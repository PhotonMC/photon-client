//! `Minecraft.gpuUtilization`: how much of a frame the GPU worked, from the last few timed frames.

use std::time::Instant;

/// The timed frames the share averages (`TimerQuery.ROTATIONS`).
const ROTATIONS: usize = 3;

#[derive(Debug, Default)]
pub struct GpuMeter {
    /// The GPU time of the last frames, in nanoseconds, newest last.
    gpu_ns: [f64; ROTATIONS],
    frame_ns: f64,
    last_frame: Option<Instant>,
}

impl GpuMeter {
    /// A frame ended: the GPU's time in it, none where the device cannot time, and the wall time
    /// since the frame before (`savedCpuDuration`, which includes the frame limiter's wait).
    pub fn frame(&mut self, gpu_ms: Option<f64>, now: Instant) {
        if let Some(last) = self.last_frame {
            if let Some(gpu_ms) = gpu_ms {
                self.gpu_ns.rotate_left(1);
                self.gpu_ns[ROTATIONS - 1] = gpu_ms * 1e6;
            }
            self.frame_ns = now.duration_since(last).as_nanos() as f64;
        }
        self.last_frame = Some(now);
    }

    /// The percentage of the frame the GPU worked: zero before a frame was timed, and not a number
    /// before the first frame ended (nothing over nothing), which the entry shows as zero.
    pub fn utilization(&self) -> f64 {
        let average = (self.gpu_ns.iter().sum::<f64>() / ROTATIONS as f64).trunc();
        average * 100.0 / self.frame_ns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn the_share_is_the_average_of_three_frames_over_the_last_frame() {
        let mut meter = GpuMeter::default();
        let start = Instant::now();
        meter.frame(Some(1.0), start);
        for (n, ms) in [2.0, 4.0, 6.0].into_iter().enumerate() {
            meter.frame(Some(ms), start + Duration::from_millis(10 * (n as u64 + 1)));
        }
        // (2 + 4 + 6) / 3 = 4 ms of a 10 ms frame.
        assert_eq!(meter.utilization(), 40.0);
    }

    #[test]
    fn a_meter_that_has_timed_nothing_reads_zero_or_not_a_number() {
        let mut meter = GpuMeter::default();
        assert!(meter.utilization().is_nan());
        let start = Instant::now();
        meter.frame(None, start);
        meter.frame(None, start + Duration::from_millis(10));
        assert_eq!(meter.utilization(), 0.0);
    }
}
