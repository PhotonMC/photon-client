//! `BlockState.getMenuProvider`: which blocks open a menu on use, which a spectator may open.
//!
//! Chests answer on any state here (no cat sits on one), as the reference does on a level
//! without entities. A lectern answers with a book.

use photon_core::ids::BlockStateId;

/// The blocks whose every state has a menu provider: the blocks with a menu of their own
/// (crafting, anvils, looms ...) and those whose block entity is one (containers, the beacon,
/// the lectern). Sorted for the lookup.
#[rustfmt::skip]
const MENU_BLOCKS: &[&str] = &[
    "minecraft:anvil", "minecraft:barrel", "minecraft:beacon", "minecraft:black_shulker_box",
    "minecraft:blast_furnace", "minecraft:blue_shulker_box", "minecraft:brewing_stand",
    "minecraft:brown_shulker_box", "minecraft:cartography_table", "minecraft:chest",
    "minecraft:chipped_anvil", "minecraft:copper_chest", "minecraft:crafter",
    "minecraft:crafting_table", "minecraft:cyan_shulker_box", "minecraft:damaged_anvil",
    "minecraft:dispenser", "minecraft:dropper", "minecraft:enchanting_table",
    "minecraft:exposed_copper_chest", "minecraft:furnace", "minecraft:gray_shulker_box",
    "minecraft:green_shulker_box", "minecraft:grindstone", "minecraft:hopper",
    "minecraft:lectern", "minecraft:light_blue_shulker_box", "minecraft:light_gray_shulker_box",
    "minecraft:lime_shulker_box", "minecraft:loom", "minecraft:magenta_shulker_box",
    "minecraft:orange_shulker_box", "minecraft:oxidized_copper_chest", "minecraft:pink_shulker_box",
    "minecraft:purple_shulker_box", "minecraft:red_shulker_box", "minecraft:shulker_box",
    "minecraft:smithing_table", "minecraft:smoker", "minecraft:stonecutter",
    "minecraft:trapped_chest", "minecraft:waxed_copper_chest", "minecraft:waxed_exposed_copper_chest",
    "minecraft:waxed_oxidized_copper_chest", "minecraft:waxed_weathered_copper_chest",
    "minecraft:weathered_copper_chest", "minecraft:white_shulker_box", "minecraft:yellow_shulker_box",
];

/// Whether the state has a menu provider. A lectern has one only with a book on it.
pub fn has_menu_provider(state: BlockStateId) -> bool {
    use photon_data::block::properties::HAS_BOOK;
    let Some((block, offset)) = photon_data::block::registry::block_for_state(state) else {
        return false;
    };
    if block.name == "minecraft:lectern" {
        return block.flag(offset, &HAS_BOOK);
    }
    MENU_BLOCKS.binary_search(&block.name).is_ok()
}
