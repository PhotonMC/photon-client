//! Client window: the reference `Window`'s sizing rules and its GLFW surface.
//!
//! [`scale`] holds the pure sizing behaviour pinned against the Java class and
//! needs no GPU or surface. [`surface`] owns the actual GLFW window and applies
//! those rules to a real framebuffer.

pub mod scale;
pub mod surface;
pub mod video_mode;

pub use scale::{
    BASE_HEIGHT, BASE_WIDTH, WindowScale, allowed_window_min_size, framebuffer_extent,
};
pub use surface::{DisplayData, WindowSurface, WindowSurfaceError};
pub use video_mode::VideoMode;
