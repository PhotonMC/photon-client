//! What the debug overlay's system entry reads of the machine (`DebugEntrySystemSpecs`).

use photon_gui::hud::debug::{DeviceType, MemoryUsage, SystemSpecs, vendor_name};
use photon_render::vulkan::BackendDescription;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

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

/// A line of a `/proc` file such as `VmRSS: 12345 kB`, in bytes.
fn proc_kibibytes(path: &str, key: &str) -> Option<i64> {
    let text = std::fs::read_to_string(path).ok()?;
    let rest = text.lines().find_map(|line| line.strip_prefix(key))?;
    let kibibytes: i64 = rest
        .trim_start_matches(':')
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(kibibytes * 1024)
}

/// What the memory entry shows: the process's resident memory is what is in use and allocated, the
/// machine's memory is the most there is, and the rate is the resident memory's change over the
/// last half second (`DebugEntryMemory.AllocationRateCalculator` without the collector count, which
/// there is none of).
#[derive(Default)]
pub struct MemorySampler {
    last: Option<(Instant, i64)>,
    rate: i64,
    usage: Option<MemoryUsage>,
}

/// `AllocationRateCalculator.UPDATE_INTERVAL_MS`.
const UPDATE_INTERVAL: Duration = Duration::from_millis(500);

impl MemorySampler {
    /// Takes a sample when half a second has passed since the last.
    pub fn update(&mut self, now: Instant) {
        if self.last.is_some_and(|(at, _)| now - at < UPDATE_INTERVAL) {
            return;
        }
        let (Some(used), Some(max)) = (
            proc_kibibytes("/proc/self/status", "VmRSS"),
            proc_kibibytes("/proc/meminfo", "MemTotal"),
        ) else {
            self.usage = None;
            return;
        };
        if let Some((at, before)) = self.last {
            let seconds = (now - at).as_secs_f64();
            self.rate = ((used - before) as f64 / seconds).round() as i64;
        }
        self.last = Some((now, used));
        self.usage = Some(MemoryUsage {
            used,
            total: used,
            max,
            rate: self.rate,
        });
    }

    pub fn usage(&self) -> Option<MemoryUsage> {
        self.usage
    }
}
