//! Each item's own component types in the version being played: the newest version's table
//! (`tables/item/component_sets.rs`), or for an older version that table with the components its
//! items have or lack (`tables/item/component_sets_older.rs`), less the types the version does not
//! register.

use std::sync::OnceLock;

use photon_core::version::Version;

use crate::item::component_sets::DEFAULT_COMPONENT_SETS;
use crate::item::component_types::{self, NAMES};
use crate::tables::item::component_sets_older::OLDER;

/// The items a change of one component applies to.
pub enum Items {
    /// Every item the version has.
    All,
    /// The named items.
    Only(&'static [&'static str]),
}

/// A component type an older version's items have, or lack, by name.
pub struct SetChange {
    pub component: &'static str,
    pub items: Items,
}

/// The component types of the item at `index` (canonical item id) in the active version, as a
/// set of canonical type ids; empty outside the registry.
pub fn of(index: usize) -> u128 {
    sets().get(index).copied().unwrap_or(0)
}

fn sets() -> &'static [u128] {
    static LISTS: [OnceLock<&'static [u128]>; Version::ALL.len()] =
        [const { OnceLock::new() }; Version::ALL.len()];
    let version = Version::active();
    LISTS[version.index()].get_or_init(|| match build(version) {
        Some(list) => Vec::leak(list),
        None => &DEFAULT_COMPONENT_SETS,
    })
}

/// The sets of an older version, or `None` for a version that has the newest table's.
fn build(version: Version) -> Option<Vec<u128>> {
    let (_, added, removed) = OLDER.iter().find(|(older, ..)| *older == version)?;
    let mut list = DEFAULT_COMPONENT_SETS.to_vec();
    for change in *added {
        apply(&mut list, change, |set, bit| set | bit);
    }
    for change in *removed {
        apply(&mut list, change, |set, bit| set & !bit);
    }
    let registered = (0..NAMES.len() as i32)
        .filter(|id| component_types::exists_in(*id, version))
        .fold(0_u128, |all, id| all | 1_u128 << id);
    for set in &mut list {
        *set &= registered;
    }
    Some(list)
}

fn apply(list: &mut [u128], change: &SetChange, step: fn(u128, u128) -> u128) {
    let id = component_types::id(change.component).expect("a listed component type");
    let bit = 1_u128 << id;
    match change.items {
        Items::All => list.iter_mut().for_each(|set| *set = step(*set, bit)),
        Items::Only(names) => {
            for name in names {
                let index = crate::item::registry::index(name).expect("a listed item");
                list[index] = step(list[index], bit);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_older_version_swaps_the_animations_and_keeps_only_what_it_registers() {
        for (version, ..) in OLDER {
            let list = build(*version).expect("a listed version has a table");
            assert_eq!(list.len(), DEFAULT_COMPONENT_SETS.len());
            let swing = 1_u128 << component_types::SWING_ANIMATION;
            let attack = 1_u128 << component_types::ATTACK_ANIMATION;
            assert!(list.iter().all(|set| set & attack == 0));
            assert!(
                list.iter()
                    .zip(crate::item::items::ITEMS.iter())
                    .filter(|(_, item)| item.exists_in(*version))
                    .all(|(set, _)| set & swing != 0)
            );
        }
    }

    #[test]
    fn the_newest_version_has_no_older_table() {
        assert!(build(*Version::ALL.last().expect("a version")).is_none());
    }
}
