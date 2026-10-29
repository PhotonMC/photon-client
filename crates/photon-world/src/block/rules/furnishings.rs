//! `getStateForPlacement` of the blocks that face the player or wait in water: shelves, furnaces,
//! anvils, pumpkins, vaults, looms, lecterns, campfires, pots, sculk sensors and the like.

use super::placement::{Placement, nearest_direction};
use super::*;

/// `FluidState.is(Fluids.WATER)` of the place: a water source.
fn in_water(ctx: &PlaceContext<'_>) -> &'static str {
    flag(is_water_source(ctx.state(ctx.clicked_pos())))
}

/// The default state facing `direction`.
fn facing_state(default: BlockStateId, direction: Direction) -> Option<BlockStateId> {
    with_property(default, "facing", direction.name())
}

/// `getHorizontalDirection().getOpposite()`: facing the player.
fn facing_player(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    facing_state(default, ctx.horizontal_direction.opposite())
}

/// The state, waterlogged by the water the place holds.
fn waterlogged(state: Option<BlockStateId>, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    with_property(state?, "waterlogged", in_water(ctx))
}

/// `DispenserBlock` and `BarrelBlock`: facing away from where the player looks.
fn facing_away(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    facing_state(default, nearest_direction(ctx).opposite())
}

/// `ShelfBlock`: facing the player, powered by a neighbour, waterlogged.
fn shelf(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    let Rule::Known(powered) = super::signal::has_neighbor_signal(ctx.clicked_pos(), ctx) else {
        return Rule::Unported;
    };
    Rule::Known(
        waterlogged(facing_player(default, ctx), ctx)
            .and_then(|state| with_property(state, "powered", flag(powered))),
    )
}

/// `RedstoneLampBlock`: lit by a neighbour's signal.
fn lamp(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    super::signal::has_neighbor_signal(ctx.clicked_pos(), ctx)
        .map(|lit| with_property(default, "lit", flag(lit)))
}

/// `CrafterBlock`: the front away from the look, the top as the look leaves it, triggered by a
/// neighbour's signal.
fn crafter(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    let Rule::Known(triggered) = super::signal::has_neighbor_signal(ctx.clicked_pos(), ctx) else {
        return Rule::Unported;
    };
    let front = nearest_direction(ctx).opposite();
    let top = match front {
        Direction::Down => ctx.horizontal_direction.opposite(),
        Direction::Up => ctx.horizontal_direction,
        _ => Direction::Up,
    };
    Rule::Known(
        with_property(
            default,
            "orientation",
            &format!("{}_{}", front.name(), top.name()),
        )
        .and_then(|state| with_property(state, "triggered", flag(triggered))),
    )
}

/// `CampfireBlock`: lit unless it replaces water, with a signal fire over hay.
fn campfire(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let in_water = is_water_source(ctx.state(pos));
    let below = ctx.state((pos.0, pos.1 - 1, pos.2));
    let on_hay =
        block(below).is_some_and(|block| block.index() == photon_data::block::ids::HAY_BLOCK);
    with_property(default, "waterlogged", flag(in_water))
        .and_then(|state| with_property(state, "signal_fire", flag(on_hay)))
        .and_then(|state| with_property(state, "lit", flag(!in_water)))
        .and_then(|state| facing_state(state, ctx.horizontal_direction))
}

/// `ConduitBlock`: waterlogged only by a full block of water.
fn conduit(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let fluid = crate::block::state_or_air(ctx.state(ctx.clicked_pos())).fluid;
    let full_water = is_water(ctx.state(ctx.clicked_pos())) && fluid.amount == 8;
    with_property(default, "waterlogged", flag(full_water))
}

/// `HugeMushroomBlock`: a face is drawn wherever the neighbour is not the same block.
fn huge_mushroom(
    block_index_placed: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let mut state = default;
    for direction in Direction::stream() {
        let (dx, dy, dz) = direction.step();
        let beside = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
        let same = block_index(beside).map(|(index, _)| index) == Some(block_index_placed);
        state = with_property(state, direction.name(), flag(!same))?;
    }
    Some(state)
}

/// The rules of this file, by the class that declares `getStateForPlacement`.
pub(super) const PLACEMENTS: &[(BlockClass, Placement)] = &[
    (BlockClass::ShelfBlock, |_, default, ctx| {
        shelf(default, ctx)
    }),
    (BlockClass::CopperGolemStatueBlock, |_, default, ctx| {
        Rule::Known(waterlogged(facing_player(default, ctx), ctx))
    }),
    (
        BlockClass::WaterloggedTransparentBlock,
        |_, default, ctx| Rule::Known(waterlogged(Some(default), ctx)),
    ),
    (BlockClass::AbstractFurnaceBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    (BlockClass::AnvilBlock, |_, default, ctx| {
        Rule::Known(facing_state(
            default,
            ctx.horizontal_direction.clockwise(Axis::Y),
        ))
    }),
    (BlockClass::CarvedPumpkinBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    (BlockClass::VaultBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    // A lectern holds no book when the client places it: that needs the server's stack data.
    (BlockClass::LecternBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    (BlockClass::LoomBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    (BlockClass::StonecutterBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    (BlockClass::BeehiveBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    (BlockClass::ChiseledBookShelfBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    (BlockClass::EndPortalFrameBlock, |_, default, ctx| {
        Rule::Known(facing_player(default, ctx))
    }),
    (BlockClass::DispenserBlock, |_, default, ctx| {
        Rule::Known(facing_away(default, ctx))
    }),
    (BlockClass::BarrelBlock, |_, default, ctx| {
        Rule::Known(facing_away(default, ctx))
    }),
    (BlockClass::HeavyCoreBlock, |_, default, ctx| {
        Rule::Known(waterlogged(Some(default), ctx))
    }),
    (BlockClass::SculkSensorBlock, |_, default, ctx| {
        Rule::Known(waterlogged(Some(default), ctx))
    }),
    (BlockClass::CalibratedSculkSensorBlock, |_, default, ctx| {
        Rule::Known(waterlogged(
            facing_state(default, ctx.horizontal_direction),
            ctx,
        ))
    }),
    (BlockClass::SculkShriekerBlock, |_, default, ctx| {
        Rule::Known(waterlogged(Some(default), ctx))
    }),
    (BlockClass::MangroveRootsBlock, |_, default, ctx| {
        Rule::Known(waterlogged(Some(default), ctx))
    }),
    (BlockClass::BarrierBlock, |_, default, ctx| {
        Rule::Known(waterlogged(Some(default), ctx))
    }),
    (BlockClass::DriedGhastBlock, |_, default, ctx| {
        Rule::Known(waterlogged(facing_player(default, ctx), ctx))
    }),
    (BlockClass::DecoratedPotBlock, |_, default, ctx| {
        Rule::Known(waterlogged(
            facing_state(default, ctx.horizontal_direction),
            ctx,
        ))
    }),
    (BlockClass::CampfireBlock, |_, default, ctx| {
        Rule::Known(campfire(default, ctx))
    }),
    (BlockClass::ConduitBlock, |_, default, ctx| {
        Rule::Known(conduit(default, ctx))
    }),
    (BlockClass::RedstoneLampBlock, |_, default, ctx| {
        lamp(default, ctx)
    }),
    (BlockClass::CrafterBlock, |_, default, ctx| {
        crafter(default, ctx)
    }),
    (BlockClass::InfestedRotatedPillarBlock, |_, default, ctx| {
        Rule::Known(super::placement::pillar_placement(default, ctx))
    }),
    (BlockClass::HugeMushroomBlock, |block, default, ctx| {
        Rule::Known(huge_mushroom(block, default, ctx))
    }),
];
