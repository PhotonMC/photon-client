//! GPU backend. The reference ships an OpenGL and a Vulkan path and defaults to
//! OpenGL; this client targets the Vulkan one, for the reasons recorded in
//! `docs/render-backend-decision.md`.

pub mod atlas;
pub mod model;
pub mod png;
pub mod vulkan;

pub use atlas::{SpriteEntry, SpritePlacement, StitchError, Stitcher};
pub use model::{Material, SlotContents, TextureSlots, TextureSlotsData, TextureSlotsResolver};
pub use png::{DecodedImage, PngError, decode_rgba};
pub use vulkan::{CapturedFrame, VulkanBackend, VulkanError};
