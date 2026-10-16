//! Lookups on the item registry ([`super::items`]).

use std::collections::HashMap;
use std::sync::OnceLock;

use photon_core::ids::ItemId;

pub use super::definition::{Item, RuleBlocks, Tool, ToolRule};
pub use super::items::ITEMS;

/// An item's registry index by its identifier.
pub fn index(name: &str) -> Option<usize> {
    static BY_NAME: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    BY_NAME
        .get_or_init(|| {
            ITEMS
                .iter()
                .enumerate()
                .map(|(index, item)| (item.name, index))
                .collect()
        })
        .get(name)
        .copied()
}

/// An item's id, resolved while compiling: a name the registry lacks stops the build.
pub const fn named(name: &str) -> ItemId {
    let mut index = 0;
    while index < ITEMS.len() {
        if crate::name::same(ITEMS[index].name, name) {
            return ItemId(index as i32);
        }
        index += 1;
    }
    panic!("the item registry has no such name");
}

/// The item that stands for an empty stack (`Items.AIR`).
pub const AIR_ITEM: ItemId = named("minecraft:air");

/// An item by its identifier.
pub fn item(name: &str) -> Option<&'static Item> {
    index(name).map(|index| &ITEMS[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_is_found_at_its_own_place() {
        for (index, item) in ITEMS.iter().enumerate() {
            assert_eq!(named(item.name).0 as usize, index, "{}", item.name);
        }
    }
}
