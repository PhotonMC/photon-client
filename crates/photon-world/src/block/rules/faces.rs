//! `updateShape` of the blocks that hold on to the faces of their neighbours: lichen, sculk
//! veins and the pale moss carpet.

use super::growth::{can_attach_to, has_face, offset};
use super::*;

fn air() -> BlockStateId {
    registry::default_state_named("minecraft:air")
}

/// `MultifaceBlock.updateShape`: a face whose neighbour can no longer hold it goes, and a block
/// left without a face is air.
pub(super) fn multiface(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let any = Direction::stream().any(|face| has_face(state, face));
    if !any {
        return air();
    }
    let held = can_attach_to(direction, ctx.state(offset(pos, direction)));
    if !has_face(state, direction) || held {
        return state;
    }
    let removed = with_property(state, direction.name(), "false").unwrap_or(state);
    if Direction::stream().any(|face| has_face(removed, face)) {
        removed
    } else {
        air()
    }
}

/// `MossyCarpetBlock.updateShape`: air where it cannot survive or has nothing left, otherwise
/// the sides recomputed.
pub(super) fn carpet(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if !super::ground::carpet_survives(state, pos, ctx) {
        return air();
    }
    let Some(updated) = super::ground::carpet_updated(state, pos, ctx, false) else {
        return state;
    };
    let sides = super::growth::HORIZONTAL
        .into_iter()
        .any(|side| property(updated, side.name()) != Some("none"));
    if property(updated, "bottom") == Some("true") || sides {
        updated
    } else {
        air()
    }
}
