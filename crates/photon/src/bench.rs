//! `PHOTON_BENCH=path`: frame timings kept for a whole scripted run and written
//! as a JSON report when it ends, so two runs can be compared by their
//! percentiles instead of by eye. `python3 tools/photon.py bench` drives it.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use photon_render::{BackendTimings, GPU_PASSES, GpuTimings, VulkanBackend};

use crate::profile;

/// How long after the first frame samples are left out, so the sections the
/// world is still meshing do not count.
const DEFAULT_WARMUP: Duration = Duration::from_secs(10);

/// The backend's spans, in the order of [`BACKEND_SPANS`].
const BACKEND_SPANS: [&str; 4] = ["fence_wait", "acquire", "record", "submit_present"];

fn backend_spans(timings: &BackendTimings) -> [Duration; 4] {
    [
        timings.fence_wait,
        timings.acquire,
        timings.record,
        timings.submit_present,
    ]
}

/// One frame, in milliseconds.
struct Sample {
    /// Ticking, meshing, building the frame, drawing it, and the whole frame.
    cpu: [f64; 5],
    /// The parts of [`profile::SPANS`].
    parts: [f64; profile::SPANS.len()],
    backend: BackendTimings,
    gpu: Option<GpuTimings>,
}

/// The facts of a run the report names beside the numbers.
pub struct RunInfo {
    pub device: String,
    pub extent: (u32, u32),
    pub render_distance: i32,
    pub loaded_chunks: usize,
    pub entities: usize,
    /// What the validation layer reported (errors, warnings), when it was on.
    pub validation: (u32, u32),
    /// How the swapchain presents, and the frame rate limit the run held to.
    pub present_mode: String,
    pub memory: photon_render::SectionMemory,
    pub framerate_limit: i32,
}

pub struct Bench {
    path: String,
    warmup: Duration,
    first_frame: Option<Instant>,
    /// When the last measured frame ended, for the frames per second of the wall clock.
    last_frame: Option<Instant>,
    samples: Vec<Sample>,
}

impl Bench {
    /// A recorder if `PHOTON_BENCH` names a report file; `PHOTON_BENCH_WARMUP` is
    /// the seconds to leave out at the start.
    pub fn from_env() -> Option<Self> {
        let path = std::env::var("PHOTON_BENCH").ok()?;
        let warmup = std::env::var("PHOTON_BENCH_WARMUP")
            .ok()
            .and_then(|seconds| seconds.parse().ok())
            .map_or(DEFAULT_WARMUP, Duration::from_secs_f64);
        Some(Self {
            path,
            warmup,
            first_frame: None,
            last_frame: None,
            samples: Vec::new(),
        })
    }

    /// One drawn frame, by when it started, ticked, meshed, built and drew.
    pub fn record(
        &mut self,
        [start, ticked, meshed, built, drawn]: [Instant; 5],
        parts: [f64; profile::SPANS.len()],
        backend: &VulkanBackend,
    ) {
        let first = *self.first_frame.get_or_insert(start);
        if start - first < self.warmup {
            return;
        }
        let ms = |from: Instant, to: Instant| (to - from).as_secs_f64() * 1000.0;
        self.last_frame = Some(drawn);
        self.samples.push(Sample {
            cpu: [
                ms(start, ticked),
                ms(ticked, meshed),
                ms(meshed, built),
                ms(built, drawn),
                ms(start, drawn),
            ],
            parts,
            backend: backend.timings(),
            gpu: backend.gpu_timings(),
        });
    }

    /// Write the report. A run with no sample past the warm-up has nothing to report.
    pub fn write(&self, info: &RunInfo) -> Result<(), String> {
        if self.samples.is_empty() {
            return Err("no frame was drawn after the warm-up; make the script longer".to_owned());
        }
        std::fs::write(&self.path, self.report(info))
            .map_err(|error| format!("{}: {error}", self.path))
    }

    fn report(&self, info: &RunInfo) -> String {
        let column =
            |pick: &dyn Fn(&Sample) -> f64| Stats::of(self.samples.iter().map(pick).collect());
        let frames = self.samples.len();
        let total = column(&|sample| sample.cpu[4]);
        // The frame without the wait for a swapchain image, which a vsynced
        // swapchain ends on the display's schedule: what the work cost.
        let work = column(&|sample| sample.cpu[4] - sample.backend.acquire.as_secs_f64() * 1000.0);
        let mut out = String::from("{\n  \"schema\": 1,\n");
        let _ = writeln!(out, "  \"device\": {:?},", info.device);
        let _ = writeln!(out, "  \"extent\": [{}, {}],", info.extent.0, info.extent.1);
        let _ = writeln!(out, "  \"render_distance\": {},", info.render_distance);
        let memory = &info.memory;
        let _ = writeln!(
            out,
            "  \"section_memory\": {{\"sections\": {}, \"vertex_bytes\": {}, \"index_bytes\": {}, \"arena_bytes\": {}}},",
            memory.sections, memory.vertex_bytes, memory.index_bytes, memory.arena_bytes
        );
        let _ = writeln!(out, "  \"present_mode\": {:?},", info.present_mode);
        let _ = writeln!(out, "  \"framerate_limit\": {},", info.framerate_limit);
        let _ = writeln!(out, "  \"loaded_chunks\": {},", info.loaded_chunks);
        let _ = writeln!(out, "  \"entities\": {},", info.entities);
        let _ = writeln!(
            out,
            "  \"validation\": {{\"errors\": {}, \"warnings\": {}}},",
            info.validation.0, info.validation.1
        );
        let _ = writeln!(out, "  \"warmup_s\": {},", self.warmup.as_secs_f64());
        let _ = writeln!(out, "  \"frames\": {frames},");
        // Frames per second of the wall clock, which a vsynced swapchain or a limit holds to the
        // display's rate or its own; the frame's work is what to compare otherwise.
        let measured = self
            .first_frame
            .zip(self.last_frame)
            .map_or(0.0, |(first, last)| {
                (last - first).as_secs_f64() - self.warmup.as_secs_f64()
            });
        let _ = writeln!(
            out,
            "  \"wall_fps\": {:.1},",
            frames as f64 / measured.max(1e-9)
        );
        let _ = writeln!(out, "  \"work_fps\": {:.1},", 1000.0 / work.mean);
        let _ = writeln!(out, "  \"work_ms\": {},", work.json());
        let _ = writeln!(out, "  \"frame_ms\": {},", total.json());
        out.push_str("  \"cpu_ms\": {\n");
        let cpu = ["tick", "mesh", "extract", "backend"];
        for (index, name) in cpu.iter().enumerate() {
            let _ = writeln!(
                out,
                "    \"{name}\": {},",
                column(&|sample| sample.cpu[index]).json()
            );
        }
        for (index, name) in BACKEND_SPANS.iter().enumerate() {
            let comma = if index + 1 < BACKEND_SPANS.len() {
                ","
            } else {
                ""
            };
            let stats =
                column(&|sample| backend_spans(&sample.backend)[index].as_secs_f64() * 1000.0);
            let _ = writeln!(out, "    \"{name}\": {}{comma}", stats.json());
        }
        out.push_str("  },\n");
        // The parts inside the spans above, so a spike can be traced to one.
        out.push_str("  \"parts_ms\": {\n");
        for (index, name) in profile::SPANS.iter().enumerate() {
            let comma = if index + 1 < profile::SPANS.len() {
                ","
            } else {
                ""
            };
            let stats = column(&|sample| sample.parts[index]);
            let _ = writeln!(out, "    \"{name}\": {}{comma}", stats.json());
        }
        out.push_str("  },\n");
        let gpu: Vec<&GpuTimings> = self
            .samples
            .iter()
            .filter_map(|sample| sample.gpu.as_ref())
            .collect();
        if gpu.is_empty() {
            out.push_str("  \"gpu_ms\": null\n}\n");
            return out;
        }
        out.push_str("  \"gpu_ms\": {\n");
        let scene = Stats::of(gpu.iter().map(|timings| timings.scene_ms).collect());
        let _ = writeln!(out, "    \"scene\": {},", scene.json());
        let passes: Vec<Stats> = (0..GPU_PASSES.len())
            .map(|index| Stats::of(gpu.iter().map(|timings| timings.passes_ms[index]).collect()))
            .collect();
        // Some drivers (MoltenVK on Apple GPUs) write every timestamp inside a
        // render pass at its boundary, which leaves one pass with all of the
        // time. The passes are only a breakdown when the time is spread.
        let spread = passes
            .iter()
            .filter(|pass| pass.mean >= scene.mean * 0.01)
            .count()
            >= 2;
        let _ = writeln!(out, "    \"pass_breakdown_resolved\": {spread},");
        for (index, (name, stats)) in GPU_PASSES.iter().zip(&passes).enumerate() {
            let comma = if index + 1 < passes.len() { "," } else { "" };
            let _ = writeln!(out, "    \"{name}\": {}{comma}", stats.json());
        }
        out.push_str("  }\n}\n");
        out
    }
}

/// The mean and the percentiles of a column of samples.
#[derive(Debug, PartialEq)]
struct Stats {
    mean: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    max: f64,
}

impl Stats {
    /// Percentiles by nearest rank: the smallest sample at least that
    /// fraction of the samples are not above.
    fn of(mut samples: Vec<f64>) -> Self {
        samples.sort_by(f64::total_cmp);
        let rank = |fraction: f64| {
            let index = (fraction * samples.len() as f64).ceil() as usize;
            samples[index.clamp(1, samples.len()) - 1]
        };
        Self {
            mean: samples.iter().sum::<f64>() / samples.len() as f64,
            p50: rank(0.50),
            p95: rank(0.95),
            p99: rank(0.99),
            max: samples[samples.len() - 1],
        }
    }

    fn json(&self) -> String {
        format!(
            "{{\"mean\": {:.3}, \"p50\": {:.3}, \"p95\": {:.3}, \"p99\": {:.3}, \"max\": {:.3}}}",
            self.mean, self.p50, self.p95, self.p99, self.max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_are_nearest_rank() {
        let stats = Stats::of((1..=100).rev().map(f64::from).collect());
        assert_eq!(
            stats,
            Stats {
                mean: 50.5,
                p50: 50.0,
                p95: 95.0,
                p99: 99.0,
                max: 100.0
            }
        );
    }

    #[test]
    fn one_sample_is_every_percentile() {
        let stats = Stats::of(vec![7.0]);
        assert_eq!(
            (stats.p50, stats.p95, stats.p99, stats.max),
            (7.0, 7.0, 7.0, 7.0)
        );
    }
}
