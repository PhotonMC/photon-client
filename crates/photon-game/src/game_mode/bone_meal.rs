//! `BoneMealItem.useOn` as the client predicts it: the swing follows `growCrop`, which needs a
//! bonemealable block with a valid target, then `growWaterPlant` on the block beside the face.

use super::context::Context;
use super::mining::{block, is_air};
use super::{Aim, UseResult};
use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;
use photon_world::block::rules::property;

/// Whether `age` is below `limit`.
fn younger_than(state: BlockStateId, limit: u8) -> bool {
    property(state, "age")
        .and_then(|age| age.parse::<u8>().ok())
        .is_some_and(|age| age < limit)
}

/// The age a crop, stem, cocoa pod or berry bush stops growing at, by block.
fn full_age(name: &str) -> Option<u8> {
    match name {
        "wheat" | "carrots" | "potatoes" | "melon_stem" | "pumpkin_stem" => Some(7),
        "beetroots" | "sweet_berry_bush" => Some(3),
        // `getMaxAge` is two but the property stops at one: never fully grown.
        "torchflower_crop" => Some(2),
        "cocoa" => Some(2),
        _ => None,
    }
}

/// The head, the direction it grows in and the block it grows into, for each growing plant: the
/// head's name and body's name come first.
fn growing_plant(name: &str) -> Option<(&'static str, (i32, bool))> {
    Some(match name {
        "kelp" | "kelp_plant" => ("kelp", (1, false)),
        "twisting_vines" | "twisting_vines_plant" => ("twisting_vines", (1, true)),
        "weeping_vines" | "weeping_vines_plant" => ("weeping_vines", (-1, true)),
        _ => return None,
    })
}

/// `GrowingPlantHeadBlock` and `GrowingPlantBodyBlock.isValidBonemealTarget` (and the cave vines'
/// own rule): the head, found by following the body along the growth direction, has a place to
/// grow into. `None` for any other block.
fn growing_plant_target(
    ctx: &Context<'_>,
    name: &str,
    state: BlockStateId,
    pos: (i32, i32, i32),
) -> Option<bool> {
    if name.starts_with("cave_vines") {
        return Some(property(state, "berries") == Some("false"));
    }
    let (head, (step, into_air)) = growing_plant(name)?;
    let body = format!("{head}_plant");
    let named = |at: (i32, i32, i32)| {
        block(ctx.state_at(at))
            .name
            .trim_start_matches("minecraft:")
    };
    let mut at = pos;
    if name == body {
        loop {
            at.1 += step;
            if named(at) != body {
                break;
            }
        }
        if named(at) != head {
            return Some(false);
        }
    }
    let grow = ctx.state_at((at.0, at.1 + step, at.2));
    Some(if into_air {
        is_air(grow)
    } else {
        block(grow).name == "minecraft:water"
    })
}

/// `TallGrassBlock.isValidBonemealTarget`: the tall plant it grows into survives where it stands
/// and the block above is air. `None` where the survival rule is not ported.
fn tall_grass_target(
    ctx: &Context<'_>,
    name: &str,
    pos: (i32, i32, i32),
    above: BlockStateId,
) -> Option<bool> {
    let grown = if name == "fern" {
        "minecraft:large_fern"
    } else {
        "minecraft:tall_grass"
    };
    let grown = photon_data::block::registry::block_index(grown)?;
    let grown = photon_data::block::registry::BLOCKS[grown].default_state_id();
    let world: &photon_world::ClientWorld = ctx.world;
    let state_at = |x: i32, y: i32, z: i32| world.block_state_or_air(x, y, z);
    let aimed = (pos, Direction::Up, (0.0, 0.0, 0.0));
    let tags = world.block_state_registry().tags();
    let place_ctx = photon_world::block::rules::PlaceContext::new(
        &state_at,
        aimed,
        None,
        ctx.player.rotation(),
        tags,
    );
    match photon_world::block::rules::can_survive(grown, pos, &place_ctx) {
        photon_world::block::rules::Rule::Known(survives) => Some(survives && is_air(above)),
        photon_world::block::rules::Rule::Unported => None,
    }
}

/// `BonemealableBlock.isValidBonemealTarget` on the client for the block at `pos`: `Some` where
/// the rule is ported (a block that is not bonemealable is `Some(false)`), `None` where it is
/// not. The client is no `ServerLevel`, so the saplings and mushrooms refuse.
fn valid_target(ctx: &Context<'_>, state: BlockStateId, pos: (i32, i32, i32)) -> Option<bool> {
    let name = block(state).name.trim_start_matches("minecraft:");
    if let Some(full) = full_age(name) {
        return Some(younger_than(state, full));
    }
    let above = ctx.state_at((pos.0, pos.1 + 1, pos.2));
    let below = ctx.state_at((pos.0, pos.1 - 1, pos.2));
    if let Some(target) = growing_plant_target(ctx, name, state, pos) {
        return Some(target);
    }
    if matches!(name, "short_grass" | "fern") {
        return tall_grass_target(ctx, name, pos, above);
    }
    surroundings_target(name, state, (above, below))
}

/// The targets decided by the blocks above and below, or by nothing at all.
fn surroundings_target(
    name: &str,
    state: BlockStateId,
    (above, below): (BlockStateId, BlockStateId),
) -> Option<bool> {
    match name {
        "grass_block" | "bamboo_sapling" | "crimson_nylium" | "warped_nylium" | "moss_block"
        | "pale_moss_block" => Some(is_air(above)),
        "seagrass" => Some(block(above).name == "minecraft:water"),
        "sea_pickle" => Some(
            property(state, "waterlogged") == Some("true")
                && block(below).name.ends_with("_coral_block")
                && !block(below).name.starts_with("minecraft:dead_"),
        ),
        "rooted_dirt" => Some(is_air(below)),
        n => fixed_target(n),
    }
}

/// The targets that do not depend on the surroundings.
fn fixed_target(name: &str) -> Option<bool> {
    match name {
        "sunflower" | "lilac" | "rose_bush" | "peony" | "pink_petals" | "wildflowers"
        | "short_dry_grass" | "small_dripleaf" => Some(true),
        "brown_mushroom" | "red_mushroom" => Some(false),
        n if n.ends_with("_sapling") || n.ends_with("azalea") => Some(false),
        n => (!bonemealable_name(n)).then_some(false),
    }
}

/// Whether the name is a block class with a `BonemealableBlock` rule not ported here.
fn bonemealable_name(name: &str) -> bool {
    matches!(
        name,
        "short_grass"
            | "fern"
            | "glow_lichen"
            | "moss_carpet"
            | "pale_moss_carpet"
            | "pale_hanging_moss"
            | "mangrove_leaves"
            | "mangrove_propagule"
            | "big_dripleaf"
            | "big_dripleaf_stem"
            | "bamboo"
            | "bamboo_sapling"
            | "crimson_fungus"
            | "warped_fungus"
            | "netherrack"
            | "tall_dry_grass"
            | "pitcher_crop"
            | "bush"
            | "firefly_bush"
    )
}

/// `BoneMealItem.useOn`: swings where the clicked block grows, or where the block beside the face
/// is a water source on a sturdy face; otherwise passes. `None` where the block's rule is not
/// ported.
pub(super) fn use_on(ctx: &Context<'_>, aim: Aim) -> Option<UseResult> {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return Some(UseResult::Pass);
    };
    let clicked = ctx.state_at((x, y, z));
    if valid_target(ctx, clicked, (x, y, z))? {
        return Some(UseResult::Success { swing: true });
    }
    let face = Direction::from_3d_data_value(i32::from(face));
    let (dx, dy, dz) = face.step();
    let beside = ctx.state_at((x + dx, y + dy, z + dz));
    let sturdy = photon_world::block::state(clicked).is_some_and(|info| info.is_face_sturdy(face));
    // `FluidState.isFull`: a source, or the falling flow `LiquidBlock` gives levels from eight up.
    let source = block(beside).name == "minecraft:water"
        && property(beside, "level")
            .and_then(|level| level.parse::<u8>().ok())
            .is_some_and(|level| level == 0 || level >= 8);
    Some(if sturdy && source {
        UseResult::Success { swing: true }
    } else {
        UseResult::Pass
    })
}
