//! Per item, in registry order.

pub mod behaviour;
pub mod components;
pub mod creative_tabs;
pub mod potions;
pub mod registry;
pub mod stacking;
pub mod tooltip;

/// An item's registry index by its identifier.
pub fn index(name: &str) -> Option<usize> {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static BY_NAME: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    BY_NAME
        .get_or_init(|| {
            registry::ITEMS
                .iter()
                .enumerate()
                .map(|(index, item)| (item.name, index))
                .collect()
        })
        .get(name)
        .copied()
}
