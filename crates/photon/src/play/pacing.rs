//! The frame's pacing: the limiter and the frame rate the debug overlay shows.

use super::*;

impl Play {
    /// The frame limiter holds the frame for what the options ask.
    pub(super) fn limit_frame(&mut self) {
        let options = &self.controls.options;
        let limit = self.limiter.limit(
            options.framerate_limit,
            options.inactivity_fps_limit,
            self.surface.is_iconified(),
            Instant::now(),
        );
        self.limiter.wait(limit);
        // `Minecraft.runTick`: after the limiter, the frame is counted.
        self.limiter.count_frame(Instant::now());
        self.controls.debug.frame = photon_gui::hud::debug::FrameStats {
            fps: self.limiter.fps(),
            limit,
            present_mode: photon_gui::hud::debug::PresentMode::from_vulkan(
                self.backend.description().present_mode.as_raw(),
            ),
        };
        self.controls.debug.system =
            Some(crate::system_info::system_specs(self.backend.description()));
    }
}
