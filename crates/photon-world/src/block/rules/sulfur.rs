//! Potent sulfur: dry, wet or a geyser, by the water above and the block below.

use super::placement::Placement;
use super::*;

/// `BlockState.is(tag)` of the block at `pos`.
fn tagged(ctx: &PlaceContext<'_>, pos: (i32, i32, i32), tag: &str) -> bool {
    block_index(ctx.state(pos)).is_some_and(|(index, _)| ctx.tags.contains(index, tag))
}

/// `PotentSulfurBlock.validBlockState`: the state the water above and the block below give.
pub(super) fn valid_state(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    let below = (pos.0, pos.1 - 1, pos.2);
    let fluid = crate::block::state_or_air(ctx.state(below)).fluid;
    let source_if_fluid = fluid.is_empty() || fluid.is_source();
    let chosen = if !is_water_source(ctx.state((pos.0, pos.1 + 1, pos.2))) {
        "dry"
    } else if tagged(ctx, below, "minecraft:causes_continuous_geyser_eruptions") && source_if_fluid
    {
        "continuous"
    } else if tagged(ctx, below, "minecraft:causes_periodic_geyser_eruptions") && source_if_fluid {
        if property(state, "potent_sulfur_state") == Some("erupting") {
            return state;
        }
        "dormant"
    } else {
        "wet"
    };
    with_property(state, "potent_sulfur_state", chosen).unwrap_or(state)
}

pub(super) const PLACEMENTS: &[(BlockClass, Placement)] =
    &[(BlockClass::PotentSulfurBlock, |_, default, ctx| {
        Rule::Known(Some(valid_state(default, ctx.clicked_pos(), ctx)))
    })];
