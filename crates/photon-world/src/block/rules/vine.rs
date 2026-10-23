//! `VineBlock.updateShape`: a vine loses the faces that no longer hold on to anything.

use super::growth::{HORIZONTAL, can_attach_to, has_face, offset, same_block, vine_supports};
use super::*;

/// `VineBlock.getUpdatedState`: the top face stays only under a block it can attach to, and a
/// side face only where it is supported or a vine above has the same face.
fn updated_state(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let Some((own, _)) = block_index(state) else {
        return state;
    };
    let above_pos = offset(pos, Direction::Up);
    let mut state = state;
    if has_face(state, Direction::Up) {
        let held = can_attach_to(Direction::Down, ctx.state(above_pos));
        state = with_property(state, "up", flag(held)).unwrap_or(state);
    }
    let above = ctx.state(above_pos);
    for direction in HORIZONTAL {
        if has_face(state, direction) {
            let held = vine_supports(own, pos, direction, ctx)
                || (same_block(above, own) && has_face(above, direction));
            state = with_property(state, direction.name(), flag(held)).unwrap_or(state);
        }
    }
    state
}

/// `VineBlock.updateShape`: below changes nothing; any other side recomputes the faces, and a vine
/// with none left is air.
pub(super) fn update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction == Direction::Down {
        return state;
    }
    let updated = updated_state(state, pos, ctx);
    let faces = [Direction::Up]
        .into_iter()
        .chain(HORIZONTAL)
        .any(|face| has_face(updated, face));
    if faces {
        updated
    } else {
        registry::default_state_named("minecraft:air")
    }
}
