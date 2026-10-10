//! The `block` family: registry tables read from the reference rather than
//! rebuilt.
//!
//! The inputs are tiny because the oracle enumerates the registry itself;
//! the expected output is the whole table, which the Rust side has to
//! reproduce.

use serde_json::{Value, json};

/// Case label and oracle operation of each whole-registry table.
const TABLES: &[(&str, &str)] = &[
    ("state-definitions", "state_definitions"),
    ("declarations", "block_declarations"),
    ("item-declarations", "item_declarations"),
    ("collision-shapes", "collision_shapes"),
    ("outline-shapes", "outline_shapes"),
    ("movement-properties", "movement_properties"),
    ("offset-properties", "offset_properties"),
    ("fluid-overlay-blocks", "fluid_overlay_blocks"),
    ("motion-blocking", "motion_blocking"),
    ("fluid-flow-properties", "fluid_flow_properties"),
    ("light-properties", "light_properties"),
    ("face-occlusion", "face_occlusion"),
    ("destroy-properties", "destroy_properties"),
    ("item-tools", "item_tools"),
    ("item-stacking", "item_stacking"),
    ("item-components", "item_components"),
    ("tooltip-registries", "tooltip_registries"),
    ("potions", "potions"),
    ("creative-tabs", "creative_tabs"),
];

/// The creative search box's tag text: path text, a namespace and a path
/// (trimmed), nothing after '#', upper case (the tree lowercases, the
/// tooltip tags do not), an untrimmed path, and nothing found.
const TAG_SEARCHES: &[(&str, &str)] = &[
    ("path", "#logs"),
    ("namespace-path", "#minecraft:logs"),
    ("trimmed", "# mine : wool "),
    ("namespace-only", "#minecraft:"),
    ("empty", "#"),
    ("upper", "#Logs"),
    ("untrimmed", "#wool "),
    ("nested-path", "#enchantable/sw"),
    ("none", "#qqq"),
];

fn table(label: &str, operation: &str) -> Value {
    json!({
        "case_id": format!("block/{label}"), "family": "block", "operation": operation,
        "rows": [{"label": "registry"}],
    })
}

pub fn cases() -> Vec<Value> {
    let mut cases: Vec<Value> = TABLES.iter().map(|(label, operation)| table(label, operation)).collect();
    let rows: Vec<Value> = TAG_SEARCHES.iter().map(|(label, text)| json!({"label": label, "text": text})).collect();
    cases.push(json!({
        "case_id": "block/creative-tag-search", "family": "block",
        "operation": "creative_tag_search", "rows": rows,
    }));
    cases.push(table("behaviours", "block_behaviours"));
    cases.push(table("tints", "block_tints"));
    cases
}
