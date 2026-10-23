//! The blocks that lie on the ground or in water and check what is above or below them: cake,
//! dirt paths, farmland, seagrass and the grindstone.

use super::placement::Placement;
use super::*;

fn above(pos: (i32, i32, i32)) -> (i32, i32, i32) {
    (pos.0, pos.1 + 1, pos.2)
}

fn solid_at(ctx: &PlaceContext<'_>, pos: (i32, i32, i32)) -> bool {
    crate::block::state(ctx.state(pos)).is_some_and(|info| info.solid)
}

/// `CakeBlock.canSurvive`: the block below is solid.
fn cake_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    solid_at(ctx, (pos.0, pos.1 - 1, pos.2))
}

/// `DirtPathBlock.canSurvive`: nothing solid above, or a fence gate.
fn path_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let over = ctx.state(above(pos));
    !solid_at(ctx, above(pos))
        || block(over).is_some_and(|block| block.class == BlockClass::FenceGateBlock)
}

/// `FarmlandBlock.canSurvive`: nothing solid above, or a plant that keeps the soil.
fn farmland_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let over = block_index(ctx.state(above(pos)));
    !solid_at(ctx, above(pos))
        || over.is_some_and(|(index, _)| ctx.tags.contains(index, "minecraft:maintains_farmland"))
}

/// `canSurvive` of the classes of this file.
pub(super) fn survives(
    class: BlockClass,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    match class {
        BlockClass::GrindstoneBlock => Rule::Known(true),
        BlockClass::CakeBlock => Rule::Known(cake_survives(pos, ctx)),
        BlockClass::DirtPathBlock => Rule::Known(path_survives(pos, ctx)),
        BlockClass::FarmlandBlock => Rule::Known(farmland_survives(pos, ctx)),
        _ => Rule::Unported,
    }
}

/// `SeagrassBlock.mayPlaceOn`: a sturdy top that is not one of the blocks that cannot hold it.
pub(super) fn seagrass_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let below = ctx.state((pos.0, pos.1 - 1, pos.2));
    let sturdy = crate::block::state(below).is_some_and(|info| info.is_face_sturdy(Direction::Up));
    sturdy
        && !block_index(below).is_some_and(|(index, _)| {
            ctx.tags
                .contains(index, "minecraft:cannot_support_seagrass")
        })
}

/// `DirtPathBlock` and `FarmlandBlock.getStateForPlacement`: dirt where the block could not stay.
fn soil(survives: bool, default: BlockStateId) -> Rule<Option<BlockStateId>> {
    if survives {
        return Rule::Known(Some(default));
    }
    Rule::Known(registry::block_index("minecraft:dirt").map(default_state))
}

/// `SeagrassBlock.getStateForPlacement`: only into a full block of water.
fn seagrass(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let place = ctx.state(ctx.clicked_pos());
    let fluid = crate::block::state_or_air(place).fluid;
    (is_water(place) && fluid.amount == 8).then_some(default)
}

/// The rules of this file, by the class that declares `getStateForPlacement`.
pub(super) const PLACEMENTS: &[(BlockClass, Placement)] = &[
    (BlockClass::DirtPathBlock, |_, default, ctx| {
        soil(path_survives(ctx.clicked_pos(), ctx), default)
    }),
    (BlockClass::FarmlandBlock, |_, default, ctx| {
        soil(farmland_survives(ctx.clicked_pos(), ctx), default)
    }),
    (BlockClass::SeagrassBlock, |_, default, ctx| {
        Rule::Known(seagrass(default, ctx))
    }),
];
