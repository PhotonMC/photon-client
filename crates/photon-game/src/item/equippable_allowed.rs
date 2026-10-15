//! Which entities an item's own `equippable` component lets wear it
//! (`Equippable.allowedEntities`). The generated item tables do not carry
//! it, so it is a table taken from the reference: every item's default
//! component patch in `tests/behavior/fixtures/item-default-patches.txt`. The
//! test below reads the fixture and compares every item against this table;
//! regenerate the rows from the fixture's `equippable` (component
//! `EQUIPPABLE`), grouped by value, when the version changes.

/// `Equippable.allowedEntities`: an entity type tag, or the types listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Allowed {
    Tag(String),
    Types(Vec<String>),
}

enum Row {
    Tag(&'static str),
    Types(&'static [&'static str]),
}

type Group = (Row, &'static [&'static str]);

#[rustfmt::skip]
static ALLOWED: &[Group] = &[
    (
        Row::Tag("minecraft:can_equip_harness"),
        &["minecraft:white_harness", "minecraft:orange_harness", "minecraft:magenta_harness", "minecraft:light_blue_harness", "minecraft:yellow_harness", "minecraft:lime_harness", "minecraft:pink_harness", "minecraft:gray_harness", "minecraft:light_gray_harness", "minecraft:cyan_harness", "minecraft:purple_harness", "minecraft:blue_harness", "minecraft:brown_harness", "minecraft:green_harness", "minecraft:red_harness", "minecraft:black_harness"],
    ),
    (
        Row::Tag("minecraft:can_equip_saddle"),
        &["minecraft:saddle"],
    ),
    (
        Row::Tag("minecraft:can_wear_horse_armor"),
        &["minecraft:copper_horse_armor", "minecraft:iron_horse_armor", "minecraft:golden_horse_armor", "minecraft:diamond_horse_armor", "minecraft:netherite_horse_armor", "minecraft:leather_horse_armor"],
    ),
    (
        Row::Tag("minecraft:can_wear_nautilus_armor"),
        &["minecraft:iron_nautilus_armor", "minecraft:golden_nautilus_armor", "minecraft:diamond_nautilus_armor", "minecraft:netherite_nautilus_armor", "minecraft:copper_nautilus_armor"],
    ),
    (
        Row::Types(&["minecraft:llama", "minecraft:trader_llama"]),
        &["minecraft:white_carpet", "minecraft:orange_carpet", "minecraft:magenta_carpet", "minecraft:light_blue_carpet", "minecraft:yellow_carpet", "minecraft:lime_carpet", "minecraft:pink_carpet", "minecraft:gray_carpet", "minecraft:light_gray_carpet", "minecraft:cyan_carpet", "minecraft:purple_carpet", "minecraft:blue_carpet", "minecraft:brown_carpet", "minecraft:green_carpet", "minecraft:red_carpet", "minecraft:black_carpet"],
    ),
    (
        Row::Types(&["minecraft:wolf"]),
        &["minecraft:wolf_armor"],
    ),
];

/// The item's own allowed entities; none for an item that allows every entity.
pub fn allowed_entities(item: &str) -> Option<Allowed> {
    let (row, _) = ALLOWED.iter().find(|(_, items)| items.contains(&item))?;
    Some(match row {
        Row::Tag(tag) => Allowed::Tag((*tag).to_owned()),
        Row::Types(types) => Allowed::Types(types.iter().map(|&name| name.to_owned()).collect()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(input: &mut &[u8]) -> i32 {
        photon_core::wire::read_varint(input).expect("a varint")
    }

    fn string(input: &mut &[u8]) -> String {
        let length = varint(input) as usize;
        let text = String::from_utf8(input[..length].to_vec()).expect("text");
        *input = &input[length..];
        text
    }

    /// Every item's own `allowed_entities` in the reference's dump is this table's.
    #[test]
    fn matches_the_reference_dump() {
        const PATCHES: &str =
            include_str!("../../../../tests/behavior/fixtures/item-default-patches.txt");
        for line in PATCHES.lines() {
            let (name, wire) = line.split_once(' ').expect("a name and its bytes");
            let bytes: Vec<u8> = (0..wire.len())
                .step_by(2)
                .map(|at| u8::from_str_radix(&wire[at..at + 2], 16).expect("hex"))
                .collect();
            let patch = crate::item_components::read_patch(&mut bytes.as_slice()).expect("a patch");
            let dumped = patch
                .added
                .iter()
                .find(|(id, _)| *id == photon_data::item::component_types::EQUIPPABLE)
                .and_then(|(_, value)| {
                    let mut input = value.as_slice();
                    varint(&mut input);
                    if varint(&mut input) == 0 {
                        string(&mut input);
                        input = &input[if input[0] == 1 { 5 } else { 1 }..];
                    }
                    for _ in 0..2 {
                        // The asset and the camera overlay.
                        let present = input[0] == 1;
                        input = &input[1..];
                        if present {
                            string(&mut input);
                        }
                    }
                    let present = input[0] == 1;
                    input = &input[1..];
                    present.then(|| match varint(&mut input) {
                        0 => Allowed::Tag(string(&mut input)),
                        count => Allowed::Types(
                            (1..count)
                                .map(|_| {
                                    photon_data::entity_type::name(photon_core::ids::EntityTypeId(
                                        varint(&mut input),
                                    ))
                                    .expect("an entity type")
                                    .to_owned()
                                })
                                .collect(),
                        ),
                    })
                });
            assert_eq!(allowed_entities(name), dumped, "{name}");
        }
    }
}
