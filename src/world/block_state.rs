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

/// A collision-shape descriptor for the pinned built-in block-state subset.
///
/// Raw bits keep the descriptor comparable so it can live in `ClientWorld`'s
/// structural state without making `VoxelShape` part of that state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientBlockStateCollisionShape {
    Empty,
    FullBlock,
    Box {
        min_x_bits: u64,
        min_y_bits: u64,
        min_z_bits: u64,
        max_x_bits: u64,
        max_y_bits: u64,
        max_z_bits: u64,
    },
    Boxes {
        boxes: Vec<[u64; 6]>,
    },
}

impl ClientBlockStateCollisionShape {
    pub(crate) fn to_voxel_shape(&self) -> VoxelShape {
        match self {
            Self::Empty => Shapes::empty(),
            Self::FullBlock => Shapes::block(),
            Self::Box {
                min_x_bits,
                min_y_bits,
                min_z_bits,
                max_x_bits,
                max_y_bits,
                max_z_bits,
            } => Shapes::box_shape(
                f64::from_bits(*min_x_bits),
                f64::from_bits(*min_y_bits),
                f64::from_bits(*min_z_bits),
                f64::from_bits(*max_x_bits),
                f64::from_bits(*max_y_bits),
                f64::from_bits(*max_z_bits),
            )
            .expect("registered block-state shape bounds must be ordered"),
            Self::Boxes { boxes } => {
                let mut shape = Shapes::empty();
                for [
                    min_x_bits,
                    min_y_bits,
                    min_z_bits,
                    max_x_bits,
                    max_y_bits,
                    max_z_bits,
                ] in boxes
                {
                    let box_shape = Shapes::box_shape(
                        f64::from_bits(*min_x_bits),
                        f64::from_bits(*min_y_bits),
                        f64::from_bits(*min_z_bits),
                        f64::from_bits(*max_x_bits),
                        f64::from_bits(*max_y_bits),
                        f64::from_bits(*max_z_bits),
                    )
                    .expect("registered block-state shape bounds must be ordered");
                    shape = Shapes::or(&shape, &box_shape);
                }
                shape
            }
        }
    }
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

pub(crate) fn client_block_state_box(
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
) -> [u64; 6] {
    [
        min_x.to_bits(),
        min_y.to_bits(),
        min_z.to_bits(),
        max_x.to_bits(),
        max_y.to_bits(),
        max_z.to_bits(),
    ]
}

/// Resolve the pinned 26.2 built-in block-state IDs used by the current
/// ClientWorld collision slice.
///
/// Java's complete built-in registry is larger. IDs outside this deliberately
/// small exact subset follow the `Block.stateById` fallback to air until their
/// state definition and context branch is implemented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBlockStateRegistry {
    pub(crate) collision_shapes: BTreeMap<i32, ClientBlockStateCollisionShape>,
    pub(crate) stair_states: BTreeMap<i32, ClientStairBlockState>,
    pub(crate) block_registry_ids_by_state: BTreeMap<i32, i32>,
    pub(crate) climbable_block_registry_ids: BTreeSet<i32>,
    pub(crate) glide_through_block_registry_ids: BTreeSet<i32>,
}

impl ClientBlockStateRegistry {
    pub fn vanilla_26_2() -> Self {
        let mut collision_shapes = BTreeMap::new();
        collision_shapes.insert(0, ClientBlockStateCollisionShape::Empty);
        collision_shapes.insert(1, ClientBlockStateCollisionShape::FullBlock);
        collision_shapes.insert(
            13331,
            ClientBlockStateCollisionShape::Box {
                min_x_bits: 0.0_f64.to_bits(),
                min_y_bits: 0.5_f64.to_bits(),
                min_z_bits: 0.0_f64.to_bits(),
                max_x_bits: 1.0_f64.to_bits(),
                max_y_bits: 1.0_f64.to_bits(),
                max_z_bits: 1.0_f64.to_bits(),
            },
        );
        collision_shapes.insert(
            13333,
            ClientBlockStateCollisionShape::Box {
                min_x_bits: 0.0_f64.to_bits(),
                min_y_bits: 0.0_f64.to_bits(),
                min_z_bits: 0.0_f64.to_bits(),
                max_x_bits: 1.0_f64.to_bits(),
                max_y_bits: 0.5_f64.to_bits(),
                max_z_bits: 1.0_f64.to_bits(),
            },
        );
        collision_shapes.insert(13335, ClientBlockStateCollisionShape::FullBlock);
        collision_shapes.insert(
            3908,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 1.0, 0.5),
                    client_block_state_box(0.0, 0.5, 0.5, 1.0, 1.0, 1.0),
                ],
            },
        );
        collision_shapes.insert(
            3918,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
                    client_block_state_box(0.0, 0.5, 0.0, 1.0, 1.0, 0.5),
                ],
            },
        );
        collision_shapes.insert(3917, collision_shapes[&3918].clone());
        collision_shapes.insert(
            3920,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
                    client_block_state_box(0.0, 0.5, 0.0, 0.5, 1.0, 1.0),
                    client_block_state_box(0.5, 0.5, 0.0, 1.0, 1.0, 0.5),
                ],
            },
        );
        collision_shapes.insert(3919, collision_shapes[&3920].clone());
        collision_shapes.insert(
            3922,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
                    client_block_state_box(0.0, 0.5, 0.0, 1.0, 1.0, 0.5),
                    client_block_state_box(0.5, 0.5, 0.5, 1.0, 1.0, 1.0),
                ],
            },
        );
        collision_shapes.insert(3921, collision_shapes[&3922].clone());
        collision_shapes.insert(
            3924,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
                    client_block_state_box(0.0, 0.5, 0.0, 0.5, 1.0, 0.5),
                ],
            },
        );
        collision_shapes.insert(3923, collision_shapes[&3924].clone());
        collision_shapes.insert(
            3926,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
                    client_block_state_box(0.5, 0.5, 0.0, 1.0, 1.0, 0.5),
                ],
            },
        );
        collision_shapes.insert(3925, collision_shapes[&3926].clone());
        collision_shapes.insert(
            3938,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
                    client_block_state_box(0.0, 0.5, 0.5, 1.0, 1.0, 1.0),
                ],
            },
        );
        collision_shapes.insert(
            3958,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
                    client_block_state_box(0.0, 0.5, 0.0, 0.5, 1.0, 1.0),
                ],
            },
        );
        collision_shapes.insert(
            3978,
            ClientBlockStateCollisionShape::Boxes {
                boxes: vec![
                    client_block_state_box(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
                    client_block_state_box(0.5, 0.5, 0.0, 1.0, 1.0, 1.0),
                ],
            },
        );
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
        let mut block_registry_ids_by_state = BTreeMap::from([(0, 0), (1, 1), (8389, 366)]);
        for state_id in 5719..=5726 {
            block_registry_ids_by_state.insert(state_id, 221);
        }
        let climbable_block_registry_ids = BTreeSet::from([221, 366]);
        let glide_through_block_registry_ids = BTreeSet::from([366]);
        Self {
            collision_shapes,
            stair_states,
            block_registry_ids_by_state,
            climbable_block_registry_ids,
            glide_through_block_registry_ids,
        }
    }

    pub fn resolve_state_id(&self, state_id: i32) -> i32 {
        if self.collision_shapes.contains_key(&state_id) {
            state_id
        } else {
            0
        }
    }

    pub fn collision_shape(&self, state_id: i32) -> VoxelShape {
        self.collision_shapes
            .get(&self.resolve_state_id(state_id))
            .expect("the registry must contain its air fallback")
            .to_voxel_shape()
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

    /// Return whether a pinned 26.2 block-state belongs to BlockTags.CLIMBABLE.
    pub fn is_climbable_state(&self, state_id: i32) -> bool {
        self.block_registry_ids_by_state
            .get(&state_id)
            .is_some_and(|block_id| self.climbable_block_registry_ids.contains(block_id))
    }

    /// Return whether a pinned 26.2 block-state belongs to BlockTags.CAN_GLIDE_THROUGH.
    pub fn is_glide_through_state(&self, state_id: i32) -> bool {
        self.block_registry_ids_by_state
            .get(&state_id)
            .is_some_and(|block_id| self.glide_through_block_registry_ids.contains(block_id))
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
    }

    pub(crate) fn collision_shapes(&self) -> BTreeMap<i32, VoxelShape> {
        self.collision_shapes
            .iter()
            .map(|(&state_id, shape)| (state_id, shape.to_voxel_shape()))
            .collect()
    }
}
