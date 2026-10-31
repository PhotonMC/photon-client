//! The `block` family: registry tables read from the reference rather than
//! rebuilt.
//!
//! The inputs are tiny because the oracle enumerates the registry itself;
//! the expected output is the whole table, which the Rust side has to
//! reproduce.

use serde_json::{Value, json};

/// Case label and oracle operation of each whole-registry table.
const TABLES: &[(&str, &str)] = &[
    ("registry-names", "registry_names"),
    ("state-definitions", "state_definitions"),
    ("declarations", "block_declarations"),
    ("item-declarations", "item_declarations"),
    ("collision-shapes", "collision_shapes"),
    ("outline-shapes", "outline_shapes"),
    ("movement-properties", "movement_properties"),
    ("offset-properties", "offset_properties"),
    ("fluid-overlay-blocks", "fluid_overlay_blocks"),
    ("motion-blocking", "motion_blocking"),
    ("menu-providers", "menu_providers"),
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
    ("creative-tabs-operator", "creative_tabs_operator"),
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
    cases.push(json!({
        "case_id": "block/hotbar-entry", "family": "block",
        "operation": "hotbar_entry", "rows": hotbar_entries(),
    }));
    cases.push(json!({
        "case_id": "block/spectator-menu", "family": "block",
        "operation": "spectator_menu", "rows": spectator_rows(),
    }));
    cases.extend(crate::block_placement::cases());
    cases.push(table("behaviours", "block_behaviours"));
    cases.push(table("tints", "block_tints"));
    cases
}

/// The hotbar file's entries: stacks stored, and compounds read, as the item codec does.
fn hotbar_entries() -> Vec<Value> {
    let store = |label: &str, item: Option<&str>, count: i32| match item {
        Some(item) => json!({"label": label, "kind": "store", "item": item, "count": count}),
        None => json!({"label": label, "kind": "store"}),
    };
    let parse = |label: &str, entry: Value| json!({"label": label, "kind": "parse", "entry": entry});
    vec![
        store("store-full", Some("minecraft:stone"), 64),
        store("store-one", Some("minecraft:diamond_sword"), 1),
        store("store-empty", None, 0),
        parse("parse-full", json!({"id": "minecraft:stone", "count": 64})),
        parse("parse-no-count", json!({"id": "minecraft:stone"})),
        parse("parse-zero", json!({"id": "minecraft:stone", "count": 0})),
        parse("parse-max", json!({"id": "minecraft:stone", "count": 99})),
        parse("parse-over", json!({"id": "minecraft:stone", "count": 100})),
        parse("parse-unknown", json!({"id": "minecraft:nothing", "count": 1})),
        parse("parse-empty", json!({})),
        parse("parse-empty-components", json!({"id": "minecraft:stone", "components": {}})),
        parse("parse-air", json!({"id": "minecraft:air", "count": 1})),
        parse("parse-no-id", json!({"count": 3})),
    ]
}

/// The spectator menu's rows: a player is its id halves, name and game mode.
fn spectator_rows() -> Vec<Value> {
    let player = |hi: &str, lo: &str, name: &str, mode: i32| json!({"hi": hi, "lo": lo, "name": name, "mode": mode});
    let nine: Vec<Value> = (1..=9).map(|n| player("0", &format!("{n:x}"), &format!("p{n}"), 0)).collect();
    let row =
        |label: &str, players: Vec<Value>, steps: &[&str]| json!({"label": label, "players": players, "steps": steps});
    vec![
        row("none", vec![], &["select:0", "select:1", "scroll:1", "scroll:-1"]),
        row(
            "one",
            vec![player("0", "1", "solo", 0)],
            &["select:0", "select:0", "select:0", "select:0", "scroll:1", "scroll:-1"],
        ),
        row(
            "spectators-hidden",
            vec![player("0", "1", "watcher", 3), player("0", "2", "walker", 0)],
            &["select:0", "select:0", "select:0", "select:0"],
        ),
        row(
            "order",
            vec![
                player("0", "5", "five", 0),
                player("8000000000000000", "1", "high", 0),
                player("ffffffffffffffff", "0", "last", 0),
                player("0", "ffffffffffffffff", "low-max", 0),
            ],
            &["select:0", "select:0", "select:2", "select:2"],
        ),
        row(
            "pages",
            nine,
            &[
                "select:0",
                "select:0",
                "select:7",
                "select:7",
                "select:1",
                "select:0",
                "select:0",
                "select:8",
                "scroll:1",
                "scroll:1",
                "scroll:-1",
            ],
        ),
        row("wheel", vec![player("0", "1", "a", 0)], &["scroll:1", "scroll:1", "scroll:-1", "scroll:-1"]),
    ]
}
