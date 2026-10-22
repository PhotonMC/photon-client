//! The shapes a moving piston pushes with.

use photon_core::shape::VoxelShape;
use photon_data::block::blocks::id;

use super::pistons::{shape_at, state_of};
use crate::block;
use crate::block::entity::MovingPiston;
use crate::block::piston::{self, Pos};

/// `PistonMovingBlockEntity.getCollisionRelatedBlockState`'s shape: a piston that draws back is
/// its head (short from the first quarter of the way), anything else the block it carries.
pub(crate) fn pushing_shape(moving: &MovingPiston, pos: Pos) -> VoxelShape {
    let carried = block::state_or_air(moving.moved_state);
    let is_piston = carried
        .block()
        .class
        .is(photon_data::block::classes::BlockClass::PistonBaseBlock);
    if moving.extending || !moving.source || !is_piston {
        return shape_at(moving.moved_state, pos);
    }
    let sticky = carried.block == id::STICKY_PISTON as u16;
    let facing = piston::facing(moving.moved_state).unwrap_or(moving.direction);
    let head = state_of(
        id::PISTON_HEAD,
        &[
            ("facing", facing.name()),
            (
                "short",
                if moving.progress > 0.25 {
                    "true"
                } else {
                    "false"
                },
            ),
            ("type", if sticky { "sticky" } else { "normal" }),
        ],
    );
    shape_at(head, pos)
}

/// The shape of the block a moving piston carries.
pub(crate) fn carried_shape(moving: &MovingPiston, pos: Pos) -> VoxelShape {
    shape_at(moving.moved_state, pos)
}

/// Whether the block a moving piston carries is honey, which takes what stands on it along.
pub(crate) fn carries_honey(moving: &MovingPiston) -> bool {
    block::state_or_air(moving.moved_state).block == id::HONEY_BLOCK as u16
}
