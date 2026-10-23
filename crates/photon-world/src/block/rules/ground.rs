//! The blocks that lie on the ground, hang or grow and check what is above or below them:
//! cake, dirt paths, farmland, seagrass, the grindstone, cocoa, snow, roots, moss and propagules.

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
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    match class {
        BlockClass::GrindstoneBlock => Rule::Known(true),
        BlockClass::CakeBlock => Rule::Known(cake_survives(pos, ctx)),
        BlockClass::DirtPathBlock => Rule::Known(path_survives(pos, ctx)),
        BlockClass::FarmlandBlock => Rule::Known(farmland_survives(pos, ctx)),
        _ => hanging_survives(class, state, pos, ctx),
    }
}

/// Whether the block at `pos` is in the block tag.
fn tagged(ctx: &PlaceContext<'_>, pos: (i32, i32, i32), tag: &str) -> bool {
    block_index(ctx.state(pos)).is_some_and(|(index, _)| ctx.tags.contains(index, tag))
}

fn below(pos: (i32, i32, i32)) -> (i32, i32, i32) {
    (pos.0, pos.1 - 1, pos.2)
}

/// `getRawBrightness(pos, 0)`, if the context can tell.
fn raw_brightness(ctx: &PlaceContext<'_>, pos: (i32, i32, i32)) -> Option<i32> {
    ctx.brightness.map(|light| light(pos.0, pos.1, pos.2))
}

/// `CropBlock.canSurvive`: light enough, over farmland or other crop soil.
fn crop_survives(state: BlockStateId, pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> Rule<bool> {
    let Some(light) = raw_brightness(ctx, pos) else {
        return Rule::Unported;
    };
    if light < 8 {
        return Rule::Known(false);
    }
    super::survive::vegetation(state, pos, ctx)
}

/// `MushroomBlock.canSurvive`: on a block that ignores the light, or dim and on a solid one.
fn mushroom_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> Rule<bool> {
    let under = below(pos);
    if tagged(ctx, under, "minecraft:overrides_mushroom_light_requirement") {
        return Rule::Known(true);
    }
    let Some(light) = raw_brightness(ctx, pos) else {
        return Rule::Unported;
    };
    let solid = crate::block::state(ctx.state(under)).is_some_and(|info| info.solid_render);
    Rule::Known(light < 13 && solid)
}

/// `canSurvive` of what grows from a neighbour: cocoa, snow, roots, moss and propagules.
fn hanging_survives(
    class: BlockClass,
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    let up = above(pos);
    match class {
        BlockClass::CropBlock => return crop_survives(state, pos, ctx),
        BlockClass::MushroomBlock => return mushroom_survives(pos, ctx),
        _ => {}
    }
    Rule::Known(match class {
        BlockClass::CocoaBlock => {
            let (dx, dy, dz) = facing(state).unwrap_or(Direction::North).step();
            tagged(
                ctx,
                (pos.0 + dx, pos.1 + dy, pos.2 + dz),
                "minecraft:supports_cocoa",
            )
        }
        BlockClass::SnowLayerBlock => snow_survives(pos, ctx),
        BlockClass::HangingRootsBlock => {
            crate::block::state(ctx.state(up)).is_some_and(|i| i.is_face_sturdy(Direction::Down))
        }
        BlockClass::HangingMossBlock => {
            let over = ctx.state(up);
            super::growth::can_attach_to(Direction::Up, over)
                || block(over).is_some_and(|b| b.class == BlockClass::HangingMossBlock)
        }
        BlockClass::MangrovePropaguleBlock => propagule_survives(state, pos, ctx),
        BlockClass::MossyCarpetBlock => carpet_survives(state, pos, ctx),
        _ => return Rule::Unported,
    })
}

/// `SnowLayerBlock.canSurvive`.
fn snow_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let under = below(pos);
    if tagged(ctx, under, "minecraft:cannot_support_snow_layer") {
        return false;
    }
    if tagged(ctx, under, "minecraft:support_override_snow_layer") {
        return true;
    }
    let state = ctx.state(under);
    let info = crate::block::state_or_air(state);
    let top = crate::block::states()
        .collision(info)
        .face_shape(Direction::Up);
    let full_snow = block(state).is_some_and(|b| b.class == BlockClass::SnowLayerBlock)
        && property(state, "layers") == Some("8");
    crate::block::state::is_shape_full_block(&top) || full_snow
}

/// `MangrovePropaguleBlock.canSurvive`: hanging from a block that holds it, or standing on one.
fn propagule_survives(state: BlockStateId, pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    if property(state, "hanging") == Some("true") {
        tagged(
            ctx,
            above(pos),
            "minecraft:supports_hanging_mangrove_propagule",
        )
    } else {
        tagged(ctx, below(pos), "minecraft:supports_mangrove_propagule")
    }
}

/// `MossyCarpetBlock.canSurvive`: a base needs something below, the rest a base carpet.
fn carpet_survives(state: BlockStateId, pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let under = ctx.state(below(pos));
    if property(state, "bottom") == Some("true") {
        return !is_air(under);
    }
    block(under).is_some_and(|b| b.name == "minecraft:pale_moss_carpet")
        && property(under, "bottom") == Some("true")
}

/// `MossyCarpetBlock.getUpdatedState` with the sides made: each side a block can be put against
/// is low, or tall where an upper carpet has it.
fn carpet_state(
    default: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let over = ctx.state(above(pos));
    let upper = block(over).is_some_and(|b| b.name == "minecraft:pale_moss_carpet")
        && property(over, "bottom") != Some("true");
    let mut state = default;
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        let (dx, dy, dz) = direction.step();
        let held = super::growth::can_attach_to(
            direction,
            ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz)),
        );
        let tall = upper && property(over, direction.name()).is_some_and(|side| side != "none");
        let side = match (held, tall) {
            (false, _) => "none",
            (true, true) => "tall",
            (true, false) => "low",
        };
        state = with_property(state, direction.name(), side)?;
    }
    Some(state)
}

/// `SnowLayerBlock.getStateForPlacement`: one layer more on snow, otherwise the default.
fn snow(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let placed = ctx.state(ctx.clicked_pos());
    if !block(placed).is_some_and(|b| b.class == BlockClass::SnowLayerBlock) {
        return Some(default);
    }
    let layers: i32 = property(placed, "layers")?.parse().ok()?;
    with_property(placed, "layers", &(layers + 1).min(8).to_string())
}

/// `CocoaBlock.getStateForPlacement`: the first horizontal direction of the look that has a
/// block to hang from.
fn cocoa(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    super::placement::nearest_looking_directions(ctx)
        .into_iter()
        .filter(|direction| direction.axis() != Axis::Y)
        .find_map(|direction| {
            let state = with_property(default, "facing", direction.name())?;
            hanging_survives(BlockClass::CocoaBlock, state, pos, ctx)
                .eq(&Rule::Known(true))
                .then_some(state)
        })
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
    (BlockClass::CocoaBlock, |_, default, ctx| {
        Rule::Known(cocoa(default, ctx))
    }),
    (BlockClass::SnowLayerBlock, |_, default, ctx| {
        Rule::Known(snow(default, ctx))
    }),
    (BlockClass::HangingRootsBlock, |_, default, ctx| {
        Rule::Known(with_property(
            default,
            "waterlogged",
            flag(is_water_source(ctx.state(ctx.clicked_pos()))),
        ))
    }),
    // The propagule is put down grown, as a sapling-like block that is not hanging.
    (BlockClass::MangrovePropaguleBlock, |_, default, ctx| {
        let water = flag(is_water_source(ctx.state(ctx.clicked_pos())));
        Rule::Known(
            with_property(default, "waterlogged", water)
                .and_then(|state| with_property(state, "age", "4")),
        )
    }),
    (BlockClass::MossyCarpetBlock, |_, default, ctx| {
        Rule::Known(carpet_state(default, ctx.clicked_pos(), ctx))
    }),
];
