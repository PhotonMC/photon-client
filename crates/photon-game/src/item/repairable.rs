//! The `repairable` component of each item: which items repair it on an
//! anvil. The generated item tables do not carry it, so it is a table taken
//! from the reference: every item's default component patch, as the
//! fixture `tests/behavior/fixtures/item-default-patches.txt` holds it
//! (`tools/generate/item_components.py`). The test below reads the fixture
//! and compares every item against this table, so a new version that moves
//! a repair material fails there; regenerate the rows from the fixture's
//! component 33 and group them by value.

use photon_core::ids::ItemId;
use photon_core::wire::write_varint;

/// `Repairable.items`: a tag, or the items listed.
pub(crate) enum RepairItems {
    Tag(&'static str),
    Items(&'static [&'static str]),
}

type Group = (RepairItems, &'static [&'static str]);

#[rustfmt::skip]
static REPAIRABLE: &[Group] = &[
    (
        RepairItems::Items(&["minecraft:breeze_rod"]),
        &["minecraft:mace"],
    ),
    (
        RepairItems::Items(&["minecraft:phantom_membrane"]),
        &["minecraft:elytra"],
    ),
    (
        RepairItems::Tag("minecraft:copper_tool_materials"),
        &["minecraft:copper_sword", "minecraft:copper_shovel", "minecraft:copper_pickaxe", "minecraft:copper_axe", "minecraft:copper_hoe", "minecraft:copper_spear"],
    ),
    (
        RepairItems::Tag("minecraft:diamond_tool_materials"),
        &["minecraft:diamond_sword", "minecraft:diamond_shovel", "minecraft:diamond_pickaxe", "minecraft:diamond_axe", "minecraft:diamond_hoe", "minecraft:diamond_spear"],
    ),
    (
        RepairItems::Tag("minecraft:gold_tool_materials"),
        &["minecraft:golden_sword", "minecraft:golden_shovel", "minecraft:golden_pickaxe", "minecraft:golden_axe", "minecraft:golden_hoe", "minecraft:golden_spear"],
    ),
    (
        RepairItems::Tag("minecraft:iron_tool_materials"),
        &["minecraft:iron_sword", "minecraft:iron_shovel", "minecraft:iron_pickaxe", "minecraft:iron_axe", "minecraft:iron_hoe", "minecraft:iron_spear"],
    ),
    (
        RepairItems::Tag("minecraft:netherite_tool_materials"),
        &["minecraft:netherite_sword", "minecraft:netherite_shovel", "minecraft:netherite_pickaxe", "minecraft:netherite_axe", "minecraft:netherite_hoe", "minecraft:netherite_spear"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_chain_armor"),
        &["minecraft:chainmail_helmet", "minecraft:chainmail_chestplate", "minecraft:chainmail_leggings", "minecraft:chainmail_boots"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_copper_armor"),
        &["minecraft:copper_helmet", "minecraft:copper_chestplate", "minecraft:copper_leggings", "minecraft:copper_boots"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_diamond_armor"),
        &["minecraft:diamond_helmet", "minecraft:diamond_chestplate", "minecraft:diamond_leggings", "minecraft:diamond_boots"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_gold_armor"),
        &["minecraft:golden_helmet", "minecraft:golden_chestplate", "minecraft:golden_leggings", "minecraft:golden_boots"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_iron_armor"),
        &["minecraft:iron_helmet", "minecraft:iron_chestplate", "minecraft:iron_leggings", "minecraft:iron_boots"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_leather_armor"),
        &["minecraft:leather_helmet", "minecraft:leather_chestplate", "minecraft:leather_leggings", "minecraft:leather_boots"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_netherite_armor"),
        &["minecraft:netherite_helmet", "minecraft:netherite_chestplate", "minecraft:netherite_leggings", "minecraft:netherite_boots"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_turtle_helmet"),
        &["minecraft:turtle_helmet"],
    ),
    (
        RepairItems::Tag("minecraft:repairs_wolf_armor"),
        &["minecraft:wolf_armor"],
    ),
    (
        RepairItems::Tag("minecraft:stone_tool_materials"),
        &["minecraft:stone_sword", "minecraft:stone_shovel", "minecraft:stone_pickaxe", "minecraft:stone_axe", "minecraft:stone_hoe", "minecraft:stone_spear"],
    ),
    (
        RepairItems::Tag("minecraft:wooden_tool_materials"),
        &["minecraft:wooden_sword", "minecraft:wooden_shovel", "minecraft:wooden_pickaxe", "minecraft:wooden_axe", "minecraft:wooden_hoe", "minecraft:shield", "minecraft:wooden_spear"],
    ),
];

/// The component's value for the item as the network writes it (a holder
/// set: zero and the tag's name, or one more than the count and the ids).
pub(crate) fn repairable_value(item: ItemId) -> Option<Vec<u8>> {
    let name = usize::try_from(item.0)
        .ok()
        .and_then(|index| photon_data::item::registry::ITEMS.get(index))?
        .name;
    let (repair, _) = REPAIRABLE.iter().find(|(_, items)| items.contains(&name))?;
    let mut out = Vec::new();
    match repair {
        RepairItems::Tag(tag) => {
            write_varint(0, &mut out);
            write_varint(tag.len() as i32, &mut out);
            out.extend_from_slice(tag.as_bytes());
        }
        RepairItems::Items(items) => {
            write_varint(items.len() as i32 + 1, &mut out);
            for name in *items {
                write_varint(crate::item::id_of(name)?.0, &mut out);
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every item's own `repairable` in the reference's dump is this table's.
    #[test]
    fn matches_the_reference_dump() {
        const PATCHES: &str =
            include_str!("../../../../tests/behavior/fixtures/item-default-patches.txt");
        for (index, line) in PATCHES.lines().enumerate() {
            let (name, wire) = line.split_once(' ').expect("a name and its bytes");
            let bytes: Vec<u8> = (0..wire.len())
                .step_by(2)
                .map(|at| u8::from_str_radix(&wire[at..at + 2], 16).expect("hex"))
                .collect();
            let patch = crate::item_components::read_patch(&mut bytes.as_slice()).expect("a patch");
            let dumped = patch
                .added
                .iter()
                .find(|(id, _)| *id == photon_data::item::component_types::REPAIRABLE)
                .map(|(_, value)| value.clone());
            assert_eq!(
                repairable_value(photon_core::ids::ItemId(index as i32)),
                dumped,
                "{name}"
            );
        }
    }
}
