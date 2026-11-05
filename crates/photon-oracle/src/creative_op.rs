//! The creative inventory's stacks as the parity rows hold them, and the operator tab.

use photon_data::item::component_types::NAMES;
use photon_data::item::creative::CreativeStack;
use photon_data::item::creative_tabs::OPERATOR_TAB_ITEMS;
use serde_json::{Value, json};

use crate::tooltip::component_json;

/// Component type ids as the oracle version's server numbers them.
fn wire_ids(ids: &[i32]) -> Vec<i32> {
    let map = &crate::case_ids::registries().component_types;
    ids.iter().map(|&id| map.wire_or_unknown(id)).collect()
}

/// A table stack: its item, the wire bytes of the stack, the component ids it sets and removes,
/// and the components by name.
pub(super) fn stack_json(
    entry: &CreativeStack,
    registries: &photon_game::registries::Registries,
) -> Value {
    let made = photon_game::menus::creative::table_stack(entry, registries);
    let mut wire = Vec::new();
    photon_core::wire::write_varint(made.item_count(), &mut wire);
    photon_core::wire::write_varint(
        crate::case_ids::wire_item(made.item_registry_id()),
        &mut wire,
    );
    let registries_for_wire = crate::case_ids::registries();
    let numbering = photon_data::item_components::Numbering::outgoing(&registries_for_wire);
    wire.extend(made.patch().trusted_for(&numbering).unwrap_or_default());
    let mut components = serde_json::Map::new();
    for component in entry.components {
        if let Some((id, _)) =
            photon_game::item::default_components::encode(component, Some(registries))
        {
            components.insert(NAMES[id as usize].to_owned(), component_json(component));
        }
    }
    json!({
        "item": entry.item,
        "wire": wire.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "positive": wire_ids(made.positive_component_type_ids()),
        "negative": wire_ids(made.negative_component_type_ids()),
        "components": components,
    })
}

/// The operator tab for a player who can use game master blocks, what it gives the search tab,
/// and the search tab's whole list then.
pub(super) fn operator_tabs(label: &str) -> Value {
    let data_root = crate::jar_root().join("data");
    let registries = photon_game::registries::Registries::vanilla(&data_root);
    let stacks = |entries: &[CreativeStack]| -> Vec<Value> {
        entries
            .iter()
            .map(|entry| stack_json(entry, &registries))
            .collect()
    };
    let search_tab = photon_game::menus::creative::search_list();
    json!({
        "label": label,
        "outcome": "listed",
        "should_display": true,
        "items": stacks(&OPERATOR_TAB_ITEMS),
        "search_items": stacks(&OPERATOR_TAB_ITEMS),
        "search_tab": stacks(search_tab),
    })
}

/// The hotbar file's item codec: a stack stored as its compound, or a compound read as its stack.
pub(super) fn hotbar_entry(label: &str, row: &Value) -> Result<Value, String> {
    use photon_core::ids::ItemId;
    use photon_core::nbt::{Field, FieldValue};
    use photon_game::hotbar::{decode_entry, encode_entry};
    let text = |key: &str| row.get(key).and_then(Value::as_str);
    let entry = if text("kind") == Some("store") {
        let stack = text("item")
            .and_then(photon_data::item::registry::index)
            .zip(row.get("count").and_then(Value::as_i64))
            .map(|(index, count)| {
                photon_world::ClientItemStackState::new(count as i32, ItemId(index as i32))
            });
        let FieldValue::Compound(fields) = encode_entry(stack.as_ref()) else {
            return Err("an entry is a compound".to_owned());
        };
        let mut object = serde_json::Map::new();
        for Field { key, value } in fields {
            let value = match value {
                FieldValue::Str(text) => json!(text),
                FieldValue::Int(number) => json!(number),
                _ => json!("other"),
            };
            object.insert(key, value);
        }
        Value::Object(object)
    } else {
        let spec = row.get("entry").ok_or("a parse row has an entry")?;
        let mut fields = Vec::new();
        if let Some(id) = spec.get("id").and_then(Value::as_str) {
            fields.push(Field {
                key: "id".to_owned(),
                value: FieldValue::Str(id.to_owned()),
            });
        }
        if let Some(count) = spec.get("count").and_then(Value::as_i64) {
            fields.push(Field {
                key: "count".to_owned(),
                value: FieldValue::Int(count as i32),
            });
        }
        if spec.get("components").is_some() {
            fields.push(Field {
                key: "components".to_owned(),
                value: FieldValue::Compound(Vec::new()),
            });
        }
        match decode_entry(&FieldValue::Compound(fields)) {
            Some(stack) => {
                let name = photon_data::item::registry::ITEMS
                    .get(stack.item_registry_id().0 as usize)
                    .map(|item| item.name);
                json!({"item": name, "count": stack.item_count()})
            }
            None => json!({}),
        }
    };
    Ok(json!({"label": label, "outcome": "coded", "entry": entry}))
}
