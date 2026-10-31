//! The swapchain and what is sized by it.

use super::*;

/// What the frame's images are used for: drawn to, cleared, and read back for the blur behind a
/// screen and for captures.
pub(super) const SWAPCHAIN_USAGE: vk::ImageUsageFlags = vk::ImageUsageFlags::from_raw(
    vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw()
        | vk::ImageUsageFlags::TRANSFER_DST.as_raw()
        | vk::ImageUsageFlags::TRANSFER_SRC.as_raw(),
);

/// The present mode for vsync on or off, from what the surface supports, by the reference's own
/// preference (`GpuSurface.PresentMode.getSupportedVsyncMode`). FIFO is always supported.
pub(super) fn choose_present_mode(
    supported: &[vk::PresentModeKHR],
    vsync: bool,
) -> vk::PresentModeKHR {
    let preferred: &[vk::PresentModeKHR] = if vsync {
        &[vk::PresentModeKHR::FIFO_RELAXED, vk::PresentModeKHR::FIFO]
    } else {
        &[
            vk::PresentModeKHR::IMMEDIATE,
            vk::PresentModeKHR::MAILBOX,
            vk::PresentModeKHR::FIFO,
        ]
    };
    preferred
        .iter()
        .copied()
        .find(|mode| supported.contains(mode))
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

impl VulkanBackend {
    /// Show frames as the display refreshes (`true`) or as soon as they are drawn. The swapchain
    /// is made again with the mode that fits, at the next [`Self::resize`].
    pub fn set_vsync(&mut self, vsync: bool) {
        if self.vsync != vsync {
            self.vsync = vsync;
            self.out_of_date = true;
        }
    }

    /// Whether frames are shown as the display refreshes.
    pub fn vsync(&self) -> bool {
        self.vsync
    }

    /// Whether the swapchain no longer fits the window and wants
    /// [`Self::resize`].
    pub fn out_of_date(&self) -> bool {
        self.out_of_date
    }

    /// The size frames are shown at: the swapchain's, which is the drawn size unless the frames
    /// are scaled up to it.
    pub fn present_extent(&self) -> (u32, u32) {
        self.targets
            .scaled
            .as_ref()
            .map_or_else(|| self.extent(), |(_, shown)| (shown.width, shown.height))
    }

    /// The size frames are drawn at, in pixels.
    pub fn extent(&self) -> (u32, u32) {
        (
            self.description.extent.width,
            self.description.extent.height,
        )
    }

    /// Rebuild the swapchain, and what is sized by it, for a new window
    /// size. A zero size (a minimised window) is left for later.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), VulkanError> {
        self.resize_scaled((width, height), (width, height))
    }

    /// [`Self::resize`] for a frame drawn at one size (`render`) and shown at another (`present`,
    /// the window's): the frame is drawn into a target of its own size and blitted up to the
    /// swapchain. Two equal sizes draw into the swapchain directly.
    pub fn resize_scaled(
        &mut self,
        render: (u32, u32),
        (width, height): (u32, u32),
    ) -> Result<(), VulkanError> {
        if width == 0 || height == 0 || render.0 == 0 || render.1 == 0 {
            return Ok(());
        }
        // SAFETY: live handles; `device_wait_idle` runs first, so the old swapchain, views, depth
        // target and semaphores are unused when they are replaced.
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|result| VulkanError::Call("vkDeviceWaitIdle", result))?;
            let capabilities = self
                .surface_instance
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)
                .map_err(|result| {
                    VulkanError::Call("vkGetPhysicalDeviceSurfaceCapabilities", result)
                })?;
            let extent = clamp_extent(&capabilities, width, height);
            if extent.width == 0 || extent.height == 0 {
                return Ok(());
            }
            let present_mode = self.replace_swapchain(&capabilities, extent)?;
            self.replace_views_and_semaphores()?;
            self.size_targets(render, extent)?;
            self.description.image_count = self.images.len() as u32;
            self.description.present_mode = present_mode;
            self.out_of_date = false;
            Ok(())
        }
    }

    /// The swapchain made again at `extent`, with the present mode the vsync setting asks for.
    ///
    /// # Safety
    ///
    /// The device is idle.
    unsafe fn replace_swapchain(
        &mut self,
        capabilities: &vk::SurfaceCapabilitiesKHR,
        extent: vk::Extent2D,
    ) -> Result<vk::PresentModeKHR, VulkanError> {
        // SAFETY: live handles; the device is idle (see `# Safety`).
        unsafe {
            let present_mode = choose_present_mode(
                &self
                    .surface_instance
                    .get_physical_device_surface_present_modes(self.physical_device, self.surface)
                    .map_err(|result| {
                        VulkanError::Call("vkGetPhysicalDeviceSurfacePresentModes", result)
                    })?,
                self.vsync,
            );
            let swapchain_info = vk::SwapchainCreateInfoKHR::default()
                .surface(self.surface)
                .min_image_count(REQUESTED_IMAGE_COUNT.max(capabilities.min_image_count))
                .image_format(self.description.surface_format)
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(SWAPCHAIN_USAGE)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .old_swapchain(self.swapchain);
            let swapchain = self
                .swapchain_device
                .create_swapchain(&swapchain_info, None)
                .map_err(|result| VulkanError::Call("vkCreateSwapchainKHR", result))?;
            for view in self.image_views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
            self.swapchain = swapchain;
            self.images = self
                .swapchain_device
                .get_swapchain_images(swapchain)
                .map_err(|result| VulkanError::Call("vkGetSwapchainImagesKHR", result))?;
            Ok(present_mode)
        }
    }

    /// A view of each swapchain image, and the semaphores that follow the image count; the acquire
    /// semaphore may have been left signalled by a failed acquire, so it is replaced too.
    ///
    /// # Safety
    ///
    /// The device is idle.
    unsafe fn replace_views_and_semaphores(&mut self) -> Result<(), VulkanError> {
        // SAFETY: live handles; the device is idle (see `# Safety`).
        unsafe {
            for image in &self.images {
                self.image_views.push(
                    self.device
                        .create_image_view(
                            &vk::ImageViewCreateInfo::default()
                                .image(*image)
                                .view_type(vk::ImageViewType::TYPE_2D)
                                .format(self.description.surface_format)
                                .subresource_range(
                                    vk::ImageSubresourceRange::default()
                                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                                        .level_count(1)
                                        .layer_count(1),
                                ),
                            None,
                        )
                        .map_err(|result| VulkanError::Call("vkCreateImageView", result))?,
                );
            }
            for semaphore in self.render_finished.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }
            for _ in 0..self.images.len() {
                self.render_finished.push(
                    self.device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?,
                );
            }
            for frame in &mut self.frames {
                self.device.destroy_semaphore(frame.image_available, None);
                frame.image_available = self
                    .device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                    .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?;
            }
            Ok(())
        }
    }

    /// The size frames are drawn at, and the targets that follow it: the colour target when it is
    /// not the swapchain's size, and the depth target.
    ///
    /// # Safety
    ///
    /// The device is idle.
    unsafe fn size_targets(
        &mut self,
        render: (u32, u32),
        extent: vk::Extent2D,
    ) -> Result<(), VulkanError> {
        // SAFETY: live handles; the device is idle (see `# Safety`).
        unsafe {
            let drawn = vk::Extent2D {
                width: render.0,
                height: render.1,
            };
            let scaled = drawn != extent;
            self.description.extent = if scaled { drawn } else { extent };
            self.targets.scaled = None;
            if scaled {
                let target = self.target_image(
                    self.description.surface_format,
                    (SWAPCHAIN_USAGE, vk::ImageAspectFlags::COLOR),
                    drawn,
                )?;
                self.targets.scaled = Some((target, extent));
            }
            if self.targets.depth.take().is_some() {
                self.targets.depth = Some(self.create_depth_target()?);
            }
            Ok(())
        }
    }

    /// A depth image, with its memory and view, for the current extent.
    ///
    /// # Safety
    ///
    /// No frame may be recording: it submits setup work and waits for the queue.
    pub(super) unsafe fn create_depth_target(&self) -> Result<Image, VulkanError> {
        // SAFETY: live handles; no frame is recording (see `# Safety`).
        unsafe {
            let depth_image = self.depth_image()?;
            let image = depth_image.image;
            // The scene's rendering expects the depth image in its attachment layout, and clears
            // it on load, so nothing needs to survive from the first use.
            let depth = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                .level_count(1)
                .layer_count(1);
            self.run_once(|command| {
                transition(
                    &self.device,
                    command,
                    image,
                    depth,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                );
            })?;
            Ok(depth_image)
        }
    }

    /// A depth image of the frame's size, in no layout yet.
    ///
    /// # Safety
    ///
    /// Live handles.
    pub(super) unsafe fn depth_image(&self) -> Result<Image, VulkanError> {
        // SAFETY: live handles.
        unsafe {
            self.target_image(
                vk::Format::D32_SFLOAT,
                (
                    vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                    vk::ImageAspectFlags::DEPTH,
                ),
                self.description.extent,
            )
        }
    }

    /// A device-local image of `extent`, in no layout yet, with its memory and view.
    ///
    /// # Safety
    ///
    /// Live handles.
    pub(super) unsafe fn target_image(
        &self,
        format: vk::Format,
        (usage, aspect): (vk::ImageUsageFlags, vk::ImageAspectFlags),
        extent: vk::Extent2D,
    ) -> Result<Image, VulkanError> {
        // SAFETY: live handles; the image's memory is bound before the view is made, and the
        // target owns all three.
        unsafe {
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: extent.width,
                            height: extent.height,
                            depth: 1,
                        })
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(usage)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .initial_layout(vk::ImageLayout::UNDEFINED),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImage", result))?;
            let requirements = self.device.get_image_memory_requirements(image);
            let memory = self.allocate(requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            self.device
                .bind_image_memory(image, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindImageMemory", result))?;
            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format)
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(aspect)
                                .level_count(1)
                                .layer_count(1),
                        ),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            Ok(Image::new(&self.device, (image, memory, view), format))
        }
    }

    /// Blit the drawn frame, left readable, over the whole swapchain image and leave that ready to
    /// present. The sizes may differ: the compositor shows what the swapchain holds.
    ///
    /// # Safety
    ///
    /// Recording, outside a rendering scope; `drawn` is the scaled target in the transfer-source
    /// layout.
    pub(super) unsafe fn record_scaled_present(&self, drawn: vk::Image, shown: vk::Image) {
        let Some((_, present)) = self.targets.scaled.as_ref() else {
            return;
        };
        let range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);
        let corner = |extent: vk::Extent2D| vk::Offset3D {
            x: extent.width as i32,
            y: extent.height as i32,
            z: 1,
        };
        let layer = vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(1);
        let region = vk::ImageBlit::default()
            .src_subresource(layer)
            .src_offsets([vk::Offset3D::default(), corner(self.description.extent)])
            .dst_subresource(layer)
            .dst_offsets([vk::Offset3D::default(), corner(*present)]);
        // SAFETY: the caller's contract; both images are live and of one format.
        unsafe {
            transition(
                &self.device,
                self.command_buffer,
                shown,
                range,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );
            self.device.cmd_blit_image(
                self.command_buffer,
                drawn,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                shown,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
                vk::Filter::LINEAR,
            );
            transition(
                &self.device,
                self.command_buffer,
                shown,
                range,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR,
            );
        }
    }

    /// The format of the depth target, none yet being undefined.
    pub(super) fn depth_format(&self) -> vk::Format {
        self.targets
            .depth
            .as_ref()
            .map_or(vk::Format::UNDEFINED, |depth| depth.format)
    }

    /// The image and view a frame is drawn into: the scaled target, or swapchain image `index`.
    pub(super) fn frame_target(&self, index: usize) -> (vk::Image, vk::ImageView) {
        match &self.targets.scaled {
            Some((target, _)) => (target.image, target.view),
            None => (self.images[index], self.image_views[index]),
        }
    }
}

/// Keep the requested size inside what the surface reports it can present.
pub(super) fn clamp_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    width: u32,
    height: u32,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }
    vk::Extent2D {
        width: width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [vk::PresentModeKHR; 4] = [
        vk::PresentModeKHR::IMMEDIATE,
        vk::PresentModeKHR::MAILBOX,
        vk::PresentModeKHR::FIFO,
        vk::PresentModeKHR::FIFO_RELAXED,
    ];

    #[test]
    fn vsync_takes_relaxed_fifo_then_fifo() {
        assert_eq!(
            choose_present_mode(&ALL, true),
            vk::PresentModeKHR::FIFO_RELAXED
        );
        assert_eq!(
            choose_present_mode(&ALL[..3], true),
            vk::PresentModeKHR::FIFO
        );
    }

    #[test]
    fn no_vsync_takes_immediate_then_mailbox_then_fifo() {
        assert_eq!(
            choose_present_mode(&ALL, false),
            vk::PresentModeKHR::IMMEDIATE
        );
        assert_eq!(
            choose_present_mode(&ALL[1..], false),
            vk::PresentModeKHR::MAILBOX
        );
        assert_eq!(
            choose_present_mode(&ALL[2..], false),
            vk::PresentModeKHR::FIFO
        );
    }
}
