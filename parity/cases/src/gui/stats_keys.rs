//! Rows of the statistics screen for the keys it takes: the tab bar's Control shortcuts and the
//! keyboard focus (Tab, the arrows, Enter and Space).

use serde_json::{Value, json};

use super::stats::{screen_row, with};

const CTRL: i64 = 2;
const SHIFT: i64 = 1;
const TAB: i64 = 258;

fn key(code: i64, modifiers: i64) -> Value {
    json!({"do": "key", "key": code, "modifiers": modifiers})
}

fn digit(digit: i64) -> i64 {
    48 + digit
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

/// `count` presses of the arrow `code` after a click at the point.
fn after_click(x: i64, y: i64, code: i64, count: usize) -> Vec<Value> {
    let mut steps = vec![click(x, y)];
    steps.extend((0..count).map(|_| key(code, 0)));
    steps
}

/// `count` presses of the arrow `code` after `tabs` presses of Tab.
fn after_tabs(tabs: usize, code: i64, count: usize) -> Vec<Value> {
    let mut steps: Vec<Value> = (0..tabs).map(|_| key(TAB, 0)).collect();
    steps.extend((0..count).map(|_| key(code, 0)));
    steps
}

/// The Items tab, its first button focused by Tab, then `count` presses of the arrow `code`.
fn items_then_arrows(code: i64, count: usize) -> Vec<Value> {
    let mut steps = items_then_tab(2, 0);
    steps.extend((0..count).map(|_| key(code, 0)));
    steps
}

fn enter_after(mut steps: Vec<Value>) -> Vec<Value> {
    steps.push(key(257, 0));
    steps
}

fn enter_twice(steps: Vec<Value>) -> Vec<Value> {
    enter_after(enter_after(steps))
}

fn space_after(mut steps: Vec<Value>) -> Vec<Value> {
    steps.push(key(32, 0));
    steps
}

/// A loaded 427x240 screen of the statistics `stats`, and what is done to it.
fn on(label: &str, stats: &Value, loaded: bool, steps: Vec<Value>) -> Value {
    with(screen_row(label, (427, 240), stats, loaded), json!({"steps": steps}))
}

/// Every tab can be chosen: general, an item used and a mob killed.
fn all_stats() -> Value {
    json!([
        ["minecraft:custom", "minecraft:jump", 1234],
        ["minecraft:used", "minecraft:diamond_sword", 5],
        ["minecraft:killed", "minecraft:zombie", 12],
    ])
}

/// The Items tab chosen by a Ctrl+digit, then `count` presses of Tab with the modifiers.
fn items_then_tab(count: usize, modifiers: i64) -> Vec<Value> {
    let mut steps = vec![key(digit(2), CTRL)];
    steps.extend((0..count).map(|_| key(TAB, modifiers)));
    steps
}

/// The keys the tab bar takes (`TabNavigationBar.keyPressed`): Control with a digit or Tab, in a
/// loaded screen with every tab usable, one with the middle tab empty, and while loading.
pub(super) fn key_rows() -> Vec<Value> {
    let all = all_stats();
    let no_items = json!([["minecraft:custom", "minecraft:jump", 1234], ["minecraft:killed", "minecraft:zombie", 12]]);
    let only_general = json!([["minecraft:custom", "minecraft:jump", 1234]]);
    let mut rows = vec![
        on("key-ctrl-2", &all, true, vec![key(digit(2), CTRL)]),
        on("key-ctrl-3", &all, true, vec![key(digit(3), CTRL)]),
        on("key-ctrl-1-stays", &all, true, vec![key(digit(1), CTRL)]),
        on("key-ctrl-0-last", &all, true, vec![key(digit(0), CTRL)]),
        on("key-ctrl-5-last", &all, true, vec![key(digit(5), CTRL)]),
        on("key-ctrl-3-inactive", &no_items, true, vec![key(digit(2), CTRL)]),
        on("key-ctrl-2-then-1", &all, true, vec![key(digit(2), CTRL), key(digit(1), CTRL)]),
        on("key-ctrl-tab", &all, true, vec![key(TAB, CTRL)]),
        on("key-ctrl-tab-twice", &all, true, vec![key(TAB, CTRL), key(TAB, CTRL)]),
        on("key-ctrl-tab-wraps", &all, true, vec![key(TAB, CTRL), key(TAB, CTRL), key(TAB, CTRL)]),
        on("key-ctrl-shift-tab", &all, true, vec![key(TAB, CTRL | SHIFT)]),
        on("key-ctrl-tab-skips-empty", &no_items, true, vec![key(TAB, CTRL)]),
        on("key-ctrl-shift-tab-skips-empty", &no_items, true, vec![key(TAB, CTRL | SHIFT), key(TAB, CTRL | SHIFT)]),
        on("key-ctrl-tab-loading", &only_general, false, vec![key(TAB, CTRL)]),
        on("key-ctrl-2-loading", &json!([]), false, vec![key(digit(2), CTRL)]),
        on("key-ctrl-tab-only-general", &only_general, true, vec![key(TAB, CTRL)]),
        on("key-digit-alone", &all, true, vec![key(digit(2), 0)]),
        on("key-shift-digit", &all, true, vec![key(digit(2), SHIFT)]),
        on("key-ctrl-keypad-digit", &all, true, vec![key(322, CTRL)]),
        on("key-ctrl-letter", &all, true, vec![key(65, CTRL)]),
        on("key-after-list-click", &all, true, vec![click(200, 40), key(digit(2), CTRL)]),
        on("key-after-list-click-back", &all, true, vec![click(200, 40), key(digit(2), CTRL), key(digit(1), CTRL)]),
        on("key-after-list-click-inactive", &no_items, true, vec![click(200, 40), key(digit(2), CTRL)]),
        on("key-focused-tab-inactive", &no_items, true, vec![click(70, 10), key(digit(2), CTRL)]),
        on(
            "key-focused-tab-inactive-keyboard",
            &no_items,
            true,
            vec![click(70, 10), key(TAB, CTRL), key(digit(2), CTRL)],
        ),
        on("key-after-tab-click", &all, true, vec![click(200, 10), key(digit(3), CTRL), key(TAB, CTRL)]),
    ];
    rows.extend(navigation_rows());
    rows
}

/// The keyboard focus (`Screen.keyPressed`): Tab, Enter, Space and the arrows.
fn navigation_rows() -> Vec<Value> {
    let mut rows = tab_order_rows();
    rows.extend(items_focus_rows());
    rows.extend(enter_rows());
    rows.extend(arrow_rows());
    rows
}

/// Tab and Shift+Tab around the bar, the list and Done.
fn tab_order_rows() -> Vec<Value> {
    let all = all_stats();
    vec![
        on("nav-tab-1", &all, true, vec![key(TAB, 0)]),
        on("nav-tab-2", &all, true, vec![key(TAB, 0), key(TAB, 0)]),
        on("nav-tab-3", &all, true, vec![key(TAB, 0), key(TAB, 0), key(TAB, 0)]),
        on("nav-tab-4", &all, true, vec![key(TAB, 0), key(TAB, 0), key(TAB, 0), key(TAB, 0)]),
        on("nav-shift-tab-1", &all, true, vec![key(TAB, SHIFT)]),
        on("nav-shift-tab-2", &all, true, vec![key(TAB, SHIFT), key(TAB, SHIFT)]),
        on("nav-shift-tab-3", &all, true, vec![key(TAB, SHIFT), key(TAB, SHIFT), key(TAB, SHIFT)]),
        on("nav-list-click-tab", &all, true, vec![click(200, 40), key(TAB, 0)]),
        on("nav-list-click-tab-2", &all, true, vec![click(200, 40), key(TAB, 0), key(TAB, 0)]),
        on("nav-list-click-shift-tab", &all, true, vec![click(200, 40), key(TAB, SHIFT)]),
        on("nav-tab-click-tab", &all, true, vec![click(200, 10), key(TAB, 0)]),
        on("nav-tab-click-tab-2", &all, true, vec![click(200, 10), key(TAB, 0), key(TAB, 0)]),
        on("nav-tab-click-shift-tab", &all, true, vec![click(200, 10), key(TAB, SHIFT)]),
        on("nav-loading-tab", &all, false, vec![key(TAB, 0)]),
        on("nav-loading-shift-tab", &all, false, vec![key(TAB, SHIFT)]),
    ]
}

/// Tab through the widgets of the items list.
fn items_focus_rows() -> Vec<Value> {
    let all = all_stats();
    vec![
        on("nav-items-tab-1", &all, true, items_then_tab(1, 0)),
        on("nav-items-tab-2", &all, true, items_then_tab(2, 0)),
        on("nav-items-tab-6", &all, true, items_then_tab(6, 0)),
        on("nav-items-tab-7", &all, true, items_then_tab(7, 0)),
        on("nav-items-tab-9", &all, true, items_then_tab(9, 0)),
        on("nav-items-tab-10", &all, true, items_then_tab(10, 0)),
        on("nav-items-tab-12", &all, true, items_then_tab(12, 0)),
        on("nav-items-shift-tab-1", &all, true, items_then_tab(1, SHIFT)),
        on("nav-items-shift-tab-2", &all, true, items_then_tab(2, SHIFT)),
        on("nav-items-shift-tab-3", &all, true, items_then_tab(3, SHIFT)),
        on("nav-items-shift-tab-6", &all, true, items_then_tab(6, SHIFT)),
        on("nav-items-shift-tab-7", &all, true, items_then_tab(7, SHIFT)),
        on("nav-items-shift-tab-8", &all, true, items_then_tab(8, SHIFT)),
        on("nav-items-shift-tab-9", &all, true, items_then_tab(9, SHIFT)),
        on("nav-items-shift-tab-10", &all, true, items_then_tab(10, SHIFT)),
        on("nav-items-enter-button", &all, true, enter_after(items_then_tab(2, 0))),
        on("nav-items-enter-twice", &all, true, enter_twice(items_then_tab(2, 0))),
        on("nav-items-space-button-2", &all, true, space_after(items_then_tab(3, 0))),
        on("nav-items-enter-slot", &all, true, enter_after(items_then_tab(8, 0))),
    ]
}

/// Enter and Space on the focused child, and Tab in the mobs list.
fn enter_rows() -> Vec<Value> {
    let all = all_stats();
    let no_items = json!([["minecraft:custom", "minecraft:jump", 1234], ["minecraft:killed", "minecraft:zombie", 12]]);
    vec![
        on("nav-done-then-tab-click", &all, true, vec![key(TAB, 0), key(TAB, 0), key(TAB, 0), click(200, 10)]),
        on("nav-done-then-list-click", &all, true, vec![key(TAB, 0), key(TAB, 0), key(TAB, 0), click(200, 70)]),
        on("nav-enter-load", &all, true, vec![key(257, 0)]),
        on("nav-tab-enter", &all, true, vec![key(TAB, 0), key(257, 0)]),
        on("nav-tab-click-enter", &all, true, vec![click(200, 10), key(257, 0)]),
        on("nav-tab-click-space", &all, true, vec![click(200, 10), key(32, 0)]),
        on("nav-inactive-tab-enter", &no_items, true, vec![click(70, 10), key(digit(2), CTRL), key(257, 0)]),
        on("nav-list-enter", &all, true, vec![key(TAB, 0), key(TAB, 0), key(257, 0)]),
        on("nav-mobs-tab-2", &all, true, vec![click(330, 10), key(TAB, 0)]),
        on("nav-mobs-tab-3", &all, true, vec![click(330, 10), key(TAB, 0), key(TAB, 0)]),
        on("nav-mobs-shift-tab-2", &all, true, vec![click(330, 10), key(TAB, SHIFT), key(TAB, SHIFT)]),
        on("nav-mobs-list-click-tab", &all, true, vec![click(330, 10), click(200, 40), key(TAB, 0)]),
    ]
}

/// The arrow keys.
fn arrow_rows() -> Vec<Value> {
    let mut rows = bar_arrow_rows();
    rows.extend(list_arrow_rows());
    rows.extend(items_arrow_rows());
    rows.extend(items_arrow_more_rows());
    rows.extend(items_arrow_last_rows());
    rows
}

/// Arrows on the tab bar.
fn bar_arrow_rows() -> Vec<Value> {
    let all = all_stats();
    let no_items = json!([["minecraft:custom", "minecraft:jump", 1234], ["minecraft:killed", "minecraft:zombie", 12]]);
    vec![
        on("arrow-mobs-bar-down", &all, true, after_click(330, 10, 264, 1)),
        on("arrow-mobs-bar-down-end", &all, true, after_click(330, 10, 264, 8)),
        on("arrow-mobs-bar-down-end-2", &all, true, after_click(330, 10, 264, 12)),
        on("arrow-load-bar-left-right", &all, true, vec![key(262, 0), key(262, 0), key(262, 0), key(263, 0)]),
        on("arrow-loading-down", &all, false, vec![key(264, 0)]),
        on("arrow-loading-up", &all, false, vec![key(265, 0)]),
        on("arrow-load-left", &all, true, vec![key(263, 0)]),
        on("arrow-load-right", &all, true, vec![key(262, 0)]),
        on("arrow-load-up", &all, true, vec![key(265, 0)]),
        on("arrow-load-down", &all, true, vec![key(264, 0)]),
        on("arrow-tabclick-right-1", &all, true, after_click(200, 10, 262, 1)),
        on("arrow-tabclick-right-2", &all, true, after_click(200, 10, 262, 2)),
        on("arrow-tabclick-right-3", &all, true, after_click(200, 10, 262, 3)),
        on("arrow-tabclick-left-1", &all, true, after_click(200, 10, 263, 1)),
        on("arrow-tabclick-left-2", &all, true, after_click(200, 10, 263, 2)),
        on("arrow-general-right-1", &no_items, true, after_click(70, 10, 262, 1)),
        on("arrow-general-right-2", &no_items, true, after_click(70, 10, 262, 2)),
        on("arrow-mobs-left-1", &no_items, true, after_click(330, 10, 263, 1)),
        on("arrow-mobs-left-2", &no_items, true, after_click(330, 10, 263, 2)),
        on("arrow-bar-down-1", &all, true, after_click(70, 10, 264, 1)),
        on("arrow-bar-down-2", &all, true, after_click(70, 10, 264, 2)),
        on("arrow-bar-down-3", &all, true, after_click(70, 10, 264, 3)),
        on("arrow-bar-down-8", &all, true, after_click(70, 10, 264, 8)),
        on("arrow-bar-down-9", &all, true, after_click(70, 10, 264, 9)),
        on("arrow-bar-down-10", &all, true, after_click(70, 10, 264, 10)),
        on("arrow-bar-down-11", &all, true, after_click(70, 10, 264, 11)),
    ]
}

/// Arrows in the general and mobs lists and on Done.
fn list_arrow_rows() -> Vec<Value> {
    let all = all_stats();
    vec![
        on("arrow-shift2-done-up", &all, true, {
            let mut v = vec![key(258, 1), key(258, 1)];
            v.push(key(265, 0));
            v
        }),
        on("arrow-shift2-done-up-2", &all, true, {
            let mut v = vec![key(258, 1), key(258, 1)];
            v.extend((0..2).map(|_| key(265, 0)));
            v
        }),
        on("arrow-shift2-done-down", &all, true, {
            let mut v = vec![key(258, 1), key(258, 1)];
            v.push(key(264, 0));
            v
        }),
        on("arrow-general-down-end", &all, true, after_click(70, 10, 264, 20)),
        on("arrow-general-down-end-up", &all, true, {
            let mut v = after_click(70, 10, 264, 20);
            v.push(key(265, 0));
            v
        }),
        on("arrow-general-down-end-tab", &all, true, {
            let mut v = after_click(70, 10, 264, 20);
            v.push(key(258, 0));
            v
        }),
        on("arrow-list-right-tab", &all, true, vec![key(258, 0), key(258, 0), key(262, 0), key(258, 0)]),
        on("arrow-list-right-shift-tab", &all, true, vec![key(258, 0), key(258, 0), key(262, 0), key(258, 1)]),
        on("arrow-list-right-up", &all, true, vec![key(258, 0), key(258, 0), key(262, 0), key(265, 0)]),
        on("arrow-list-click-right", &all, true, after_click(200, 40, 262, 1)),
        on("arrow-done-up-nosel", &all, true, {
            let mut v = vec![key(258, 1), key(258, 1)];
            v.push(key(265, 0));
            v.push(key(265, 0));
            v
        }),
        on("arrow-done-up-1", &all, true, after_tabs(3, 265, 1)),
        on("arrow-done-up-2", &all, true, after_tabs(3, 265, 2)),
        on("arrow-done-down", &all, true, after_tabs(3, 264, 1)),
        on("arrow-done-left", &all, true, after_tabs(3, 263, 1)),
        on("arrow-list-right", &all, true, after_tabs(2, 262, 1)),
        on("arrow-list-left", &all, true, after_tabs(2, 263, 1)),
        on("arrow-list-up", &all, true, after_tabs(2, 265, 1)),
        on("arrow-list-right-then-down", &all, true, vec![key(258, 0), key(258, 0), key(262, 0), key(264, 0)]),
        on("arrow-list-click-down", &all, true, after_click(200, 40, 264, 1)),
        on("arrow-list-click-up", &all, true, after_click(200, 40, 265, 1)),
        on("arrow-list-click-up-3", &all, true, after_click(200, 40, 265, 3)),
    ]
}

/// Arrows in the items list.
fn items_arrow_rows() -> Vec<Value> {
    let all = all_stats();
    vec![
        on("arrow-items-right-5", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.extend((0..5).map(|_| key(262, 0)));
            v
        }),
        on("arrow-items-right-7", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.extend((0..7).map(|_| key(262, 0)));
            v
        }),
        on("arrow-items-right-8", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.extend((0..8).map(|_| key(262, 0)));
            v
        }),
        on("arrow-items-left-at-start", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.push(key(263, 0));
            v
        }),
        on("arrow-items-left-at-start-2", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.extend((0..2).map(|_| key(263, 0)));
            v
        }),
        on("arrow-items-slot-right", &all, true, {
            let mut v = items_then_tab(8, 0);
            v.push(key(262, 0));
            v
        }),
        on("arrow-items-slot-left", &all, true, {
            let mut v = items_then_tab(8, 0);
            v.push(key(263, 0));
            v
        }),
        on("arrow-items-right-6-down", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.extend((0..6).map(|_| key(262, 0)));
            v.push(key(264, 0));
            v
        }),
        on("arrow-items-bar-down", &all, true, after_click(200, 10, 264, 1)),
        on("arrow-items-bar-down-2", &all, true, after_click(200, 10, 264, 2)),
        on("arrow-items-bar-down-3", &all, true, after_click(200, 10, 264, 3)),
    ]
}

/// More arrows in the items list.
fn items_arrow_more_rows() -> Vec<Value> {
    let all = all_stats();
    vec![
        on("arrow-items-bar-down-up", &all, true, {
            let mut v = after_click(200, 10, 264, 2);
            v.push(key(265, 0));
            v
        }),
        on("arrow-items-down-then-left", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.extend([264, 263].map(|c| key(c, 0)));
            v
        }),
        on("arrow-items-right-6", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.extend((0..6).map(|_| key(262, 0)));
            v
        }),
        on("arrow-items-left-from-nothing", &all, true, {
            let mut v = after_click(200, 10, 263, 1);
            v.push(key(258, 0));
            v
        }),
        on("arrow-items-down-1", &all, true, items_then_arrows(264, 1)),
        on("arrow-items-down-2", &all, true, items_then_arrows(264, 2)),
        on("arrow-items-down-3", &all, true, items_then_arrows(264, 3)),
        on("arrow-items-down-4", &all, true, items_then_arrows(264, 4)),
        on("arrow-items-down-5", &all, true, items_then_arrows(264, 5)),
        on("arrow-items-down-6", &all, true, items_then_arrows(264, 6)),
        on("arrow-items-down-7", &all, true, items_then_arrows(264, 7)),
        on("arrow-items-down-8", &all, true, items_then_arrows(264, 8)),
        on("arrow-items-up-1", &all, true, items_then_arrows(265, 1)),
    ]
}

/// The last arrows in the items list.
fn items_arrow_last_rows() -> Vec<Value> {
    let all = all_stats();
    vec![
        on("arrow-items-up-2", &all, true, items_then_arrows(265, 2)),
        on("arrow-items-up-3", &all, true, items_then_arrows(265, 3)),
        on("arrow-items-tab2-right", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.push(key(262, 0));
            v
        }),
        on("arrow-items-tab2-right-3", &all, true, {
            let mut v = items_then_tab(2, 0);
            v.extend((0..3).map(|_| key(262, 0)));
            v
        }),
        on("arrow-items-tab2-left", &all, true, {
            let mut v = items_then_tab(3, 0);
            v.push(key(263, 0));
            v
        }),
        on("arrow-items-tab2-down", &all, true, {
            let mut v = items_then_tab(5, 0);
            v.push(key(264, 0));
            v
        }),
        on("arrow-items-tab2-down-2", &all, true, {
            let mut v = items_then_tab(5, 0);
            v.extend((0..2).map(|_| key(264, 0)));
            v
        }),
        on("arrow-items-slot-up", &all, true, {
            let mut v = items_then_tab(8, 0);
            v.push(key(265, 0));
            v
        }),
    ]
}
