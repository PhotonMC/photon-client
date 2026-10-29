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
