//! The ids a version puts on the wire, and the canonical ids game code uses
//! ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)).
//!
//! A canonical id is a definition's place in the union list. A version's wire id is its place in
//! that list with the definitions the version does not have left out, the way the block registry
//! numbers states already, and a version that orders some entries differently puts them in its own
//! order (`versions/`). Both are computed from the definitions, not stored. For the newest version
//! only what a newer version removed (`until`) is left out.

use photon_core::version::Version;

use crate::block::registry::BLOCKS;
use crate::item::registry::ITEMS;
use crate::named::presence;
use crate::particle_type::PARTICLE_TYPES;
use crate::tables::entity_types::ENTITY_TYPES;
use crate::tables::item::attributes::ATTRIBUTES;
use crate::tables::item::component_types;
use crate::tables::registry_names::{BLOCK_ENTITY_TYPES, MENU_TYPES};
use crate::tables::sound_events::SOUND_EVENTS;
use crate::versions;

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

    /// The map of a registry numbered in `wire_order`: the canonical ids the version has, by wire
    /// id. Canonical ids left out are the ones the version lacks.
    pub fn from_wire_order(canonical_len: usize, wire_order: Vec<u32>) -> Self {
        let mut to_wire = vec![None; canonical_len];
        for (wire, canonical) in wire_order.iter().enumerate() {
            to_wire[*canonical as usize] = Some(wire as u32);
        }
        Self {
            to_wire,
            to_canonical: wire_order,
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

    /// The canonical id of a wire id as the wire carries it. An id the version does not have
    /// stays outside the canonical range (and a negative one stays as it is), so a bad id from a
    /// server is still a bad id after the translation, as it is for the reference.
    pub fn canonical_or_unknown(&self, wire: i32) -> i32 {
        let Ok(index) = u32::try_from(wire) else {
            return wire;
        };
        match self.to_canonical(index) {
            Some(canonical) => canonical as i32,
            None => wire.saturating_add((self.canonical_len() - self.wire_len()) as i32),
        }
    }

    /// The wire id of a canonical id as game code holds it, for a packet going out; an id the
    /// version does not have stays outside the wire range.
    pub fn wire_or_unknown(&self, canonical: i32) -> i32 {
        let Ok(index) = u32::try_from(canonical) else {
            return canonical;
        };
        match self.to_wire(index) {
            Some(wire) => wire as i32,
            None => canonical.saturating_sub((self.canonical_len() - self.wire_len()) as i32),
        }
    }

    /// Whether every canonical id is its own wire id.
    pub fn is_identity(&self) -> bool {
        self.canonical_len() == self.wire_len()
            && self.to_canonical.iter().zip(0..).all(|(c, i)| *c == i)
    }
}

/// The canonical ids of the entries a version has, in the order it numbers them: the union
/// list without what the version lacks, with the window `patch` names put in the version's order
/// (empty when the version orders them like the union list). A patch that does not describe a
/// window of exactly its names is left unapplied; a test of every version catches that.
fn wire_order<'a>(entries: impl Iterator<Item = (&'a str, bool)>, patch: &[&str]) -> Vec<u32> {
    let entries: Vec<_> = entries
        .enumerate()
        .filter(|(_, (_, present))| *present)
        .collect();
    let mut order: Vec<u32> = entries
        .iter()
        .map(|(canonical, _)| *canonical as u32)
        .collect();
    let bare = |name: &str| {
        name.split_once(':')
            .map_or(name, |(_, bare)| bare)
            .to_owned()
    };
    let in_patch = |name: &str| patch.contains(&bare(name).as_str());
    let Some(start) = entries.iter().position(|(_, (name, _))| in_patch(name)) else {
        return order;
    };
    let Some(window) = entries.get(start..start + patch.len()) else {
        return order;
    };
    if !window.iter().all(|(_, (name, _))| in_patch(name)) {
        return order;
    }
    for (place, name) in patch.iter().enumerate() {
        let found = window.iter().find(|(_, (n, _))| bare(n) == *name);
        let Some((canonical, _)) = found else {
            return order;
        };
        order[start + place] = *canonical as u32;
    }
    order
}

/// The block states in the order a version numbers its blocks: each block's states stay
/// together. Also the canonical count, which is the union list's.
fn state_order(block_order: &[u32]) -> (usize, Vec<u32>) {
    let mut starts = Vec::with_capacity(BLOCKS.len());
    let mut total = 0u32;
    for block in &BLOCKS {
        starts.push(total);
        total += block.state_count();
    }
    let order = block_order
        .iter()
        .flat_map(|block| {
            let start = starts[*block as usize];
            start..start + BLOCKS[*block as usize].state_count()
        })
        .collect();
    (total as usize, order)
}

/// The numbering of every registry whose ids come over the wire as numbers, for one version.
/// Built once per session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRegistries {
    version: Version,
    pub items: IdMap,
    /// The block registry, which tags name their blocks by.
    pub blocks: IdMap,
    pub block_states: IdMap,
    pub entity_types: IdMap,
    pub sound_events: IdMap,
    pub particle_types: IdMap,
    pub menu_types: IdMap,
    pub block_entity_types: IdMap,
    pub attributes: IdMap,
    /// The data component types, which an item's component patch names its entries by.
    pub component_types: IdMap,
}

impl VersionRegistries {
    /// The numbering of `version`, computed by walking the union list filtered to it.
    pub fn for_version(version: Version) -> Self {
        let item_order = wire_order(
            ITEMS
                .iter()
                .map(|item| (item.name, item.exists_in(version))),
            versions::item_patch(version),
        );
        let block_order = wire_order(
            BLOCKS
                .iter()
                .map(|block| (block.name, block.exists_in(version))),
            versions::block_patch(version),
        );
        let (state_count, state_order) = state_order(&block_order);
        Self {
            version,
            items: IdMap::from_wire_order(ITEMS.len(), item_order),
            blocks: IdMap::from_wire_order(BLOCKS.len(), block_order),
            block_states: IdMap::from_wire_order(state_count, state_order),
            entity_types: IdMap::from_presence(presence(&ENTITY_TYPES, version)),
            sound_events: IdMap::from_presence(presence(&SOUND_EVENTS, version)),
            particle_types: IdMap::from_presence(
                PARTICLE_TYPES
                    .iter()
                    .map(|particle| particle.exists_in(version)),
            ),
            menu_types: IdMap::from_presence(presence(&MENU_TYPES, version)),
            block_entity_types: IdMap::from_presence(presence(&BLOCK_ENTITY_TYPES, version)),
            attributes: IdMap::from_presence(
                ATTRIBUTES
                    .iter()
                    .map(|attribute| attribute.exists_in(version)),
            ),
            component_types: IdMap::from_presence(
                (0..component_types::NAMES.len() as i32)
                    .map(|id| component_types::exists_in(id, version)),
            ),
        }
    }

    /// A numbering where every canonical id is its own wire id. For tests of code that only
    /// passes ids through, so they do not change when a version adds or drops an entry; the
    /// translation itself is tested on [`IdMap`] and by the registry parity family.
    pub fn without_gaps() -> Self {
        let real = Self::for_version(Version::NEWEST);
        let dense =
            |map: &IdMap| IdMap::from_presence(std::iter::repeat_n(true, map.canonical_len()));
        Self {
            version: real.version,
            items: dense(&real.items),
            blocks: dense(&real.blocks),
            block_states: dense(&real.block_states),
            entity_types: dense(&real.entity_types),
            sound_events: dense(&real.sound_events),
            particle_types: dense(&real.particle_types),
            menu_types: dense(&real.menu_types),
            block_entity_types: dense(&real.block_entity_types),
            attributes: dense(&real.attributes),
            component_types: dense(&real.component_types),
        }
    }

    /// The numbering of `version`, built on first use and shared by every session of it.
    pub fn of(version: Version) -> std::sync::Arc<Self> {
        static BUILT: [std::sync::OnceLock<std::sync::Arc<VersionRegistries>>; Version::ALL.len()] =
            [const { std::sync::OnceLock::new() }; Version::ALL.len()];
        BUILT[version.index()]
            .get_or_init(|| std::sync::Arc::new(Self::for_version(version)))
            .clone()
    }

    /// The version the numbering is for.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Whether the item named exists in this version; the filter for lists built from the union
    /// tables (creative tabs, the recipe book, command suggestions).
    pub fn has_item(&self, name: &str) -> bool {
        crate::item::registry::index(name)
            .is_some_and(|index| self.items.to_wire(index as u32).is_some())
    }

    /// Every registry's name and map, for checks that cover them all.
    pub fn all(&self) -> [(&'static str, &IdMap); 10] {
        [
            ("items", &self.items),
            ("blocks", &self.blocks),
            ("block_states", &self.block_states),
            ("entity_types", &self.entity_types),
            ("sound_events", &self.sound_events),
            ("particle_types", &self.particle_types),
            ("menu_types", &self.menu_types),
            ("block_entity_types", &self.block_entity_types),
            ("attributes", &self.attributes),
            ("component_types", &self.component_types),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_item_tells_a_listed_item_from_an_unknown_name() {
        let registries = VersionRegistries::of(Version::NEWEST);
        assert!(registries.has_item("minecraft:stone"));
        assert!(!registries.has_item("minecraft:not_an_item"));
    }

    #[test]
    fn every_version_numbers_every_registry_densely() {
        for version in Version::ALL.iter().copied() {
            let registries = VersionRegistries::for_version(version);
            for (name, map) in registries.all() {
                assert!(map.wire_len() > 0, "{version:?} {name}");
                assert!(map.wire_len() <= map.canonical_len(), "{version:?} {name}");
                let present = (0..map.canonical_len() as u32)
                    .filter(|id| map.to_wire(*id).is_some())
                    .count();
                assert_eq!(present, map.wire_len(), "{version:?} {name}");
            }
            let items = ITEMS.iter().filter(|item| item.exists_in(version)).count();
            assert_eq!(registries.items.wire_len(), items, "{version:?}");
            assert_eq!(registries.items.canonical_len(), ITEMS.len());
        }
    }

    #[test]
    fn the_union_has_as_many_canonical_ids_as_the_tables_list() {
        let registries = VersionRegistries::of(Version::NEWEST);
        assert_eq!(
            registries.block_states.canonical_len(),
            crate::block::registry::state_count() as usize
        );
    }

    #[test]
    fn ids_round_trip_through_the_wire() {
        for version in Version::ALL.iter().copied() {
            let map = VersionRegistries::for_version(version).items;
            for canonical in 0..map.canonical_len() as u32 {
                let Some(wire) = map.to_wire(canonical) else {
                    continue;
                };
                assert!((wire as usize) < map.wire_len(), "{version:?}");
                assert_eq!(map.to_canonical(wire), Some(canonical), "{version:?}");
            }
            assert_eq!(map.to_wire(map.canonical_len() as u32), None);
            assert_eq!(map.to_canonical(map.wire_len() as u32), None);
        }
    }

    #[test]
    fn an_unknown_id_keeps_its_distance_and_a_negative_one_is_left_alone() {
        for version in Version::ALL.iter().copied() {
            let map = VersionRegistries::for_version(version).items;
            let gap = (map.canonical_len() - map.wire_len()) as i32;
            let past_the_end = map.canonical_len() as i32 + 5;
            assert_eq!(map.wire_or_unknown(past_the_end), past_the_end - gap);
            let wire_past_the_end = map.wire_len() as i32 + 5;
            assert_eq!(map.canonical_or_unknown(wire_past_the_end), past_the_end);
            assert_eq!(map.canonical_or_unknown(-3), -3);
            assert_eq!(map.wire_or_unknown(-3), -3);
        }
    }

    #[test]
    fn a_named_entry_follows_its_span_into_the_numbering() {
        use crate::named::Named;
        // Only one version exists yet, so no span can leave a gap here; the gap itself is
        // covered by `a_missing_entry_leaves_a_gap_and_shifts_the_ids_after_it`.
        let entries = [
            Named::new("minecraft:all"),
            Named::new("minecraft:since").since(Version::V26_2),
            Named::new("minecraft:until").until(Version::V26_2),
        ];
        let present: Vec<bool> = presence(&entries, Version::V26_2).collect();
        assert_eq!(present, [true, true, true]);
        assert!(IdMap::from_presence(present).is_identity());
    }

    #[test]
    fn a_gap_shifts_the_ids_after_it_both_ways() {
        let map = IdMap::from_presence([true, false, true, true]);
        assert_eq!(map.canonical_or_unknown(1), 2);
        assert_eq!(map.canonical_or_unknown(2), 3);
        assert_eq!(map.wire_or_unknown(3), 2);
        assert_eq!(map.wire_or_unknown(1), 0);
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
