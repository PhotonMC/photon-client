//! What a version numbers differently from the union lists ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)),
//! one module per version that does.

use photon_core::version::Version;

pub mod v26_1;

/// The window of blocks the version numbers in another order than the union list, in its order;
/// empty for a version that follows the union list.
pub fn block_patch(version: Version) -> &'static [&'static str] {
    match version {
        Version::V26_1 => &v26_1::BLOCKS,
        _ => &[],
    }
}

/// The same for items.
pub fn item_patch(version: Version) -> &'static [&'static str] {
    match version {
        Version::V26_1 => &v26_1::ITEMS,
        _ => &[],
    }
}
