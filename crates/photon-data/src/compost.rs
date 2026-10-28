//! The items a composter takes (`ComposterBlock.COMPOSTABLES`).

use crate::tables::compost::COMPOSTABLES;

/// The chance an item fills a layer of a composter; none for an item it does not take.
pub fn chance(item: &str) -> Option<f32> {
    COMPOSTABLES
        .iter()
        .find(|(name, _)| *name == item)
        .map(|&(_, chance)| chance)
}

/// Every item a composter takes, in registry order: its name and chance.
pub fn all() -> &'static [(&'static str, f32)] {
    &COMPOSTABLES
}
