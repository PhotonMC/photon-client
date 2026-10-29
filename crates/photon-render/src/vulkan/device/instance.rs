use super::*;

/// What the instance stage makes: the loader, the instance, the surface of the window and the
/// validation messenger when it is asked for.
pub(super) struct InstanceParts {
    pub(super) entry: ash::Entry,
    pub(super) instance: ash::Instance,
    pub(super) surface_instance: ash::khr::surface::Instance,
    pub(super) surface: vk::SurfaceKHR,
    pub(super) validation: Option<validation::Validation>,
}

/// Load Vulkan, make an instance with the extensions the window needs, and the surface.
///
/// # Safety
///
/// `display_handle` and `window_handle` must refer to a window that outlives the parts; the
/// surface borrows them.
pub(super) unsafe fn create_instance(
    display_handle: raw_window_handle::RawDisplayHandle,
    window_handle: raw_window_handle::RawWindowHandle,
) -> Result<InstanceParts, VulkanError> {
    // SAFETY: the caller keeps the window alive (see `# Safety`); each handle is made from
    // handles made before it, and the caller's backend owns them all afterwards.
    unsafe {
        let entry = ash::Entry::load().map_err(|error| VulkanError::Loader(error.to_string()))?;

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

        let mut layers: Vec<*const std::ffi::c_char> = Vec::new();
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
        Ok(InstanceParts {
            entry,
            instance,
            surface_instance,
            surface,
            validation,
        })
    }
}
