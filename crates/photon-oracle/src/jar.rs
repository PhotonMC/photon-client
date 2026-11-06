//! Where the game's own resources (the client jar's `assets` and `data`) are read from.

use std::path::PathBuf;

/// The directory holding the client jar's `assets` and `data` for the version under test: the
/// parent of `PHOTON_ASSETS` when the parity run names one, otherwise the newest local copy.
pub(crate) fn jar_root() -> PathBuf {
    if let Ok(assets) = photon_core::content_root::assets()
        && let Some(root) = PathBuf::from(assets).parent()
    {
        return root.to_path_buf();
    }
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.data/minecraft/assets/jar")
}
