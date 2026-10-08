//! `PHOTON_PROFILE`: where the frame time went, printed every two seconds,
//! with the slowest frame and the 99th percentile beside the mean so a spike
//! shows instead of averaging away.

use std::cell::Cell;
use std::fmt::Write as _;
use std::time::{Duration, Instant};

use crate::world::sections::SectionMeshes;

/// The parts of a frame timed inside its coarse spans.
#[derive(Clone, Copy)]
pub enum Span {
    /// Applying the packets the reader queued.
    Drain,
    /// The queued light and the light engine.
    Light,
    /// Copying what the occlusion flood reads for its thread.
    Graph,
    /// Copying regions for the mesh workers.
    HandOut,
    /// Uploading the meshes the workers finished.
    ShowBuilt,
    /// Entities, particles and the other overlays.
    Overlays,
}

/// The names of [`Span`]'s parts, in its order.
pub const SPANS: [&str; 6] = [
    "drain",
    "light",
    "graph",
    "hand_out",
    "show_built",
    "overlays",
];

thread_local! {
    static PARTS: Cell<[Duration; SPANS.len()]> = const { Cell::new([Duration::ZERO; SPANS.len()]) };
}

/// Run `work`, adding how long it took to this frame's `span`.
pub fn timed<T>(span: Span, work: impl FnOnce() -> T) -> T {
    let started = Instant::now();
    let result = work();
    let took = started.elapsed();
    PARTS.with(|parts| {
        let mut sums = parts.get();
        sums[span as usize] += took;
        parts.set(sums);
    });
    result
}

/// This frame's parts, in milliseconds, reset for the next frame.
pub fn take_parts() -> [f64; SPANS.len()] {
    PARTS
        .with(|parts| parts.replace([Duration::ZERO; SPANS.len()]))
        .map(|took| took.as_secs_f64() * 1000.0)
}

/// The names of the coarse spans, then the parts.
const COLUMNS: [&str; 5 + SPANS.len()] = [
    "tick",
    "mesh",
    "build",
    "draw",
    "total",
    "drain",
    "light",
    "graph",
    "hand_out",
    "show_built",
    "overlays",
];

/// The frames since the last print, one row of [`COLUMNS`] each, in
/// milliseconds.
pub struct FrameProfile {
    frames: Vec<[f64; COLUMNS.len()]>,
    started: Instant,
    /// The median frame of the last print: a frame twice as long is logged.
    median: Option<f64>,
}

impl FrameProfile {
    /// A profile if `PHOTON_PROFILE` asks for one.
    pub fn from_env() -> Option<Self> {
        std::env::var_os("PHOTON_PROFILE").is_some().then(|| Self {
            frames: Vec::new(),
            started: Instant::now(),
            median: None,
        })
    }

    /// One frame, by when it started, ticked, meshed, built and drew, and
    /// its parts.
    pub fn record(
        &mut self,
        [start, ticked, meshed, built, drawn]: [Instant; 5],
        parts: [f64; SPANS.len()],
        sections: &mut SectionMeshes,
    ) {
        let ms = |from: Instant, to: Instant| (to - from).as_secs_f64() * 1000.0;
        let mut row = [0.0; COLUMNS.len()];
        row[..5].copy_from_slice(&[
            ms(start, ticked),
            ms(ticked, meshed),
            ms(meshed, built),
            ms(built, drawn),
            ms(start, drawn),
        ]);
        row[5..].copy_from_slice(&parts);
        if self.median.is_some_and(|median| row[4] > median * 2.0) {
            log::debug!(
                "spike {}",
                columns_text(&row, |value| format!("{value:.2}"))
            );
        }
        self.frames.push(row);
        if self.started.elapsed() < Duration::from_secs(2) {
            return;
        }
        let count = self.frames.len() as f64;
        let column = |index: usize| {
            let mut values: Vec<f64> = self.frames.iter().map(|row| row[index]).collect();
            values.sort_by(f64::total_cmp);
            let p99 =
                values[((values.len() as f64 * 0.99).ceil() as usize).clamp(1, values.len()) - 1];
            (
                values.iter().sum::<f64>() / count,
                p99,
                values[values.len() - 1],
                values[values.len() / 2],
            )
        };
        let stats: Vec<(f64, f64, f64, f64)> = (0..COLUMNS.len()).map(column).collect();
        println!(
            "profile fps={:.0} ms/frame (mean/p99/max): {} pending={}",
            count / self.started.elapsed().as_secs_f64(),
            columns_text(&stats, |(mean, p99, max, _)| format!(
                "{mean:.2}/{p99:.2}/{max:.2}"
            )),
            sections.pending(),
        );
        sections.report();
        self.median = Some(stats[4].3);
        self.frames.clear();
        self.started = Instant::now();
    }
}

/// `name=value` for each column.
fn columns_text<T>(row: &[T], value: impl Fn(&T) -> String) -> String {
    let mut text = String::new();
    for (name, cell) in COLUMNS.iter().zip(row) {
        let separator = if text.is_empty() { "" } else { " " };
        let _ = write!(text, "{separator}{name}={}", value(cell));
    }
    text
}
