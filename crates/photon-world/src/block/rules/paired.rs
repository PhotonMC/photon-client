//! `updateShape` of the blocks whose state follows a neighbour: the two halves of beds and
//! doors, and the campfire over hay.

use super::growth::{offset, same_block};
use super::*;

fn air() -> BlockStateId {
    registry::default_state_named("minecraft:air")
}

/// `BedBlock.updateShape`: the half toward the other half takes its `occupied`; a bed without its
/// other half is air.
pub(super) fn bed(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let Some(facing) = facing(state) else {
        return state;
    };
    let part = property(state, "part");
    let toward = if part == Some("foot") {
        facing
    } else {
        facing.opposite()
    };
    if direction != toward {
        return state;
    }
    let other = ctx.state(offset(pos, direction));
    let own = block_index(state).map(|(index, _)| index);
    let partner = own.is_some_and(|own| same_block(other, own)) && property(other, "part") != part;
    match property(other, "occupied") {
        Some(occupied) if partner => with_property(state, "occupied", occupied).unwrap_or(state),
        _ => air(),
    }
}

/// `CampfireBlock.updateShape`: the signal fire follows hay below.
pub(super) fn campfire(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction != Direction::Down {
        return state;
    }
    let hay =
        block(ctx.state(offset(pos, direction))).is_some_and(|b| b.name == "minecraft:hay_block");
    with_property(state, "signal_fire", flag(hay)).unwrap_or(state)
}

/// `DoorBlock.updateShape`: the half toward the other half becomes a copy of it with its own
/// half; a lower half that cannot survive is air.
pub(super) fn door(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    let half = property(state, "half");
    let lower = half == Some("lower");
    if direction.axis() != Axis::Y || lower != (direction == Direction::Up) {
        if lower && direction == Direction::Down {
            return match can_survive(state, pos, ctx) {
                Rule::Known(false) => Rule::Known(air()),
                Rule::Known(true) => Rule::Known(state),
                Rule::Unported => Rule::Unported,
            };
        }
        return Rule::Known(state);
    }
    let other = ctx.state(offset(pos, direction));
    let is_door = block(other).is_some_and(|b| b.class.is(BlockClass::DoorBlock));
    match half {
        Some(half) if is_door && property(other, "half") != Some(half) => {
            Rule::Known(with_property(other, "half", half).unwrap_or(other))
        }
        _ => Rule::Known(air()),
    }
}
