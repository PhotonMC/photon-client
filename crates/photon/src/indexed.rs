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

/// The asset index file under `root`: the one of the game version the jar's assets belong to,
/// else the one the installed game's version metadata names, else the only one there is (the
/// development setup keeps no version metadata beside its index).
fn index_file(root: &Path, version: Option<photon_core::version::Version>) -> Option<PathBuf> {
    if let Some(version) = version {
        let own = root
            .join("indexes")
            .join(format!("{}.json", version.asset_index()));
        if own.is_file() {
            return Some(own);
        }
    }
    if let Some(id) = photon_content::asset_index_id(root) {
        return Some(root.join("indexes").join(format!("{}.json", id.as_str())));
    }
    // Several indexes and no metadata (a development folder that keeps more than one version):
    // the highest-numbered one is the newest game's.
    let mut found: Vec<(u64, PathBuf)> = std::fs::read_dir(root.join("indexes"))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            let number = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.parse().ok())
                .unwrap_or(0);
            (number, path)
        })
        .collect();
    found.sort();
    if found.len() > 1 {
        log::warn!(
            "{}: several asset indexes and no version metadata, using the newest",
            root.display()
        );
    }
    found.pop().map(|(_, path)| path)
}

/// Whether `text` is a SHA-1 digest as the index writes it. Only one is safe in a file path.
fn is_digest(text: &str) -> bool {
    text.len() == 40 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

impl IndexedAssets {
    /// The index of the assets that sit above the jar's folder: two folders up in the development
    /// layout (`<assets>/jar/assets`), three in an installed game (`<cache>/jar/<hash>/assets`).
    pub(crate) fn open(jar_assets: &Path) -> Option<Self> {
        let root = jar_assets
            .ancestors()
            .skip(2)
            .take(2)
            .find(|folder| folder.join("indexes").is_dir())?
            .to_path_buf();
        let index = photon_core::json::read_object(&index_file(
            &root,
            photon_core::content_root::assets_version(jar_assets),
        )?)?;
        let hashes = index
            .object("objects")?
            .entries()
            .filter_map(|(key, entry)| {
                let JsonValue::Object(entry) = entry else {
                    return None;
                };
                let hash = entry.text("hash").filter(|hash| is_digest(hash))?;
                Some((String::from_utf16(key).ok()?, hash))
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

#[cfg(test)]
mod tests {
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("photon-indexed-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("indexes")).unwrap();
        dir
    }

    #[test]
    fn both_layouts_find_the_index_above_the_jar() {
        for (name, jar) in [("dev", "jar/assets"), ("installed", "jar/abc/assets")] {
            let root = scratch(name);
            std::fs::write(root.join("indexes/32.json"), r#"{"objects":{}}"#).unwrap();
            let jar_assets = root.join(jar);
            std::fs::create_dir_all(&jar_assets).unwrap();
            let found = super::IndexedAssets::open(&jar_assets);
            assert!(found.is_some_and(|found| found.root == root), "{name}");
            std::fs::remove_dir_all(&root).unwrap();
        }
    }

    #[test]
    fn the_only_index_is_taken_and_of_two_without_metadata_the_newest() {
        let root = scratch("only");
        std::fs::write(root.join("indexes/32.json"), "{}").unwrap();
        assert_eq!(
            super::index_file(&root, None),
            Some(root.join("indexes/32.json"))
        );
        std::fs::write(root.join("indexes/33.json"), "{}").unwrap();
        assert_eq!(
            super::index_file(&root, None),
            Some(root.join("indexes/33.json"))
        );
        // A jar of a known version takes that version's own index, whatever else is there.
        std::fs::write(root.join("indexes/34.json"), "{}").unwrap();
        let version = photon_core::version::Version::V26_2;
        assert_eq!(
            super::index_file(&root, Some(version)),
            Some(root.join("indexes/32.json"))
        );
        assert_eq!(
            super::index_file(&root, None),
            Some(root.join("indexes/34.json"))
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn only_a_digest_names_an_object_file() {
        assert!(super::is_digest("da39a3ee5e6b4b0d3255bfef95601890afd80709"));
        for bad in [
            "",
            "ab",
            "../../etc/passwd",
            "DA39A3EE5E6B4B0D3255BFEF95601890AFD80709",
        ] {
            assert!(!super::is_digest(bad), "{bad}");
        }
    }
}
