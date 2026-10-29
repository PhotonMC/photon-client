//! The ids a case carries are the reference's own: wire ids of the version whose Java answers
//! the case. The client's code numbers content by canonical id, so every id that enters from a
//! case goes through here, and every id that leaves for a comparison with the reference's
//! answer goes back.

use std::sync::{Arc, OnceLock};

use photon_core::ids::{BlockStateId, EntityTypeId, ItemId};
use photon_core::version::Version;
use photon_data::version_registries::VersionRegistries;

/// The version the oracle's Java is: `tests/behavior/java/v26_2`. It is not the newest version.
pub(crate) const ORACLE_VERSION: Version = Version::V26_2;

/// The numbering of [`ORACLE_VERSION`].
pub(crate) fn registries() -> Arc<VersionRegistries> {
    static REGISTRIES: OnceLock<Arc<VersionRegistries>> = OnceLock::new();
    REGISTRIES
        .get_or_init(|| Arc::new(VersionRegistries::for_version(ORACLE_VERSION)))
        .clone()
}

/// The entity type a case names by wire id.
pub(crate) fn entity_type(wire: i32) -> EntityTypeId {
    EntityTypeId(registries().entity_types.canonical_or_unknown(wire))
}

/// The item a case names by wire id.
pub(crate) fn item(wire: i32) -> ItemId {
    ItemId(registries().items.canonical_or_unknown(wire))
}

/// The block state a case names by wire id.
pub(crate) fn block_state(wire: i32) -> BlockStateId {
    BlockStateId(registries().block_states.canonical_or_unknown(wire))
}

/// The blocks of [`ORACLE_VERSION`] in registry order: the position in this list is the
/// block's place in the reference's registry.
pub(crate) fn blocks() -> impl Iterator<Item = &'static photon_data::block::registry::Block> {
    photon_data::block::registry::BLOCKS
        .iter()
        .filter(|block| block.exists_in(ORACLE_VERSION))
}

/// The items of [`ORACLE_VERSION`] in registry order.
pub(crate) fn items() -> impl Iterator<Item = &'static photon_data::item::registry::Item> {
    photon_data::item::registry::ITEMS
        .iter()
        .filter(|item| item.exists_in(ORACLE_VERSION))
}

/// How many block states [`ORACLE_VERSION`] has.
pub(crate) fn state_count() -> u32 {
    registries().block_states.wire_len() as u32
}

/// The chunk sections a payload holds, read with the numbering of [`ORACLE_VERSION`].
pub(crate) fn sections(
    bytes: &[u8],
) -> Result<Vec<photon_protocol::play::ChunkSectionObservation>, String> {
    photon_protocol::play::decode_chunk_section_data(bytes, &registries())
}

/// A world that speaks the oracle's version, so the wire ids a case names reach it as they would
/// from a server of that version.
pub(crate) fn world(dimension: photon_world::DimensionBounds) -> photon_world::ClientWorld {
    let mut world = photon_world::ClientWorld::new(dimension);
    world.set_versions(registries());
    world
}

/// The wire id of an item, as the oracle's Java numbers it.
pub(crate) fn wire_item(item: ItemId) -> i32 {
    registries().items.wire_or_unknown(item.0)
}

/// The wire id of a block state, as the oracle's Java numbers it.
pub(crate) fn wire_state(state: BlockStateId) -> i32 {
    registries().block_states.wire_or_unknown(state.0)
}
