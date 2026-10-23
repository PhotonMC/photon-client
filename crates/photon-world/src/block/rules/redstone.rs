//! `RedStoneWireBlock.updateShape`: the wire reconnects to what changed beside or above it.

use super::growth::{HORIZONTAL, fill_lone_sides, is_conductor, offset, wire_side, wire_stands_on};
use super::*;

fn connected(side: Option<&str>) -> bool {
    side.is_some_and(|side| side != "none")
}

fn is_cross(state: BlockStateId) -> bool {
    HORIZONTAL
        .iter()
        .all(|direction| connected(property(state, direction.name())))
}

fn is_dot(state: BlockStateId) -> bool {
    HORIZONTAL
        .iter()
        .all(|direction| !connected(property(state, direction.name())))
}

/// `RedStoneWireBlock.getConnectionState`: every side recomputed, a dot staying a dot and a wire
/// with a lone side or axis completed to a line. The power is kept.
fn connection_state(
    power: &str,
    was_dot: bool,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let can_connect_up = !is_conductor(ctx.state(offset(pos, Direction::Up)));
    let sides = HORIZONTAL.map(|direction| wire_side(pos, direction, can_connect_up, ctx));
    let sides = if was_dot && sides.iter().all(|side| *side == "none") {
        sides
    } else {
        fill_lone_sides(sides)
    };
    let mut state = with_property(
        registry::default_state_named("minecraft:redstone_wire"),
        "power",
        power,
    )?;
    for (direction, side) in HORIZONTAL.iter().zip(sides) {
        state = with_property(state, direction.name(), side)?;
    }
    Some(state)
}

pub(super) fn update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    if super::survive::has_dynamic_neighbour(pos, ctx) {
        return Rule::Unported;
    }
    let Some(power) = property(state, "power") else {
        return Rule::Known(state);
    };
    let rebuilt = match direction {
        Direction::Down => {
            let stands = wire_stands_on(ctx.state(offset(pos, direction)));
            return Rule::Known(if stands {
                state
            } else {
                registry::default_state_named("minecraft:air")
            });
        }
        Direction::Up => connection_state(power, is_dot(state), pos, ctx),
        _ => {
            let can_connect_up = !is_conductor(ctx.state(offset(pos, Direction::Up)));
            let side = wire_side(pos, direction, can_connect_up, ctx);
            let name = direction.name();
            if connected(Some(side)) == connected(property(state, name)) && !is_cross(state) {
                with_property(state, name, side)
            } else {
                connection_state(power, false, pos, ctx)
            }
        }
    };
    Rule::Known(rebuilt.unwrap_or(state))
}
