//! Chorus plants and flowers, and the coral blocks that ask for water: the rules of the blocks
//! that join or lean on others of their kind.

use super::placement::Placement;
use super::*;
use photon_core::direction::Plane;

fn offset(pos: (i32, i32, i32), direction: Direction) -> (i32, i32, i32) {
    let (dx, dy, dz) = direction.step();
    (pos.0 + dx, pos.1 + dy, pos.2 + dz)
}

fn is_named(state: BlockStateId, name: &str) -> bool {
    block(state).is_some_and(|block| block.name == name)
}

fn same_block(state: BlockStateId, other: BlockStateId) -> bool {
    block_index(state).map(|(index, _)| index) == block_index(other).map(|(index, _)| index)
}

/// Whether the block at `pos` is in the block tag.
fn tagged(ctx: &PlaceContext<'_>, pos: (i32, i32, i32), tag: &str) -> bool {
    block_index(ctx.state(pos)).is_some_and(|(index, _)| ctx.tags.contains(index, tag))
}

/// `ChorusPlantBlock.getStateWithConnections`: a side joins a chorus plant of the same kind or a
/// flower, and the underside also a block the plant grows from.
fn plant_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let joins = |direction: Direction| {
        let there = ctx.state(offset(pos, direction));
        same_block(there, default) || is_named(there, "minecraft:chorus_flower")
    };
    let mut state = default;
    for direction in Direction::stream() {
        let joined = joins(direction)
            || (direction == Direction::Down
                && tagged(
                    ctx,
                    offset(pos, direction),
                    "minecraft:supports_chorus_plant",
                ));
        state = with_property(state, direction.name(), flag(joined))?;
    }
    Some(state)
}

/// `ChorusPlantBlock.canSurvive`: standing on its own kind or on a block it grows from, or
/// leaning on a neighbouring plant that does, with nothing above and below at once.
pub(super) fn plant_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let supports = |at: (i32, i32, i32)| {
        same_block(ctx.state(at), state) || tagged(ctx, at, "minecraft:supports_chorus_plant")
    };
    let below = offset(pos, Direction::Down);
    let block_above_or_below =
        !is_air(ctx.state(offset(pos, Direction::Up))) && !is_air(ctx.state(below));
    for direction in Plane::Horizontal.directions() {
        let beside = offset(pos, *direction);
        if same_block(ctx.state(beside), state) {
            if block_above_or_below {
                return false;
            }
            if supports(offset(beside, Direction::Down)) {
                return true;
            }
        }
    }
    supports(below)
}

/// `ChorusPlantBlock.updateShape`: a plant that can no longer stand keeps its state (it is
/// ticked away); any other joins or leaves the changed neighbour.
pub(super) fn plant_update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if !plant_survives(state, pos, ctx) {
        return state;
    }
    let at = offset(pos, direction);
    let there = ctx.state(at);
    let joined = same_block(there, state)
        || is_named(there, "minecraft:chorus_flower")
        || direction == Direction::Down && tagged(ctx, at, "minecraft:supports_chorus_plant");
    with_property(state, direction.name(), flag(joined)).unwrap_or(state)
}

/// `TripWireBlock.updateShape`: the side towards the changed neighbour.
pub(super) fn tripwire_update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction.axis() == Axis::Y {
        return state;
    }
    let there = ctx.state(offset(pos, direction));
    let joined = if is_named(there, "minecraft:tripwire_hook") {
        property(there, "facing") == Some(direction.opposite().name())
    } else {
        same_block(there, state)
    };
    with_property(state, direction.name(), flag(joined)).unwrap_or(state)
}

/// `ChorusFlowerBlock.canSurvive`: on its plant or a block it grows from, or in the air with
/// exactly one plant beside it and nothing else.
pub(super) fn flower_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let below = offset(pos, Direction::Down);
    let under = ctx.state(below);
    if is_named(under, "minecraft:chorus_plant")
        || tagged(ctx, below, "minecraft:supports_chorus_flower")
    {
        return true;
    }
    if !is_air(under) {
        return false;
    }
    let mut one_neighbour = false;
    for direction in Plane::Horizontal.directions() {
        let beside = ctx.state(offset(pos, *direction));
        if is_named(beside, "minecraft:chorus_plant") {
            if one_neighbour {
                return false;
            }
            one_neighbour = true;
        } else if !is_air(beside) {
            return false;
        }
    }
    one_neighbour
}

/// `CreakingHeartBlock.getStateForPlacement`: the axis of the face clicked. Between two logs of
/// the axis it wakes by the time of day, which is not known here.
fn creaking_heart(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    let axis = match ctx.face.axis() {
        Axis::X => "x",
        Axis::Y => "y",
        Axis::Z => "z",
    };
    let Some(state) = with_property(default, "axis", axis) else {
        return Rule::Known(None);
    };
    let pos = ctx.clicked_pos();
    let between_logs = Direction::stream()
        .filter(|direction| direction.axis() == ctx.face.axis())
        .all(|direction| {
            let log = offset(pos, direction);
            tagged(ctx, log, "minecraft:pale_oak_logs")
                && property(ctx.state(log), "axis") == Some(axis)
        });
    if between_logs {
        Rule::Unported
    } else {
        Rule::Known(Some(state))
    }
}

/// `TripWireBlock.getStateForPlacement`: each side joins a tripwire or a hook facing it. The
/// block's `onPlace` updates the hooks along the line only on a server.
fn tripwire(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let mut state = default;
    for direction in Plane::Horizontal.directions() {
        let there = ctx.state(offset(pos, *direction));
        let joined = if is_named(there, "minecraft:tripwire_hook") {
            property(there, "facing") == Some(direction.opposite().name())
        } else {
            same_block(there, default)
        };
        state = with_property(state, direction.name(), flag(joined))?;
    }
    Some(state)
}

/// Whether the hook placed at `pos` finds a hook facing back along its line, within 41 blocks:
/// `TripWireHookBlock.calculateState` then also sets that hook, plays a sound and attaches the
/// wire, which is not reproduced. The scan stops at the first hook of any facing.
pub(super) fn hook_has_receiver(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let Some(facing) = property(state, "facing").and_then(Direction::from_name) else {
        return false;
    };
    let (dx, dy, dz) = facing.step();
    (1..42).find_map(|i| {
        let there = ctx.state((pos.0 + dx * i, pos.1 + dy * i, pos.2 + dz * i));
        is_named(there, "minecraft:tripwire_hook")
            .then(|| property(there, "facing") == Some(facing.opposite().name()))
    }) == Some(true)
}

/// The rules of this file, by the class that declares `getStateForPlacement`. The coral block
/// also schedules a tick and draws from the level random when no water is near; a client runs no
/// ticks and its random is unseeded, so neither shows.
pub(super) const PLACEMENTS: &[(BlockClass, Placement)] = &[
    (BlockClass::CoralBlock, |_, default, _| {
        Rule::Known(Some(default))
    }),
    (BlockClass::ChorusPlantBlock, |_, default, ctx| {
        Rule::Known(plant_placement(default, ctx))
    }),
    (BlockClass::TripWireBlock, |_, default, ctx| {
        Rule::Known(tripwire(default, ctx))
    }),
    (BlockClass::CreakingHeartBlock, |_, default, ctx| {
        creaking_heart(default, ctx)
    }),
];
