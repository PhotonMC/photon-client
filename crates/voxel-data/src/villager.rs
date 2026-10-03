//! The villager types and professions: their order in the registries (what the
//! wire carries) and the hat each one's texture asks for
//! (`VillagerMetadataSection.Hat`).

use crate::generated::villager_registries::{VILLAGER_PROFESSIONS, VILLAGER_TYPES};

/// How much of the head a texture's hat covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hat {
    None,
    Partial,
    Full,
}

fn entry(table: &'static [(&'static str, Hat)], id: i32) -> Option<(&'static str, Hat)> {
    usize::try_from(id)
        .ok()
        .and_then(|index| table.get(index))
        .copied()
}

/// The type with this registry id: its name (`plains`) and hat.
pub fn villager_type(id: i32) -> Option<(&'static str, Hat)> {
    entry(&VILLAGER_TYPES, id)
}

/// The profession with this registry id: its name (`farmer`) and hat.
pub fn profession(id: i32) -> Option<(&'static str, Hat)> {
    entry(&VILLAGER_PROFESSIONS, id)
}

/// Every type, in registry order.
pub fn types() -> &'static [(&'static str, Hat)] {
    &VILLAGER_TYPES
}

/// Every profession, in registry order.
pub fn professions() -> &'static [(&'static str, Hat)] {
    &VILLAGER_PROFESSIONS
}
