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

/// `block_state` select: the stack's merged `block_state` component, a property it holds or lacks.
fn select_rows() -> Vec<Value> {
    let ask = |label: &str, property: &str, item: &str, extra: Value| row(label, json!(property), item, extra);
    vec![
        ask("light-default", "level", "minecraft:light", json!({})),
        ask("light-level-7", "level", "minecraft:light", json!({"block_state": {"level": "7"}})),
        ask("light-removed", "level", "minecraft:light", json!({"remove_block_state": true})),
        ask("light-asks-other", "waterlogged", "minecraft:light", json!({})),
        ask("light-patch-other-key", "level", "minecraft:light", json!({"block_state": {"waterlogged": "true"}})),
        ask("beehive-plain", "honey_level", "minecraft:beehive", json!({})),
        ask("beehive-honey-5", "honey_level", "minecraft:beehive", json!({"block_state": {"honey_level": "5"}})),
        ask("stone-nothing", "level", "minecraft:stone", json!({})),
        ask(
            "statue-pose",
            "copper_golem_pose",
            "minecraft:oxidized_copper_golem_statue",
            json!({"block_state": {"copper_golem_pose": "sitting"}}),
        ),
    ]
}

/// The value the `block_state` select reads from a stack.
pub fn item_select() -> Value {
    gui("item-select", "item_select", select_rows())
}

/// The needle's wobbler over a run of game ticks: still on a tick, moving on the next.
fn wobble_rows() -> Vec<Value> {
    let run_with = |label: &str, (wobble, factor): (bool, f64), steps: &[(i64, f64)]| {
        let steps: Vec<Value> = steps.iter().map(|(tick, target)| json!({"tick": tick, "target": target})).collect();
        json!({"label": label, "kind": "wobble", "wobble": wobble, "factor": factor, "steps": steps})
    };
    let run = |label: &str, wobble: bool, steps: &[(i64, f64)]| run_with(label, (wobble, 0.8), steps);
    let climb: Vec<(i64, f64)> = (1..=12).map(|tick| (tick, 0.3)).collect();
    vec![
        run("wobble-climbs-to-the-target", true, &climb),
        run("wobble-same-tick-twice", true, &[(4, 0.9), (4, 0.1), (5, 0.9)]),
        run("wobble-starts-at-tick-zero", true, &[(0, 0.4), (1, 0.4)]),
        run("wobble-wraps-the-short-way", true, &[(1, 0.95), (2, 0.95), (3, 0.95), (4, 0.95)]),
        run("plain-follows-every-call", false, &[(4, 0.9), (4, 0.1), (5, 0.6)]),
        run_with("clock-factor-climbs", (true, 0.9), &climb),
        run("wobble-long-gap", true, &[(1, 0.5), (900, 0.5), (901, 0.2)]),
    ]
}

/// The needle turn of a holder that is not the local player, towards a block.
fn towards_rows() -> Vec<Value> {
    let at = |label: &str, (x, y, z): (f64, f64, f64), yaw: f64, block: [i32; 3]| json!({"label": label, "kind": "towards", "tick": 10, "owner": {"x": x, "y": y, "z": z, "yaw": yaw}, "block": block});
    vec![
        at("north-of-the-block", (0.5, 64.0, 20.5), 0.0, [0, 64, 0]),
        at("east-of-the-block", (-30.5, 64.0, 0.5), 90.0, [0, 64, 0]),
        at("looking-away", (10.0, 70.0, -4.0), 213.5, [-3, 12, 40]),
        at("negative-yaw", (123.25, 5.0, -77.75), -47.0, [100, 80, -100]),
        at("yaw-wraps", (0.0, 0.0, 0.0), 725.0, [5, 0, 5]),
        at("same-column", (0.5, 90.0, 0.5), 33.0, [0, 10, 0]),
        at("far", (30000000.0, 64.0, -30000000.0), 12.5, [-30000000, 64, 30000000]),
    ]
}

/// The compass needle: the wobbler and the turn towards a block.
pub fn item_needle() -> Value {
    gui("item-needle", "item_needle", [wobble_rows(), towards_rows()].concat())
}
