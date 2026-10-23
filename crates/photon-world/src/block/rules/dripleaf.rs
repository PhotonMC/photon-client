//! Big and small dripleaves: they face the player or the dripleaf below, and stand on clay-like
//! ground.

use super::placement::Placement;
use super::*;

fn below(pos: (i32, i32, i32)) -> (i32, i32, i32) {
    (pos.0, pos.1 - 1, pos.2)
}

fn tagged(ctx: &PlaceContext<'_>, pos: (i32, i32, i32), tag: &str) -> bool {
    block_index(ctx.state(pos)).is_some_and(|(index, _)| ctx.tags.contains(index, tag))
}

fn named(state: BlockStateId, name: &str) -> bool {
    block(state).is_some_and(|block| block.name == name)
}

/// `FluidState.isSourceOfType(WATER)` of the block at `pos`.
fn water_source(ctx: &PlaceContext<'_>, pos: (i32, i32, i32)) -> bool {
    is_water_source(ctx.state(pos))
}

/// `BigDripleafBlock.getStateForPlacement`: waterlogged by a source, facing the way a dripleaf
/// below faces, otherwise towards the player.
fn big_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let under = ctx.state(below(pos));
    let facing =
        if named(under, "minecraft:big_dripleaf") || named(under, "minecraft:big_dripleaf_stem") {
            property(under, "facing")?
        } else {
            ctx.horizontal_direction.opposite().name()
        };
    let state = with_property(default, "waterlogged", flag(water_source(ctx, pos)))?;
    with_property(state, "facing", facing)
}

/// `BigDripleafBlock.canSurvive`: on a dripleaf, its stem or ground that carries it.
pub(super) fn big_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let under = ctx.state(below(pos));
    named(under, "minecraft:big_dripleaf")
        || named(under, "minecraft:big_dripleaf_stem")
        || tagged(ctx, below(pos), "minecraft:supports_big_dripleaf")
}

/// `BigDripleafBlock.updateShape`: a leaf with another on top becomes a stem. One that can no
/// longer stand turns into air.
pub(super) fn big_update(
    state: BlockStateId,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> BlockStateId {
    if direction == Direction::Down && !big_survives(pos, ctx) {
        return registry::default_state_named("minecraft:air");
    }
    let on_top = ctx.state((pos.0, pos.1 + 1, pos.2));
    if direction != Direction::Up || !named(on_top, "minecraft:big_dripleaf") {
        return state;
    }
    const STEM: BlockStateId = registry::default_state_named("minecraft:big_dripleaf_stem");
    [
        ("facing", property(state, "facing")),
        ("waterlogged", property(state, "waterlogged")),
    ]
    .into_iter()
    .try_fold(STEM, |stem, (name, value)| {
        with_property(stem, name, value?)
    })
    .unwrap_or(state)
}

/// `SmallDripleafBlock.getStateForPlacement`: a double plant that faces the player and is
/// waterlogged by any water at the place.
fn small_placement(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    super::tall::double_plant(default, ctx).map(|state| {
        let state = with_property(state?, "facing", ctx.horizontal_direction.opposite().name())?;
        with_property(
            state,
            "waterlogged",
            flag(is_water(ctx.state(ctx.clicked_pos()))),
        )
    })
}

/// `SmallDripleafBlock.canSurvive`: a lower half on ground that carries it, or on ground that
/// carries plants under a water source; an upper half as any double plant.
pub(super) fn small_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    if property(state, "half") == Some("upper") {
        return super::tall::double_plant_survives(state, pos, ctx);
    }
    let ground = below(pos);
    if tagged(ctx, ground, "minecraft:supports_small_dripleaf") {
        return Rule::Known(true);
    }
    Rule::Known(water_source(ctx, pos) && tagged(ctx, ground, "minecraft:supports_vegetation"))
}

/// The rules of this file, by the class that declares `getStateForPlacement`.
pub(super) const PLACEMENTS: &[(BlockClass, Placement)] = &[
    (BlockClass::BigDripleafBlock, |_, default, ctx| {
        Rule::Known(big_placement(default, ctx))
    }),
    (BlockClass::SmallDripleafBlock, |_, default, ctx| {
        small_placement(default, ctx)
    }),
];
