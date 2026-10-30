//! The backend on the render thread (ADR 0003, "rendering one frame behind"): the frame is built
//! on this thread, and recording, submitting and presenting it run on the render thread while the
//! next frame's ticks and building go on. Any later touch of the backend waits for it.

use std::time::Instant;

use photon_render::{Handoff, VulkanBackend, VulkanError};

use super::*;
use crate::settings::{RenderMode, Settings};

/// The backend as `render.pipelined` asks for it: with a render thread, or without.
pub(super) fn handoff(backend: VulkanBackend) -> Handoff<VulkanBackend> {
    match Settings::get().render {
        RenderMode::Pipelined => Handoff::threaded(backend),
        RenderMode::Serial => Handoff::serial(backend),
    }
}

impl Play {
    /// Draws the scene on the render thread (inline in serial mode). The error of the frame
    /// before comes back here, a frame late.
    pub(super) fn submit_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<()> {
        if let Some(error) = self.backend.take_error::<VulkanError>() {
            return Err(error.into());
        }
        self.flush_bench();
        let presented = self.bench.as_ref().map(Bench::presented);
        self.backend.submit_result(move |backend| {
            let drawn = backend.draw_scene(colour, view_projection);
            if let Some(presented) = presented {
                *presented.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
            }
            drawn
        });
        Ok(())
    }

    /// Records the frame that was submitted last, now that its backend timings are final.
    pub(super) fn flush_bench(&mut self) {
        if let Some(bench) = self.bench.as_mut() {
            bench.flush(&self.backend);
        }
    }
}
