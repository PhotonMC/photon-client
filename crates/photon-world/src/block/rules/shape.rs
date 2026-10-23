//! `BlockBehaviour.updateShape`: how a block reshapes when a neighbour changes.

use super::*;

/// The state of the block at `pos` after the neighbour in `direction` changed, as the class that
/// declares `updateShape` decides it. A class whose rule is not written is `Unported`.
pub(super) fn update_shape(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    let Some(class) = state_owner(state, Method::UpdateShape) else {
        return Rule::Known(state);
    };
    match class {
        BlockClass::StairBlock => Rule::Known(stairs(state, pos, direction, ctx)),
        BlockClass::SpeleothemBlock => {
            Rule::Known(super::speleothem::update(state, pos, direction, ctx))
        }
        BlockClass::NoteBlock => Rule::Known(super::note::update(state, pos, direction, ctx)),
        BlockClass::PotentSulfurBlock => Rule::Known(super::sulfur::valid_state(state, pos, ctx)),
        BlockClass::SnowyBlock => Rule::Known(snowy(state, pos, direction, ctx)),
        BlockClass::ChorusPlantBlock => {
            Rule::Known(super::chorus::plant_update(state, pos, direction, ctx))
        }
        BlockClass::TripWireBlock => {
            Rule::Known(super::chorus::tripwire_update(state, pos, direction, ctx))
        }
        BlockClass::BigDripleafBlock => {
            Rule::Known(super::dripleaf::big_update(state, pos, direction, ctx))
        }
        _ => connecting(class, state, pos, direction, ctx),
    }
}

/// The classes that join their neighbours.
fn connecting(
    class: BlockClass,
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> Rule<BlockStateId> {
    match class {
        BlockClass::FenceBlock => {
            Rule::Known(super::connect::fence_update(state, pos, direction, ctx))
        }
        BlockClass::IronBarsBlock => {
            Rule::Known(super::connect::bars_update(state, pos, direction, ctx))
        }
        BlockClass::WallBlock => super::connect::wall_update(state, pos, direction, ctx),
        BlockClass::FenceGateBlock => {
            Rule::Known(super::connect::gate_update(state, pos, direction, ctx))
        }
        _ => Rule::Unported,
    }
}

/// `StairBlock.updateShape`: a side neighbour reshapes the corner.
fn stairs(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction.axis() == Axis::Y {
        return state;
    }
    let shape = stairs_shape(state, ctx.state_at, pos);
    with_property(state, "shape", shape).unwrap_or(state)
}

/// `SnowyBlock.updateShape`: snowy under a block of the snow tag.
fn snowy(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction != Direction::Up {
        return state;
    }
    let above = ctx.state((pos.0, pos.1 + 1, pos.2));
    let snowy =
        block_index(above).is_some_and(|(index, _)| ctx.tags.contains(index, "minecraft:snow"));
    with_property(state, "snowy", flag(snowy)).unwrap_or(state)
}
