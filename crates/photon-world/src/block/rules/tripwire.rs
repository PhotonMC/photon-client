//! `TripWireHookBlock.setPlacedBy`: a hook put down at the end of a tripwire line pairs with the
//! hook at the other end and attaches the string between them.

use super::item::Write;
use super::*;

/// How far along the line `calculateState` looks (the reference's `42`, from one).
const REACH: i32 = 42;

const HOOK: &str = concat!("minecraft:", "tripwire_hook");
const STRING: &str = concat!("minecraft:", "tripwire");

fn is_named(state: BlockStateId, name: &str) -> bool {
    block(state).is_some_and(|block| block.name == name)
}

fn is_set(state: BlockStateId, name: &str) -> bool {
    property(state, name) == Some("true")
}

/// What a hook finds along its line: whether the string is attached and powered, how far the
/// hook that faces back is (zero for none), and the string states passed on the way.
struct Line {
    attached: bool,
    powered: bool,
    receiver: i32,
    wires: Vec<Option<BlockStateId>>,
}

/// The scan of `TripWireHookBlock.calculateState` along the line a hook faces.
fn scan(pos: (i32, i32, i32), direction: Direction, ctx: &PlaceContext<'_>) -> Line {
    let mut line = Line {
        attached: true,
        powered: false,
        receiver: 0,
        wires: vec![None; REACH as usize],
    };
    for i in 1..REACH {
        let state = ctx.state(step(pos, direction, i));
        if is_named(state, HOOK) {
            if super::super::piston::facing(state) == Some(direction.opposite()) {
                line.receiver = i;
            }
            break;
        }
        if is_named(state, STRING) {
            line.powered |= !is_set(state, "disarmed") && is_set(state, "powered");
            line.wires[i as usize] = Some(state);
        } else {
            line.attached = false;
        }
    }
    line.attached &= line.receiver > 1;
    line.powered &= line.attached;
    line
}

/// `TripWireHookBlock.calculateState` for a placement (no wire source, nothing destroyed): the
/// writes after the hook is down, in the order the reference makes them. The receiver hook goes
/// first, then this hook, then the string between them when the line's attachment changed.
pub(super) fn placed_hook(
    placed: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Vec<Write> {
    let (Some(direction), Some((index, _))) =
        (super::super::piston::facing(placed), block_index(placed))
    else {
        return Vec::new();
    };
    let line = scan(pos, direction, ctx);
    let hook = |facing: Direction| {
        with_property(default_state(index), "attached", flag(line.attached))
            .and_then(|state| with_property(state, "powered", flag(line.powered)))
            .and_then(|state| with_property(state, "facing", facing.name()))
    };
    let write = |at, state| Write {
        pos: at,
        state,
        flags: 3,
    };
    let mut writes = Vec::new();
    let receiver = (line.receiver > 0)
        .then(|| hook(direction.opposite()))
        .flatten();
    writes.extend(receiver.map(|state| write(step(pos, direction, line.receiver), state)));
    writes.extend(hook(direction).map(|state| write(pos, state)));
    if is_set(placed, "attached") != line.attached {
        writes.extend(
            attached_string(&line, (pos, direction), ctx)
                .into_iter()
                .map(|(at, state)| write(at, state)),
        );
    }
    writes
}

/// The string between the hooks, set to the line's attachment where it is still string.
fn attached_string(
    line: &Line,
    (pos, direction): ((i32, i32, i32), Direction),
    ctx: &PlaceContext<'_>,
) -> Vec<((i32, i32, i32), BlockStateId)> {
    (1..line.receiver)
        .filter_map(|i| {
            let at = step(pos, direction, i);
            let current = ctx.state(at);
            let wire = line.wires[i as usize]
                .filter(|_| is_named(current, STRING) || is_named(current, HOOK))?;
            Some((at, with_property(wire, "attached", flag(line.attached))?))
        })
        .collect()
}

fn step(pos: (i32, i32, i32), direction: Direction, distance: i32) -> (i32, i32, i32) {
    let (dx, dy, dz) = direction.step();
    (
        pos.0 + dx * distance,
        pos.1 + dy * distance,
        pos.2 + dz * distance,
    )
}
