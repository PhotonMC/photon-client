//! GPU backend. The reference ships an OpenGL and a Vulkan path and defaults to
//! OpenGL; this client targets the Vulkan one, for the reasons recorded in
//! `docs/render-backend-decision.md`.

pub mod atlas;
pub mod geometry;
pub mod mipmap;
pub mod model;
pub mod png;
pub mod resolve;
pub mod rotation;
pub mod scene;
pub mod vulkan;

pub use atlas::{SpriteEntry, SpritePlacement, StitchError, Stitcher};
pub use geometry::{
    FaceUvs, Quadrant, WindingError, closest_direction, default_face_uv, face_vertex_uv,
    face_vertices, recalculate_winding, triangle_normal,
};
pub use mipmap::{SrgbTables, generate_mip_levels};
pub use model::{
    CuboidModel, ElementRotation, GuiLight, Material, ModelElement, ModelFace,
    ModelParseError, SlotContents, TextureSlots, TextureSlotsData, TextureSlotsResolver,
    parse_model,
};
pub use scene::{Mesh, SpriteRect, Vertex, full_cube};
pub use resolve::{AssetSource, BuiltAtlas, ResolveError, ResolvedModel, build_atlas, build_mesh, resolve_model};
pub use rotation::{
    Matrix4, RotationAxis, element_transform, euler_element_transform, model_state_transform, rotate_vertex,
};
pub use png::{DecodedImage, PngError, decode_rgba};
pub use vulkan::{CapturedFrame, VulkanBackend, VulkanError};
