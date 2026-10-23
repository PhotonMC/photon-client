//! `FireBlock.updateShape`: fire is placed again from its surroundings, keeping its age.

use super::*;
use crate::block::fire::{can_burn, placement_state};

const ALL: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// `FireBlock.canSurvive`: a sturdy top below or anything beside that burns.
fn survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let below = ctx.state((pos.0, pos.1 - 1, pos.2));
    crate::block::state(below).is_some_and(|info| info.is_face_sturdy(Direction::Up))
        || ALL
            .into_iter()
            .any(|direction| can_burn(ctx.state(growth::offset(pos, direction))))
}

/// A fire that cannot survive becomes air, which a client never applies, so it stays as it is.
pub(super) fn update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if !survives(pos, ctx) {
        return state;
    }
    let state_at = |(x, y, z)| (ctx.state_at)(x, y, z);
    let Some(placed) = placement_state(state_at, pos) else {
        return state;
    };
    match property(state, "age") {
        Some(age) if property(placed, "age").is_some() => {
            with_property(placed, "age", age).unwrap_or(placed)
        }
        _ => placed,
    }
}
