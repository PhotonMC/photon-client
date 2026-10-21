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
        self.controls.debug.gpu.frame(
            self.backend.gpu_timings().map(|timings| timings.scene_ms),
            Instant::now(),
        );
        self.controls.debug.memory.update(Instant::now());
        self.controls.debug.sound_mood = self.session.player().map(|_| self.sound_mood());
        // `Minecraft.pauseIfInactive`.
        self.controls
            .pause_if_inactive(&mut self.surface, &self.screen_assets, Instant::now());
        // The machine does not change while the game runs, the framebuffer does.
        let described = self.backend.description();
        let system = self
            .controls
            .debug
            .system
            .get_or_insert_with(|| crate::system_info::system_specs(described));
        system.display = (
            described.extent.width as i32,
            described.extent.height as i32,
        );
    }

    /// `DebugEntrySoundMood`'s reading: the channels the sound engine has in use (none without a
    /// device, as the library has no pools then) and the biome handler's mood.
    fn sound_mood(&self) -> photon_gui::hud::debug::SoundMood {
        let channels = self
            .sound
            .as_ref()
            .map_or([(0, 0); 2], |sound| sound.channel_usage())
            .map(|(used, size)| (used as i32, size as i32));
        photon_gui::hud::debug::SoundMood {
            channels,
            mood: self.ambient.moodiness(),
        }
    }
}
