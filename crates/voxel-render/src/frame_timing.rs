//! Where a frame's time goes inside the backend: the CPU's spans, and the
//! GPU's time in each pass of the scene.

use std::time::Duration;

/// The passes the GPU timestamps divide the scene into, in draw order. A pass
/// ends where the next begins; the sky's span includes beginning the
/// rendering and clearing the depth.
pub const GPU_PASSES: [&str; 8] = [
    "sky",
    "terrain",
    "entities",
    "overlays",
    "translucent",
    "clouds_weather",
    "hand",
    "gui",
];

/// The timestamps written for a scene: one before the first pass and one
/// after each.
pub(crate) const TIMESTAMP_COUNT: u32 = GPU_PASSES.len() as u32 + 1;

/// The CPU time one frame spent in the backend, by what it waited for or did.
#[derive(Debug, Clone, Copy, Default)]
pub struct BackendTimings {
    /// Waiting for the previous frame to finish, in every place the frame waited since the last
    /// one was submitted: the uploads each wait for it too.
    pub fence_wait: Duration,
    /// Waiting for a swapchain image, which a vsynced swapchain makes the wait for the display.
    pub acquire: Duration,
    /// Recording the command buffer.
    pub record: Duration,
    /// Submitting it and presenting the image.
    pub submit_present: Duration,
}

/// The GPU time of one finished frame's scene, in milliseconds.
#[derive(Debug, Clone, Copy)]
pub struct GpuTimings {
    /// Each pass of [`GPU_PASSES`].
    pub passes_ms: [f64; GPU_PASSES.len()],
    /// From the scene's first timestamp to its last.
    pub scene_ms: f64,
}

impl GpuTimings {
    /// From the raw timestamps: `valid_bits` of each are meaningful, and one
    /// tick lasts `period_ns` nanoseconds.
    pub(crate) fn from_ticks(ticks: &[u64], valid_bits: u32, period_ns: f64) -> Self {
        let mask = if valid_bits >= 64 {
            u64::MAX
        } else {
            (1_u64 << valid_bits) - 1
        };
        let span_ms = |from: u64, to: u64| (to.wrapping_sub(from) & mask) as f64 * period_ns / 1e6;
        let mut passes_ms = [0.0; GPU_PASSES.len()];
        for (pass, pair) in passes_ms.iter_mut().zip(ticks.windows(2)) {
            *pass = span_ms(pair[0], pair[1]);
        }
        Self {
            passes_ms,
            scene_ms: span_ms(ticks[0], ticks[ticks.len() - 1]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_are_the_gaps_between_timestamps() {
        let ticks = [100, 150, 150, 400, 410, 410, 500, 500, 1100];
        let timings = GpuTimings::from_ticks(&ticks, 64, 1_000_000.0);
        assert_eq!(
            timings.passes_ms,
            [50.0, 0.0, 250.0, 10.0, 0.0, 90.0, 0.0, 600.0]
        );
        assert_eq!(timings.scene_ms, 1000.0);
    }

    #[test]
    fn a_counter_that_wrapped_still_gives_the_gap() {
        // A 32-bit counter that wrapped between the two timestamps.
        let ticks = [0xFFFF_FFF0, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10];
        let timings = GpuTimings::from_ticks(&ticks, 32, 1_000_000.0);
        assert_eq!(timings.passes_ms[0], 32.0);
        assert_eq!(timings.scene_ms, 32.0);
    }
}
