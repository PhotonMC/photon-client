//! `ClientWorld` block_state layer split out of the former single-file module.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    binary::BlockPos,
    direction::Direction,
    shape::{Shapes, VoxelShape},
};

/// A block collision shape returned in Java's cursor traversal order.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientBlockCollision {
    pub position: BlockPos,
    pub shape: VoxelShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClientStairsShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}

impl ClientStairsShape {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Straight => "straight",
            Self::InnerLeft => "inner_left",
            Self::InnerRight => "inner_right",
            Self::OuterLeft => "outer_left",
            Self::OuterRight => "outer_right",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClientStairBlockState {
    pub(crate) facing: Direction,
    pub(crate) top: bool,
    pub(crate) shape: ClientStairsShape,
    pub(crate) waterlogged: bool,
}

pub(crate) fn client_stair_block_state(
    facing: Direction,
    top: bool,
    shape: ClientStairsShape,
    waterlogged: bool,
) -> ClientStairBlockState {
    ClientStairBlockState {
        facing,
        top,
        shape,
        waterlogged,
    }
}

/// Resolve the pinned 26.2 built-in block-state IDs used by the current
/// ClientWorld collision slice.
///
/// Java's complete built-in registry is larger. IDs outside this deliberately
/// small exact subset follow the `Block.stateById` fallback to air until their
/// state definition and context branch is implemented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBlockStateRegistry {
    pub(crate) stair_states: BTreeMap<i32, ClientStairBlockState>,
    pub(crate) climbable_block_registry_ids: BTreeSet<i32>,
    pub(crate) glide_through_block_registry_ids: BTreeSet<i32>,
    /// Every block tag the server sent, by tag id, as block registry ids.
    pub(crate) block_tags: BTreeMap<String, BTreeSet<i32>>,
}

impl ClientBlockStateRegistry {
    pub fn vanilla_26_2() -> Self {
        let stair_states = BTreeMap::from([
            (
                3908,
                client_stair_block_state(
                    Direction::North,
                    true,
                    ClientStairsShape::Straight,
                    false,
                ),
            ),
            (
                3917,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::Straight,
                    true,
                ),
            ),
            (
                3918,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::Straight,
                    false,
                ),
            ),
            (
                3919,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::InnerLeft,
                    true,
                ),
            ),
            (
                3920,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::InnerLeft,
                    false,
                ),
            ),
            (
                3921,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::InnerRight,
                    true,
                ),
            ),
            (
                3922,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::InnerRight,
                    false,
                ),
            ),
            (
                3923,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::OuterLeft,
                    true,
                ),
            ),
            (
                3924,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::OuterLeft,
                    false,
                ),
            ),
            (
                3925,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::OuterRight,
                    true,
                ),
            ),
            (
                3926,
                client_stair_block_state(
                    Direction::North,
                    false,
                    ClientStairsShape::OuterRight,
                    false,
                ),
            ),
            (
                3938,
                client_stair_block_state(
                    Direction::South,
                    false,
                    ClientStairsShape::Straight,
                    false,
                ),
            ),
            (
                3958,
                client_stair_block_state(
                    Direction::West,
                    false,
                    ClientStairsShape::Straight,
                    false,
                ),
            ),
            (
                3978,
                client_stair_block_state(
                    Direction::East,
                    false,
                    ClientStairsShape::Straight,
                    false,
                ),
            ),
        ]);
        let climbable_block_registry_ids = BTreeSet::from([221, 366]);
        let glide_through_block_registry_ids = BTreeSet::from([366]);
        Self {
            stair_states,
            climbable_block_registry_ids,
            glide_through_block_registry_ids,
            block_tags: BTreeMap::new(),
        }
    }

    /// Java `Block.stateById`: a number outside the registry reads as air.
    pub fn resolve_state_id(&self, state_id: i32) -> i32 {
        match u32::try_from(state_id) {
            Ok(id) if crate::world::block_registry::block_for_state(id).is_some() => state_id,
            _ => 0,
        }
    }

    /// The shape a state blocks movement with, when no entity is asking.
    ///
    /// This is the pinned table for every state the registry has. A handful of
    /// blocks are flagged there as depending on their position or on the
    /// asking entity; see [`crate::world::block_collision_table`].
    pub fn collision_shape(&self, state_id: i32) -> VoxelShape {
        table_collision_shape(self.resolve_state_id(state_id) as u32)
    }

    pub fn stair_shape_name(&self, state_id: i32) -> Option<&'static str> {
        self.stair_states
            .get(&state_id)
            .map(|state| state.shape.name())
    }

    pub(crate) fn stair_shape_update_state_id(
        &self,
        state_id: i32,
        neighbors: &[Option<i32>; 6],
    ) -> Option<i32> {
        let state = self.stair_states.get(&state_id)?;
        let facing = state.facing;
        let shape = if let Some(behind_id) = neighbors[facing.data3d() as usize]
            && let Some(behind) = self.stair_states.get(&behind_id)
            && state.top == behind.top
            && behind.facing.axis() != facing.axis()
            && self.stair_can_take_shape(state, neighbors, behind.facing.opposite())
        {
            if behind.facing
                == facing
                    .counter_clockwise_y()
                    .expect("stair facing is horizontal")
            {
                ClientStairsShape::OuterLeft
            } else {
                ClientStairsShape::OuterRight
            }
        } else if let Some(front_id) = neighbors[facing.opposite().data3d() as usize]
            && let Some(front) = self.stair_states.get(&front_id)
            && state.top == front.top
            && front.facing.axis() != facing.axis()
            && self.stair_can_take_shape(state, neighbors, front.facing)
        {
            if front.facing
                == facing
                    .counter_clockwise_y()
                    .expect("stair facing is horizontal")
            {
                ClientStairsShape::InnerLeft
            } else {
                ClientStairsShape::InnerRight
            }
        } else {
            ClientStairsShape::Straight
        };

        self.stair_state_id_for_shape(state, shape)
    }

    pub(crate) fn stair_can_take_shape(
        &self,
        state: &ClientStairBlockState,
        neighbors: &[Option<i32>; 6],
        direction: Direction,
    ) -> bool {
        let Some(neighbor_id) = neighbors[direction.data3d() as usize] else {
            return true;
        };
        let Some(neighbor) = self.stair_states.get(&neighbor_id) else {
            return true;
        };
        neighbor.facing != state.facing || neighbor.top != state.top
    }

    pub(crate) fn stair_state_id_for_shape(
        &self,
        state: &ClientStairBlockState,
        shape: ClientStairsShape,
    ) -> Option<i32> {
        if state.facing != Direction::North || state.top {
            return None;
        }
        Some(match (state.waterlogged, shape) {
            (false, ClientStairsShape::Straight) => 3918,
            (false, ClientStairsShape::InnerLeft) => 3920,
            (false, ClientStairsShape::InnerRight) => 3922,
            (false, ClientStairsShape::OuterLeft) => 3924,
            (false, ClientStairsShape::OuterRight) => 3926,
            (true, ClientStairsShape::Straight) => 3917,
            (true, ClientStairsShape::InnerLeft) => 3919,
            (true, ClientStairsShape::InnerRight) => 3921,
            (true, ClientStairsShape::OuterLeft) => 3923,
            (true, ClientStairsShape::OuterRight) => 3925,
        })
    }

    /// The registry number of the block a state belongs to.
    ///
    /// The pinned registry numbers blocks in the same order it numbers their
    /// states, so the block's position in it is its registry number.
    pub fn block_registry_id(&self, state_id: i32) -> Option<i32> {
        let id = u32::try_from(state_id).ok()?;
        crate::world::block_registry::block_index_for_state(id).map(|(index, _)| index as i32)
    }

    /// Return whether a block-state belongs to BlockTags.CLIMBABLE.
    pub fn is_climbable_state(&self, state_id: i32) -> bool {
        self.block_registry_id(state_id)
            .is_some_and(|block_id| self.climbable_block_registry_ids.contains(&block_id))
    }

    /// Return whether a block-state belongs to BlockTags.CAN_GLIDE_THROUGH.
    pub fn is_glide_through_state(&self, state_id: i32) -> bool {
        self.block_registry_id(state_id)
            .is_some_and(|block_id| self.glide_through_block_registry_ids.contains(&block_id))
    }

    pub(crate) fn apply_update_tags(&mut self, registries: &[crate::tags::RegistryTags]) {
        let Some(block_registry) = registries.iter().find(|registry| {
            registry.registry.namespace() == "minecraft" && registry.registry.path() == "block"
        }) else {
            return;
        };

        let mut climbable_block_registry_ids = BTreeSet::new();
        let mut glide_through_block_registry_ids = BTreeSet::new();
        for tag in &block_registry.tags {
            let target = if tag.id.namespace() == "minecraft" && tag.id.path() == "climbable" {
                Some(&mut climbable_block_registry_ids)
            } else if tag.id.namespace() == "minecraft" && tag.id.path() == "can_glide_through" {
                Some(&mut glide_through_block_registry_ids)
            } else {
                None
            };
            if let Some(target) = target {
                target.extend(tag.values.iter().copied());
            }
        }
        self.climbable_block_registry_ids = climbable_block_registry_ids;
        self.glide_through_block_registry_ids = glide_through_block_registry_ids;
        self.block_tags = block_registry
            .tags
            .iter()
            .map(|tag| (tag.id.as_string(), tag.values.iter().copied().collect()))
            .collect();
    }

    /// `BlockState.is(tag)`, against the tags the server last sent. `tag` is
    /// the tag id without the leading `#`.
    pub fn is_state_in_tag(&self, state_id: i32, tag: &str) -> bool {
        self.block_registry_id(state_id).is_some_and(|block_id| {
            self.block_tags
                .get(tag)
                .is_some_and(|blocks| blocks.contains(&block_id))
        })
    }
}

/// The distinct shapes of the pinned table, built once.
fn table_shapes() -> &'static [VoxelShape] {
    static BUILT: std::sync::OnceLock<Vec<VoxelShape>> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        crate::world::block_collision_table::SHAPES
            .iter()
            .map(|boxes| shape_from_boxes(boxes))
            .collect()
    })
}

fn outline_shapes() -> &'static [VoxelShape] {
    static BUILT: std::sync::OnceLock<Vec<VoxelShape>> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        crate::world::block_outline_table::SHAPES
            .iter()
            .map(|boxes| shape_from_boxes(boxes))
            .collect()
    })
}

/// The rebuilt outline shape for one entry of the table, for the parity
/// adapter.
pub fn outline_table_shape(index: usize) -> Option<VoxelShape> {
    outline_shapes().get(index).cloned()
}

/// `BlockState.getShape` with no entity in the question: the outline a
/// click aims at, moved by the block's random offset where it has one.
pub fn outline_shape(state_id: u32, position: [i32; 3]) -> VoxelShape {
    let Some((index, offset)) = crate::world::block_registry::block_index_for_state(state_id)
    else {
        return Shapes::empty();
    };
    let outline = &crate::world::block_outline_table::BLOCKS[index];
    let shape_index = if outline.shapes.len() == 1 {
        outline.shapes[0]
    } else {
        outline.shapes[offset as usize]
    };
    let shape = &outline_shapes()[usize::from(shape_index)];
    if outline.offset {
        let [x, y, z] = block_offset(crate::world::block_registry::BLOCKS[index].name, position);
        shape.move_by(x, y, z)
    } else {
        shape.clone()
    }
}

/// `BlockState.getOffset`: the random nudge some blocks (plants, bamboo,
/// dripstone) take off the grid at a position.
pub fn block_offset(block: &str, position: [i32; 3]) -> [f64; 3] {
    let Some(&(_, vertical, horizontal_max, vertical_max)) =
        crate::render::block_offset_table::OFFSETS
            .iter()
            .find(|(name, ..)| *name == block)
    else {
        return [0.0; 3];
    };
    let horizontal_max = f64::from(f32::from_bits(horizontal_max));
    let seed = crate::math::mth_get_seed(position[0], 0, position[2]);
    let axis = |bits: i64| {
        let value = (f64::from((bits & 15) as f32 / 15.0) - 0.5) * 0.5;
        if value < -horizontal_max {
            -horizontal_max
        } else {
            value.min(horizontal_max)
        }
    };
    let y = if vertical {
        (f64::from((seed >> 4 & 15) as f32 / 15.0) - 1.0) * f64::from(f32::from_bits(vertical_max))
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
fn shape_from_boxes(boxes: &[crate::world::block_collision_table::ShapeBox]) -> VoxelShape {
    let full = [0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0].map(f64::to_bits);
    if boxes.len() == 1 && boxes[0] == full {
        return Shapes::block();
    }
    let mut shape = Shapes::empty();
    for bounds in boxes {
        let [min_x, min_y, min_z, max_x, max_y, max_z] = bounds.map(f64::from_bits);
        let part = Shapes::box_shape(min_x, min_y, min_z, max_x, max_y, max_z)
            .expect("the reference's boxes are ordered");
        shape = Shapes::or(&shape, &part);
    }
    shape
}

/// The rebuilt shape for one entry of the table, for the parity adapter.
pub fn table_shape(index: usize) -> Option<VoxelShape> {
    table_shapes().get(index).cloned()
}

/// The table's shape for one state; a state outside the registry is air.
pub(crate) fn table_collision_shape(state_id: u32) -> VoxelShape {
    let Some((index, offset)) = crate::world::block_registry::block_index_for_state(state_id)
    else {
        return Shapes::empty();
    };
    let shapes = crate::world::block_collision_table::BLOCKS[index].shapes;
    let shape = if shapes.len() == 1 {
        shapes[0]
    } else {
        shapes[offset as usize]
    };
    table_shapes()[usize::from(shape)].clone()
}
