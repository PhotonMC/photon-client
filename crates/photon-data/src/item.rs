//! Per item, in registry order.

pub mod attribute;
pub mod component;
pub mod creative;
pub mod definition;
pub mod effect;
pub mod registry;

pub use crate::tables::item::{
    attributes, classes, component_sets, component_types, creative_tabs, items, mob_effects,
    potions,
};

// The component sets are indexed by canonical item id; a new item must grow both.
const _: () = assert!(component_sets::DEFAULT_COMPONENT_SETS.len() == items::ITEMS.len());
