//! Every entry of every built-in registry with its wire id, from the version's registries.

use photon_core::version::Version;
use photon_data::block::registry::BLOCKS;
use photon_data::item::registry::ITEMS;
use photon_data::version_registries::{IdMap, VersionRegistries};
use serde_json::{Value, json};

/// The answer for the `entries` operation: per row the entries of the registry it names.
pub(crate) fn registry_result(request: &Value) -> Result<Value, String> {
    if request["operation"].as_str() != Some("entries") {
        return Err("unknown registry operation".into());
    }
    let version = match request["version"].as_str() {
        Some("26.2") => Version::V26_2,
        _ => return Err("unsupported registry version".into()),
    };
    let registries = VersionRegistries::for_version(version);
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let label = row["label"].as_str().ok_or("label")?;
        rows.push(json!({"label": label, "entries": entries(label, &registries, version)?}));
    }
    Ok(json!({"rows": rows}))
}

/// The entries of `names` the version has, with the wire id `map` gives each canonical place.
fn listed(names: impl Iterator<Item = String>, map: &IdMap) -> Vec<Value> {
    names
        .enumerate()
        .filter_map(|(index, name)| {
            map.to_wire(index as u32)
                .map(|id| json!({"name": name, "id": id}))
        })
        .collect()
}

fn entries(
    label: &str,
    registries: &VersionRegistries,
    version: Version,
) -> Result<Vec<Value>, String> {
    let strings =
        |names: &[&str]| -> Vec<String> { names.iter().map(|name| (*name).to_owned()).collect() };
    Ok(match label {
        "item" => listed(
            ITEMS.iter().map(|item| item.name.to_owned()),
            &registries.items,
        ),
        "block" => blocks(version),
        "block_state" => block_states(registries, version),
        "entity_type" => listed(
            strings(photon_data::entity_type::names()).into_iter(),
            &registries.entity_types,
        ),
        "sound_event" => listed(
            strings(photon_data::sound_event::names()).into_iter(),
            &registries.sound_events,
        ),
        "particle_type" => listed(
            photon_data::particle_type::PARTICLE_TYPES
                .iter()
                .map(|particle| format!("minecraft:{}", particle.name)),
            &registries.particle_types,
        ),
        "menu" => listed(
            strings(photon_data::menu_type::names()).into_iter(),
            &registries.menu_types,
        ),
        "block_entity_type" => listed(
            strings(photon_data::block_entity_type::names()).into_iter(),
            &registries.block_entity_types,
        ),
        _ => return Err("unknown registry".into()),
    })
}

/// The blocks the version has, numbered in order.
fn blocks(version: Version) -> Vec<Value> {
    BLOCKS
        .iter()
        .filter(|block| block.exists_in(version))
        .enumerate()
        .map(|(id, block)| json!({"name": block.name, "id": id}))
        .collect()
}

/// Every state of the blocks the version has, with the property values that name it.
fn block_states(registries: &VersionRegistries, version: Version) -> Vec<Value> {
    let mut canonical = 0u32;
    let mut states = Vec::new();
    for block in &BLOCKS {
        for offset in 0..block.state_count() {
            if let Some(id) = registries.block_states.to_wire(canonical) {
                debug_assert!(block.exists_in(version));
                let properties: Vec<String> = block
                    .properties_at(offset)
                    .unwrap_or_default()
                    .iter()
                    .map(|(name, value)| format!("{name}={value}"))
                    .collect();
                states.push(json!({"name": block.name, "id": id, "properties": properties}));
            }
            canonical += 1;
        }
    }
    states
}
