//! Block models baked once at load (`ModelBakery`): each variant's quads
//! turned by its rotation, their texture kept in place under UV lock and
//! mapped onto the atlas, with the cull face and tint slot the mesher reads.
//! Meshing a block then only picks a variant and lights its quads.

use voxel_core::direction::Direction;

use crate::resolve::{BuiltAtlas, ResolvedModel};

/// One quad of a baked model, in the block's unit cube.
#[derive(Debug, Clone)]
pub struct BakedQuad {
    pub corners: [[f32; 3]; 4],
    /// Atlas coordinates of each corner.
    pub uvs: [[f32; 2]; 4],
    /// The side the quad faces.
    pub direction: Direction,
    /// The side whose neighbour may hide it.
    pub cull: Option<Direction>,
    /// The tint slot the quad takes its colour from; negative for none.
    pub tint_index: i32,
    /// Whether the quad takes the per-side shade.
    pub shade: bool,
}

/// A variant's model, baked.
#[derive(Debug, Clone)]
pub struct BakedModel {
    pub quads: Vec<BakedQuad>,
    pub ambient_occlusion: bool,
}

impl BakedModel {
    /// Whether every quad hides behind a side, so a block closed in on all
    /// six draws nothing.
    pub fn closes_in(&self) -> bool {
        self.quads.iter().all(|quad| quad.cull.is_some())
    }
}

/// What one baked model is made from: a model and a variant's turn.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BakeKey {
    pub model: String,
    /// Degrees about x, y and z.
    pub rotation: (i32, i32, i32),
    pub uv_lock: bool,
}

/// Bake a model for one variant; `None` when the turn is not a quarter turn
/// or a texture is missing from the atlas.
pub fn bake(model: &ResolvedModel, atlas: &BuiltAtlas, key: &BakeKey) -> Option<BakedModel> {
    let (rotate_x, rotate_y, rotate_z) = key.rotation;
    let state = crate::rotation::model_state_transform(rotate_x, rotate_y, rotate_z)?;
    let identity = rotate_x % 360 == 0 && rotate_y % 360 == 0 && rotate_z % 360 == 0;
    let uv_transform = |facing| {
        crate::rotation::inverse_face_transform(rotate_x, rotate_y, rotate_z, facing, key.uv_lock)
            .unwrap_or(crate::rotation::FLAT_IDENTITY)
    };
    let mesh = crate::resolve::build_mesh(
        model,
        atlas,
        (!identity).then_some(&state),
        &[],
        uv_transform,
        |_| false,
    )
    .ok()?;
    let quads = mesh
        .quads
        .iter()
        .enumerate()
        .map(|(quad, info)| {
            let vertex = |corner: usize| &mesh.vertices[quad * 4 + corner];
            BakedQuad {
                corners: std::array::from_fn(|corner| vertex(corner).position),
                uvs: std::array::from_fn(|corner| vertex(corner).uv),
                direction: info.direction,
                cull: info.cull,
                tint_index: info.tint_index,
                shade: info.shade,
            }
        })
        .collect();
    Some(BakedModel {
        quads,
        ambient_occlusion: model.ambient_occlusion,
    })
}
