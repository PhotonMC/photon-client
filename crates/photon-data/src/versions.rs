//! What a version numbers differently from the union lists ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)),
//! one module per version that does, and the lists that are worked out per version.

use photon_core::version::Version;

pub mod block_patches;
pub mod component_sets;
pub mod model_layers;
pub mod v26_1;
pub mod v26_2;

/// The windows of blocks the version numbers in another order than the union list, each in its
/// order; empty for a version that follows the union list. 26.1 has the slab window 26.2
/// regrouped as 26.2 numbers it, and its own copper window.
pub fn block_patch(version: Version) -> &'static [&'static [&'static str]] {
    match version {
        Version::V26_1 => &[&v26_2::BLOCKS, &v26_1::BLOCKS],
        Version::V26_2 => &[&v26_2::BLOCKS],
        _ => &[],
    }
}

/// The same for items.
pub fn item_patch(version: Version) -> &'static [&'static [&'static str]] {
    match version {
        Version::V26_1 => &[&v26_1::ITEMS],
        _ => &[],
    }
}

/// The custom statistics the version does not register (a version's registry has no entry for
/// them, so the ids it sends skip them). Names without the namespace.
pub fn custom_stats_lacking(version: Version) -> &'static [&'static str] {
    match version {
        Version::V26_1 | Version::V26_2 => &["sleep_in_straw_bed"],
        _ => &[],
    }
}
