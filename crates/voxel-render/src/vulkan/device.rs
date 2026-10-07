//! Bringing the backend up: instance, device and the first swapchain, and the one-shot submissions that follow.

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
        reason = "Vulkan objects created in the order the API requires"
    )]
    pub unsafe fn create(
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, VulkanError> {
        // SAFETY: the caller keeps the window alive (see `# Safety`). Each handle is made from
        // handles created before it, in the order Vulkan requires, and ends up in the returned
        // backend, whose `Drop` destroys them.
        unsafe {
            let entry =
                ash::Entry::load().map_err(|error| VulkanError::Loader(error.to_string()))?;

            // The reference takes its instance extensions from GLFW and names
            // the application the same way on every run.
            let mut extensions = ash_window::enumerate_required_extensions(display_handle)
                .map_err(|result| VulkanError::Call("enumerate surface extensions", result))?
                .to_vec();
            extensions.retain(|name| !name.is_null());

            // Layered drivers such as MoltenVK register as portability drivers,
            // which the loader hides unless the instance opts in. Opting in only
            // widens enumeration; conformant drivers are listed either way.
            let portability = entry
                .enumerate_instance_extension_properties(None)
                .map_err(|result| VulkanError::Call("enumerate instance extensions", result))?
                .iter()
                .any(|available| {
                    CStr::from_ptr(available.extension_name.as_ptr())
                        == ash::khr::portability_enumeration::NAME
                });
            let mut instance_flags = vk::InstanceCreateFlags::empty();
            if portability {
                extensions.push(ash::khr::portability_enumeration::NAME.as_ptr());
                instance_flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
            }

            let mut layers: Vec<*const i8> = Vec::new();
            let validation_names = validation::requested()
                .then(|| validation::names(&entry))
                .flatten();
            if let Some((layer, extension)) = validation_names {
                layers.push(layer);
                extensions.push(extension);
            }

            let application_name = CString::new("Minecraft Java Edition").expect("static name");
            let engine_name = CString::new("MinecraftJE").expect("static name");
            let application_info = vk::ApplicationInfo::default()
                .application_name(&application_name)
                .engine_name(&engine_name)
                .engine_version(0)
                .api_version(API_VERSION);
            let instance_info = vk::InstanceCreateInfo::default()
                .flags(instance_flags)
                .application_info(&application_info)
                .enabled_layer_names(&layers)
                .enabled_extension_names(&extensions);
            let instance = entry
                .create_instance(&instance_info, None)
                .map_err(|result| VulkanError::Call("vkCreateInstance", result))?;

            let validation = if validation_names.is_some() {
                Some(validation::Validation::new(&entry, &instance)?)
            } else {
                None
            };
            let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);
            let surface =
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
                    .map_err(|result| VulkanError::Call("vkCreateSurfaceKHR", result))?;

            let (physical_device, queue_family, properties) =
                select_physical_device(&instance, &surface_instance, surface)?;

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
            let mut features_divisor =
                vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT::default()
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

            let surface_format = pick_surface_format(&surface_instance, physical_device, surface)?;
            let capabilities = surface_instance
                .get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|result| {
                    VulkanError::Call("vkGetPhysicalDeviceSurfaceCapabilities", result)
                })?;
            let extent = clamp_extent(&capabilities, width, height);

            let swapchain_device = ash::khr::swapchain::Device::new(&instance, &device);
            let dynamic_rendering = ash::khr::dynamic_rendering::Device::new(&instance, &device);
            let swapchain_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(REQUESTED_IMAGE_COUNT.max(capabilities.min_image_count))
                .image_format(surface_format)
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                )
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
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

            let command_pool = device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .queue_family_index(queue_family)
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateCommandPool", result))?;
            let command_buffer = device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1),
                )
                .map_err(|result| VulkanError::Call("vkAllocateCommandBuffers", result))?[0];

            let image_available = device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?;
            // Present waits are per image: reusing one semaphore across images
            // is what trips validation on a FIFO swapchain.
            let mut render_finished = Vec::with_capacity(images.len());
            for _ in 0..images.len() {
                render_finished.push(
                    device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?,
                );
            }
            let in_flight = device
                .create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateFence", result))?;

            let valid_bits = instance.get_physical_device_queue_family_properties(physical_device)
                [queue_family as usize]
                .timestamp_valid_bits;
            let timestamps =
                if valid_bits > 0 && properties.limits.timestamp_compute_and_graphics == vk::TRUE {
                    let pool = device
                        .create_query_pool(
                            &vk::QueryPoolCreateInfo::default()
                                .query_type(vk::QueryType::TIMESTAMP)
                                .query_count(TIMESTAMP_COUNT),
                            None,
                        )
                        .map_err(|result| VulkanError::Call("vkCreateQueryPool", result))?;
                    Some(TimestampQueries {
                        pool,
                        period_ns: f64::from(properties.limits.timestamp_period),
                        valid_bits,
                        pending: false,
                        last: None,
                    })
                } else {
                    None
                };
            let device_name = CStr::from_ptr(properties.device_name.as_ptr())
                .to_string_lossy()
                .into_owned();
            let description = BackendDescription {
                device_name,
                discrete: properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU,
                api_version: properties.api_version,
                surface_format,
                image_count: images.len() as u32,
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
                readback: None,
                depth: None,
                out_of_date: false,
                scene: None,
                sky_buffer: None,
                pending_sky: None,
                cloud_buffer: None,
                weather_buffer: None,
                weather_counts: (0, 0),
                entity_buffer: None,
                entity_count: 0,
                ground_buffer: None,
                ground_count: 0,
                name_tag_see_buffer: None,
                name_tag_see_count: 0,
                orb_buffer: None,
                orb_count: 0,
                shadow_buffer: None,
                shadow_count: 0,
                glow_buffer: None,
                glow_count: 0,
                swirl_buffer: None,
                swirl_count: 0,
                glint_buffer: None,
                glint_count: 0,
                map_buffer: None,
                map_count: 0,
                map_decoration_buffer: None,
                map_decoration_count: 0,
                dissolve_buffer: None,
                dissolve_count: 0,
                dissolve_mask_offset: [0.0; 2],
                rays_buffer: None,
                rays_count: 0,
                name_tag_buffer: None,
                name_tag_count: 0,
                hand_buffer: None,
                hand_count: 0,
                held_count: 0,
                outline_buffer: None,
                outline_count: 0,
                breaking_buffer: None,
                breaking_count: 0,
                hand_projection: [[0.0; 4]; 4],
                gui_buffer: None,
                gui_batches: Vec::new(),
                gui_size: [1.0, 1.0],
                gui_item_buffer: None,
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
                image_available,
                render_finished,
                in_flight,
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
    pub(super) unsafe fn run_once(
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

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        // SAFETY: `device_wait_idle` runs first, then objects go children before parents (scene,
        // buffers, views, sync objects, pool, swapchain, device, surface, instance), each once,
        // since `Drop` runs once.
        unsafe {
            let _ = self.device.device_wait_idle();
            self.destroy_scene();
            // The host buffers free themselves; they go here, while the device is live.
            self.sky_buffer = None;
            self.cloud_buffer = None;
            self.weather_buffer = None;
            for buffer in [
                self.entity_buffer.take(),
                self.ground_buffer.take(),
                self.name_tag_see_buffer.take(),
                self.orb_buffer.take(),
                self.shadow_buffer.take(),
                self.glow_buffer.take(),
                self.swirl_buffer.take(),
                self.glint_buffer.take(),
                self.map_buffer.take(),
                self.map_decoration_buffer.take(),
                self.dissolve_buffer.take(),
                self.rays_buffer.take(),
                self.name_tag_buffer.take(),
                self.hand_buffer.take(),
                self.outline_buffer.take(),
                self.breaking_buffer.take(),
                self.gui_buffer.take(),
                self.gui_item_buffer.take(),
            ] {
                drop(buffer);
            }
            for block in self.arena.drain(..) {
                self.device.destroy_buffer(block.buffer, None);
                self.device.free_memory(block.memory, None);
            }
            for chunk in self.staging.drain(..) {
                self.device.destroy_buffer(chunk.buffer, None);
                self.device.free_memory(chunk.memory, None);
            }
            for view in self.image_views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            for semaphore in self.render_finished.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.device.destroy_semaphore(self.image_available, None);
            self.device.destroy_fence(self.in_flight, None);
            self.device
                .destroy_pipeline_cache(self.pipeline_cache, None);
            if let Some(queries) = self.timestamps.take() {
                self.device.destroy_query_pool(queries.pool, None);
            }
            self.device.destroy_command_pool(self.command_pool, None);
            self.free_readback();
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
