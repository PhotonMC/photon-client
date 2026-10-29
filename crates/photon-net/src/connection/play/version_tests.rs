//! The version the connection tests speak: the newest, named in one place.

use std::sync::Arc;

use photon_core::version::Version;
use photon_data::version_registries::VersionRegistries;

/// The numbering of the version the tests run as.
pub(super) fn registries() -> Arc<VersionRegistries> {
    VersionRegistries::of(Version::NEWEST)
}

/// The protocol number that version answers a status request with.
pub(super) fn protocol() -> i32 {
    Version::NEWEST.protocol()
}
