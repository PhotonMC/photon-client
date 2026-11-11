//! `Item.useOn` of the axe, hoe and shovel classes (before 26.3), where each class carries its
//! own table: the server changes the block, the client decides whether the tool applies and
//! plays the sound the server leaves out for the one using it. The axe also sets the block on the
//! client, the hoe and the shovel do not. 26.3 replaces all three by the `block_transformer`
//! component (see `component`).

use super::super::context::Context;
use super::super::mining::{DIRECTION_DOWN, block, is_air};
use super::super::{Aim, UseResult};
use super::{dowse_lit_campfire, play_at_block};

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
/// campfire is dowsed (the server puts it out; the client shows the smoke), the bottom face and
/// anything else pass.
fn shovel_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    if face == DIRECTION_DOWN {
        return UseResult::Pass;
    }
    let state = ctx.state_at((x, y, z));
    let name = block(state).name;
    if flattenable(name) && is_air(ctx.state_at((x, y + 1, z))) {
        play_at_block(ctx, "minecraft:item.shovel.flatten", (x, y, z));
        return UseResult::Success { swing: true };
    }
    if dowse_lit_campfire(ctx, (x, y, z), state) {
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

/// `AxeItem.useOn` for the client: not with a blocking item in the off hand unless sneaking; a
/// log strips to its stripped block (axis kept), sounding the axe and setting the block as the
/// reference does on the client too; weathered copper scrapes one age back and waxed copper loses
/// its wax; anything else passes.
fn axe_use_on(ctx: &mut Context<'_>, hand: u8, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, .. } = aim else {
        return UseResult::Pass;
    };
    if super::blocking_intent(ctx, hand) {
        return UseResult::Pass;
    }
    let state = ctx.state_at((x, y, z));
    let name = block(state).name;
    let Some(target) = stripped(name) else {
        let position = ctx.player.position();
        let (changed, sound) = match photon_world::block::rules::scraped(state) {
            Some(scraped) => (scraped, "minecraft:item.axe.scrape"),
            None => match photon_world::block::rules::waxed_off(state) {
                Some(unwaxed) => (unwaxed, "minecraft:item.axe.wax_off"),
                None => return UseResult::Pass,
            },
        };
        play_at_block(ctx, sound, (x, y, z));
        ctx.world
            .set_block_with_updates((x, y, z), changed, 11, 512, position);
        return UseResult::Success { swing: true };
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

/// `HoeItem.useOn` for the client: a tillable block with its predicate met sounds the hoe and
/// succeeds; any other passes.
fn hoe_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let Some(needs_air) = tillable(block(ctx.state_at((x, y, z))).name) else {
        return UseResult::Pass;
    };
    // `HoeItem.onlyIfAirAbove`: the face is not down, and the block above is air.
    if needs_air && (face == DIRECTION_DOWN || !is_air(ctx.state_at((x, y + 1, z)))) {
        return UseResult::Pass;
    }
    play_at_block(ctx, "minecraft:item.hoe.till", (x, y, z));
    UseResult::Success { swing: true }
}

/// `Item.useOn` of an axe, hoe or shovel class.
pub(super) fn use_on(
    ctx: &mut Context<'_>,
    (hand, class): (u8, photon_data::item::classes::ItemClass),
    aim: Aim,
) -> UseResult {
    use photon_data::item::classes::ItemClass as C;
    match class {
        C::HoeItem => hoe_use_on(ctx, aim),
        C::AxeItem => axe_use_on(ctx, hand, aim),
        _ => shovel_use_on(ctx, aim),
    }
}

#[cfg(test)]
mod tests {
    use super::{stripped, tillable};

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
