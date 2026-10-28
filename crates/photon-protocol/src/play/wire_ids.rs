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
    fn the_newest_version_reads_ids_as_they_are() {
        let registries = VersionRegistries::for_version(Version::NEWEST);
        let mut input: &[u8] = &[5, 200, 1];
        let first = WireItemId::read(&mut input, "item").unwrap();
        let second = WireBlockStateId::read(&mut input, "state").unwrap();
        assert_eq!(first.canonical(&registries), ItemId(5));
        assert_eq!(second.canonical(&registries), BlockStateId(200));
        assert_eq!(WireItemId::from_canonical(ItemId(5), &registries), first);
        assert_eq!(WireEntityTypeId(7).canonical(&registries), EntityTypeId(7));
    }

    #[test]
    fn an_id_past_the_registry_stays_past_it() {
        let registries = VersionRegistries::for_version(Version::NEWEST);
        let far = i32::MAX / 2;
        assert_eq!(WireItemId(far).canonical(&registries), ItemId(far));
        assert_eq!(WireItemId(-1).canonical(&registries), ItemId(-1));
    }
}
