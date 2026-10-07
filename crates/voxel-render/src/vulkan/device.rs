//! Bringing the backend up: instance, device and the first swapchain, and the one-shot submissions that follow.

use super::*;

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        // SAFETY: `device_wait_idle` runs first, then objects go children before parents (scene,
        // buffers, views, sync objects, pool, swapchain, device, surface, instance), each once,
        // since `Drop` runs once.
        unsafe {
            let _ = self.device.device_wait_idle();
            self.destroy_scene();
            // The host buffers free themselves; they go here, while the device is live.
            self.cloud_buffer = None;
            self.sky_buffer.clear();
            self.weather_buffer.clear();
            self.entity.buffer.clear();
            self.ground.buffer.clear();
            self.name_tag_see.buffer.clear();
            self.orb.buffer.clear();
            self.shadow.buffer.clear();
            self.glow.buffer.clear();
            self.swirl.buffer.clear();
            self.glint.buffer.clear();
            self.map.buffer.clear();
            self.map_decoration.buffer.clear();
            self.dissolve.buffer.clear();
            self.rays.buffer.clear();
            self.name_tag.buffer.clear();
            self.hand.buffer.clear();
            self.outline.buffer.clear();
            self.breaking.buffer.clear();
            self.gui_buffer.clear();
            self.gui_item_buffer.clear();
            self.arena.clear();
            self.staging.clear();
            self.readback = None;
            for view in self.image_views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            for semaphore in self.render_finished.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }
            for frame in self.frames.drain(..) {
                self.device.destroy_semaphore(frame.image_available, None);
                self.device.destroy_fence(frame.in_flight, None);
            }
            self.device
                .destroy_pipeline_cache(self.pipeline_cache, None);
            if let Some(queries) = self.timestamps.take() {
                self.device.destroy_query_pool(queries.pool, None);
            }
            self.device.destroy_command_pool(self.command_pool, None);
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_instance.destroy_surface(self.surface, None);
            if let Some(validation) = self.validation.take() {
                validation.destroy();
            }
            self.instance.destroy_instance(None);
        }
    }
}

/// Pick a device the way the reference does: skip anything that cannot meet the
/// required extensions or the minimum API version, take the first that can, and
/// upgrade to a discrete GPU if a later one turns out to be discrete.
///
/// # Safety
///
/// `instance`, `surface_instance` and `surface` must be live and belong together.
pub(super) unsafe fn select_physical_device(
    instance: &ash::Instance,
    surface_instance: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32, vk::PhysicalDeviceProperties), VulkanError> {
    // SAFETY: live instance and surface (see `# Safety`). The extension names are NUL-terminated
    // arrays the driver fills in.
    unsafe {
        let devices = instance
            .enumerate_physical_devices()
            .map_err(|result| VulkanError::Call("vkEnumeratePhysicalDevices", result))?;
        let mut selected: Option<(vk::PhysicalDevice, u32, vk::PhysicalDeviceProperties)> = None;

        for device in devices {
            let properties = instance.get_physical_device_properties(device);
            if properties.api_version < MINIMUM_DEVICE_API_VERSION {
                continue;
            }
            let Ok(extensions) = instance.enumerate_device_extension_properties(device) else {
                continue;
            };
            let has_all = REQUIRED_DEVICE_EXTENSIONS.iter().all(|required| {
                extensions
                    .iter()
                    .any(|available| CStr::from_ptr(available.extension_name.as_ptr()) == *required)
            });
            if !has_all {
                continue;
            }
            let Some(queue_family) =
                graphics_present_family(instance, surface_instance, surface, device)
            else {
                continue;
            };

            let discrete = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
            match &selected {
                None => selected = Some((device, queue_family, properties)),
                Some((_, _, chosen)) => {
                    if discrete && chosen.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
                        selected = Some((device, queue_family, properties));
                    }
                }
            }
        }

        selected.ok_or(VulkanError::NoSuitableDevice)
    }
}

/// First queue family that can both render and present to this surface.
///
/// # Safety
///
/// `instance`, `surface_instance` and `surface` must be live and belong together, and `device` must
/// come from `instance`.
pub(super) unsafe fn graphics_present_family(
    instance: &ash::Instance,
    surface_instance: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    device: vk::PhysicalDevice,
) -> Option<u32> {
    // SAFETY: live instance and surface; `device` comes from this instance.
    unsafe {
        instance
            .get_physical_device_queue_family_properties(device)
            .iter()
            .enumerate()
            .find_map(|(index, family)| {
                let index = index as u32;
                let graphics = family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                let present = surface_instance
                    .get_physical_device_surface_support(device, index, surface)
                    .unwrap_or(false);
                (graphics && present).then_some(index)
            })
    }
}

/// The reference accepts only these two formats, and only in sRGB non-linear.
///
/// # Safety
///
/// `surface_instance` and `surface` must be live and belong together, and `device` must come from
/// the same instance.
pub(super) unsafe fn pick_surface_format(
    surface_instance: &ash::khr::surface::Instance,
    device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<vk::Format, VulkanError> {
    // SAFETY: live instance and surface; `device` comes from this instance.
    unsafe {
        let formats = surface_instance
            .get_physical_device_surface_formats(device, surface)
            .map_err(|result| VulkanError::Call("vkGetPhysicalDeviceSurfaceFormats", result))?;
        formats
            .iter()
            .find(|format| {
                format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    && ACCEPTED_SURFACE_FORMATS.contains(&format.format)
            })
            .map(|format| format.format)
            .ok_or(VulkanError::NoSuitableSurfaceFormat)
    }
}

mod create;
mod instance;
mod logical;
mod presentation;
mod sync;

use instance::*;
use logical::*;
use presentation::*;
use sync::*;
