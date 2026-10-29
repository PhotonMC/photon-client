//! `updateShape` of the blocks whose state follows a neighbour: the two halves of beds and
//! doors, the campfire over hay, the attached stems, bamboo and bells.

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
    let hay = block(ctx.state(offset(pos, direction)))
        .is_some_and(|b| b.index() == photon_data::block::ids::HAY_BLOCK);
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

/// `AttachedStemBlock.updateShape`: a neighbour other than the fruit in the direction it faces
/// turns it back into the stem at full age. The fruit is the block the stem is named after.
pub(super) fn attached_stem(
    state: BlockStateId,
    direction: Direction,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let Some(name) = block(state).and_then(|b| b.name.strip_prefix("minecraft:attached_")) else {
        return state;
    };
    let fruit = name.trim_end_matches("_stem");
    let neighbour = ctx.state(offset(pos, direction));
    let is_fruit = block(neighbour).is_some_and(|b| b.name == format!("minecraft:{fruit}"));
    if is_fruit || Some(direction) != facing(state) {
        return state;
    }
    let stem = registry::default_state_named(&format!("minecraft:{fruit}_stem"));
    with_property(stem, "age", "7").unwrap_or(stem)
}

fn is_bamboo(state: BlockStateId) -> bool {
    block(state).is_some_and(|b| b.index() == photon_data::block::ids::BAMBOO)
}

/// `BambooSaplingBlock.updateShape`: air where it cannot survive; bamboo above it makes it
/// bamboo.
pub(super) fn bamboo_sapling(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    match can_survive(state, pos, ctx) {
        Rule::Known(false) => Rule::Known(air()),
        Rule::Known(true) => {
            let grown = direction == Direction::Up && is_bamboo(ctx.state(offset(pos, direction)));
            Rule::Known(if grown {
                registry::default_state_named("minecraft:bamboo")
            } else {
                state
            })
        }
        Rule::Unported => Rule::Unported,
    }
}

/// `BambooStalkBlock.updateShape`: a stalk below a stalk with a greater age takes the other age
/// (the tick it schedules where it cannot survive is not state).
pub(super) fn bamboo_stalk(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let above = ctx.state(offset(pos, direction));
    let age = |state| property(state, "age").and_then(|age| age.parse::<u8>().ok());
    if direction == Direction::Up && is_bamboo(above) && age(above) > age(state) {
        let next = if age(state) == Some(0) { "1" } else { "0" };
        return with_property(state, "age", next).unwrap_or(state);
    }
    state
}

fn sturdy(state: BlockStateId, face: Direction) -> bool {
    crate::block::state(state).is_some_and(|info| info.is_face_sturdy(face))
}

/// `BellBlock.updateShape`: a bell between two walls becomes single where the neighbour on the
/// axis no longer holds it, and a single one becomes double when the side opposite its support
/// gets a block that does.
pub(super) fn bell(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let attachment = property(state, "attachment");
    let facing = facing(state).unwrap_or(Direction::North);
    // `getConnectedDirection(state).getOpposite()`: the way the bell hangs from.
    let connected = match attachment {
        Some("floor") => Direction::Down,
        Some("ceiling") => Direction::Up,
        _ => facing,
    };
    if connected == direction
        && attachment != Some("double_wall")
        && !super::survive::bell_survives(state, pos, ctx)
    {
        return air();
    }
    if direction.axis() != facing.axis() {
        return state;
    }
    bell_wall(
        state,
        (facing, connected),
        direction,
        ctx.state(offset(pos, direction)),
    )
}

/// The wall half of `BellBlock.updateShape`, for a neighbour on the bell's facing axis.
fn bell_wall(
    state: BlockStateId,
    (facing, connected): (Direction, Direction),
    direction: Direction,
    neighbour: BlockStateId,
) -> BlockStateId {
    match property(state, "attachment") {
        Some("double_wall") if !sturdy(neighbour, direction) => {
            with_property(state, "attachment", "single_wall")
                .and_then(|state| with_property(state, "facing", direction.opposite().name()))
                .unwrap_or(state)
        }
        Some("single_wall") if connected.opposite() == direction && sturdy(neighbour, facing) => {
            with_property(state, "attachment", "double_wall").unwrap_or(state)
        }
        _ => state,
    }
}
