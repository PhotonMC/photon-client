//! The part of `BlockItem.place` the client predicts before it changes the level.

use super::*;

/// What `BlockItem.place` decides before it touches the level: whether the clicked block gives
/// way, the state the block takes and whether it may stand there. `Err` is the result of the
/// use where the prediction ends: a refusal, or a rule the client does not predict yet.
pub(super) fn predicted_state(
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
    let state = match rules::state_for_placement(block, place_ctx) {
        Rule::Known(Some(state)) => state,
        Rule::Known(None) => return Err(UseResult::Fail),
        Rule::Unported => return Err(unpredicted),
    };
    // `BlockItem.canPlace`: it must stand there and not be in the way.
    match rules::can_survive(state, pos, place_ctx) {
        Rule::Known(true) => Ok((pos, state)),
        Rule::Known(false) => Err(UseResult::Fail),
        Rule::Unported => Err(unpredicted),
    }
}
