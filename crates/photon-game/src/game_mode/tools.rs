//! The client's part of `Item.useOn` for the tools that change a block: the server changes it,
//! the client only decides whether the tool applies and plays the sound the server leaves out
//! for the one using it.

use super::context::Context;
use super::mining::{block, is_air};
use super::{Aim, UseResult};
use photon_world::local_player::{LocalSound, LocalSoundSource};

/// `HoeItem.TILLABLES`: whether the hoe works on the block (by name), which of them need the
/// face not to be the bottom one and air above.
fn tillable(name: &str) -> Option<bool> {
    match name {
        "minecraft:grass_block"
        | "minecraft:dirt_path"
        | "minecraft:dirt"
        | "minecraft:coarse_dirt" => Some(true),
        "minecraft:rooted_dirt" => Some(false),
        _ => None,
    }
}

/// Plays `event` at the middle of a block, loud and at normal pitch, as
/// `Level.playSound(player, pos, ...)` does for the one using the tool.
fn play_at_block(ctx: &mut Context<'_>, event: &'static str, (x, y, z): (i32, i32, i32)) {
    ctx.world.push_sound(LocalSound::at(
        event,
        LocalSoundSource::Blocks,
        (1.0, 1.0),
        (f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5),
    ));
}

/// `ShovelItem.FLATTENABLES`.
fn flattenable(name: &str) -> bool {
    matches!(
        name,
        "minecraft:grass_block"
            | "minecraft:dirt"
            | "minecraft:podzol"
            | "minecraft:coarse_dirt"
            | "minecraft:mycelium"
            | "minecraft:rooted_dirt"
    )
}

/// `ShovelItem.useOn` for the client: a flattenable block with air above sounds the shovel, a lit
/// campfire is dowsed (its smoke is not drawn), the bottom face and anything else pass.
pub(super) fn shovel_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    if face == super::mining::DIRECTION_DOWN {
        return UseResult::Pass;
    }
    let state = ctx.state_at((x, y, z));
    let name = block(state).name;
    if flattenable(name) && is_air(ctx.state_at((x, y + 1, z))) {
        play_at_block(ctx, "minecraft:item.shovel.flatten", (x, y, z));
        return UseResult::Success { swing: true };
    }
    let campfire = matches!(name, "minecraft:campfire" | "minecraft:soul_campfire");
    if campfire && photon_world::block::rules::property(state, "lit") == Some("true") {
        return UseResult::Success { swing: true };
    }
    UseResult::Pass
}

/// `AxeItem.STRIPPABLES`: the stripped block's name for a log, wood, stem, hyphae or bamboo block.
fn stripped(name: &str) -> Option<String> {
    let path = name.strip_prefix("minecraft:")?;
    let strippable = path == "bamboo_block"
        || [
            "oak", "dark_oak", "pale_oak", "acacia", "cherry", "birch", "jungle", "spruce",
            "mangrove",
        ]
        .iter()
        .any(|tree| path == format!("{tree}_log") || path == format!("{tree}_wood"))
        || ["warped", "crimson"]
            .iter()
            .any(|fungus| path == format!("{fungus}_stem") || path == format!("{fungus}_hyphae"));
    strippable.then(|| format!("minecraft:stripped_{path}"))
}

/// A block the axe scrapes or waxes off (the keys of `WeatheringCopper.PREVIOUS_BY_BLOCK` and
/// `HoneycombItem.WAX_OFF_BY_BLOCK`: every copper block that is exposed, weathered, oxidized or
/// waxed), whose tables are not ported: the click counts as done.
fn weathering_copper(name: &str) -> bool {
    name.strip_prefix("minecraft:").is_some_and(|path| {
        ["exposed_", "weathered_", "oxidized_", "waxed_"]
            .iter()
            .any(|prefix| path.starts_with(prefix))
    })
}

/// `AxeItem.useOn` for the client: not with a blocking item in the off hand unless sneaking; a
/// log strips to its stripped block (axis kept), sounding the axe and setting the block as the
/// reference does on the client too. Copper is not ported; anything else passes.
pub(super) fn axe_use_on(ctx: &mut Context<'_>, hand: u8, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, .. } = aim else {
        return UseResult::Pass;
    };
    // `playerHasBlockingItemUseIntent`.
    let off_hand_blocks = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(super::OFF_HAND)))
        .is_some_and(|stack| {
            crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
                .has(photon_data::item::component_types::BLOCKS_ATTACKS)
        });
    if hand == super::MAIN_HAND && off_hand_blocks && !ctx.player.is_shift_key_down() {
        return UseResult::Pass;
    }
    let state = ctx.state_at((x, y, z));
    let name = block(state).name;
    let Some(target) = stripped(name) else {
        return if weathering_copper(name) {
            UseResult::Success { swing: true }
        } else {
            UseResult::Pass
        };
    };
    let axis = photon_world::block::rules::property(state, "axis");
    let new_state = photon_data::block::registry::BLOCKS
        .iter()
        .find(|candidate| candidate.name == target)
        .map(|candidate| candidate.default_state_id())
        .and_then(|default| {
            photon_world::block::rules::with_property(default, "axis", axis.unwrap_or("y"))
        });
    let Some(new_state) = new_state else {
        return UseResult::Success { swing: true };
    };
    play_at_block(ctx, "minecraft:item.axe.strip", (x, y, z));
    let position = ctx.player.position();
    ctx.world
        .set_block_with_updates((x, y, z), new_state, 11, 512, position);
    UseResult::Success { swing: true }
}

/// `ShearsItem.useOn` for the client: a growing plant's head that is not at its full age (kelp,
/// twisting, weeping and cave vines) sounds the shears and takes the full age at once; anything
/// else passes (`Item.useOn`).
pub(super) fn shears_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, .. } = aim else {
        return UseResult::Pass;
    };
    let state = ctx.state_at((x, y, z));
    let head = matches!(
        block(state).name,
        "minecraft:kelp"
            | "minecraft:twisting_vines"
            | "minecraft:weeping_vines"
            | "minecraft:cave_vines"
    );
    let age = photon_world::block::rules::property(state, "age");
    if !head || age == Some("25") {
        return UseResult::Pass;
    }
    let Some(grown) = photon_world::block::rules::with_property(state, "age", "25") else {
        return UseResult::Success { swing: true };
    };
    play_at_block(ctx, "minecraft:block.growing_plant.crop", (x, y, z));
    let position = ctx.player.position();
    ctx.world
        .set_block_with_updates((x, y, z), grown, 3, 512, position);
    UseResult::Success { swing: true }
}

/// `HoeItem.useOn` for the client: a tillable block with its predicate met sounds the hoe and
/// succeeds; any other passes.
pub(super) fn hoe_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let Some(needs_air) = tillable(block(ctx.state_at((x, y, z))).name) else {
        return UseResult::Pass;
    };
    // `HoeItem.onlyIfAirAbove`: the face is not down, and the block above is air.
    if needs_air && (face == super::mining::DIRECTION_DOWN || !is_air(ctx.state_at((x, y + 1, z))))
    {
        return UseResult::Pass;
    }
    play_at_block(ctx, "minecraft:item.hoe.till", (x, y, z));
    UseResult::Success { swing: true }
}

#[cfg(test)]
mod tests {
    use super::{stripped, tillable, weathering_copper};

    #[test]
    fn logs_strip_and_stripped_logs_do_not() {
        assert_eq!(
            stripped("minecraft:oak_log").as_deref(),
            Some("minecraft:stripped_oak_log")
        );
        assert_eq!(
            stripped("minecraft:crimson_hyphae").as_deref(),
            Some("minecraft:stripped_crimson_hyphae")
        );
        assert_eq!(
            stripped("minecraft:bamboo_block").as_deref(),
            Some("minecraft:stripped_bamboo_block")
        );
        assert_eq!(stripped("minecraft:stripped_oak_log"), None);
        assert_eq!(stripped("minecraft:oak_planks"), None);
        for name in [
            "minecraft:waxed_cut_copper",
            "minecraft:exposed_lightning_rod",
            "minecraft:waxed_lightning_rod",
        ] {
            assert!(weathering_copper(name), "{name}");
        }
        for name in [
            "minecraft:copper_block",
            "minecraft:copper_torch",
            "minecraft:raw_copper_block",
            "minecraft:copper_ore",
        ] {
            assert!(!weathering_copper(name), "{name}");
        }
    }

    #[test]
    fn the_shovel_flattens_six_blocks() {
        for name in [
            "minecraft:grass_block",
            "minecraft:podzol",
            "minecraft:mycelium",
        ] {
            assert!(super::flattenable(name), "{name}");
        }
        assert!(!super::flattenable("minecraft:dirt_path"));
        assert!(!super::flattenable("minecraft:stone"));
    }

    #[test]
    fn the_hoe_tills_five_blocks() {
        for name in [
            "minecraft:grass_block",
            "minecraft:dirt_path",
            "minecraft:dirt",
            "minecraft:coarse_dirt",
        ] {
            assert_eq!(tillable(name), Some(true), "{name}");
        }
        assert_eq!(tillable("minecraft:rooted_dirt"), Some(false));
        assert_eq!(tillable("minecraft:stone"), None);
    }
}
