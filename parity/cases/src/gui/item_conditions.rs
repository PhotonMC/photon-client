//! The conditions an item model decides from a stack alone.

use serde_json::{Value, json};

use super::gui;

/// One stack and the property asked of it.
fn row(label: &str, property: Value, item: &str, extra: Value) -> Value {
    let mut row = json!({"label": label, "property": property, "item": item, "components": []});
    for (key, value) in extra.as_object().into_iter().flatten() {
        row[key] = value.clone();
    }
    row
}

/// `broken`: a damageable stack one point from its end, either side of that point.
fn broken_rows() -> Vec<Value> {
    let broken = || json!({"property": "minecraft:broken"});
    let elytra = |label: &str, extra: Value| row(label, broken(), "minecraft:elytra", extra);
    vec![
        elytra("elytra-new", json!({})),
        elytra("elytra-430", json!({"damage": 430})),
        elytra("elytra-431", json!({"damage": 431})),
        elytra("elytra-432", json!({"damage": 432})),
        elytra("elytra-over", json!({"damage": 500})),
        elytra("elytra-negative", json!({"damage": -3})),
        elytra("elytra-unbreakable", json!({"damage": 431, "unbreakable": true})),
        elytra("elytra-max-damage-10-at-9", json!({"damage": 9, "max_damage": 10})),
        elytra("elytra-max-damage-10-at-8", json!({"damage": 8, "max_damage": 10})),
        elytra("elytra-max-damage-1", json!({"max_damage": 1})),
        elytra("elytra-max-damage-0", json!({"damage": 3, "max_damage": 0})),
        row("stone-damaged", broken(), "minecraft:stone", json!({"damage": 5})),
        row("stick-with-max-damage", broken(), "minecraft:stick", json!({"damage": 4, "max_damage": 5})),
        elytra("elytra-damage-removed-at-max", json!({"damage": 431, "remove": ["damage"]})),
        elytra("elytra-max-removed", json!({"damage": 431, "remove": ["max_damage"]})),
        elytra("elytra-max-1-damage-removed", json!({"max_damage": 1, "remove": ["damage"]})),
        row("stone-max-damage-only", broken(), "minecraft:stone", json!({"max_damage": 1})),
        row("stone-max-and-damage", broken(), "minecraft:stone", json!({"max_damage": 5, "damage": 4})),
        row("stone-max-and-damage-below", broken(), "minecraft:stone", json!({"max_damage": 5, "damage": 3})),
        row("sword-431", broken(), "minecraft:diamond_sword", json!({"damage": 1560})),
        row("sword-1561", broken(), "minecraft:diamond_sword", json!({"damage": 1561})),
    ]
}

/// `has_component`: a component the stack's own patch adds, with and without `ignore_default`.
fn component_rows() -> Vec<Value> {
    let has = |name: &str, ignore: bool| json!({"property": "minecraft:has_component", "component": name, "ignore_default": ignore});
    let dyed = "minecraft:dyed_color";
    let tracker = "minecraft:lodestone_tracker";
    vec![
        row("wolf-armor-plain", has(dyed, false), "minecraft:wolf_armor", json!({})),
        row("wolf-armor-dyed", has(dyed, false), "minecraft:wolf_armor", json!({"components": [dyed]})),
        row("wolf-armor-dyed-ignore-default", has(dyed, true), "minecraft:wolf_armor", json!({"components": [dyed]})),
        row("wolf-armor-tracker-asks-dyed", has(dyed, false), "minecraft:wolf_armor", json!({"components": [tracker]})),
        row("compass-plain", has(tracker, false), "minecraft:compass", json!({})),
        row("compass-tracked", has(tracker, false), "minecraft:compass", json!({"components": [tracker]})),
        row(
            "compass-tracked-ignore-default",
            has(tracker, true),
            "minecraft:compass",
            json!({"components": [tracker]}),
        ),
        row("compass-both", has(tracker, false), "minecraft:compass", json!({"components": [dyed, tracker]})),
        row("leather-dyed", has(dyed, false), "minecraft:leather_helmet", json!({"components": [dyed]})),
        row("leather-plain", has(dyed, false), "minecraft:leather_helmet", json!({})),
    ]
}

/// The conditions an item model decides from a stack alone (`Broken`, `HasComponent`).
pub fn item_condition() -> Value {
    gui("item-condition", "item_condition", [broken_rows(), component_rows()].concat())
}
