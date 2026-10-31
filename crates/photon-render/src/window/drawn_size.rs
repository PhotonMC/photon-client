//! The size a frame is drawn at, where the window cannot change the monitor's mode.
//!
//! A Wayland compositor lists one mode per monitor and does no mode switching, so a fullscreen
//! resolution the player picks is drawn by the client and blitted up to the window. The GUI scale
//! and the layout follow the drawn size, as they follow a real mode's.

use super::scale::framebuffer_extent;
use super::surface::WindowSurface;
use super::video_mode::VideoMode;

impl WindowSurface {
    /// The framebuffer the window really has, in pixels.
    pub fn native_framebuffer(&self) -> (i32, i32) {
        self.framebuffer.0
    }

    /// The size frames are drawn at when it is not the window's: a fullscreen resolution the
    /// platform cannot switch to.
    pub fn drawn_framebuffer(&self) -> Option<(i32, i32)> {
        self.framebuffer.1
    }

    /// Whether the platform leaves the monitor's mode alone (Wayland).
    pub(super) fn keeps_monitor_mode(glfw: &glfw::Glfw) -> bool {
        glfw.get_platform() == glfw::Platform::Wayland
    }

    /// The preferred mode, drawn by the client where fullscreen cannot switch to it, and the GUI
    /// scale from the size that results.
    pub(super) fn refresh_drawn_size(&mut self) {
        let native = self.framebuffer.0;
        let listed = |mode: &VideoMode| {
            self.monitor_modes
                .as_ref()
                .is_some_and(|modes| modes.contains(mode))
        };
        let drawn = self
            .preferred_mode
            .filter(|_| Self::keeps_monitor_mode(&self.glfw))
            .filter(listed)
            .and_then(|mode| self.scaled_framebuffer(mode, native));
        self.framebuffer.1 = drawn;
        let (width, height) = drawn.unwrap_or(native);
        self.scale
            .set_framebuffer_size(framebuffer_extent(width), framebuffer_extent(height));
        let chosen = self
            .scale
            .calculate_scale(self.gui_scale_option, self.enforce_unicode);
        self.scale.set_gui_scale(chosen);
    }
}

impl WindowSurface {
    /// The window's framebuffer cut down by the share `mode` is of the monitor's own mode, so a
    /// tiled window that cannot change the mode still renders fewer pixels. None where it would
    /// not be fewer.
    fn scaled_framebuffer(&self, mode: VideoMode, native: (i32, i32)) -> Option<(i32, i32)> {
        let monitor = self
            .monitor_modes
            .as_ref()?
            .iter()
            .map(|m| m.height)
            .max()?;
        if mode.height >= monitor || monitor <= 0 {
            return None;
        }
        let share = f64::from(mode.height) / f64::from(monitor);
        let cut = |side: i32| ((f64::from(side) * share).round() as i32).max(1);
        Some((cut(native.0), cut(native.1))).filter(|&drawn| drawn != native)
    }
}

/// Whether two sizes are the same shape within two percent, so that a frame drawn at one fills
/// the other without bars or stretching.
pub(super) fn same_shape(
    (width, height): (i32, i32),
    (other_width, other_height): (i32, i32),
) -> bool {
    let (a, b) = (
        i64::from(width) * i64::from(other_height),
        i64::from(height) * i64::from(other_width),
    );
    (a - b).abs() * 50 <= a
}

#[cfg(test)]
mod tests {
    use super::same_shape;

    #[test]
    fn sizes_of_one_shape_fit_each_other() {
        assert!(same_shape((1280, 720), (1920, 1080)));
        assert!(same_shape((854, 480), (1920, 1080)));
        assert!(!same_shape((1680, 1050), (1920, 1080)));
        assert!(!same_shape((1024, 768), (1920, 1080)));
    }
}
