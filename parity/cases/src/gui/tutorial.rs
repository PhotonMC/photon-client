//! The `tutorial` case: the tutorial's steps through scripts of events.

use serde_json::{Value, json};

use super::gui;

struct Player {
    mode: &'static str,
    inventory: &'static [&'static str],
    mined: &'static [&'static str],
    crafted: &'static [&'static str],
}

const SURVIVOR: Player = Player { mode: "SURVIVAL", inventory: &[], mined: &[], crafted: &[] };

const CREATIVE: Player = Player { mode: "CREATIVE", ..SURVIVOR };

fn row(label: &str, step: &str, player: &Player, events: Vec<Value>) -> Value {
    json!({
        "label": label, "step": step, "mode": player.mode, "inventory": player.inventory,
        "mined": player.mined, "crafted": player.crafted, "events": events,
    })
}

fn ticks(n: i64) -> Value {
    json!({"do": "tick", "n": n, "keys": [], "mouse": []})
}

fn moving(n: i64, keys: &[&str], mouse: &[[f64; 2]]) -> Value {
    json!({"do": "tick", "n": n, "keys": keys, "mouse": mouse})
}

fn look(block: &str) -> Value {
    json!({"do": "look", "block": block})
}

fn destroy(block: &str, percent: f64) -> Value {
    json!({"do": "destroy", "block": block, "percent": percent})
}

fn get_item(item: &str) -> Value {
    json!({"do": "get_item", "item": item})
}

fn inventory() -> Value {
    json!({"do": "inventory"})
}

fn update(at: i64) -> Value {
    json!({"do": "update", "at": at})
}

fn mode(value: &str) -> Value {
    json!({"do": "mode", "value": value})
}

fn stop() -> Value {
    json!({"do": "stop"})
}

fn movement_rows() -> Vec<Value> {
    let mut rows = movement_start_rows();
    rows.extend(movement_end_rows());
    rows
}

fn movement_start_rows() -> Vec<Value> {
    let turn = [[0.5, 0.0]];
    let fast = [[0.0, 0.5]];
    vec![
        row(
            "movement-hint-edge",
            "movement",
            &SURVIVOR,
            vec![ticks(1), ticks(98), ticks(1), update(0), ticks(1), update(100)],
        ),
        row(
            "movement-keys",
            "movement",
            &SURVIVOR,
            vec![
                ticks(1),
                moving(39, &["forward"], &[]),
                moving(1, &["forward"], &[]),
                moving(1, &["forward"], &[]),
                moving(1, &["left"], &[]),
                update(0),
                moving(5, &["jump"], &[]),
                ticks(1),
            ],
        ),
        row(
            "movement-each-key",
            "movement",
            &SURVIVOR,
            vec![
                ticks(1),
                moving(10, &["forward"], &[]),
                moving(10, &["backward"], &[]),
                moving(10, &["left"], &[]),
                moving(10, &["right"], &[]),
                moving(2, &["jump"], &[]),
            ],
        ),
        row(
            "movement-mouse",
            "movement",
            &SURVIVOR,
            vec![ticks(1), moving(39, &[], &turn), moving(1, &[], &turn), moving(1, &[], &fast), update(0)],
        ),
        row(
            "movement-mouse-thresholds",
            "movement",
            &SURVIVOR,
            vec![
                ticks(1),
                moving(30, &[], &[[0.006, 0.006], [0.006, 0.0]]),
                moving(30, &[], &[[0.01, 0.01]]),
                moving(30, &[], &[[-0.0100001, 0.0]]),
                moving(30, &[], &[[0.0, -0.0100001]]),
            ],
        ),
    ]
}

fn movement_end_rows() -> Vec<Value> {
    let turn = [[0.5, 0.0]];
    let mut rows = vec![
        row(
            "movement-both-survival",
            "movement",
            &SURVIVOR,
            vec![ticks(1), moving(41, &["forward"], &turn), ticks(1), update(0), ticks(1)],
        ),
        row(
            "movement-look-hint",
            "movement",
            &SURVIVOR,
            vec![ticks(1), moving(42, &["forward"], &[]), ticks(18), ticks(1), ticks(1), update(0), ticks(1)],
        ),
        row(
            "movement-look-first",
            "movement",
            &SURVIVOR,
            vec![ticks(1), moving(42, &[], &turn), ticks(99), ticks(1), update(0), moving(60, &["forward"], &[])],
        ),
    ];
    rows.push(row(
        "movement-both-creative",
        "movement",
        &CREATIVE,
        vec![ticks(1), moving(41, &["forward"], &turn), ticks(1), update(0), ticks(1)],
    ));
    rows.push(row(
        "movement-mode-changes",
        "movement",
        &SURVIVOR,
        vec![ticks(1), moving(41, &["forward"], &turn), mode("CREATIVE"), ticks(1), ticks(1)],
    ));
    rows
}

/// The find-tree step: the hint, what completes it, and what the first tick sees.
fn find_tree_rows() -> Vec<Value> {
    let mut rows = vec![
        row(
            "find-tree-hint",
            "find_tree",
            &SURVIVOR,
            vec![ticks(1), ticks(5998), ticks(1), ticks(1), update(0), ticks(1)],
        ),
        row("find-tree-look-stone", "find_tree", &SURVIVOR, vec![ticks(1), look("minecraft:stone"), ticks(1)]),
    ];
    for block in [
        "minecraft:oak_log",
        "minecraft:oak_leaves",
        "minecraft:crimson_stem",
        "minecraft:nether_wart_block",
        "minecraft:mangrove_roots",
        "minecraft:dirt",
    ] {
        rows.push(row(
            &format!("find-tree-look-{block}"),
            "find_tree",
            &SURVIVOR,
            vec![ticks(1), look(block), ticks(1), ticks(1)],
        ));
    }
    rows.push(row(
        "find-tree-look-before-start",
        "find_tree",
        &SURVIVOR,
        vec![look("minecraft:oak_log"), ticks(1), ticks(1)],
    ));
    rows.push(row(
        "find-tree-get-item",
        "find_tree",
        &SURVIVOR,
        vec![ticks(1), get_item("minecraft:oak_log"), ticks(1)],
    ));
    rows.push(row(
        "find-tree-get-other-item",
        "find_tree",
        &SURVIVOR,
        vec![ticks(1), get_item("minecraft:stick"), get_item("minecraft:oak_planks"), ticks(1)],
    ));
    rows.extend(find_tree_shortcut_rows());
    rows
}

fn find_tree_shortcut_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let logs = Player { inventory: &["minecraft:oak_log"], ..SURVIVOR };
    rows.push(row("find-tree-inventory-first-tick", "find_tree", &logs, vec![ticks(1), ticks(1), ticks(1)]));
    let leaves = Player { inventory: &["minecraft:oak_leaves"], ..SURVIVOR };
    rows.push(row("find-tree-inventory-leaves", "find_tree", &leaves, vec![ticks(1), ticks(1)]));
    let mined = Player { mined: &["minecraft:birch_log"], ..SURVIVOR };
    rows.push(row("find-tree-mined-before", "find_tree", &mined, vec![ticks(1), ticks(1), ticks(1)]));
    let dirt = Player { mined: &["minecraft:dirt"], inventory: &["minecraft:stick"], ..SURVIVOR };
    rows.push(row("find-tree-mined-other", "find_tree", &dirt, vec![ticks(1), ticks(1), ticks(1)]));
    rows.push(row("find-tree-creative", "find_tree", &CREATIVE, vec![ticks(1), ticks(1), ticks(1)]));
    rows.push(row(
        "find-tree-shortcut-second-tick-only",
        "find_tree",
        &SURVIVOR,
        vec![ticks(1), ticks(1), get_item("minecraft:stick"), ticks(3)],
    ));
    rows
}

/// The punch-tree step, and the tree toast of the step before it, which may still be shown.
fn punch_tree_rows() -> Vec<Value> {
    let mut rows = punch_tree_start_rows();
    rows.extend(punch_tree_end_rows());
    rows.extend(tree_toast_rows());
    rows
}

fn punch_tree_start_rows() -> Vec<Value> {
    vec![
        row(
            "punch-tree-hint",
            "punch_tree",
            &SURVIVOR,
            vec![ticks(1), ticks(598), ticks(1), ticks(1), update(0), ticks(1)],
        ),
        row(
            "punch-tree-progress",
            "punch_tree",
            &SURVIVOR,
            vec![
                ticks(1),
                destroy("minecraft:oak_log", 0.3),
                ticks(600),
                destroy("minecraft:oak_log", 0.3),
                destroy("minecraft:oak_log", 0.6),
                destroy("minecraft:oak_log", -1.0),
                destroy("minecraft:stone", 0.5),
                destroy("minecraft:oak_log", 0.0),
                update(0),
                destroy("minecraft:oak_log", 1.0),
                ticks(1),
            ],
        ),
    ]
}

fn punch_tree_end_rows() -> Vec<Value> {
    let mut rows = vec![
        row(
            "punch-tree-resets",
            "punch_tree",
            &SURVIVOR,
            vec![
                ticks(1),
                destroy("minecraft:oak_log", -1.0),
                destroy("minecraft:oak_log", -1.0),
                destroy("minecraft:oak_log", -1.0),
                ticks(1),
                destroy("minecraft:oak_log", -1.0),
                ticks(1),
                ticks(1),
            ],
        ),
        row(
            "punch-tree-resets-of-other-blocks",
            "punch_tree",
            &SURVIVOR,
            vec![
                ticks(1),
                destroy("minecraft:stone", -1.0),
                destroy("minecraft:stone", -1.0),
                destroy("minecraft:stone", -1.0),
                destroy("minecraft:stone", -1.0),
                destroy("minecraft:stone", 0.5),
                ticks(1),
            ],
        ),
        row(
            "punch-tree-finish-before-toast",
            "punch_tree",
            &SURVIVOR,
            vec![ticks(1), destroy("minecraft:oak_log", 1.0), ticks(1), ticks(1)],
        ),
        row("punch-tree-get-log", "punch_tree", &SURVIVOR, vec![ticks(1), get_item("minecraft:birch_log"), ticks(1)]),
        row(
            "punch-tree-get-other",
            "punch_tree",
            &SURVIVOR,
            vec![ticks(1), get_item("minecraft:oak_planks"), ticks(1)],
        ),
    ];
    let logs = Player { inventory: &["minecraft:oak_log"], ..SURVIVOR };
    rows.push(row("punch-tree-inventory-log", "punch_tree", &logs, vec![ticks(1), ticks(1)]));
    let wart = Player { inventory: &["minecraft:nether_wart_block"], ..SURVIVOR };
    rows.push(row("punch-tree-inventory-wart", "punch_tree", &wart, vec![ticks(1), ticks(1)]));
    let mined = Player { mined: &["minecraft:oak_leaves"], ..SURVIVOR };
    rows.push(row("punch-tree-mined", "punch_tree", &mined, vec![ticks(1), ticks(1)]));
    rows
}

/// The find-tree toast and the punch-tree toast have the same icon.
fn tree_toast_rows() -> Vec<Value> {
    // The find-tree toast is up when the player looks at a tree, and the punch-tree toast has the
    // same icon: the progress belongs to the new one.
    vec![
        row(
            "tree-toasts-waiting",
            "find_tree",
            &SURVIVOR,
            vec![
                ticks(1),
                ticks(6000),
                look("minecraft:oak_log"),
                ticks(1),
                ticks(600),
                destroy("minecraft:oak_log", 0.5),
                destroy("minecraft:oak_log", 0.75),
                update(0),
                update(1000),
                destroy("minecraft:oak_log", -1.0),
            ],
        ),
        row(
            "tree-toasts-shown",
            "find_tree",
            &SURVIVOR,
            vec![
                ticks(1),
                ticks(6000),
                update(0),
                update(700),
                look("minecraft:oak_log"),
                ticks(1),
                ticks(600),
                update(701),
                destroy("minecraft:oak_log", 0.5),
                update(702),
                destroy("minecraft:oak_log", 0.75),
                update(1500),
                destroy("minecraft:oak_log", -1.0),
                update(2000),
                update(4000),
                update(9000),
            ],
        ),
    ]
}

/// The last steps: the inventory, the planks, and the end.
fn late_rows() -> Vec<Value> {
    let mut rows = inventory_rows();
    rows.extend(craft_planks_rows());
    rows.extend(ending_rows());
    rows
}

fn inventory_rows() -> Vec<Value> {
    vec![
        row(
            "open-inventory-hint",
            "open_inventory",
            &SURVIVOR,
            vec![ticks(1), ticks(598), ticks(1), ticks(1), update(0), ticks(1)],
        ),
        row(
            "open-inventory-opened",
            "open_inventory",
            &SURVIVOR,
            vec![ticks(1), ticks(700), update(0), inventory(), ticks(1), update(1000)],
        ),
        row("open-inventory-before-start", "open_inventory", &SURVIVOR, vec![inventory(), ticks(1), ticks(1)]),
        row("open-inventory-creative", "open_inventory", &CREATIVE, vec![ticks(1), ticks(1), ticks(1)]),
    ]
}

fn craft_planks_rows() -> Vec<Value> {
    let planks = Player { inventory: &["minecraft:oak_planks"], ..SURVIVOR };
    let crafted = Player { crafted: &["minecraft:spruce_planks"], ..SURVIVOR };
    let other = Player { crafted: &["minecraft:stick"], inventory: &["minecraft:oak_log"], ..SURVIVOR };
    vec![
        row(
            "craft-planks-hint",
            "craft_planks",
            &SURVIVOR,
            vec![ticks(1), ticks(1198), ticks(1), ticks(1), update(0), ticks(1)],
        ),
        row(
            "craft-planks-got",
            "craft_planks",
            &SURVIVOR,
            vec![ticks(1), ticks(1300), update(0), get_item("minecraft:birch_planks"), ticks(1), update(2000)],
        ),
        row(
            "craft-planks-got-other",
            "craft_planks",
            &SURVIVOR,
            vec![ticks(1), get_item("minecraft:oak_log"), ticks(1)],
        ),
        row("craft-planks-inventory", "craft_planks", &planks, vec![ticks(1), ticks(1), ticks(1)]),
        row("craft-planks-crafted", "craft_planks", &crafted, vec![ticks(1), ticks(1), ticks(1)]),
        row("craft-planks-other-things", "craft_planks", &other, vec![ticks(1), ticks(1), ticks(1)]),
        row("craft-planks-creative", "craft_planks", &CREATIVE, vec![ticks(1), ticks(1)]),
    ]
}

fn ending_rows() -> Vec<Value> {
    vec![
        row(
            "none-step",
            "none",
            &SURVIVOR,
            vec![
                ticks(1),
                moving(5, &["forward"], &[[1.0, 1.0]]),
                look("minecraft:oak_log"),
                destroy("minecraft:oak_log", 1.0),
                inventory(),
                get_item("minecraft:oak_log"),
                ticks(1),
            ],
        ),
        row("stored-step-unknown", "something_else", &SURVIVOR, vec![ticks(1), ticks(1)]),
        row(
            "stop-in-movement",
            "movement",
            &SURVIVOR,
            vec![
                ticks(1),
                moving(30, &["forward"], &[]),
                ticks(80),
                update(0),
                stop(),
                update(1000),
                ticks(1),
                ticks(98),
                ticks(1),
            ],
        ),
        row(
            "stop-with-toast-waiting",
            "punch_tree",
            &SURVIVOR,
            vec![ticks(1), ticks(600), stop(), stop(), update(0), ticks(1)],
        ),
        row(
            "events-before-start",
            "punch_tree",
            &SURVIVOR,
            vec![
                look("minecraft:oak_log"),
                destroy("minecraft:oak_log", 1.0),
                inventory(),
                get_item("minecraft:oak_log"),
                stop(),
                ticks(1),
                ticks(1),
            ],
        ),
    ]
}

/// The tutorial: each step's hint and way out, the thresholds of the mouse, the shortcuts the
/// first tick takes, the game mode, the level going away, and two steps' toasts sharing an icon.
pub fn tutorial() -> Value {
    let mut rows = movement_rows();
    rows.extend(find_tree_rows());
    rows.extend(punch_tree_rows());
    rows.extend(late_rows());
    gui("tutorial", "tutorial", rows)
}
