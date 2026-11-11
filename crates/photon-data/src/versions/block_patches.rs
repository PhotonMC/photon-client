//! What the active version's blocks declare differently from the block table, which is the newest
//! version's (`tables/block_older.rs` lists the differences of an older version).

use std::collections::HashMap;
use std::sync::OnceLock;

use photon_core::version::Version;

use crate::block::definition::{PushReaction, Rule};
use crate::tables::block_older::OLDER;

/// The declarations of one block that an older version has otherwise than the table.
pub struct BlockPatch {
    pub name: &'static str,
    pub explosion_resistance: Option<f32>,
    pub force_solid_on: Option<bool>,
    pub push_reaction: Option<PushReaction>,
    /// The rule the version declares for the block. An older version declares one only for some
    /// blocks; the others derive it.
    pub view_blocking: Option<Rule>,
}

/// The patch of the block named `name` in the active version, if the version declares it
/// otherwise than the table.
pub fn of(name: &str) -> Option<&'static BlockPatch> {
    patches().get(name).copied()
}

fn patches() -> &'static HashMap<&'static str, &'static BlockPatch> {
    static MAPS: [OnceLock<HashMap<&'static str, &'static BlockPatch>>; Version::ALL.len()] =
        [const { OnceLock::new() }; Version::ALL.len()];
    let version = Version::active();
    MAPS[version.index()].get_or_init(|| build(version))
}

/// The patches of an older version by block name; none for a version that has the table's.
fn build(version: Version) -> HashMap<&'static str, &'static BlockPatch> {
    OLDER
        .iter()
        .filter(|(older, _)| *older == version)
        .flat_map(|(_, list)| list.iter())
        .map(|patch| (patch.name, patch))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_patch_names_a_block_once_and_changes_something() {
        for (version, list) in OLDER {
            let found = build(*version);
            assert_eq!(found.len(), list.len(), "{version:?} names a block twice");
            for patch in *list {
                assert!(
                    crate::block::registry::BLOCKS
                        .iter()
                        .any(|block| block.name == patch.name),
                    "{}",
                    patch.name
                );
                assert!(
                    patch.explosion_resistance.is_some()
                        || patch.force_solid_on.is_some()
                        || patch.push_reaction.is_some()
                        || patch.view_blocking.is_some()
                );
            }
        }
    }

    #[test]
    fn the_newest_version_has_no_patches() {
        assert!(build(*Version::ALL.last().expect("a version")).is_empty());
    }
}
