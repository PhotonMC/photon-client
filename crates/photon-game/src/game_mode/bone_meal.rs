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

/// Whether `grown` survives at `pos` (`BlockState.canSurvive`); `None` where the survival rule is
/// not ported.
fn survives_at(ctx: &Context<'_>, grown: &str, pos: (i32, i32, i32)) -> Option<bool> {
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
        photon_world::block::rules::Rule::Known(survives) => Some(survives),
        photon_world::block::rules::Rule::Unported => None,
    }
}

/// `TallGrassBlock.isValidBonemealTarget`: the tall plant it grows into survives where it stands
/// and the block above is air.
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
    Some(survives_at(ctx, grown, pos)? && is_air(above))
}

/// `NetherrackBlock.isValidBonemealTarget`: the block above lets skylight through and nylium is
/// within one block in every direction.
fn netherrack_target(ctx: &Context<'_>, pos: (i32, i32, i32), above: BlockStateId) -> bool {
    if !photon_world::block::state_or_air(above).propagates_skylight_down {
        return false;
    }
    (-1..=1).any(|dx| {
        (-1..=1).any(|dy| {
            (-1..=1).any(|dz| {
                let near = ctx.state_at((pos.0 + dx, pos.1 + dy, pos.2 + dz));
                matches!(
                    block(near).name,
                    "minecraft:crimson_nylium" | "minecraft:warped_nylium"
                )
            })
        })
    })
}

/// `BambooStalkBlock.isValidBonemealTarget`: the stalk is under sixteen blocks tall, its top is
/// not at the last stage, and the block above the top is empty.
fn bamboo_target(ctx: &Context<'_>, pos: (i32, i32, i32)) -> bool {
    let is_bamboo = |y: i32| block(ctx.state_at((pos.0, y, pos.2))).name == "minecraft:bamboo";
    let above = (1..=16).take_while(|step| is_bamboo(pos.1 + step)).count() as i32;
    let below = (1..=16).take_while(|step| is_bamboo(pos.1 - step)).count() as i32;
    let top = ctx.state_at((pos.0, pos.1 + above, pos.2));
    above + below + 1 < 16
        && property(top, "stage") != Some("1")
        && is_air(ctx.state_at((pos.0, pos.1 + above + 1, pos.2)))
}

/// `BonemealableBlock.hasSpreadableNeighbourPos`: a horizontal neighbour that is empty and where
/// the plant spreads to would survive.
fn spreader_target(ctx: &Context<'_>, name: &str, pos: (i32, i32, i32)) -> Option<bool> {
    let seed = match name {
        "glow_lichen" => {
            return Some(photon_world::block::multiface::lichen_can_spread(
                &|at| ctx.state_at(at),
                pos,
            ));
        }
        "tall_dry_grass" => "minecraft:short_dry_grass",
        "bush" => "minecraft:bush",
        "firefly_bush" => "minecraft:firefly_bush",
        _ => return None,
    };
    for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let beside = (pos.0 + dx, pos.1, pos.2 + dz);
        if is_air(ctx.state_at(beside)) && survives_at(ctx, seed, beside)? {
            return Some(true);
        }
    }
    Some(false)
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
    match name {
        "netherrack" => return Some(netherrack_target(ctx, pos, above)),
        "bamboo" => return Some(bamboo_target(ctx, pos)),
        "mangrove_propagule" => {
            let hanging = property(state, "hanging") == Some("true");
            return Some(!hanging || property(state, "age") != Some("4"));
        }
        _ => {}
    }
    if let Some(target) = spreader_target(ctx, name, pos) {
        return Some(target);
    }
    if let Some(target) = column_target(ctx, name, (state, pos)) {
        return Some(target);
    }
    surroundings_target(name, state, (above, below))
}

/// A big dripleaf (or its stem) grows into air, water or a small dripleaf above its head; pale
/// hanging moss grows into the air below its tip; a pitcher crop by its own rule.
fn column_target(
    ctx: &Context<'_>,
    name: &str,
    (state, pos): (BlockStateId, (i32, i32, i32)),
) -> Option<bool> {
    let walk = |same: &str, step: i32| {
        let mut at = pos;
        while block(ctx.state_at(at))
            .name
            .trim_start_matches("minecraft:")
            == same
        {
            at.1 += step;
        }
        at
    };
    let room = |at: (i32, i32, i32)| {
        let name = block(ctx.state_at(at))
            .name
            .trim_start_matches("minecraft:");
        matches!(
            name,
            "air" | "cave_air" | "void_air" | "water" | "small_dripleaf"
        )
    };
    match name {
        "big_dripleaf" => Some(room((pos.0, pos.1 + 1, pos.2))),
        "big_dripleaf_stem" => {
            let head = walk("big_dripleaf_stem", 1);
            let leaf = block(ctx.state_at(head)).name == "minecraft:big_dripleaf";
            Some(leaf && room((head.0, head.1 + 1, head.2)))
        }
        "pale_hanging_moss" => Some(is_air(ctx.state_at(walk("pale_hanging_moss", -1)))),
        "pitcher_crop" => Some(pitcher_target(ctx, state, pos)),
        _ => None,
    }
}

/// `PitcherCropBlock.isValidBonemealTarget`: the lower half (found from either) can grow one more
/// age: not full grown, lit enough, below the build height, and where it turns double, with
/// air or a pitcher crop above.
fn pitcher_target(ctx: &Context<'_>, state: BlockStateId, pos: (i32, i32, i32)) -> bool {
    let is_lower = |state: BlockStateId| {
        block(state).name == "minecraft:pitcher_crop" && property(state, "half") == Some("lower")
    };
    let lower = if is_lower(state) {
        pos
    } else if is_lower(ctx.state_at((pos.0, pos.1 - 1, pos.2))) {
        (pos.0, pos.1 - 1, pos.2)
    } else {
        return false;
    };
    let age = property(ctx.state_at(lower), "age")
        .and_then(|age| age.parse::<u8>().ok())
        .unwrap_or(0);
    let dimension = ctx.world.dimension();
    let above = ctx.state_at((lower.0, lower.1 + 1, lower.2));
    age < 4
        && ctx.world.max_local_raw_brightness(lower, 0) >= 8
        && lower.1 + 1 < dimension.min_y + dimension.height
        && (age + 1 < 3 || is_air(above) || block(above).name == "minecraft:pitcher_crop")
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
        "rooted_dirt" | "mangrove_leaves" => Some(is_air(below)),
        "crimson_fungus" => Some(block(below).name == "minecraft:crimson_nylium"),
        "warped_fungus" => Some(block(below).name == "minecraft:warped_nylium"),
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
    name == "pale_moss_carpet"
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
