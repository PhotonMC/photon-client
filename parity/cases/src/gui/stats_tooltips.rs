//! Rows of the statistics screen for the tooltips of its widgets (the sort buttons and the tabs
//! with nothing in them), which wait before they show: the pointer or the keyboard focus on the
//! widget, frames drawn, time passing.

use serde_json::{Value, json};

use super::stats::{screen_row, with};

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn key(code: i64, modifiers: i64) -> Value {
    json!({"do": "key", "key": code, "modifiers": modifiers})
}

fn frame() -> Value {
    json!({"do": "frame"})
}

fn wait(ms: i64) -> Value {
    json!({"do": "wait", "ms": ms})
}

fn on(label: &str, stats: &Value, mouse: [i64; 2], steps: Vec<Value>) -> Value {
    with(screen_row(label, (427, 240), stats, true), json!({"steps": steps, "mouse": mouse}))
}

/// The sort buttons of the items list and the tab with nothing in it.
pub(super) fn tooltip_rows() -> Vec<Value> {
    let all = json!([
        ["minecraft:custom", "minecraft:jump", 1234],
        ["minecraft:used", "minecraft:diamond_sword", 5],
        ["minecraft:killed", "minecraft:zombie", 12],
    ]);
    let no_items = json!([["minecraft:custom", "minecraft:jump", 1234], ["minecraft:killed", "minecraft:zombie", 12]]);
    let items = || click(200, 10);
    let button = [140, 38];
    let tab = [200, 10];
    let away = [0, 0];
    vec![
        on("tip-button-final", &all, button, vec![items()]),
        on("tip-button-frame", &all, button, vec![items(), frame()]),
        on("tip-button-wait-100", &all, button, vec![items(), frame(), wait(100)]),
        on("tip-button-wait-600", &all, button, vec![items(), frame(), wait(600)]),
        on("tip-button-wait-600-frames", &all, button, vec![items(), frame(), wait(600), frame()]),
        on("tip-button-wait-1500", &all, button, vec![items(), frame(), wait(1500)]),
        on("tip-button-last", &all, [340, 38], vec![items(), frame(), wait(1500)]),
        on("tip-button-focus", &all, away, vec![items(), key(258, 0), key(258, 0), frame(), wait(1500)]),
        on("tip-button-focus-frame", &all, away, vec![items(), key(258, 0), key(258, 0), frame()]),
        on("tip-button-arrow", &all, away, vec![items(), key(264, 0), key(264, 0), frame(), wait(1500)]),
        on("tip-empty-tab-final", &no_items, tab, vec![]),
        on("tip-empty-tab-frame", &no_items, tab, vec![frame()]),
        on("tip-empty-tab-wait-100", &no_items, tab, vec![frame(), wait(100)]),
        on("tip-empty-tab-wait-600", &no_items, tab, vec![frame(), wait(600)]),
        on("tip-empty-tab-wait-1500", &no_items, tab, vec![frame(), wait(1500)]),
        on("tip-empty-tab-focus", &no_items, away, vec![click(70, 10), key(50, 2), frame(), wait(1500)]),
        on("tip-empty-tab-focus-frame", &no_items, away, vec![click(70, 10), key(50, 2), frame()]),
        on("tip-empty-tab-focus-keyboard", &no_items, away, vec![key(263, 0), key(50, 2), frame(), wait(1500)]),
        on("tip-empty-tab-focus-keyboard-frame", &no_items, away, vec![key(263, 0), key(50, 2), frame()]),
        on("tip-empty-tab-click", &no_items, tab, vec![click(200, 10), frame(), wait(1500)]),
        on("tip-tab-hover", &all, tab, vec![frame(), wait(1500)]),
    ]
}
