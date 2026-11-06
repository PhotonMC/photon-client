//! Registry ids in cases for a version other than the base.
//!
//! The cases are written once, with the ids of the base version's registries. Another version
//! numbers its registries differently where it adds or removes entries, so for it each id is
//! translated by name (and, for block states, properties) through the two versions'
//! registries (the oracle binary answers it). The registry family checks those against Java.

use std::collections::HashMap;

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
    /// The translation to `version`, or none when it is the base or the client's registries
    /// cannot be asked.
    pub fn load(version: &str) -> Option<Self> {
        if version == BASE {
            return None;
        }
        let root = super::super::process::root();
        super::super::process::run(
            "cargo",
            &[
                "build",
                "--quiet",
                "--release",
                "--package",
                "photon-oracle",
            ],
        )
        .ok()?;
        let output = std::process::Command::new(root.join("target/release/photon-oracle"))
            .args(["translate-ids", BASE, version])
            .output()
            .ok()?;
        let answer: Value = serde_json::from_slice(&output.stdout).ok()?;
        let mut maps = HashMap::new();
        for (field, registry) in FIELDS {
            let map = answer[registry]
                .as_object()?
                .iter()
                .filter_map(|(old, new)| Some((old.parse::<i64>().ok()?, new.as_i64()?)))
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
