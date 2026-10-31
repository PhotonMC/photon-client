use super::*;

impl VulkanBackend {
    /// Bring the backend up against an existing window surface.
    ///
    /// # Safety
    ///
    /// `display_handle` and `window_handle` must refer to a window that outlives
    /// this backend; the surface borrows them.
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
            let description = describe(
                (&instance, physical_device, &properties),
                (surface_format, images.len(), present_mode, extent),
            );

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
                command_buffer: frames[0].command_buffer,
                frames,
                frame_slot: 0,
                frames_submitted: 0,
                completed: std::cell::Cell::new(0),
                readback: None,
                targets: Default::default(),
                out_of_date: false,
                gui_only: false,
                filtering: Default::default(),
                scene: None,
                sky_buffer: DynamicBuffer::default(),
                pending_sky: None,
                panorama_view: None,
                panorama_blur: 0,
                cloud_buffer: None,
                weather_buffer: DynamicBuffer::default(),
                weather_counts: (0, 0),
                world_border_buffer: DynamicBuffer::default(),
                world_border_count: 0,
                screen_effect_buffer: DynamicBuffer::default(),
                screen_effect_counts: [0; 3],
                screen_effect_projection: [[0.0; 4]; 4],
                entity: DrawList::default(),
                ground: DrawList::default(),
                name_tag_see: DrawList::default(),
                orb: DrawList::default(),
                particle: DrawList::default(),
                block_particle: DrawList::default(),
                flat: FlatDraws::default(),
                map: DrawList::default(),
                map_decoration: DrawList::default(),
                dissolve: DrawList::default(),
                dissolve_mask_offset: [0.0; 2],
                rays: DrawList::default(),
                portal: DrawList::default(),
                portal_scene: None,
                portal_time: 0.0,
                name_tag: Default::default(),
                hand: DrawList::default(),
                held_counts: [0; 3],
                lines: LineDraws::default(),
                glint_strength: 0.75,
                breaking: DrawList::default(),
                hand_projection: [[0.0; 4]; 4],
                gui_buffer: DynamicBuffer::default(),
                gui_runs: GuiRuns::default(),
                gui_size: [1.0, 1.0],
                gui_item_buffer: DynamicBuffer::default(),
                sections: Default::default(),
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
                cache_identity: pipeline_cache::CacheIdentity::of(&properties),
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

/// What the backend tells of the device it chose and the surface it draws to.
fn describe(
    (instance, physical_device, properties): (
        &ash::Instance,
        vk::PhysicalDevice,
        &vk::PhysicalDeviceProperties,
    ),
    (surface_format, image_count, present_mode, extent): (
        vk::Format,
        usize,
        vk::PresentModeKHR,
        vk::Extent2D,
    ),
) -> BackendDescription {
    // SAFETY: the driver fills the name with a terminated string.
    let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    BackendDescription {
        device_name,
        discrete: properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU,
        api_version: properties.api_version,
        vendor_id: properties.vendor_id,
        device_type: properties.device_type.as_raw(),
        driver_info: driver_info(instance, physical_device, properties.api_version),
        surface_format,
        image_count: image_count as u32,
        present_mode,
        extent,
    }
}

/// `VulkanPhysicalDevice.driverInfo`: the version the API reports in `major.minor.patch`, then
/// the driver's name and its info. Drivers older than Vulkan 1.2 have neither without an
/// extension, so those parts are empty.
fn driver_info(instance: &ash::Instance, device: vk::PhysicalDevice, api_version: u32) -> String {
    let (name, info) = if api_version >= vk::API_VERSION_1_2 {
        let mut driver = vk::PhysicalDeviceDriverProperties::default();
        let mut query = vk::PhysicalDeviceProperties2::default().push_next(&mut driver);
        // SAFETY: the device belongs to the instance and the chain holds one live structure.
        unsafe { instance.get_physical_device_properties2(device, &mut query) };
        let text = |chars: &[std::ffi::c_char]| {
            // SAFETY: the driver fills these with terminated strings.
            unsafe { CStr::from_ptr(chars.as_ptr()) }
                .to_string_lossy()
                .into_owned()
        };
        (text(&driver.driver_name), text(&driver.driver_info))
    } else {
        (String::new(), String::new())
    };
    format!(
        "{}.{}.{} {name} {info}",
        api_version >> 22 & 127,
        api_version >> 12 & 1023,
        api_version & 4095
    )
}
