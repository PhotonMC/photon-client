//! GLFW surface creation mirroring the reference `Window` constructor.
//!
//! The reference creates the window hidden, sizes it from `DisplayData` clamped
//! to at least one pixel, centres it on the primary monitor's preferred video
//! mode, then reads the framebuffer size back and derives the GUI scale from it.
//! This module reproduces that order. Client-API hints follow the selected
//! backend: the Vulkan path the project targets asks GLFW for no client API,
//! because the surface is created through Vulkan rather than by GLFW.

use std::fmt;

use glfw::{WindowHint, WindowMode};

use super::scale::{WindowScale, allowed_window_min_size, framebuffer_extent};

/// Reference default window size, as shipped in the client's argument parser.
pub const DEFAULT_WIDTH: i32 = 854;
/// Reference default window size, as shipped in the client's argument parser.
pub const DEFAULT_HEIGHT: i32 = 480;

/// The reference `DisplayData` record, limited to the fields this slice uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayData {
    /// Requested windowed width before the minimum-size clamp.
    pub width: i32,
    /// Requested windowed height before the minimum-size clamp.
    pub height: i32,
    /// Whether the window should start fullscreen.
    pub fullscreen: bool,
}

impl Default for DisplayData {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            fullscreen: false,
        }
    }
}

/// Everything that can stop the surface from coming up.
#[derive(Debug)]
pub enum WindowSurfaceError {
    /// GLFW itself refused to initialise.
    Init(glfw::InitError),
    /// GLFW initialised but would not create the window.
    Create,
}

impl fmt::Display for WindowSurfaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init(error) => write!(formatter, "GLFW initialisation failed: {error}"),
            Self::Create => formatter.write_str("GLFW window creation failed"),
        }
    }
}

impl std::error::Error for WindowSurfaceError {}

/// A live GLFW window plus the sizing state derived from its framebuffer.
pub struct WindowSurface {
    glfw: glfw::Glfw,
    window: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    scale: WindowScale,
    /// Where and how big the window was before it went fullscreen.
    windowed: Option<(i32, i32, i32, i32)>,
}

impl WindowSurface {
    /// Create the window in the reference's order: hidden, clamped size,
    /// centred on the primary monitor, then framebuffer read back.
    pub fn create(display: DisplayData, title: &str) -> Result<Self, WindowSurfaceError> {
        let mut glfw = glfw::init_no_callbacks().map_err(WindowSurfaceError::Init)?;

        // The Vulkan backend asks GLFW for no client API; the reference also
        // creates every window hidden and shows it once sizing is settled.
        glfw.window_hint(WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(WindowHint::Visible(false));

        let width = allowed_window_min_size(display.width);
        let height = allowed_window_min_size(display.height);

        let (mut window, events) = glfw
            .create_window(width as u32, height as u32, title, WindowMode::Windowed)
            .ok_or(WindowSurfaceError::Create)?;

        // Centre on the primary monitor's current video mode, as the reference
        // does from the monitor origin and mode size.
        let centre = glfw.with_primary_monitor(|_, monitor| {
            let monitor = monitor?;
            let mode = monitor.get_video_mode()?;
            let (monitor_x, monitor_y) = monitor.get_pos();
            Some((
                monitor_x + mode.width as i32 / 2 - width / 2,
                monitor_y + mode.height as i32 / 2 - height / 2,
            ))
        });
        if let Some((x, y)) = centre {
            window.set_pos(x, y);
        }

        window.set_framebuffer_size_polling(true);
        window.set_size_polling(true);
        window.set_focus_polling(true);
        window.set_key_polling(true);
        window.set_close_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_scroll_polling(true);

        let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
        let mut scale = WindowScale::new(
            framebuffer_extent(framebuffer_width),
            framebuffer_extent(framebuffer_height),
        );
        scale.set_gui_scale(scale.calculate_scale(0, false));

        window.show();
        Ok(Self {
            glfw,
            window,
            events,
            scale,
            windowed: None,
        })
    }

    /// Current sizing state, refreshed on every framebuffer resize.
    pub fn scale(&self) -> WindowScale {
        self.scale
    }

    /// Whether the user asked the window to close.
    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    /// Ask the window to close on the next poll.
    pub fn request_close(&mut self) {
        self.window.set_should_close(true);
    }

    /// Drain pending events, re-deriving the GUI scale after a resize.
    ///
    /// Returns the events seen this pass so callers can log or act on them
    /// while the input layer is still being built.
    pub fn poll(&mut self) -> Vec<glfw::WindowEvent> {
        self.glfw.poll_events();
        let mut seen = Vec::new();
        for (_, event) in glfw::flush_messages(&self.events) {
            if let glfw::WindowEvent::FramebufferSize(width, height) = event {
                self.scale
                    .set_framebuffer_size(framebuffer_extent(width), framebuffer_extent(height));
                let chosen = self.scale.calculate_scale(0, false);
                self.scale.set_gui_scale(chosen);
            }
            seen.push(event);
        }
        seen
    }

    /// Switch between a window and fullscreen on the primary monitor at its
    /// current video mode, as the reference's fullscreen key does, and back
    /// to the window's old place and size.
    pub fn toggle_fullscreen(&mut self) {
        if let Some((x, y, width, height)) = self.windowed.take() {
            self.window.set_monitor(
                WindowMode::Windowed,
                x,
                y,
                width.max(1) as u32,
                height.max(1) as u32,
                None,
            );
            return;
        }
        let (x, y) = self.window.get_pos();
        let (width, height) = self.window.get_size();
        let window = &mut self.window;
        let switched = self.glfw.with_primary_monitor(|_, monitor| {
            let monitor = monitor?;
            let mode = monitor.get_video_mode()?;
            window.set_monitor(
                WindowMode::FullScreen(monitor),
                0,
                0,
                mode.width,
                mode.height,
                Some(mode.refresh_rate),
            );
            Some(())
        });
        if switched.is_some() {
            self.windowed = Some((x, y, width, height));
        }
    }

    /// Hide and hold the cursor for mouse look, or give it back.
    ///
    /// The reference grabs the mouse with GLFW's disabled cursor mode and
    /// asks for raw motion where the platform offers it.
    pub fn set_cursor_captured(&mut self, captured: bool) {
        if captured {
            self.window.set_cursor_mode(glfw::CursorMode::Disabled);
            if self.glfw.supports_raw_motion() {
                self.window.set_raw_mouse_motion(true);
            }
        } else {
            self.window.set_cursor_mode(glfw::CursorMode::Normal);
        }
    }

    /// The window's size in screen coordinates, which cursor positions are
    /// measured in.
    pub fn window_size(&self) -> (i32, i32) {
        self.window.get_size()
    }

    /// Put the cursor at a place in the window, in screen coordinates.
    pub fn set_cursor_position(&mut self, x: f64, y: f64) {
        self.window.set_cursor_pos(x, y);
    }

    /// Window position in screen coordinates, as the reference tracks it.
    pub fn position(&self) -> (i32, i32) {
        self.window.get_pos()
    }

    /// Raw handles for surface creation, valid while this window is alive.
    pub fn raw_handles(
        &self,
    ) -> Result<
        (
            raw_window_handle::RawDisplayHandle,
            raw_window_handle::RawWindowHandle,
        ),
        raw_window_handle::HandleError,
    > {
        use raw_window_handle::{HasDisplayHandle as _, HasWindowHandle as _};
        Ok((
            self.window.display_handle()?.as_raw(),
            self.window.window_handle()?.as_raw(),
        ))
    }

    /// Block until at least one event arrives, then drain as `poll` does.
    pub fn wait(&mut self) -> Vec<glfw::WindowEvent> {
        self.glfw.wait_events();
        self.poll()
    }
}
