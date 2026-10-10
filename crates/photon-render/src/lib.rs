//! GPU backend. The reference ships an OpenGL and a Vulkan path and defaults to
//! OpenGL; this client targets the Vulkan one, for the reasons recorded in
//! `docs/adr/0001-vulkan-renderer.md`.

pub mod block;
pub mod block_entity;
pub mod border;
pub mod camera;
pub mod entity;
pub mod frame_timing;
pub mod gui;
pub mod item_model;
pub mod joml;
pub mod model;
pub mod objects;
pub mod panorama;
pub mod particles;
pub mod render_types;
pub mod resolve;
pub mod scene;
pub mod screen_effect;
pub mod section_workers;
pub mod sky;
pub mod texture;
pub mod visibility;
pub mod vulkan;
pub mod window;

pub use block::blockstate::{
    BlockStateFile, Condition, Operation, Selector, Term, Variant, parse_block_state,
};
pub use block::geometry::{
    FaceUvs, Quadrant, WindingError, approximate_nearest_direction, closest_direction,
    default_face_uv, face_vertex_uv, face_vertices, recalculate_winding, triangle_normal,
};
pub use block::rotation::{
    Matrix4, RotationAxis, element_transform, euler_element_transform, model_state_transform,
    rotate_vertex,
};
pub use block::tint::{
    ColourMap, ColourMapOutOfBounds, FOLIAGE_MANGROVE, LILY_PAD_IN_WORLD, colour_map_lookup,
    constant_tints, multiply,
};
pub use block::world_mesh::{WorldMesh, build_world_mesh};
pub use frame_timing::{BackendTimings, GPU_PASSES, GpuTimings};
pub use model::{
    CuboidModel, ElementRotation, GuiLight, Material, ModelElement, ModelFace, ModelParseError,
    SlotContents, TextureSlots, TextureSlotsData, TextureSlotsResolver, parse_model,
};
pub use resolve::{
    AssetSource, BuiltAtlas, ResolveError, ResolvedModel, build_atlas, build_atlas_for, build_mesh,
    resolve_model,
};
pub use scene::{BlockVertex, Mesh, SpriteRect, Vertex, full_cube};
pub use texture::atlas::{AtlasSprite, SpriteEntry, SpritePlacement, StitchError, Stitcher};
pub use texture::metadata::{AnimationFrames, TextureMetadata};
pub use texture::mipmap::{
    MipmapStrategy, SrgbTables, Transparency, compute_transparency, generate_mip_levels,
    generate_mip_levels_with,
};
pub use texture::png::{DecodedImage, PngError, decode_rgba};
pub use vulkan::{CapturedFrame, SectionMemory, VulkanBackend, VulkanError, validation_findings};
