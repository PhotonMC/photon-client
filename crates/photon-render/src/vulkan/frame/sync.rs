use super::*;

impl VulkanBackend {
    /// Wait until every frame up to number `through` has finished on the GPU. The wait is counted
    /// against the frame being built, since that is the frame that stood still for it.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(in super::super) unsafe fn complete_through(
        &self,
        through: u64,
    ) -> Result<(), VulkanError> {
        let through = through.min(self.frames_submitted);
        let start = std::time::Instant::now();
        for number in self.completed.get() + 1..=through {
            let slot = &self.frames[number as usize % FRAMES_IN_FLIGHT];
            // A slot that took a later frame had to finish this one first, so only a slot still
            // holding this frame has a fence to wait on.
            if slot.stamp == number {
                // SAFETY: live handles; the fence is waited on, never reset, here.
                let waited = unsafe {
                    self.device
                        .wait_for_fences(&[slot.in_flight], true, u64::MAX)
                };
                waited.map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            }
            self.completed.set(number);
        }
        self.fence_wait.set(self.fence_wait.get() + start.elapsed());
        Ok(())
    }

    /// Give back the section meshes whose last frame has finished, along with the copies still
    /// waiting to fill them.
    ///
    /// # Safety
    ///
    /// The GPU must be done with every frame up to `completed`.
    pub(in super::super) unsafe fn free_retired(&mut self) {
        let completed = self.completed.get();
        let (done, waiting): (Vec<_>, Vec<_>) = std::mem::take(&mut self.retired)
            .into_iter()
            .partition(|(stamp, _)| *stamp <= completed);
        self.retired = waiting;
        for (_, section) in done {
            // A copy into a mesh nobody will draw has nowhere safe to go: its range is free now.
            self.staged
                .retain(|copy| copy.target != section.vertices && copy.target != section.indices);
            // SAFETY: the section's last frame has finished (see `# Safety`).
            unsafe { self.destroy_section(section) };
        }
    }

    /// Wait until every frame submitted so far has finished.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(in super::super) unsafe fn wait_in_flight(&self) -> Result<(), VulkanError> {
        // SAFETY: the caller's contract.
        unsafe { self.complete_through(self.frames_submitted) }
    }

    /// Write a timestamp after the work recorded so far, if the device can.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording, with the timestamp queries reset in it.
    pub(in super::super) unsafe fn stamp(&self, query: u32) {
        if let Some(queries) = &self.timestamps {
            let query = (self.frame_slot as u32) * TIMESTAMP_COUNT + query;
            // SAFETY: recording (see `# Safety`), and the pool is live and owned by `self`.
            unsafe {
                self.device.cmd_write_timestamp(
                    self.command_buffer,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    queries.pool,
                    query,
                );
            }
        }
    }

    /// Read back the timestamps the last frame of this slot wrote, if it wrote any.
    ///
    /// # Safety
    ///
    /// The caller has waited for that frame, so it has finished.
    pub(in super::super) unsafe fn read_timestamps(&mut self) {
        let slot = self.frame_slot;
        let Some(queries) = self
            .timestamps
            .as_mut()
            .filter(|queries| queries.pending[slot])
        else {
            return;
        };
        queries.pending[slot] = false;
        let mut ticks = [0_u64; TIMESTAMP_COUNT as usize];
        // SAFETY: live pool; the frame that wrote it has finished (see `# Safety`). A pass the
        // frame did not reach leaves the query unavailable and the call fails, which keeps the
        // previous reading.
        let read = unsafe {
            self.device.get_query_pool_results(
                queries.pool,
                slot as u32 * TIMESTAMP_COUNT,
                &mut ticks,
                vk::QueryResultFlags::TYPE_64,
            )
        };
        if read.is_ok() {
            queries.last = Some(GpuTimings::from_ticks(
                &ticks,
                queries.valid_bits,
                queries.period_ns,
            ));
        }
    }

    /// The CPU time the last frame spent in the backend, by span.
    pub fn timings(&self) -> BackendTimings {
        self.timings
    }

    /// The GPU's time in each pass of the last frame whose timestamps have
    /// been read back, or nothing when the device cannot write timestamps.
    pub fn gpu_timings(&self) -> Option<GpuTimings> {
        self.timestamps.as_ref().and_then(|queries| queries.last)
    }

    /// What the backend selected.
    pub fn description(&self) -> &BackendDescription {
        &self.description
    }

    /// Acquire, clear to one colour, and present a single frame.
    ///
    /// The colour is a placeholder: no reference pixel has been compared yet.
    pub fn draw_clear_frame(&mut self, colour: [f32; 4]) -> Result<(), VulkanError> {
        self.render_frame(colour, false, IDENTITY).map(|_| ())
    }

    /// Draw the loaded scene under one view-projection matrix.
    pub fn draw_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<(), VulkanError> {
        self.render_frame(colour, false, view_projection)
            .map(|_| ())
    }

    /// Draw the loaded scene and read the presented image back.
    pub fn capture_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<CapturedFrame, VulkanError> {
        let pixels = self.render_frame(colour, true, view_projection)?;
        Ok(CapturedFrame {
            width: self.description.extent.width,
            height: self.description.extent.height,
            rgba: pixels.unwrap_or_default(),
        })
    }

    /// Render one frame and read the presented image back as RGBA8 rows.
    ///
    /// Readback is the same path frame comparison will need later, so it copies
    /// the image the compositor receives rather than re-rendering into an
    /// offscreen target.
    pub fn capture_frame(&mut self, colour: [f32; 4]) -> Result<CapturedFrame, VulkanError> {
        let pixels = self.render_frame(colour, true, IDENTITY)?;
        Ok(CapturedFrame {
            width: self.description.extent.width,
            height: self.description.extent.height,
            rgba: pixels.unwrap_or_default(),
        })
    }

    pub(in super::super) fn render_frame(
        &mut self,
        colour: [f32; 4],
        capture: bool,
        view_projection: [[f32; 4]; 4],
    ) -> Result<Option<Vec<u8>>, VulkanError> {
        // SAFETY: live handles. The fence wait comes first, so the previous frame is done before
        // its resources are recycled; the command buffer is begun here and ended before the
        // submit, and the readback is mapped only after the fence has signalled.
        unsafe {
            // The frame about to be recorded takes the slot of the one before the last, which must
            // have finished: the last frame may still be drawing.
            let number = self.frames_submitted + 1;
            self.frame_slot = number as usize % FRAMES_IN_FLIGHT;
            self.command_buffer = self.frames[self.frame_slot].command_buffer;
            self.complete_through(self.frames[self.frame_slot].stamp)?;
            self.read_timestamps();
            self.free_retired();
            let completed = self.completed.get();
            self.staging_in_flight
                .retain(|&(stamp, _)| stamp > completed);
            self.trim_staging();
            if capture {
                self.make_readback()?;
            }

            let acquire_start = std::time::Instant::now();
            let (index, _) = match self.swapchain_device.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.frames[self.frame_slot].image_available,
                vk::Fence::null(),
            ) {
                Ok(value) => value,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.out_of_date = true;
                    return Ok(None);
                }
                Err(result) => return Err(VulkanError::Call("vkAcquireNextImageKHR", result)),
            };
            self.timings.acquire = acquire_start.elapsed();
            self.device
                .reset_fences(&[self.frames[self.frame_slot].in_flight])
                .map_err(|result| VulkanError::Call("vkResetFences", result))?;

            let image = self.images[index as usize];
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(1);

            self.device
                .begin_command_buffer(
                    self.command_buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|result| VulkanError::Call("vkBeginCommandBuffer", result))?;
            let record_start = std::time::Instant::now();
            if let Some(queries) = &self.timestamps {
                self.device.cmd_reset_query_pool(
                    self.command_buffer,
                    queries.pool,
                    self.frame_slot as u32 * TIMESTAMP_COUNT,
                    TIMESTAMP_COUNT,
                );
            }
            self.record_staged_copies();

            transition(
                &self.device,
                self.command_buffer,
                image,
                range,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );
            self.device.cmd_clear_color_image(
                self.command_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearColorValue { float32: colour },
                &[range],
            );

            if self.scene.is_some() {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                );
                self.record_scene(index as usize, view_projection);
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
            }
            if capture {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                );
                let region = vk::BufferImageCopy::default()
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .image_extent(vk::Extent3D {
                        width: self.description.extent.width,
                        height: self.description.extent.height,
                        depth: 1,
                    });
                self.device.cmd_copy_image_to_buffer(
                    self.command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    self.readback.as_ref().expect("made for the capture").buffer,
                    &[region],
                );
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
            } else {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
            }

            self.device
                .end_command_buffer(self.command_buffer)
                .map_err(|result| VulkanError::Call("vkEndCommandBuffer", result))?;
            self.timings.record = record_start.elapsed();

            let submit_start = std::time::Instant::now();
            let wait_semaphores = [self.frames[self.frame_slot].image_available];
            let wait_stages = [vk::PipelineStageFlags::TRANSFER];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished[index as usize]];
            let submit = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            self.device
                .queue_submit(
                    self.queue,
                    &[submit],
                    self.frames[self.frame_slot].in_flight,
                )
                .map_err(|result| VulkanError::Call("vkQueueSubmit", result))?;

            let swapchains = [self.swapchain];
            let indices = [index];
            let present = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);
            match self.swapchain_device.queue_present(self.queue, &present) {
                Ok(false) => {}
                Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR) => {
                    self.out_of_date = true;
                }
                Err(result) => return Err(VulkanError::Call("vkQueuePresentKHR", result)),
            }
            self.timings.submit_present = submit_start.elapsed();
            self.timings.fence_wait = self.fence_wait.take();
            self.frames_submitted = number;
            self.frames[self.frame_slot].stamp = number;
            let (slot, wrote_scene) = (self.frame_slot, self.scene.is_some());
            if let Some(queries) = self.timestamps.as_mut() {
                queries.pending[slot] = wrote_scene;
            }

            if !capture {
                return Ok(None);
            }
            self.wait_in_flight()?;
            let byte_count =
                (self.description.extent.width * self.description.extent.height * 4) as usize;
            let memory = self.readback.as_ref().expect("made for the capture").memory;
            let mapped = self
                .device
                .map_memory(memory, 0, byte_count as u64, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            let mut rgba = vec![0_u8; byte_count];
            std::ptr::copy_nonoverlapping(mapped, rgba.as_mut_ptr(), byte_count);
            self.device.unmap_memory(memory);
            self.readback = None;
            if self.description.surface_format == vk::Format::B8G8R8A8_UNORM {
                for pixel in rgba.as_chunks_mut::<4>().0 {
                    pixel.swap(0, 2);
                }
            }
            Ok(Some(rgba))
        }
    }
}
