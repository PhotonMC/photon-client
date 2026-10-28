//! The block entity type registry: each type's name, and its id (its place in
//! the list, which is what the wire carries).

use crate::tables::registry_names::BLOCK_ENTITY_TYPES;

/// The name of the type with this id (`minecraft:mob_spawner`).
pub fn name(id: i32) -> Option<&'static str> {
    usize::try_from(id)
        .ok()
        .and_then(|index| BLOCK_ENTITY_TYPES.get(index))
        .map(|entry| entry.name)
}

/// The id of the type with this name.
pub fn id(name: &str) -> Option<i32> {
    BLOCK_ENTITY_TYPES
        .iter()
        .position(|candidate| candidate.name == name)
        .map(|index| index as i32)
}

/// Every type's name, in id order.
pub fn names() -> impl Iterator<Item = &'static str> {
    crate::named::names(&BLOCK_ENTITY_TYPES)
}
