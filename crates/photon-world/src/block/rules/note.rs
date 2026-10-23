//! Note blocks: the instrument follows the blocks above and below.

use super::placement::Placement;
use super::*;

/// `NoteBlockInstrument.worksAboveNoteBlock`: the mob heads and the custom head.
fn works_above(instrument: &str) -> bool {
    matches!(
        instrument,
        "zombie" | "skeleton" | "creeper" | "dragon" | "wither_skeleton" | "piglin" | "custom_head"
    )
}

/// `BlockState.instrument` of the block at `pos`.
fn instrument_at(ctx: &PlaceContext<'_>, pos: (i32, i32, i32)) -> &'static str {
    block(ctx.state(pos)).map_or("harp", |block| block.instrument)
}

/// `NoteBlock.setInstrument`: a head above gives its sound, else the block below does.
pub(super) fn with_instrument(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let above = instrument_at(ctx, (pos.0, pos.1 + 1, pos.2));
    let below = instrument_at(ctx, (pos.0, pos.1 - 1, pos.2));
    let chosen = if works_above(above) {
        above
    } else if works_above(below) {
        "harp"
    } else {
        below
    };
    with_property(state, "instrument", chosen).unwrap_or(state)
}

/// `NoteBlock.updateShape`: a block above or below sets the instrument again.
pub(super) fn update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction.axis() == Axis::Y {
        with_instrument(state, pos, ctx)
    } else {
        state
    }
}

pub(super) const PLACEMENTS: &[(BlockClass, Placement)] =
    &[(BlockClass::NoteBlock, |_, default, ctx| {
        Rule::Known(Some(with_instrument(default, ctx.clicked_pos(), ctx)))
    })];
