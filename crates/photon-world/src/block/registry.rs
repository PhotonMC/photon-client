//! The client's block registry (the tags the server sent) and what a state
//! collides with and what a click aims at.

use std::sync::Arc;

use photon_core::{
    binary::BlockPos,
    ids::BlockStateId,
    shape::{Shapes, VoxelShape},
};

use crate::block::tags::{BlockTags, ItemTags};

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
    /// The item tags the server last sent.
    pub(crate) item_tags: Arc<ItemTags>,
    /// The banner pattern tags the server last sent.
    pub(crate) banner_pattern_tags: Arc<ItemTags>,
    /// The entity type tags the server last sent.
    pub(crate) entity_type_tags: Arc<ItemTags>,
}

impl ClientBlockStateRegistry {
    /// Java `Block.stateById`: a number outside the registry reads as air.
    pub fn resolve_state_id(&self, state_id: i32) -> i32 {
        match photon_data::block::registry::block_for_state(BlockStateId(state_id)) {
            Some(_) => state_id,
            None => 0,
        }
    }

    /// The shape a state blocks movement with, when no entity is asking,
    /// as a block at the origin has it: one that shifts by position is
    /// shifted by the origin's shift.
    pub fn collision_shape(&self, state_id: i32) -> VoxelShape {
        collision_shape_at(BlockStateId(state_id), [0; 3])
    }

    /// The same for a block at `position`.
    pub fn collision_shape_at(&self, state_id: i32, position: [i32; 3]) -> VoxelShape {
        collision_shape_at(BlockStateId(state_id), position)
    }

    /// A stair state's `shape`.
    pub fn stair_shape_name(&self, state_id: i32) -> Option<&'static str> {
        crate::block::rules::property(state_id, "shape")
    }

    /// The registry number of the block a state belongs to.
    ///
    /// The pinned registry numbers blocks in the same order it numbers their
    /// states, so the block's position in it is its registry number.
    pub fn block_registry_id(&self, state_id: i32) -> Option<i32> {
        photon_data::block::registry::block_index_for_state(BlockStateId(state_id))
            .map(|(index, _)| index as i32)
    }

    /// `BlockTags.CLIMBABLE`.
    pub fn is_climbable_state(&self, state_id: i32) -> bool {
        self.is_state_in_tag(state_id, "minecraft:climbable")
    }

    /// `BlockTags.CAN_GLIDE_THROUGH`.
    pub fn is_glide_through_state(&self, state_id: i32) -> bool {
        self.is_state_in_tag(state_id, "minecraft:can_glide_through")
    }

    /// The banner pattern tags the server last sent.
    pub fn banner_pattern_tags(&self) -> &Arc<ItemTags> {
        &self.banner_pattern_tags
    }

    /// The entity type tags the server last sent, by entity type registry id.
    pub fn entity_type_tags(&self) -> &Arc<ItemTags> {
        &self.entity_type_tags
    }

    /// The item tags the server last sent.
    pub fn item_tags(&self) -> &Arc<ItemTags> {
        &self.item_tags
    }

    /// The block tags the server last sent.
    pub fn tags(&self) -> &Arc<BlockTags> {
        &self.tags
    }

    pub(crate) fn apply_update_tags(&mut self, registries: &[photon_core::tags::RegistryTags]) {
        if let Some(tags) = BlockTags::from_registries(registries) {
            self.tags = Arc::new(tags);
        }
        if let Some(tags) = ItemTags::from_registries(registries) {
            self.item_tags = Arc::new(tags);
        }
        if let Some(tags) = ItemTags::of_registry(registries, "minecraft:banner_pattern") {
            self.banner_pattern_tags = Arc::new(tags);
        }
        if let Some(tags) = ItemTags::of_registry(registries, "minecraft:entity_type") {
            self.entity_type_tags = Arc::new(tags);
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
pub fn outline_shape(state_id: BlockStateId, position: [i32; 3]) -> VoxelShape {
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

/// `BlockState.getCollisionShape` with no entity in the question, moved by
/// the block's random offset at `position` where it has one.
pub fn collision_shape_at(state_id: BlockStateId, position: [i32; 3]) -> VoxelShape {
    let Some(info) = crate::block::state(state_id) else {
        return Shapes::empty();
    };
    let shape = crate::block::states().collision(info).clone();
    let block = info.block();
    if crate::block::moves_with_position(block) {
        let [x, y, z] = crate::block::offset(block, position);
        shape.move_by(x, y, z)
    } else {
        shape
    }
}
