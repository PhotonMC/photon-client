use super::*;

/// What the device stage makes: the physical device picked, its queue family and properties, the
/// logical device and its queue.
pub(super) struct DeviceParts {
    pub(super) physical_device: vk::PhysicalDevice,
    pub(super) queue_family: u32,
    pub(super) properties: vk::PhysicalDeviceProperties,
    pub(super) device: ash::Device,
    pub(super) queue: vk::Queue,
}

/// Pick a physical device the way the reference does and make a device on it with the features
/// the renderer needs.
///
/// # Safety
///
/// `parts` must be live.
pub(super) unsafe fn create_logical_device(
    parts: &InstanceParts,
) -> Result<DeviceParts, VulkanError> {
    let InstanceParts {
        instance,
        surface_instance,
        surface,
        ..
    } = parts;
    let surface = *surface;
    // SAFETY: the caller's contract; the device is made on a device of this instance.
    unsafe {
        let (physical_device, queue_family, properties) =
            select_physical_device(instance, surface_instance, surface)?;

        let queue_priorities = [1.0_f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family)
            .queue_priorities(&queue_priorities)];
        let mut extension_names: Vec<*const i8> = REQUIRED_DEVICE_EXTENSIONS
            .iter()
            .map(|name| name.as_ptr())
            .collect();
        // A portability driver's device must be created with this extension
        // enabled whenever it advertises it.
        let portability_subset = instance
            .enumerate_device_extension_properties(physical_device)
            .map_err(|result| VulkanError::Call("enumerate device extensions", result))?
            .iter()
            .any(|available| {
                CStr::from_ptr(available.extension_name.as_ptr())
                    == ash::khr::portability_subset::NAME
            });
        if portability_subset {
            extension_names.push(ash::khr::portability_subset::NAME.as_ptr());
        }

        // The reference enables these by name; the ones that gate rendering
        // are requested here so device selection matches what it would pick.
        let mut features_11 =
            vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);
        let mut features_12 = vk::PhysicalDeviceVulkan12Features::default()
            .timeline_semaphore(true)
            .host_query_reset(true);
        let mut features_sync2 =
            vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);
        let mut features_dynamic =
            vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);
        let mut features_divisor = vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT::default()
            .vertex_attribute_instance_rate_divisor(true);
        let features_10 = vk::PhysicalDeviceFeatures::default()
            .multi_draw_indirect(true)
            .fill_mode_non_solid(true)
            .sampler_anisotropy(true);

        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&extension_names)
            .enabled_features(&features_10)
            .push_next(&mut features_11)
            .push_next(&mut features_12)
            .push_next(&mut features_sync2)
            .push_next(&mut features_dynamic)
            .push_next(&mut features_divisor);
        let device = instance
            .create_device(physical_device, &device_info, None)
            .map_err(|result| VulkanError::Call("vkCreateDevice", result))?;
        let queue = device.get_device_queue(queue_family, 0);
        Ok(DeviceParts {
            physical_device,
            queue_family,
            properties,
            device,
            queue,
        })
    }
}
