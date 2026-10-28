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
