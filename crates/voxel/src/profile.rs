//! `VOXEL_PROFILE`: where the frame time went, printed every two seconds.

use std::time::{Duration, Instant};

use crate::world::sections::SectionMeshes;

/// The frame's spans summed since the last print: ticking, meshing,
/// building the frame, drawing it, and the whole frame.
pub struct FrameProfile {
    spans: [Duration; 5],
    frames: u32,
    started: Instant,
}

impl FrameProfile {
    /// A profile if `VOXEL_PROFILE` asks for one.
    pub fn from_env() -> Option<Self> {
        std::env::var_os("VOXEL_PROFILE").is_some().then(|| Self {
            spans: [Duration::ZERO; 5],
            frames: 0,
            started: Instant::now(),
        })
    }

    /// One frame, by when it started, ticked, meshed, built and drew.
    pub fn record(
        &mut self,
        [start, ticked, meshed, built, drawn]: [Instant; 5],
        sections: &mut SectionMeshes,
    ) {
        for (span, (from, to)) in self.spans.iter_mut().zip([
            (start, ticked),
            (ticked, meshed),
            (meshed, built),
            (built, drawn),
            (start, drawn),
        ]) {
            *span += to - from;
        }
        self.frames += 1;
        if self.started.elapsed() < Duration::from_secs(2) {
            return;
        }
        let per_frame = |span: Duration| span.as_secs_f64() * 1000.0 / f64::from(self.frames);
        println!(
            "profile fps={:.0} ms/frame: tick={:.2} mesh={:.2} build={:.2} draw={:.2} total={:.2} pending={}",
            f64::from(self.frames) / self.started.elapsed().as_secs_f64(),
            per_frame(self.spans[0]),
            per_frame(self.spans[1]),
            per_frame(self.spans[2]),
            per_frame(self.spans[3]),
            per_frame(self.spans[4]),
            sections.pending(),
        );
        sections.report();
        self.spans = [Duration::ZERO; 5];
        self.frames = 0;
        self.started = Instant::now();
    }
}
