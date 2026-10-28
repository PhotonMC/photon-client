//! The menu type registry: each type's name, and its id (its place in
//! the list, which is what the wire carries).

use crate::tables::registry_names::MENU_TYPES;

/// The name of the type with this id (`minecraft:generic_9x3`).
pub fn name(id: i32) -> Option<&'static str> {
    usize::try_from(id)
        .ok()
        .and_then(|index| MENU_TYPES.get(index))
        .map(|entry| entry.name)
}

/// The id of the type with this name.
pub fn id(name: &str) -> Option<i32> {
    MENU_TYPES
        .iter()
        .position(|candidate| candidate.name == name)
        .map(|index| index as i32)
}

/// Every type's name, in id order.
pub fn names() -> impl Iterator<Item = &'static str> {
    crate::named::names(&MENU_TYPES)
}
