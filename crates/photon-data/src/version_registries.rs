//! The ids a version puts on the wire, and the canonical ids game code uses
//! ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)).
//!
//! A canonical id is a definition's place in the union list. A version's wire id is its place in
//! that list with the definitions the version does not have left out, the way the block registry
//! numbers states already. Both are computed from the definitions, not stored. For the newest
//! version nothing is left out and the maps are the identity.

use photon_core::version::Version;

use crate::block::registry::BLOCKS;
use crate::generated::entity_types::ENTITY_TYPES;
use crate::generated::sound_events::SOUND_EVENTS;
use crate::item::registry::ITEMS;
use crate::particle_type::PARTICLE_TYPES;
use crate::{block_entity_type, menu_type};

/// The numbering of one registry in one version: wire id <-> canonical id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdMap {
    /// By canonical id: the wire id, or `None` where the version does not have the entry.
    to_wire: Vec<Option<u32>>,
    /// By wire id: the canonical id.
    to_canonical: Vec<u32>,
}

impl IdMap {
    /// The map of a registry whose entries are present or not, in canonical order.
    pub fn from_presence(present: impl IntoIterator<Item = bool>) -> Self {
        let mut to_wire = Vec::new();
        let mut to_canonical = Vec::new();
        for (canonical, present) in present.into_iter().enumerate() {
            if present {
                to_wire.push(Some(to_canonical.len() as u32));
                to_canonical.push(canonical as u32);
            } else {
                to_wire.push(None);
            }
        }
        Self {
            to_wire,
            to_canonical,
        }
    }

    /// How many entries the union list has.
    pub fn canonical_len(&self) -> usize {
        self.to_wire.len()
    }

    /// How many entries the version has.
    pub fn wire_len(&self) -> usize {
        self.to_canonical.len()
    }

    /// The wire id of a canonical id, if the version has the entry.
    pub fn to_wire(&self, canonical: u32) -> Option<u32> {
        self.to_wire.get(canonical as usize).copied().flatten()
    }

    /// The canonical id of a wire id, if it is one of the version's.
    pub fn to_canonical(&self, wire: u32) -> Option<u32> {
        self.to_canonical.get(wire as usize).copied()
    }

    /// Whether every canonical id is its own wire id.
    pub fn is_identity(&self) -> bool {
        self.canonical_len() == self.wire_len()
            && self.to_canonical.iter().zip(0..).all(|(c, i)| *c == i)
    }
}

/// The numbering of every registry whose ids come over the wire as numbers, for one version.
/// Built once per session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRegistries {
    version: Version,
    pub items: IdMap,
    pub block_states: IdMap,
    pub entity_types: IdMap,
    pub sound_events: IdMap,
    pub particle_types: IdMap,
    pub menu_types: IdMap,
    pub block_entity_types: IdMap,
}

impl VersionRegistries {
    /// The numbering of `version`, computed by walking the union list filtered to it.
    pub fn for_version(version: Version) -> Self {
        let all = |len: usize| IdMap::from_presence(std::iter::repeat_n(true, len));
        Self {
            version,
            items: IdMap::from_presence(ITEMS.iter().map(|item| item.exists_in(version))),
            block_states: IdMap::from_presence(BLOCKS.iter().flat_map(|block| {
                std::iter::repeat_n(block.exists_in(version), block.state_count() as usize)
            })),
            entity_types: all(ENTITY_TYPES.len()),
            sound_events: all(SOUND_EVENTS.len()),
            particle_types: all(PARTICLE_TYPES.len()),
            menu_types: all(menu_type::names().len()),
            block_entity_types: all(block_entity_type::names().len()),
        }
    }

    /// The version the numbering is for.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Every registry's name and map, for checks that cover them all.
    pub fn all(&self) -> [(&'static str, &IdMap); 7] {
        [
            ("items", &self.items),
            ("block_states", &self.block_states),
            ("entity_types", &self.entity_types),
            ("sound_events", &self.sound_events),
            ("particle_types", &self.particle_types),
            ("menu_types", &self.menu_types),
            ("block_entity_types", &self.block_entity_types),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_version_numbers_every_registry_as_the_identity() {
        let registries = VersionRegistries::for_version(Version::V26_2);
        for (name, map) in registries.all() {
            assert!(map.is_identity(), "{name}");
            assert!(map.canonical_len() > 0, "{name}");
        }
        assert_eq!(registries.items.canonical_len(), ITEMS.len());
        assert_eq!(
            registries.block_states.canonical_len(),
            crate::block::registry::state_count() as usize
        );
    }

    #[test]
    fn ids_round_trip_through_the_wire() {
        let map = VersionRegistries::for_version(Version::V26_2).items;
        for canonical in 0..map.canonical_len() as u32 {
            let wire = map.to_wire(canonical).expect("the newest version has it");
            assert_eq!(map.to_canonical(wire), Some(canonical));
        }
        assert_eq!(map.to_wire(map.canonical_len() as u32), None);
        assert_eq!(map.to_canonical(map.wire_len() as u32), None);
    }

    #[test]
    fn a_missing_entry_leaves_a_gap_and_shifts_the_ids_after_it() {
        let map = IdMap::from_presence([true, false, true, true]);
        assert_eq!(map.to_wire(1), None);
        assert_eq!(
            [map.to_wire(0), map.to_wire(2), map.to_wire(3)],
            [Some(0), Some(1), Some(2)]
        );
        assert_eq!(map.to_canonical(1), Some(2));
        assert!(!map.is_identity());
    }
}
