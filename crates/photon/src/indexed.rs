//! The asset index's objects (`<assets>/indexes/<id>.json` and `objects/`): what the game reads
//! from outside its jar, the sounds and the title screen's panorama among them. Java's vanilla
//! pack looks there before the jar, whose panorama faces are one-pixel stand-ins.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use photon_core::json::JsonValue;

/// The asset index's objects, kept by the hash of their content.
pub(crate) struct IndexedAssets {
    root: PathBuf,
    /// Index path (`minecraft/sounds/x.ogg`) to hash.
    hashes: HashMap<String, String>,
}

impl IndexedAssets {
    /// The index of the assets that sit above the jar's folder (`<assets>/jar/assets`).
    pub(crate) fn open(jar_assets: &Path) -> Option<Self> {
        let root = jar_assets.parent()?.parent()?.to_path_buf();
        let index = photon_core::json::read_object(&root.join("indexes/32.json"))?;
        let hashes = index
            .object("objects")?
            .entries()
            .filter_map(|(key, entry)| {
                let JsonValue::Object(entry) = entry else {
                    return None;
                };
                Some((String::from_utf16(key).ok()?, entry.text("hash")?))
            })
            .collect();
        Some(Self { root, hashes })
    }

    /// Whether the index holds a path (`minecraft/sounds/x.ogg`).
    pub(crate) fn contains(&self, index_path: &str) -> bool {
        self.hashes.contains_key(index_path)
    }

    /// The paths the index names (`minecraft/sounds/x.ogg`).
    pub(crate) fn paths(&self) -> impl Iterator<Item = &str> {
        self.hashes.keys().map(String::as_str)
    }

    /// Where the object of a path is stored.
    pub(crate) fn object_path(&self, index_path: &str) -> Option<PathBuf> {
        let hash = self.hashes.get(index_path)?;
        Some(self.root.join("objects").join(&hash[..2]).join(hash))
    }

    pub(crate) fn object(&self, index_path: &str) -> Option<Vec<u8>> {
        std::fs::read(self.object_path(index_path)?).ok()
    }
}
