//! `updateShape` of the growing plants: kelp, weeping, twisting and cave vines.

use super::growth::{growing_plant_survives, is_named, offset, plant_parts};
use super::*;

/// `GrowingPlantHeadBlock.updateShape`: a head with the plant ahead of it becomes the body
/// (cave vines keep their berries).
pub(super) fn head(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let Some((growth, head, body)) = block(state).and_then(|b| plant_parts(b.name)) else {
        return state;
    };
    let plant = |at: (i32, i32, i32)| {
        let state = ctx.state(at);
        is_named(state, head) || is_named(state, body)
    };
    let behind = direction == growth.opposite() && growing_plant_survives(state, pos, ctx);
    if !(behind && plant(offset(pos, growth)))
        && !(direction == growth && plant(offset(pos, direction)))
    {
        return state;
    }
    let converted = registry::default_state_named(body);
    match property(state, "berries") {
        Some(berries) => with_property(converted, "berries", berries).unwrap_or(converted),
        None => converted,
    }
}

/// `GrowingPlantBodyBlock.updateShape`: a body whose front is no longer the plant becomes a head
/// with a random age, which the client cannot predict.
pub(super) fn body(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    let Some((growth, head, body)) = block(state).and_then(|b| plant_parts(b.name)) else {
        return Rule::Known(state);
    };
    let ahead = ctx.state(offset(pos, direction));
    if direction == growth && !is_named(ahead, head) && !is_named(ahead, body) {
        return Rule::Unported;
    }
    Rule::Known(state)
}
