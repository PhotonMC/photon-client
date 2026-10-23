//! `SignalGetter.hasNeighborSignal` and what it reads: the signal a block gives towards a
//! direction, directly or through a conductor. A block whose signal is not written here makes the
//! whole query [`Rule::Unported`].

use super::*;

/// The signal of a block that gives one, `None` where its class is not written.
fn own_signal(state: BlockStateId) -> Option<i32> {
    let powered = || i32::from(property(state, "powered") == Some("true")) * 15;
    match state_owner(state, Method::OwnSignal) {
        None => Some(0),
        Some(BlockClass::PoweredBlock) => Some(15),
        Some(BlockClass::LeverBlock | BlockClass::ButtonBlock) => Some(powered()),
        Some(BlockClass::BasePressurePlateBlock) => match property(state, "power") {
            Some(power) => power.parse().ok(),
            None => Some(powered()),
        },
        Some(BlockClass::RedstoneTorchBlock) => {
            Some(i32::from(property(state, "lit") == Some("true")) * 15)
        }
        Some(BlockClass::ObserverBlock) => Some(powered()),
        Some(BlockClass::DiodeBlock) => diode_signal(state),
        Some(_) => None,
    }
}

/// `DiodeBlock.ownSignal`: a comparator's output is held by its block entity.
fn diode_signal(state: BlockStateId) -> Option<i32> {
    if property(state, "powered") != Some("true") {
        return Some(0);
    }
    let comparator = block(state).is_some_and(|b| b.class.is(BlockClass::ComparatorBlock));
    (!comparator).then_some(15)
}

/// `BlockState.getSignal`: the signal given towards `direction`.
fn signal_of(state: BlockStateId, direction: Direction) -> Option<i32> {
    let facing = facing(state);
    match state_owner(state, Method::GetSignal) {
        None => own_signal(state),
        Some(BlockClass::RedstoneTorchBlock) => {
            if direction == Direction::Up {
                Some(0)
            } else {
                own_signal(state)
            }
        }
        Some(BlockClass::RedstoneWallTorchBlock) => {
            if facing == Some(direction) {
                Some(0)
            } else {
                own_signal(state)
            }
        }
        Some(BlockClass::ObserverBlock | BlockClass::DiodeBlock) => {
            if facing == Some(direction) {
                own_signal(state)
            } else {
                Some(0)
            }
        }
        // A powered wire gives to the sides it connects to, which needs its surroundings.
        Some(BlockClass::RedStoneWireBlock) => match property(state, "power") {
            Some("0") => Some(0),
            _ => None,
        },
        Some(_) => None,
    }
}

/// `BlockState.getDirectSignal`.
fn direct_signal_of(state: BlockStateId, direction: Direction) -> Option<i32> {
    match state_owner(state, Method::GetDirectSignal) {
        None => Some(0),
        Some(BlockClass::LeverBlock | BlockClass::ButtonBlock) => {
            if super::survive::connected_direction(state) == direction {
                own_signal(state)
            } else {
                Some(0)
            }
        }
        Some(BlockClass::BasePressurePlateBlock) => {
            if direction == Direction::Up {
                own_signal(state)
            } else {
                Some(0)
            }
        }
        Some(BlockClass::RedstoneTorchBlock) => {
            if direction == Direction::Down {
                signal_of(state, direction)
            } else {
                Some(0)
            }
        }
        Some(BlockClass::ObserverBlock | BlockClass::DiodeBlock) => signal_of(state, direction),
        Some(BlockClass::RedStoneWireBlock) => signal_of(state, direction),
        Some(_) => None,
    }
}

fn next(pos: (i32, i32, i32), direction: Direction) -> (i32, i32, i32) {
    let (dx, dy, dz) = direction.step();
    (pos.0 + dx, pos.1 + dy, pos.2 + dz)
}

/// `SignalGetter.getDirectSignalTo`, which stops once it reaches 15.
fn direct_signal_to(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> Option<i32> {
    let mut result = 0;
    for direction in Direction::stream() {
        let at = ctx.state(next(pos, direction));
        result = result.max(direct_signal_of(at, direction)?);
        if result >= 15 {
            break;
        }
    }
    Some(result)
}

/// `SignalGetter.getSignal`: the signal at `pos` towards `direction`, and what a conductor there
/// passes on from its neighbours.
fn signal_at(pos: (i32, i32, i32), direction: Direction, ctx: &PlaceContext<'_>) -> Option<i32> {
    let state = ctx.state(pos);
    let signal = signal_of(state, direction)?;
    let conductor = crate::block::state(state).is_some_and(|info| info.redstone_conductor);
    if conductor {
        Some(signal.max(direct_signal_to(pos, ctx)?))
    } else {
        Some(signal)
    }
}

/// `SignalGetter.hasNeighborSignal`.
pub(super) fn has_neighbor_signal(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> Rule<bool> {
    for direction in Direction::stream() {
        match signal_at(next(pos, direction), direction, ctx) {
            None => return Rule::Unported,
            Some(signal) if signal > 0 => return Rule::Known(true),
            Some(_) => {}
        }
    }
    Rule::Known(false)
}
