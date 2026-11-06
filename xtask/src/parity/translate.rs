//! Registry ids in cases for a version other than the base.
//!
//! The cases are written once, with the ids of the base version's registries. Another version
//! numbers its registries differently where it adds or removes entries, so for it each id is
//! translated by name (and, for block states, properties) through the two versions' registry
//! captures. A capture that is missing leaves the cases as written.

use std::collections::HashMap;
use std::io::Read;

use serde_json::Value;

/// The version the cases' ids are numbered by.
const BASE: &str = "26.2";

/// The case fields that hold a registry id, and the registry each one indexes.
const FIELDS: [(&str, &str); 6] = [
    ("state_id", "block_state"),
    ("entity_type_id", "entity_type"),
    ("type_id", "entity_type"),
    ("passenger_type_id", "entity_type"),
    ("vehicle_type_id", "entity_type"),
    ("remote_type_id", "entity_type"),
];

/// The operations whose `type_id` is not an entity type (it is a particle type).
const OTHER_TYPES: [&str; 1] = ["client_world_entity_effect_particles"];

/// The ids of each field that differ in the version under test.
pub struct Translation {
    maps: HashMap<&'static str, HashMap<i64, i64>>,
}

impl Translation {
    /// The translation to `version`, or none when it is the base or a registry capture is missing.
    pub fn load(version: &str) -> Option<Self> {
        if version == BASE {
            return None;
        }
        let mut maps = HashMap::new();
        for (field, registry) in FIELDS {
            let (old, new) = (entries(BASE, registry)?, entries(version, registry)?);
            let by_name: HashMap<&str, i64> =
                new.iter().map(|(id, key)| (key.as_str(), *id)).collect();
            let map = old
                .iter()
                .filter_map(|(id, key)| by_name.get(key.as_str()).map(|new| (*id, *new)))
                .filter(|(old, new)| old != new)
                .collect();
            maps.insert(field, map);
        }
        Some(Self { maps })
    }

    /// The line with every id field translated, or the line itself when none changes.
    pub fn line(&self, line: String) -> String {
        let Ok(mut value) = serde_json::from_str::<Value>(&line) else {
            return line;
        };
        if OTHER_TYPES.contains(&value["operation"].as_str().unwrap_or_default())
            || !self.walk(&mut value)
        {
            return line;
        }
        value.to_string()
    }

    fn walk(&self, value: &mut Value) -> bool {
        let mut changed = false;
        match value {
            Value::Object(object) => {
                for (key, child) in object.iter_mut() {
                    let new =
                        FIELDS
                            .iter()
                            .find(|(field, _)| field == key)
                            .and_then(|(field, _)| {
                                child.as_i64().and_then(|id| self.maps[field].get(&id))
                            });
                    match new {
                        Some(new) => {
                            *child = Value::from(*new);
                            changed = true;
                        }
                        None => changed |= self.walk(child),
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    changed |= self.walk(item);
                }
            }
            _ => {}
        }
        changed
    }
}

/// A registry's entries in a version's capture: each id with a key naming the entry.
fn entries(version: &str, registry: &str) -> Option<Vec<(i64, String)>> {
    let file = std::fs::File::open(format!(".data/parity/{version}/registry.jsonl.gz")).ok()?;
    let mut text = String::new();
    flate2::read::GzDecoder::new(file)
        .read_to_string(&mut text)
        .ok()?;
    for line in text.lines() {
        let record: Value = serde_json::from_str(line).ok()?;
        for row in record["record"]["result"]["rows"].as_array()? {
            if row["label"].as_str() != Some(registry) {
                continue;
            }
            return row["entries"]
                .as_array()?
                .iter()
                .map(|entry| {
                    let key = format!("{}{}", entry["name"].as_str()?, entry["properties"]);
                    Some((entry["id"].as_i64()?, key))
                })
                .collect();
        }
    }
    None
}
