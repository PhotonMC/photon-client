//! The creative inventory's stacks as the parity rows hold them, and the operator tab.

use photon_data::item::component_types::NAMES;
use photon_data::item::creative::CreativeStack;
use photon_data::item::creative_tabs::OPERATOR_TAB_ITEMS;
use serde_json::{Value, json};

use crate::tooltip::component_json;

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
    wire.extend(made.patch().trusted());
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
        "positive": made.positive_component_type_ids(),
        "negative": made.negative_component_type_ids(),
        "components": components,
    })
}

/// The operator tab for a player who can use game master blocks, what it gives the search tab,
/// and the search tab's whole list then.
pub(super) fn operator_tabs(label: &str) -> Value {
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
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
