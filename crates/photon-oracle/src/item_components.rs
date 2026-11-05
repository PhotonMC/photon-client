//! The `item_components` case: the reference's default component patches, read by this port and
//! written back, and the default components of each item as the declarations case lists them.

use super::*;

/// The reference's whole default patch of each item, kept as a fixture, is read by this port, and
/// both forms are written back from what it read.
pub(super) fn item_components(label: &str) -> Value {
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    // The reference's whole default patch of each item, kept as a
    // fixture, is read by this port, and both forms are written
    // back from what it read.
    const PATCHES: &str = include_str!("../../../tests/behavior/fixtures/item-default-patches.txt");
    use photon_data::item::component_sets::DEFAULT_COMPONENT_SETS;
    let items: Vec<Value> = PATCHES
        .lines()
        .map(|line| {
            let (name, wire) = line.split_once(' ').expect("a name and its bytes");
            let index = photon_data::item::registry::index(name).expect("a listed item");
            let set = DEFAULT_COMPONENT_SETS.get(index).copied().unwrap_or(0);
            let map = &crate::case_ids::registries().component_types;
            let types: Vec<u32> = (0..128u32)
                .filter(|id| set >> id & 1 == 1)
                .filter_map(|id| map.to_wire(id))
                .collect();
            let mut types = types;
            types.sort_unstable();
            let bytes: Vec<u8> = (0..wire.len())
                .step_by(2)
                .map(|at| u8::from_str_radix(&wire[at..at + 2], 16).expect("hex"))
                .collect();
            let mut input = bytes.as_slice();
            let registries = crate::case_ids::registries();
            let outgoing = photon_game::item_components::Numbering::outgoing(&registries);
            let patch = photon_game::item_components::read_wire_patch(&mut input, &registries);
            match patch {
                Ok(patch) if input.is_empty() => {
                    let trusted = patch.trusted_for(&outgoing);
                    let delimited = patch.delimited_for(&outgoing);
                    match (trusted, delimited) {
                        (Some(trusted), Some(delimited)) => {
                            let mut components: Vec<_> = patch
                                .added
                                .iter()
                                .filter_map(|(id, _)| map.to_wire(*id as u32))
                                .collect();
                            components.sort_unstable();
                            json!({
                                "name": name,
                                "wire": hex(&trusted),
                                "delimited": hex(&delimited),
                                "components": components,
                                "types": types,
                            })
                        }
                        _ => json!({"name": name, "wire": "unreadable"}),
                    }
                }
                _ => json!({"name": name, "wire": "unreadable"}),
            }
        })
        .collect();
    json!({"label": label, "outcome": "listed", "items": items})
}

/// An item's default components the client reads, as the codec's JSON and
/// as the network's bytes, keyed by component type name.
pub(super) fn default_components(
    item: &photon_data::item::definition::Item,
    registries: &photon_game::registries::Registries,
) -> (Value, Value) {
    use photon_data::item::component::Component;
    use photon_data::item::component_types::NAMES;
    let mut json_map = serde_json::Map::new();
    let mut wire_map = serde_json::Map::new();
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let oracle = crate::case_ids::registries();
    let mut put = |id: i32, json: Value, bytes: &[u8]| {
        // A component the oracle's version does not have is not one of its defaults.
        if oracle.component_types.to_wire(id as u32).is_none() {
            return;
        }
        let name = NAMES[id as usize];
        json_map.insert(name.to_owned(), json);
        wire_map.insert(name.to_owned(), json!(hex(bytes)));
    };
    let declared_name = item
        .components
        .iter()
        .any(|component| matches!(component, Component::ItemName(_)));
    if !declared_name {
        let key = photon_game::item::default_components::description_key(item);
        put(
            photon_data::item::component_types::ITEM_NAME,
            json!({"translate": key}),
            &photon_game::item::default_components::translatable(&key),
        );
    }
    for component in item.components {
        if let Some((id, bytes)) =
            photon_game::item::default_components::encode(component, Some(registries))
        {
            put(id, component_json(component), &bytes);
        }
    }
    (Value::Object(json_map), Value::Object(wire_map))
}
