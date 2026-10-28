//! The `registry` family: every entry of every built-in registry, with the id the network gives
//! it, in order. A wrong `since` or `until` shifts every later id; this is the case that sees it.

use serde_json::{Value, json};

use crate::case::case;

/// The registries compared, each a row.
const REGISTRIES: &[&str] =
    &["item", "block", "block_state", "entity_type", "sound_event", "particle_type", "menu", "block_entity_type"];

/// The versions the oracle can capture; a new one adds its value and its oracle build.
const VERSIONS: &[&str] = &["26.2"];

pub fn cases() -> Vec<Value> {
    VERSIONS
        .iter()
        .map(|version| {
            let rows: Vec<Value> = REGISTRIES.iter().map(|name| json!({"label": name})).collect();
            case(
                format!("registry/{version}/entries"),
                "registry",
                "entries",
                json!({"version": version, "rows": rows}),
            )
        })
        .collect()
}
