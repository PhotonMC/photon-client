use super::*;

/// What the presentation stage makes: the swapchain with its images and views, and what
/// describes them.
pub(super) struct PresentationParts {
    pub(super) swapchain_device: ash::khr::swapchain::Device,
    pub(super) dynamic_rendering: ash::khr::dynamic_rendering::Device,
    pub(super) swapchain: vk::SwapchainKHR,
    pub(super) images: Vec<vk::Image>,
    pub(super) image_views: Vec<vk::ImageView>,
    pub(super) surface_format: vk::Format,
    pub(super) present_mode: vk::PresentModeKHR,
    pub(super) extent: vk::Extent2D,
}

/// The first swapchain, with a view of each of its images.
///
/// # Safety
///
/// Both parts must be live.
pub(super) unsafe fn create_presentation(
    instance_parts: &InstanceParts,
    device_parts: &DeviceParts,
    width: u32,
    height: u32,
) -> Result<PresentationParts, VulkanError> {
    let InstanceParts {
        instance,
        surface_instance,
        surface,
        ..
    } = instance_parts;
    let surface = *surface;
    let DeviceParts {
        physical_device,
        device,
        ..
    } = device_parts;
    let physical_device = *physical_device;
    // SAFETY: the caller's contract; the swapchain is made for this surface and device.
    unsafe {
        let surface_format = pick_surface_format(surface_instance, physical_device, surface)?;
        let capabilities = surface_instance
            .get_physical_device_surface_capabilities(physical_device, surface)
            .map_err(|result| {
                VulkanError::Call("vkGetPhysicalDeviceSurfaceCapabilities", result)
            })?;
        let extent = clamp_extent(&capabilities, width, height);

        let swapchain_device = ash::khr::swapchain::Device::new(instance, device);
        let dynamic_rendering = ash::khr::dynamic_rendering::Device::new(instance, device);
        let present_mode = choose_present_mode(
            &surface_instance
                .get_physical_device_surface_present_modes(physical_device, surface)
                .map_err(|result| {
                    VulkanError::Call("vkGetPhysicalDeviceSurfacePresentModes", result)
                })?,
            true,
        );
        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(REQUESTED_IMAGE_COUNT.max(capabilities.min_image_count))
            .image_format(surface_format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(super::super::swapchain::SWAPCHAIN_USAGE)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        let swapchain = swapchain_device
            .create_swapchain(&swapchain_info, None)
            .map_err(|result| VulkanError::Call("vkCreateSwapchainKHR", result))?;
        let images = swapchain_device
            .get_swapchain_images(swapchain)
            .map_err(|result| VulkanError::Call("vkGetSwapchainImagesKHR", result))?;

        let mut image_views = Vec::with_capacity(images.len());
        for image in &images {
            image_views.push(
                device
                    .create_image_view(
                        &vk::ImageViewCreateInfo::default()
                            .image(*image)
                            .view_type(vk::ImageViewType::TYPE_2D)
                            .format(surface_format)
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
        Ok(PresentationParts {
            swapchain_device,
            dynamic_rendering,
            swapchain,
            images,
            image_views,
            surface_format,
            present_mode,
            extent,
        })
    }
}
