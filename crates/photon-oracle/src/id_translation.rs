//! The registry ids of the parity cases, which are written with one version's numbering, for
//! another version's: `photon-oracle translate-ids <from> <to>`. The numbering of a version is
//! the client's own (`VersionRegistries`); the registry family checks it against Java.

use photon_core::version::Version;
use photon_data::version_registries::{IdMap, VersionRegistries};
use serde_json::{Map, Value, json};

/// For each registry, the ids that differ in `to`, as `{"<from id>": <to id>}`.
pub(super) fn translate(from: &str, to: &str) -> Result<Value, String> {
    let (from, to) = (version(from)?, version(to)?);
    let (old, new) = (VersionRegistries::of(from), VersionRegistries::of(to));
    Ok(json!({
        "block_state": differing(&old.block_states, &new.block_states),
        "entity_type": differing(&old.entity_types, &new.entity_types),
    }))
}

fn version(name: &str) -> Result<Version, String> {
    Version::from_name(name).ok_or_else(|| format!("unknown version {name}"))
}

/// Every id of `old` that `new` numbers differently, matched by the canonical entry.
fn differing(old: &IdMap, new: &IdMap) -> Value {
    let mut map = Map::new();
    for wire in 0..old.wire_len() as u32 {
        let target = old
            .to_canonical(wire)
            .and_then(|canonical| new.to_wire(canonical));
        if let Some(target) = target.filter(|target| *target != wire) {
            map.insert(wire.to_string(), json!(target));
        }
    }
    Value::Object(map)
}
