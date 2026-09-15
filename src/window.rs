//! Window framebuffer sizing: GUI scale selection and scaled-size rounding.
//!
//! This is the sizing half of the reference `Window`; it owns no GLFW handle and
//! performs no drawing. The two behaviours here decide every later GUI and HUD
//! coordinate, so they are pinned against the Java class before any surface
//! exists.

/// Smallest GUI area the reference keeps addressable at the chosen scale.
pub const BASE_WIDTH: i32 = 320;
/// Smallest GUI area the reference keeps addressable at the chosen scale.
pub const BASE_HEIGHT: i32 = 240;

/// Framebuffer size plus the GUI scale derived from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowScale {
    framebuffer_width: i32,
    framebuffer_height: i32,
    gui_scale: i32,
    gui_scaled_width: i32,
    gui_scaled_height: i32,
}

impl WindowScale {
    /// Start from a framebuffer size, before any scale has been chosen.
    pub fn new(framebuffer_width: i32, framebuffer_height: i32) -> Self {
        Self {
            framebuffer_width,
            framebuffer_height,
            gui_scale: 0,
            gui_scaled_width: 0,
            gui_scaled_height: 0,
        }
    }

    /// Java `Window.calculateScale`.
    ///
    /// The walk stops at the cap, at a scale the framebuffer can no longer
    /// exceed, or as soon as one more step would leave less than the base area
    /// on either axis. `max_scale` of zero never matches the counter, which is
    /// how the reference expresses "pick the largest that still fits".
    pub fn calculate_scale(&self, max_scale: i32, enforce_unicode: bool) -> i32 {
        let mut scale = 1;
        while scale != max_scale
            && scale < self.framebuffer_width
            && scale < self.framebuffer_height
            && self.framebuffer_width / (scale + 1) >= BASE_WIDTH
            && self.framebuffer_height / (scale + 1) >= BASE_HEIGHT
        {
            scale += 1;
        }
        if enforce_unicode && scale % 2 != 0 {
            scale += 1;
        }
        scale
    }

    /// Java `Window.setGuiScale`.
    ///
    /// The reference divides as a `double`, narrows toward zero, and adds one
    /// whenever the division left a remainder. The narrowing and the increment
    /// keep Java's semantics: the cast saturates and the add wraps, which only
    /// becomes observable for a zero scale.
    pub fn set_gui_scale(&mut self, gui_scale: i32) {
        self.gui_scale = gui_scale;
        let double_gui_scale = f64::from(gui_scale);
        self.gui_scaled_width = scaled_extent(self.framebuffer_width, double_gui_scale);
        self.gui_scaled_height = scaled_extent(self.framebuffer_height, double_gui_scale);
    }

    /// Framebuffer width in pixels.
    pub fn framebuffer_width(&self) -> i32 {
        self.framebuffer_width
    }

    /// Framebuffer height in pixels.
    pub fn framebuffer_height(&self) -> i32 {
        self.framebuffer_height
    }

    /// Replace the framebuffer size, leaving the scaled size for the next
    /// `set_gui_scale` call, as the reference does on a resize.
    pub fn set_framebuffer_size(&mut self, framebuffer_width: i32, framebuffer_height: i32) {
        self.framebuffer_width = framebuffer_width;
        self.framebuffer_height = framebuffer_height;
    }

    /// Currently applied GUI scale.
    pub fn gui_scale(&self) -> i32 {
        self.gui_scale
    }

    /// GUI width in scaled units.
    pub fn gui_scaled_width(&self) -> i32 {
        self.gui_scaled_width
    }

    /// GUI height in scaled units.
    pub fn gui_scaled_height(&self) -> i32 {
        self.gui_scaled_height
    }
}

/// One axis of `setGuiScale`, kept separate so both axes share the exact
/// division, narrowing and rounding the reference performs.
fn scaled_extent(framebuffer_extent: i32, double_gui_scale: f64) -> i32 {
    let scaled = f64::from(framebuffer_extent) / double_gui_scale;
    let truncated = scaled as i32;
    if scaled > f64::from(truncated) {
        truncated.wrapping_add(1)
    } else {
        truncated
    }
}
