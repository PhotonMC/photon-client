//! Pointed dripstone and sulfur spikes: they point away from the block they hang from or stand
//! on, and thin towards the tip.

use super::placement::Placement;
use super::*;

fn offset(pos: (i32, i32, i32), direction: Direction) -> (i32, i32, i32) {
    let (dx, dy, dz) = direction.step();
    (pos.0 + dx, pos.1 + dy, pos.2 + dz)
}

/// `isSpeleothemWithDirection`: a block of the speleothem tag whose tip points `tip`.
fn with_direction(ctx: &PlaceContext<'_>, state: BlockStateId, tip: Direction) -> bool {
    block_index(state).is_some_and(|(index, _)| ctx.tags.contains(index, "minecraft:speleothems"))
        && property(state, "vertical_direction") == Some(tip.name())
}

fn same_block(a: BlockStateId, b: BlockStateId) -> bool {
    matches!((block_index(a), block_index(b)), (Some(a), Some(b)) if a.0 == b.0)
}

/// `isValidSpeleothemPlacement`: a sturdy face, or a speleothem of the same block, behind.
fn valid_placement(
    this: BlockStateId,
    pos: (i32, i32, i32),
    tip: Direction,
    ctx: &PlaceContext<'_>,
) -> bool {
    let behind = ctx.state(offset(pos, tip.opposite()));
    crate::block::state(behind).is_some_and(|info| info.is_face_sturdy(tip))
        || with_direction(ctx, behind, tip) && same_block(behind, this)
}

/// `calculateSpeleothemThickness`.
fn thickness(
    this: BlockStateId,
    pos: (i32, i32, i32),
    tip: Direction,
    merge: bool,
    ctx: &PlaceContext<'_>,
) -> &'static str {
    let base = tip.opposite();
    let in_front = ctx.state(offset(pos, tip));
    if with_direction(ctx, in_front, base) && same_block(in_front, this) {
        return if !merge && property(in_front, "thickness") != Some("tip_merge") {
            "tip"
        } else {
            "tip_merge"
        };
    }
    if !with_direction(ctx, in_front, tip) {
        return "tip";
    }
    if matches!(property(in_front, "thickness"), Some("tip" | "tip_merge")) {
        return "frustum";
    }
    let behind = ctx.state(offset(pos, base));
    if with_direction(ctx, behind, tip) {
        "middle"
    } else {
        "base"
    }
}

/// `SpeleothemBlock.getStateForPlacement`: pointing away from the player's look if a block holds
/// it that way, else the other way.
fn placement(
    _: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    let wanted = super::placement::nearest_vertical(ctx).opposite();
    let holds = |tip| valid_placement(default, pos, tip, ctx);
    let tip = if holds(wanted) {
        wanted
    } else if holds(wanted.opposite()) {
        wanted.opposite()
    } else {
        return Rule::Known(None);
    };
    let state = with_property(default, "vertical_direction", tip.name())
        .and_then(|state| {
            with_property(
                state,
                "thickness",
                thickness(default, pos, tip, !ctx.secondary_use, ctx),
            )
        })
        .and_then(|state| {
            with_property(state, "waterlogged", flag(is_water_source(ctx.state(pos))))
        });
    Rule::Known(state)
}

pub(super) const PLACEMENTS: &[(BlockClass, Placement)] =
    &[(BlockClass::SpeleothemBlock, placement)];

/// `SpeleothemBlock.canSurvive`.
pub(super) fn survives(state: BlockStateId, pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    Direction::from_name(property(state, "vertical_direction").unwrap_or("up"))
        .is_some_and(|tip| valid_placement(state, pos, tip, ctx))
}

/// `SpeleothemBlock.updateShape`: the thickness follows a block above or below. One that lost its
/// holder keeps its state and is ticked away.
pub(super) fn update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let Some(tip) = property(state, "vertical_direction").and_then(Direction::from_name) else {
        return state;
    };
    if direction.axis() != Axis::Y || direction == tip.opposite() && !survives(state, pos, ctx) {
        return state;
    }
    let merge = property(state, "thickness") == Some("tip_merge");
    with_property(state, "thickness", thickness(state, pos, tip, merge, ctx)).unwrap_or(state)
}
