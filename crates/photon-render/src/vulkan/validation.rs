//! The Khronos validation layer, asked for with `PHOTON_VALIDATION=1`: what it finds is logged and
//! counted, so a run can say whether the backend used Vulkan correctly.

use std::ffi::{CStr, c_void};
use std::sync::atomic::{AtomicU32, Ordering};

use super::*;

/// The layer's name.
const LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

static ERRORS: AtomicU32 = AtomicU32::new(0);
static WARNINGS: AtomicU32 = AtomicU32::new(0);

/// How many errors and warnings the validation layer has reported so far.
pub fn validation_findings() -> (u32, u32) {
    (
        ERRORS.load(Ordering::Relaxed),
        WARNINGS.load(Ordering::Relaxed),
    )
}

/// Whether `PHOTON_VALIDATION` asks for the layer.
pub(super) fn requested() -> bool {
    std::env::var_os("PHOTON_VALIDATION").is_some_and(|value| value != "0")
}

/// The layer and extension names to enable, if both exist; otherwise a note that they do not.
pub(super) fn names(entry: &ash::Entry) -> Option<(*const i8, *const i8)> {
    // SAFETY: the loader is live; the properties are read, and each name is a NUL-terminated array
    // the loader fills in.
    let (layers, extensions) = unsafe {
        (
            entry.enumerate_instance_layer_properties().ok()?,
            entry.enumerate_instance_extension_properties(None).ok()?,
        )
    };
    // SAFETY: as above.
    let has_layer = layers
        .iter()
        .any(|layer| unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) } == LAYER);
    // SAFETY: as above.
    let has_extension = extensions.iter().any(|extension| {
        // SAFETY: as above.
        let name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };
        name == ash::ext::debug_utils::NAME
    });
    if has_layer && has_extension {
        Some((LAYER.as_ptr(), ash::ext::debug_utils::NAME.as_ptr()))
    } else {
        log::warn!("PHOTON_VALIDATION is set but the validation layer is not installed");
        None
    }
}

/// The messenger that carries the layer's findings to the log.
pub(super) struct Validation {
    loader: ash::ext::debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl Validation {
    /// Start listening.
    ///
    /// # Safety
    ///
    /// `instance` must be live and made with the layer's extension enabled.
    pub(super) unsafe fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<Self, VulkanError> {
        let loader = ash::ext::debug_utils::Instance::new(entry, instance);
        let info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(report));
        // SAFETY: the instance is live and has the extension (see `# Safety`); `report` is a
        // function that outlives the messenger.
        let messenger = unsafe { loader.create_debug_utils_messenger(&info, None) }
            .map_err(|result| VulkanError::Call("vkCreateDebugUtilsMessengerEXT", result))?;
        Ok(Self { loader, messenger })
    }

    /// Stop listening.
    ///
    /// # Safety
    ///
    /// The instance the messenger was made on must still be live.
    pub(super) unsafe fn destroy(self) {
        // SAFETY: the messenger is owned by `self`, consumed here, so it is destroyed once.
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None);
        };
    }
}

/// The layer's callback: log the message and count it.
unsafe extern "system" fn report(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _kind: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user: *mut c_void,
) -> vk::Bool32 {
    // SAFETY: the layer passes a valid callback structure for the duration of the call.
    let message = unsafe { data.as_ref() }
        .and_then(|data| unsafe { data.message_as_c_str() })
        .map_or_else(|| "(no message)".into(), CStr::to_string_lossy);
    if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
        ERRORS.fetch_add(1, Ordering::Relaxed);
        log::error!("vulkan validation: {message}");
    } else {
        WARNINGS.fetch_add(1, Ordering::Relaxed);
        log::warn!("vulkan validation: {message}");
    }
    vk::FALSE
}
