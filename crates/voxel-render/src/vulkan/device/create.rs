use super::*;

impl VulkanBackend {
    /// Bring the backend up against an existing window surface.
    ///
    /// # Safety
    ///
    /// `display_handle` and `window_handle` must refer to a window that outlives
    /// this backend; the surface borrows them.
    #[expect(
        clippy::too_many_lines,
        reason = "the backend's fields, each given its starting value"
    )]
    pub unsafe fn create(
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, VulkanError> {
        // SAFETY: the caller keeps the window alive (see `# Safety`). Each stage makes its handles
        // from the ones the stages before it made, in the order Vulkan requires, and they all end
        // up in the returned backend, whose `Drop` destroys them.
        unsafe {
            let instance_parts = create_instance(display_handle, window_handle)?;
            let device_parts = create_logical_device(&instance_parts)?;
            let presentation = create_presentation(&instance_parts, &device_parts, width, height)?;
            let sync = create_sync(&instance_parts, &device_parts, presentation.images.len())?;
            let InstanceParts {
                entry,
                instance,
                surface_instance,
                surface,
                validation,
            } = instance_parts;
            let DeviceParts {
                physical_device,
                properties,
                device,
                queue,
                ..
            } = device_parts;
            let PresentationParts {
                swapchain_device,
                dynamic_rendering,
                swapchain,
                images,
                image_views,
                surface_format,
                present_mode,
                extent,
            } = presentation;
            let SyncParts {
                command_pool,
                frames,
                render_finished,
                timestamps,
            } = sync;
            let command_buffer = frames[0].command_buffer;
            let device_name = CStr::from_ptr(properties.device_name.as_ptr())
                .to_string_lossy()
                .into_owned();
            let description = BackendDescription {
                device_name,
                discrete: properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU,
                api_version: properties.api_version,
                surface_format,
                image_count: images.len() as u32,
                present_mode,
                extent,
            };

            Ok(Self {
                _entry: entry,
                instance,
                surface_instance,
                surface,
                physical_device,
                device,
                queue,
                swapchain_device,
                dynamic_rendering,
                swapchain,
                images,
                image_views,
                command_pool,
                command_buffer,
                frames,
                frame_slot: 0,
                frames_submitted: 0,
                completed: std::cell::Cell::new(0),
                readback: None,
                depth: None,
                out_of_date: false,
                scene: None,
                sky_buffer: DynamicBuffer::default(),
                pending_sky: None,
                cloud_buffer: None,
                weather_buffer: DynamicBuffer::default(),
                weather_counts: (0, 0),
                entity: DrawList::default(),
                ground: DrawList::default(),
                name_tag_see: DrawList::default(),
                orb: DrawList::default(),
                particle: DrawList::default(),
                block_particle: DrawList::default(),
                shadow: DrawList::default(),
                glow: DrawList::default(),
                swirl: DrawList::default(),
                glint: DrawList::default(),
                map: DrawList::default(),
                map_decoration: DrawList::default(),
                dissolve: DrawList::default(),
                dissolve_mask_offset: [0.0; 2],
                rays: DrawList::default(),
                name_tag: DrawList::default(),
                hand: DrawList::default(),
                held_count: 0,
                outline: DrawList::default(),
                breaking: DrawList::default(),
                hand_projection: [[0.0; 4]; 4],
                gui_buffer: DynamicBuffer::default(),
                gui_batches: Vec::new(),
                gui_size: [1.0, 1.0],
                gui_item_buffer: DynamicBuffer::default(),
                sections: std::collections::BTreeMap::new(),
                visible_sections: None,
                moving_sections: Vec::new(),
                arena: Vec::new(),
                staging: Vec::new(),
                staging_filling: Vec::new(),
                staging_in_flight: Vec::new(),
                staged: Vec::new(),
                retired: Vec::new(),
                sort_origin: [0.0; 3],
                lightmap: DEFAULT_LIGHTMAP,
                fog: NO_FOG,
                render_finished,
                vsync: true,
                validation,
                pipeline_cache: vk::PipelineCache::null(),
                pipeline_cache_path: None,
                cache_identity: pipeline_cache::CacheIdentity {
                    vendor_id: properties.vendor_id,
                    device_id: properties.device_id,
                    uuid: properties.pipeline_cache_uuid,
                },
                timestamps,
                timings: BackendTimings::default(),
                fence_wait: std::cell::Cell::default(),
                description,
            })
        }
    }

    /// Run one command buffer to completion, for setup work.
    ///
    /// # Safety
    ///
    /// The caller must have exclusive use of the queue, and `record` must record only commands that
    /// are valid outside a render pass.
    pub(in super::super) unsafe fn run_once(
        &self,
        record: impl FnOnce(vk::CommandBuffer),
    ) -> Result<(), VulkanError> {
        // SAFETY: live handles. The command buffer is allocated here, ended before the submit, and
        // freed after `queue_wait_idle`, so the GPU is done with it; the queue is used from `&mut
        // self` callers only.
        unsafe {
            let buffer = self
                .device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(self.command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1),
                )
                .map_err(|result| VulkanError::Call("vkAllocateCommandBuffers", result))?[0];
            self.device
                .begin_command_buffer(
                    buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|result| VulkanError::Call("vkBeginCommandBuffer", result))?;
            record(buffer);
            self.device
                .end_command_buffer(buffer)
                .map_err(|result| VulkanError::Call("vkEndCommandBuffer", result))?;
            let buffers = [buffer];
            let submit = vk::SubmitInfo::default().command_buffers(&buffers);
            self.device
                .queue_submit(self.queue, &[submit], vk::Fence::null())
                .map_err(|result| VulkanError::Call("vkQueueSubmit", result))?;
            self.device
                .queue_wait_idle(self.queue)
                .map_err(|result| VulkanError::Call("vkQueueWaitIdle", result))?;
            self.device
                .free_command_buffers(self.command_pool, &buffers);
            Ok(())
        }
    }
}
