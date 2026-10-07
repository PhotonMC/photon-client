//! The swapchain and what is sized by it.

use super::*;

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

    /// The swapchain's size in pixels.
    pub fn extent(&self) -> (u32, u32) {
        (
            self.description.extent.width,
            self.description.extent.height,
        )
    }

    /// Rebuild the swapchain, and what is sized by it, for a new window
    /// size. A zero size (a minimised window) is left for later.
    #[expect(
        clippy::too_many_lines,
        reason = "Vulkan objects created in the order the API requires"
    )]
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), VulkanError> {
        if width == 0 || height == 0 {
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
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                )
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
            // The acquire semaphore may have been left signalled by a failed
            // acquire, so it is replaced too.
            for frame in &mut self.frames {
                self.device.destroy_semaphore(frame.image_available, None);
                frame.image_available = self
                    .device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                    .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?;
            }
            self.description.extent = extent;
            self.description.image_count = self.images.len() as u32;
            self.description.present_mode = present_mode;

            if self.depth.take().is_some() {
                self.depth = Some(self.create_depth_target()?);
            }
            self.out_of_date = false;
            Ok(())
        }
    }

    /// A depth image, with its memory and view, for the current extent.
    ///
    /// # Safety
    ///
    /// No frame may be recording: it submits setup work and waits for the queue.
    pub(super) unsafe fn create_depth_target(&self) -> Result<Image, VulkanError> {
        // SAFETY: live handles; the image's memory is bound before the view is made, and the
        // target owns all three.
        unsafe {
            let format = vk::Format::D32_SFLOAT;
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: self.description.extent.width,
                            height: self.description.extent.height,
                            depth: 1,
                        })
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
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
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .level_count(1)
                                .layer_count(1),
                        ),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            let depth_image = Image::new(&self.device, (image, memory, view), format);
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
