//! Where the game's own resources (the client jar's `assets` and `data`) are read from.

use std::path::PathBuf;

/// The directory holding the client jar's `assets` and `data` for the version under test: the
/// parent of `PHOTON_ASSETS` when the parity run names one, otherwise the newest local copy.
pub(crate) fn jar_root() -> PathBuf {
    if let Some(assets) = std::env::var_os("PHOTON_ASSETS")
        && let Some(root) = PathBuf::from(assets).parent()
    {
        return root.to_path_buf();
    }
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.data/minecraft/assets/jar")
}

/// The release the reference jar is: the folder's `jar-<version>` suffix, 26.2 for the plain one.
pub(crate) fn jar_version() -> String {
    let root = jar_root();
    let name = root.file_name().and_then(|name| name.to_str());
    name.and_then(|name| name.strip_prefix("jar-"))
        .unwrap_or("26.2")
        .to_owned()
}
