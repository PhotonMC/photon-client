//! GPU backend. The reference ships an OpenGL and a Vulkan path and defaults to
//! OpenGL; this client targets the Vulkan one, for the reasons recorded in
//! `docs/render-backend-decision.md`.

pub mod vulkan;

pub use vulkan::{CapturedFrame, VulkanBackend, VulkanError};
