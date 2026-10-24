//! `PHOTON_PROFILE`: where the frame time went, printed every two seconds,
//! with the slowest frame and the 99th percentile beside the mean so a spike
//! shows instead of averaging away.

use std::cell::Cell;
use std::collections::HashMap;
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

/// Frames a dropped section has to come back within to count as a blink.
const BLINK_FRAMES: u64 = 60;

thread_local! {
    static BLINKS: Cell<u32> = const { Cell::new(0) };
}

/// Blinks since the last call: sections that left the drawn set and were
/// back within [`BLINK_FRAMES`] frames, the flicker the eye sees.
pub fn take_blinks() -> u32 {
    BLINKS.with(|blinks| blinks.replace(0))
}

/// Watches the occlusion flood's results for sections that blink, when a
/// bench run asks (`PHOTON_BENCH`): a section in view that one result
/// drops and a later one brings back.
#[derive(Default)]
pub struct BlinkWatch {
    frame: u64,
    dropped: HashMap<(i32, i32, i32), u64>,
}

impl BlinkWatch {
    /// A watch if a bench run is on.
    pub fn from_env() -> Option<Self> {
        std::env::var_os("PHOTON_BENCH")
            .is_some()
            .then(Self::default)
    }

    /// A frame passes.
    pub fn frame(&mut self) {
        self.frame += 1;
    }

    /// The flood's result `old` was replaced by `new`; `in_view` says
    /// whether a section is in the frustum.
    pub fn swapped(
        &mut self,
        old: &[(i32, i32, i32)],
        new: &[(i32, i32, i32)],
        in_view: impl Fn(&(i32, i32, i32)) -> bool,
    ) {
        let now: std::collections::HashSet<_> = new.iter().copied().collect();
        let frame = self.frame;
        let back = new
            .iter()
            .filter(|key| {
                self.dropped
                    .remove(key)
                    .is_some_and(|at| frame - at <= BLINK_FRAMES)
            })
            .count() as u32;
        for key in old.iter().filter(|key| !now.contains(key) && in_view(key)) {
            self.dropped.insert(*key, frame);
        }
        self.dropped.retain(|_, at| frame - *at <= BLINK_FRAMES);
        BLINKS.with(|blinks| blinks.set(blinks.get() + back));
    }
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
