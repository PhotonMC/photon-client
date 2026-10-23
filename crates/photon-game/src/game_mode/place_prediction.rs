//! The part of `BlockItem.place` the client predicts before it changes the level.

use super::*;

/// What `BlockItem.place` decides before it touches the level: whether the clicked block gives
/// way, the state the block takes and whether it may stand there. `Err` is the result of the
/// use where the prediction ends: a refusal, or a rule the client does not predict yet.
pub(super) fn predicted_state(
    kind: rules::ItemPlacement,
    block: usize,
    place_ctx: &mut rules::PlaceContext<'_>,
) -> Result<((i32, i32, i32), photon_core::ids::BlockStateId), UseResult> {
    let unpredicted = UseResult::Success { swing: true };
    let at =
        |ctx: &rules::PlaceContext<'_>, pos: (i32, i32, i32)| (ctx.state_at)(pos.0, pos.1, pos.2);
    // `BlockPlaceContext`: the clicked block is replaced if it may be.
    let clicked = at(place_ctx, place_ctx.hit_pos);
    place_ctx.replace_clicked = match rules::can_be_replaced(clicked, place_ctx) {
        Rule::Known(replaced) => replaced,
        Rule::Unported => return Err(unpredicted),
    };
    let pos = place_ctx.clicked_pos();
    // `canPlace`.
    if !place_ctx.replace_clicked {
        match rules::can_be_replaced(at(place_ctx, pos), place_ctx) {
            Rule::Known(true) => {}
            Rule::Known(false) => return Err(UseResult::Fail),
            Rule::Unported => return Err(unpredicted),
        }
    }
    // `updatePlacementContext`: scaffolding moves the click along its stack.
    match rules::update_context(kind, place_ctx) {
        Rule::Known(Some(updated)) => *place_ctx = updated,
        Rule::Known(None) => return Err(UseResult::Fail),
        Rule::Unported => return Err(unpredicted),
    }
    let pos = place_ctx.clicked_pos();
    // `getPlacementState`, with the survival check `canPlace` makes.
    match rules::placement_state(kind, block, place_ctx) {
        Rule::Known(Some(state)) => Ok((pos, state)),
        Rule::Known(None) => Err(UseResult::Fail),
        Rule::Unported => Err(unpredicted),
    }
}

/// The build height and the world border of the level the player is in.
pub(super) fn limits_of(world: &ClientWorld) -> rules::Limits {
    let dimension = world.dimension();
    let border = world.world_border();
    rules::Limits {
        max_y: dimension.min_y + dimension.height - 1,
        border: [
            border.min_x(0.0),
            border.max_x(0.0),
            border.min_z(0.0),
            border.max_z(0.0),
        ],
    }
}
