//! `tests/behavior/reference.json`: the pinned game version, its jars and
//! the oracle sources of every family.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha1::{Digest, Sha1};

use super::digest;
use crate::process;

#[derive(Deserialize)]
pub struct Pins {
    pub version: String,
    pub protocol: u32,
    pub java_major: u32,
    pub client_file: String,
    pub client_sha1: String,
    pub libraries: BTreeMap<String, Library>,
    pub families: BTreeMap<String, Family>,
}

#[derive(Deserialize)]
pub struct Library {
    pub file: String,
    pub sha1: String,
}

#[derive(Deserialize)]
pub struct Family {
    pub source: String,
    #[serde(default)]
    pub extra_sources: Vec<String>,
    pub libraries: Vec<String>,
}

/// The oracle's Java sources directory.
pub fn java_dir() -> PathBuf {
    process::root().join("tests/behavior/java")
}

/// Where an oracle source lives: `common/` holds the dispatcher and `Reflect`, which name no
/// game class, `v26_2/` the classes written against the 26.2 game.
fn source_path(name: &str) -> PathBuf {
    let common = java_dir().join("common").join(name);
    if common.is_file() {
        common
    } else {
        java_dir().join("v26_2").join(name)
    }
}

impl Pins {
    pub fn load() -> Result<Self, String> {
        let path = process::root().join("tests/behavior/reference.json");
        let text =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
    }

    /// The library keys every family needs, with Linux natives renamed to
    /// this machine's natives.
    pub fn library_keys(&self) -> BTreeSet<String> {
        self.families
            .values()
            .flat_map(|family| &family.libraries)
            .map(|key| native_key(key))
            .collect()
    }

    /// The client jar and every library, each checked against its pinned hash.
    pub fn classpath(&self, runtime: &Path) -> Result<Vec<PathBuf>, String> {
        let mut verified = Verified::load();
        let client = runtime.join(&self.client_file);
        verified.check(&client, &self.client_sha1)?;
        let mut classpath = vec![client];
        for key in self.library_keys() {
            let library = self
                .libraries
                .get(&key)
                .ok_or_else(|| format!("reference.json lists no library `{key}`"))?;
            let path = runtime.join("libs").join(&library.file);
            verified.check(&path, &library.sha1)?;
            classpath.push(path);
        }
        verified.save();
        Ok(classpath)
    }

    /// The sources one family compiles from: the dispatcher, its own oracle
    /// and the helpers it names.
    pub fn family_sources(&self, family: &str) -> Result<Vec<PathBuf>, String> {
        let entry = self
            .families
            .get(family)
            .ok_or_else(|| format!("unknown parity family `{family}`"))?;
        let mut sources: Vec<PathBuf> = [
            "Oracle.java",
            "Reflect.java",
            "Game26_2.java",
            "Access26_2.java",
            "ScratchDirectory.java",
        ]
        .into_iter()
        .map(source_path)
        .collect();
        sources.push(source_path(&entry.source));
        sources.extend(entry.extra_sources.iter().map(|extra| source_path(extra)));
        Ok(sources)
    }

    /// Every family's sources, compiled together into one class directory.
    pub fn all_sources(&self) -> Result<Vec<PathBuf>, String> {
        let mut all = BTreeSet::new();
        for family in self.families.keys() {
            all.extend(self.family_sources(family)?);
        }
        Ok(all.into_iter().collect())
    }

    /// A digest of a family's oracle sources: part of every case key.
    pub fn family_digest(&self, family: &str) -> Result<String, String> {
        let mut parts = Vec::new();
        for path in self.family_sources(family)? {
            let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            parts.push(format!("{}={}", path.display(), digest(&bytes)));
        }
        parts.sort();
        Ok(digest(parts.join("\n").as_bytes()))
    }
}

/// LWJGL's name for this machine's natives, as the version file lists them.
fn native_platform() -> &'static str {
    let arm = matches!(std::env::consts::ARCH, "aarch64" | "arm");
    match std::env::consts::OS {
        "macos" if arm => "macos-arm64",
        "macos" => "macos",
        "windows" if arm => "windows-arm64",
        "windows" if std::env::consts::ARCH == "x86" => "windows-x86",
        "windows" => "windows",
        _ => "linux",
    }
}

fn native_key(key: &str) -> String {
    match key.strip_suffix("-natives-linux") {
        Some(base) => format!("{base}-natives-{}", native_platform()),
        None => key.to_owned(),
    }
}

/// Jar hashes already checked, keyed by path, size and modification time,
/// so a run does not re-hash 200 MB of unchanged jars.
struct Verified {
    entries: BTreeMap<String, String>,
    changed: bool,
}

impl Verified {
    fn path() -> PathBuf {
        process::root().join(".data/parity/verified.json")
    }

    fn load() -> Self {
        let entries = fs::read_to_string(Self::path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Self {
            entries,
            changed: false,
        }
    }

    fn check(&mut self, path: &Path, sha1: &str) -> Result<(), String> {
        let metadata = fs::metadata(path)
            .map_err(|_| format!("missing reference file {} (run setup)", path.display()))?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |since| since.as_nanos());
        let stamp = format!("{}:{}:{modified}", path.display(), metadata.len());
        if self.entries.get(&stamp).is_some_and(|known| known == sha1) {
            return Ok(());
        }
        let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let actual = format!("{:x}", Sha1::digest(&bytes));
        if actual != sha1 {
            return Err(format!("reference hash mismatch: {}", path.display()));
        }
        self.entries.insert(stamp, actual);
        self.changed = true;
        Ok(())
    }

    fn save(&self) {
        if self.changed {
            let path = Self::path();
            let _ = fs::create_dir_all(path.parent().unwrap_or(&path));
            let _ = fs::write(
                &path,
                serde_json::to_string(&self.entries).unwrap_or_default(),
            );
        }
    }
}
