//! The registry numbering of every supported version against the data reports of that
//! version's own server (`cargo xtask server --version <id> --reports`). A version whose reports
//! are not fetched is reported as missing, not counted as checked.

use std::collections::BTreeMap;
use std::path::PathBuf;

use photon_core::version::Version;
use photon_data::version_registries::VersionRegistries;
use serde_json::Value;

use super::entries;

fn reports(version: Version) -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/reports")
        .join(version.name())
        .join("reports");
    path.is_dir().then_some(path)
}

fn read(path: PathBuf) -> Value {
    let text = std::fs::read_to_string(&path).expect("report file");
    serde_json::from_str(&text).expect("report json")
}

/// The names of a registry in the order the server numbers them.
fn reference_order(registries: &Value, key: &str) -> Vec<String> {
    let mut entries: Vec<(u64, String)> = registries[key]["entries"]
        .as_object()
        .expect("entries")
        .iter()
        .map(|(name, entry)| (entry["protocol_id"].as_u64().expect("id"), name.clone()))
        .collect();
    entries.sort();
    entries.into_iter().map(|(_, name)| name).collect()
}

fn ours(label: &str, registries: &VersionRegistries, version: Version) -> Vec<String> {
    let mut rows: Vec<(u64, String)> = entries(label, registries, version)
        .expect("registry")
        .iter()
        .map(|row| {
            (
                row["id"].as_u64().expect("id"),
                row["name"].as_str().expect("name").to_owned(),
            )
        })
        .collect();
    rows.sort();
    rows.into_iter().map(|(_, name)| name).collect()
}

#[test]
fn every_version_numbers_its_registries_like_its_own_server() {
    for version in Version::ALL.iter().copied() {
        let Some(dir) = reports(version) else {
            eprintln!("missing reference: no data reports for {}", version.name());
            continue;
        };
        let registries = VersionRegistries::for_version(version);
        let reference = read(dir.join("registries.json"));
        for (label, key) in [
            ("item", "minecraft:item"),
            ("block", "minecraft:block"),
            ("entity_type", "minecraft:entity_type"),
            ("sound_event", "minecraft:sound_event"),
            ("particle_type", "minecraft:particle_type"),
            ("menu", "minecraft:menu"),
            ("block_entity_type", "minecraft:block_entity_type"),
            ("attribute", "minecraft:attribute"),
            ("component_type", "minecraft:data_component_type"),
        ] {
            let want = reference_order(&reference, key);
            let got = ours(label, &registries, version);
            let first = want.iter().zip(&got).position(|(w, g)| w != g);
            assert!(
                want == got,
                "{} {label}: {} entries, want {}; first difference at {first:?}",
                version.name(),
                got.len(),
                want.len()
            );
        }
        let blocks = read(dir.join("blocks.json"));
        let mut want: BTreeMap<u64, String> = BTreeMap::new();
        for (name, block) in blocks.as_object().expect("blocks") {
            for state in block["states"].as_array().expect("states") {
                want.insert(state["id"].as_u64().expect("id"), name.clone());
            }
        }
        let got: BTreeMap<u64, String> = entries("block_state", &registries, version)
            .expect("states")
            .iter()
            .map(|row| {
                (
                    row["id"].as_u64().expect("id"),
                    row["name"].as_str().expect("name").to_owned(),
                )
            })
            .collect();
        assert!(want == got, "{} block states differ", version.name());
    }
}
