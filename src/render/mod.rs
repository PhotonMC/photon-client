//! GPU backend. The reference ships an OpenGL and a Vulkan path and defaults to
//! OpenGL; this client targets the Vulkan one, for the reasons recorded in
//! `docs/render-backend-decision.md`.

pub mod atlas;
pub mod biome_tint;
pub mod block_light;
pub mod blockstate;
pub mod face_culling;
pub mod face_occlusion_table;
pub mod geometry;
pub mod mipmap;
pub mod model;
pub mod png;
pub mod resolve;
pub mod rotation;
pub mod scene;
pub mod sky;
pub mod texture_metadata;
pub mod tint;
mod uv_lock_table;
pub mod vulkan;
pub mod world_mesh;

pub use atlas::{AtlasSprite, SpriteEntry, SpritePlacement, StitchError, Stitcher};
pub use blockstate::{
    BlockStateFile, Condition, Operation, Quadrant as VariantQuadrant, Selector, Term, Variant,
    parse_block_state,
};
pub use geometry::{
    FaceUvs, Quadrant, WindingError, approximate_nearest_direction, closest_direction,
    default_face_uv, face_vertex_uv, face_vertices, recalculate_winding, triangle_normal,
};
pub use mipmap::{
    MipmapStrategy, SrgbTables, Transparency, compute_transparency, generate_mip_levels,
    generate_mip_levels_with,
};
pub use model::{
    CuboidModel, ElementRotation, GuiLight, Material, ModelElement, ModelFace, ModelParseError,
    SlotContents, TextureSlots, TextureSlotsData, TextureSlotsResolver, parse_model,
};
pub use png::{DecodedImage, PngError, decode_rgba};
pub use resolve::{
    AssetSource, BuiltAtlas, ResolveError, ResolvedModel, build_atlas, build_atlas_for, build_mesh,
    resolve_model,
};
pub use rotation::{
    Matrix4, RotationAxis, element_transform, euler_element_transform, model_state_transform,
    rotate_vertex,
};
pub use scene::{Mesh, SpriteRect, Vertex, full_cube};
pub use texture_metadata::{AnimationFrames, TextureMetadata};
pub use tint::{
    ColourMap, ColourMapOutOfBounds, FOLIAGE_MANGROVE, LILY_PAD_IN_WORLD, colour_map_lookup,
    constant_tints, multiply,
};
pub use vulkan::{CapturedFrame, VulkanBackend, VulkanError};
pub use world_mesh::{WorldMesh, build_world_mesh};
