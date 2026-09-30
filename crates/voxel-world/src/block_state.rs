//! The client's block registry (the tags the server sent) and the pinned
//! shape tables: what a state collides with and what a click aims at.

use std::sync::Arc;

use voxel_core::{
    binary::BlockPos,
    shape::{Shapes, VoxelShape},
};

use crate::block::tags::BlockTags;
use voxel_data::block::shape::{BlockShapes, ShapeBox};

/// A block collision shape returned in Java's cursor traversal order.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientBlockCollision {
    pub position: BlockPos,
    pub shape: VoxelShape,
}

/// What the client's block registry holds beyond the pinned declarations:
/// the tags the server sent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientBlockStateRegistry {
    /// The block tags the server last sent.
    pub(crate) tags: Arc<BlockTags>,
}

impl ClientBlockStateRegistry {
    /// Java `Block.stateById`: a number outside the registry reads as air.
    pub fn resolve_state_id(&self, state_id: i32) -> i32 {
        match u32::try_from(state_id) {
            Ok(id) if voxel_data::block::registry::block_for_state(id).is_some() => state_id,
            _ => 0,
        }
    }

    /// The shape a state blocks movement with, when no entity is asking.
    ///
    /// This is the pinned table for every state the registry has. A handful of
    /// blocks are flagged there as depending on their position or on the
    /// asking entity; see [`voxel_data::block::collision`].
    pub fn collision_shape(&self, state_id: i32) -> VoxelShape {
        table_collision_shape(self.resolve_state_id(state_id) as u32)
    }

    /// A stair state's `shape`.
    pub fn stair_shape_name(&self, state_id: i32) -> Option<&'static str> {
        crate::block_rules::property(state_id, "shape")
    }

    /// The registry number of the block a state belongs to.
    ///
    /// The pinned registry numbers blocks in the same order it numbers their
    /// states, so the block's position in it is its registry number.
    pub fn block_registry_id(&self, state_id: i32) -> Option<i32> {
        let id = u32::try_from(state_id).ok()?;
        voxel_data::block::registry::block_index_for_state(id).map(|(index, _)| index as i32)
    }

    /// `BlockTags.CLIMBABLE`.
    pub fn is_climbable_state(&self, state_id: i32) -> bool {
        self.is_state_in_tag(state_id, "minecraft:climbable")
    }

    /// `BlockTags.CAN_GLIDE_THROUGH`.
    pub fn is_glide_through_state(&self, state_id: i32) -> bool {
        self.is_state_in_tag(state_id, "minecraft:can_glide_through")
    }

    /// The block tags the server last sent.
    pub fn tags(&self) -> &Arc<BlockTags> {
        &self.tags
    }

    pub(crate) fn apply_update_tags(&mut self, registries: &[voxel_core::tags::RegistryTags]) {
        if let Some(tags) = BlockTags::from_registries(registries) {
            self.tags = Arc::new(tags);
        }
    }

    /// `BlockState.is(tag)`, against the tags the server last sent. `tag` is
    /// the tag id without the leading `#`.
    pub fn is_state_in_tag(&self, state_id: i32, tag: &str) -> bool {
        self.block_registry_id(state_id)
            .is_some_and(|block| self.tags.contains(block as usize, tag))
    }
}

/// The distinct shapes of a shape table, rebuilt once each.
fn built_shapes(
    cell: &'static std::sync::OnceLock<Vec<VoxelShape>>,
    table: &'static [&'static [ShapeBox]],
) -> &'static [VoxelShape] {
    cell.get_or_init(|| table.iter().map(|boxes| shape_from_boxes(boxes)).collect())
}

fn collision_shapes() -> &'static [VoxelShape] {
    static BUILT: std::sync::OnceLock<Vec<VoxelShape>> = std::sync::OnceLock::new();
    built_shapes(&BUILT, &voxel_data::block::collision::SHAPES)
}

fn outline_shapes() -> &'static [VoxelShape] {
    static BUILT: std::sync::OnceLock<Vec<VoxelShape>> = std::sync::OnceLock::new();
    built_shapes(&BUILT, &voxel_data::block::outline::SHAPES)
}

/// The block (registry index) of a state, and its shape in a shape table;
/// `None` outside the registry.
fn state_shape(
    shapes: &'static [VoxelShape],
    blocks: &'static [BlockShapes],
    state_id: u32,
) -> Option<(usize, &'static BlockShapes, &'static VoxelShape)> {
    let (index, offset) = voxel_data::block::registry::block_index_for_state(state_id)?;
    let block = &blocks[index];
    let shape = &shapes[usize::from(voxel_data::block::per_state(block.shapes, offset)?)];
    Some((index, block, shape))
}

/// The rebuilt outline shape for one entry of the table, for the parity
/// adapter.
pub fn outline_table_shape(index: usize) -> Option<VoxelShape> {
    outline_shapes().get(index).cloned()
}

/// `BlockState.getShape` with no entity in the question: the outline a
/// click aims at, moved by the block's random offset where it has one.
pub fn outline_shape(state_id: u32, position: [i32; 3]) -> VoxelShape {
    let Some((index, block, shape)) = state_shape(
        outline_shapes(),
        &voxel_data::block::outline::BLOCKS,
        state_id,
    ) else {
        return Shapes::empty();
    };
    if block.offset {
        let [x, y, z] = block_offset(voxel_data::block::registry::BLOCKS[index].name, position);
        shape.move_by(x, y, z)
    } else {
        shape.clone()
    }
}

/// `BlockState.getOffset`: the random nudge some blocks (plants, bamboo,
/// dripstone) take off the grid at a position.
pub fn block_offset(block: &str, position: [i32; 3]) -> [f64; 3] {
    let Some(offset) = voxel_data::block::offset::OFFSETS
        .iter()
        .find(|offset| offset.name == block)
    else {
        return [0.0; 3];
    };
    let horizontal_max = f64::from(offset.max_horizontal);
    let seed = voxel_core::math::mth_get_seed(position[0], 0, position[2]);
    let axis = |bits: i64| {
        let value = (f64::from((bits & 15) as f32 / 15.0) - 0.5) * 0.5;
        if value < -horizontal_max {
            -horizontal_max
        } else {
            value.min(horizontal_max)
        }
    };
    let y = if offset.vertical {
        (f64::from((seed >> 4 & 15) as f32 / 15.0) - 1.0) * f64::from(offset.max_vertical)
    } else {
        0.0
    };
    [axis(seed), y, axis(seed >> 8)]
}

/// Rebuild a shape from the boxes the reference reported for it.
///
/// A single whole-block box is the reference's own full-block shape rather
/// than a box that happens to fill the block, because the collision walk
/// tests for that shape by identity and takes a shortcut when it finds it.
fn shape_from_boxes(boxes: &[ShapeBox]) -> VoxelShape {
    let full = [0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0].map(f64::to_bits);
    if boxes.len() == 1 && boxes[0].map(f64::to_bits) == full {
        return Shapes::block();
    }
    let mut shape = Shapes::empty();
    for &[min_x, min_y, min_z, max_x, max_y, max_z] in boxes {
        let part = Shapes::box_shape(min_x, min_y, min_z, max_x, max_y, max_z)
            .expect("the reference's boxes are ordered");
        shape = Shapes::or(&shape, &part);
    }
    shape
}

/// The rebuilt shape for one entry of the table, for the parity adapter.
pub fn table_shape(index: usize) -> Option<VoxelShape> {
    collision_shapes().get(index).cloned()
}

/// The table's shape for one state; a state outside the registry is air.
pub fn table_collision_shape(state_id: u32) -> VoxelShape {
    state_shape(
        collision_shapes(),
        &voxel_data::block::collision::BLOCKS,
        state_id,
    )
    .map_or_else(Shapes::empty, |(_, _, shape)| shape.clone())
}
