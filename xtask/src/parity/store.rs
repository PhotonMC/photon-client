//! Captured Java outputs, one compressed JSONL file per version and family.
//!
//! Each line is `{"key": ..., "record": ...}`. Rewriting a family keeps only
//! the keys its current cases use, so stale captures disappear on their own.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde_json::{Value, json};

use crate::process;

pub struct Store {
    dir: PathBuf,
    records: HashMap<String, Value>,
    /// Lines each family's file held when loaded.
    loaded: BTreeMap<String, usize>,
    changed: BTreeSet<String>,
}

impl Store {
    pub fn open(version: &str) -> Self {
        Self {
            dir: process::root().join(".data/parity").join(version),
            records: HashMap::new(),
            loaded: BTreeMap::new(),
            changed: BTreeSet::new(),
        }
    }

    fn file(&self, family: &str) -> PathBuf {
        self.dir.join(format!("{family}.jsonl.gz"))
    }

    /// Loads the captures of the given families.
    pub fn load(&mut self, families: &[String]) -> Result<(), String> {
        for family in families {
            let Ok(file) = File::open(self.file(family)) else {
                continue;
            };
            let mut count = 0;
            for line in BufReader::new(GzDecoder::new(file)).lines() {
                count += 1;
                let line = line.map_err(|error| format!("{family} captures: {error}"))?;
                let mut entry: Value =
                    serde_json::from_str(&line).map_err(|error| error.to_string())?;
                let key = entry["key"].as_str().unwrap_or_default().to_owned();
                self.records.insert(key, entry["record"].take());
            }
            self.loaded.insert(family.clone(), count);
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.records.get(key)
    }

    pub fn insert(&mut self, family: &str, key: String, record: Value) {
        self.records.insert(key, record);
        self.changed.insert(family.to_owned());
    }

    /// Rewrites each family whose captures changed or whose stored keys are
    /// not exactly the keys in use, keeping only the keys in use.
    pub fn save(&self, in_use: &BTreeMap<String, Vec<String>>) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|error| error.to_string())?;
        for (family, keys) in in_use {
            // Every key in use is present (missing ones were captured, which
            // marks the family changed), so an equal count means no stale lines.
            let current =
                !self.changed.contains(family) && self.loaded.get(family) == Some(&keys.len());
            if current {
                continue;
            }
            let path = self.file(family);
            let temporary = path.with_extension("tmp");
            let mut encoder = GzEncoder::new(
                File::create(&temporary).map_err(|error| error.to_string())?,
                Compression::default(),
            );
            for key in keys {
                if let Some(record) = self.records.get(key) {
                    let line = json!({"key": key, "record": record});
                    writeln!(encoder, "{line}").map_err(|error| error.to_string())?;
                }
            }
            encoder.finish().map_err(|error| error.to_string())?;
            fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}
