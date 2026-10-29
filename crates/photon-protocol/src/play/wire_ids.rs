//! Registry ids as the wire carries them, and the one place they become canonical ids
//! ([ADR 0013](../../../../docs/adr/0013-multi-version-content-model.md)).
//!
//! A wire id is a number of the connection's version; game code only holds canonical ids. The
//! types here are read from the wire and turned into canonical ids with the session's
//! [`VersionRegistries`]; nothing else in the crate wraps a raw read into an id.

use photon_core::ids::{BlockStateId, EntityTypeId, ItemId};
use photon_data::version_registries::VersionRegistries;

use super::{ConnectionError, read_varint};

/// An item id as the wire carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireItemId(pub i32);

/// A block state id as the wire carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireBlockStateId(pub i32);

/// An entity type id as the wire carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireEntityTypeId(pub i32);

/// An attribute id as the wire carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireAttributeId(pub i32);

impl WireAttributeId {
    /// A varint attribute id.
    pub fn read(input: &mut &[u8], name: &'static str) -> Result<Self, ConnectionError> {
        read_varint(input, name).map(Self)
    }

    /// The canonical id.
    pub fn canonical(self, registries: &VersionRegistries) -> i32 {
        registries.attributes.canonical_or_unknown(self.0)
    }
}

impl WireItemId {
    /// A varint item id.
    pub fn read(input: &mut &[u8], name: &'static str) -> Result<Self, ConnectionError> {
        read_varint(input, name).map(Self)
    }

    /// The canonical id.
    pub fn canonical(self, registries: &VersionRegistries) -> ItemId {
        ItemId(registries.items.canonical_or_unknown(self.0))
    }

    /// The wire id of a canonical one, for a packet going out.
    pub fn from_canonical(item: ItemId, registries: &VersionRegistries) -> Self {
        Self(registries.items.wire_or_unknown(item.0))
    }
}

impl WireBlockStateId {
    /// A varint block state id.
    pub fn read(input: &mut &[u8], name: &'static str) -> Result<Self, ConnectionError> {
        read_varint(input, name).map(Self)
    }

    /// The canonical id.
    pub fn canonical(self, registries: &VersionRegistries) -> BlockStateId {
        BlockStateId(registries.block_states.canonical_or_unknown(self.0))
    }

    /// The wire id of a canonical one, for a packet going out.
    pub fn from_canonical(state: BlockStateId, registries: &VersionRegistries) -> Self {
        Self(registries.block_states.wire_or_unknown(state.0))
    }
}

impl WireEntityTypeId {
    /// A varint entity type id.
    pub fn read(input: &mut &[u8], name: &'static str) -> Result<Self, ConnectionError> {
        read_varint(input, name).map(Self)
    }

    /// The canonical id.
    pub fn canonical(self, registries: &VersionRegistries) -> EntityTypeId {
        EntityTypeId(registries.entity_types.canonical_or_unknown(self.0))
    }

    /// The wire id of a canonical one, for a packet going out.
    pub fn from_canonical(entity_type: EntityTypeId, registries: &VersionRegistries) -> Self {
        Self(registries.entity_types.wire_or_unknown(entity_type.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::version::Version;

    #[test]
    fn a_wire_id_reads_as_the_canonical_id_of_its_version() {
        for version in Version::ALL.iter().copied() {
            let registries = VersionRegistries::for_version(version);
            for wire in [0, 5, 100] {
                let item = WireItemId(wire).canonical(&registries);
                assert_eq!(
                    registries.items.to_canonical(wire as u32),
                    Some(item.0 as u32)
                );
                assert_eq!(WireItemId::from_canonical(item, &registries).0, wire);
                let state = WireBlockStateId(wire).canonical(&registries);
                assert_eq!(
                    registries.block_states.to_wire(state.0 as u32),
                    Some(wire as u32)
                );
                let entity = WireEntityTypeId(wire).canonical(&registries);
                assert_eq!(
                    registries.entity_types.to_canonical(wire as u32),
                    Some(entity.0 as u32)
                );
            }
        }
    }

    #[test]
    fn an_id_past_the_registry_stays_past_it() {
        let registries = VersionRegistries::of(Version::NEWEST);
        let far = i32::MAX / 2;
        let gap = (registries.items.canonical_len() - registries.items.wire_len()) as i32;
        assert_eq!(WireItemId(far).canonical(&registries), ItemId(far + gap));
        assert_eq!(WireItemId(-1).canonical(&registries), ItemId(-1));
    }
}
