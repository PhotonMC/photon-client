//! What the debug overlay's system entry reads of the machine (`DebugEntrySystemSpecs`).

use photon_gui::hud::debug::{DeviceType, SystemSpecs, vendor_name};
use photon_render::vulkan::BackendDescription;
use std::sync::OnceLock;

/// `GLX._getCpuInfo`: the logical processors and the processor's name, blanks squeezed to one.
fn cpu_info() -> &'static str {
    static CPU: OnceLock<String> = OnceLock::new();
    CPU.get_or_init(|| {
        let name = std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|text| {
                text.lines()
                    .find_map(|line| line.strip_prefix("model name"))
                    .and_then(|rest| rest.split_once(':').map(|(_, name)| name.to_owned()))
            });
        let Some(name) = name else {
            return "<unknown>".to_owned();
        };
        let count = std::thread::available_parallelism().map_or(1, usize::from);
        format!("{count}x {name}")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    })
}

/// The entry's machine: this build's compiler, the processor, the framebuffer and the device the
/// backend chose.
pub fn system_specs(backend: &BackendDescription) -> SystemSpecs {
    SystemSpecs {
        runtime: ("Rust".to_owned(), env!("PHOTON_RUSTC_VERSION").to_owned()),
        cpu: cpu_info().to_owned(),
        display: (backend.extent.width as i32, backend.extent.height as i32),
        vendor: vendor_name(backend.vendor_id),
        device_name: backend.device_name.clone(),
        device_type: DeviceType::from_vulkan(backend.device_type),
        backend: "Vulkan".to_owned(),
        driver_info: backend.driver_info.clone(),
    }
}
