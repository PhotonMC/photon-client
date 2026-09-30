//! The client's block registry (the tags the server sent) and what a state
//! collides with and what a click aims at.

use std::sync::Arc;

use voxel_core::{
    binary::BlockPos,
    shape::{Shapes, VoxelShape},
};

use crate::block::tags::BlockTags;

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
    /// A block that shifts by position answers with the shift at the origin.
    pub fn collision_shape(&self, state_id: i32) -> VoxelShape {
        let info = crate::block::state_or_air(i64::from(state_id));
        crate::block::states().collision(info).clone()
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

/// `BlockState.getShape` with no entity in the question: the outline a
/// click aims at, moved by the block's random offset where it has one.
pub fn outline_shape(state_id: u32, position: [i32; 3]) -> VoxelShape {
    let Some(info) = crate::block::state(state_id) else {
        return Shapes::empty();
    };
    let shape = crate::block::states().outline(info).clone();
    let block = info.block();
    if crate::block::moves_with_position(block) {
        let [x, y, z] = crate::block::offset(block, position);
        shape.move_by(x, y, z)
    } else {
        shape
    }
}
