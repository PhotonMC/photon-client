//! Turning loaded chunks into something the renderer can draw.
//!
//! The server sends a block state as one number. The registry turns that back
//! into a block and its properties, the block's blockstate file picks a
//! variant, and the variant names a model. This walks a box of the world and
//! does that for every position in it.

use std::collections::{BTreeMap, HashMap};

use crate::block::bake::{BakeKey, BakedModel};
use crate::resolve::{AssetSource, BuiltAtlas, ResolveError, ResolvedModel};
use crate::scene::Mesh;
use photon_core::direction::Direction;
use photon_core::identifier::Identifier;
use photon_data::block::registry;

/// Every block's models and the one atlas they are drawn from, prepared
/// once, the way the reference bakes its block models at resource load.
pub struct BlockRenderer {
    pub(super) models: BTreeMap<String, ResolvedModel>,
    /// Every variant's model, baked, by the index the states name.
    pub(super) baked: Vec<BakedModel>,
    /// Each item's client item model tree, by registry id.
    pub(super) items: Vec<Option<crate::item_model::ClientItem>>,
    /// The alpha of every texture a flat item extrudes, with its size.
    pub(super) layer_alpha: BTreeMap<String, crate::item_model::LayerAlpha>,
    /// What each block state draws, by state id; after the registry's come the item frames'
    /// fake states ([`Self::item_frame_state`]).
    pub(in crate::block::world_mesh) states: Vec<StateRender>,
    /// The sprite each state's flakes show, by state id.
    pub(super) particle_sprites: Vec<Option<String>>,
    /// The id of the first fake state (the number of states the registry has).
    pub(super) frame_base: u32,
    /// The block atlas, with the fluid and sky sprites too.
    pub atlas: BuiltAtlas,
    /// Blocks or models that could not be prepared, with why.
    pub skipped: Vec<(String, String)>,
}

mod conduit;
pub use conduit::ConduitMeshes;
mod fluid;
use fluid::*;
mod item_frame;
pub use item_frame::*;
mod mesh;
mod model;
mod particle;
pub use mesh::*;
use model::*;
mod states;
use states::*;

/// How a world box turned out.
pub struct WorldMesh {
    /// The mesh, in block units with the box's near corner at the origin.
    pub mesh: Mesh,
    /// The atlas every sprite in it came from.
    pub atlas: BuiltAtlas,
    /// How many positions held something other than air.
    pub solid_count: usize,
    /// How many distinct models were used.
    pub model_count: usize,
    /// Block names the build could not resolve, with why, in the order first
    /// seen. Reported rather than swallowed: a world will always have blocks
    /// whose models need machinery that is not built yet.
    pub skipped: Vec<(String, String)>,
}

/// A quad's four corners in the model's unit cube, and the way it faces.
pub type FacedQuad = ([[f32; 3]; 4], Direction);

/// One meshed region.
pub struct RegionMesh {
    pub mesh: Mesh,
    /// How many positions held something other than air.
    pub solid_count: usize,
}

/// What meshing reads of the world around a region: block states, light,
/// biome colours and the server's block tags.
pub struct MeshWorld<'a, R, L, T> {
    pub read: &'a R,
    pub light: &'a L,
    pub biome_tint: &'a T,
    pub tags: &'a photon_world::block::tags::BlockTags,
    /// The dimension type's side shades.
    pub cardinal: photon_world::CardinalLightType,
}
