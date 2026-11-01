//! The in-world game rules screen: the loading text, the list of the rules the server named, the
//! search, the controls, and the keys, clicks and wheel turns that move through it. A row changes
//! one rule at most: the reference writes the changed rules down in the order of a hash set.

use serde_json::{Value, json};

use super::gui;

/// Rules of three categories, two of them numbers, as a server answers.
fn some() -> Value {
    json!([
        ["minecraft:advance_time", "true"],
        ["minecraft:block_drops", "false"],
        ["minecraft:max_entity_cramming", "24"],
        ["minecraft:random_tick_speed", "3"],
        ["minecraft:respawn_radius", "10"],
        ["minecraft:spawn_monsters", "true"],
        ["minecraft:keep_inventory", "false"],
    ])
}

struct Screen {
    label: &'static str,
    size: (i64, i64),
    values: Option<Value>,
    steps: Vec<Value>,
    mouse: (i64, i64),
}

fn screen(label: &'static str) -> Screen {
    Screen { label, size: (427, 240), values: Some(some()), steps: Vec::new(), mouse: (0, 0) }
}

impl Screen {
    fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "width": self.size.0, "height": self.size.1,
            "steps": self.steps, "mouse": [self.mouse.0, self.mouse.1],
        });
        if let Some(values) = self.values {
            row["values"] = values;
        }
        row
    }
}

fn key(code: i64) -> Value {
    json!({"do": "key", "key": code})
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn typed(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

const TAB: i64 = 258;
const ENTER: i64 = 257;
const ESCAPE: i64 = 256;
const DOWN: i64 = 264;
const UP: i64 = 265;
const LEFT: i64 = 263;
const RIGHT: i64 = 262;

/// What the screen shows before and after the answer, for a few windows.
fn shown_rows() -> Vec<Screen> {
    vec![
        Screen { values: None, ..screen("loading") },
        Screen { values: None, size: (854, 480), ..screen("loading-large") },
        Screen { values: None, size: (320, 110), ..screen("loading-small") },
        screen("listed"),
        Screen { size: (854, 480), ..screen("listed-large") },
        Screen { size: (320, 110), ..screen("listed-small") },
        Screen { values: Some(json!([])), ..screen("none-named") },
        Screen {
            values: Some(json!([["minecraft:bogus", "true"], ["minecraft:advance_time", "false"]])),
            ..screen("unknown-skipped")
        },
        Screen {
            values: Some(json!([["minecraft:respawn_radius", "oops"], ["minecraft:advance_time", "maybe"]])),
            ..screen("unreadable-values")
        },
        Screen { steps: vec![json!({"do": "values", "values": some()})], values: None, ..screen("answer-after-draw") },
        Screen {
            steps: vec![json!({"do": "values", "values": [["minecraft:advance_time", "false"]]})],
            ..screen("second-answer-ignored")
        },
    ]
}

/// The search and the wheel.
fn search_rows() -> Vec<Screen> {
    let search = |label, text: &str| Screen { steps: vec![typed(text)], ..screen(label) };
    vec![
        search("search-id", "radius"),
        search("search-name", "cramming"),
        search("search-category", "player"),
        search("search-description", "monsters"),
        search("search-case", "ADVANCE"),
        search("search-none", "zzzz"),
        Screen {
            size: (320, 110),
            steps: vec![json!({"do": "scroll", "x": 160, "y": 60, "amount": -3})],
            ..screen("small-scrolled")
        },
        Screen { mouse: (213, 60), ..screen("hover-category") },
        Screen { mouse: (100, 85), ..screen("hover-rule") },
    ]
}

/// The controls: a click on a toggle, a number typed, a bad number, Done and Cancel.
fn control_rows() -> Vec<Screen> {
    vec![
        Screen { steps: vec![click(300, 76)], ..screen("click-toggle") },
        Screen { steps: vec![click(300, 76), click(300, 76)], ..screen("click-toggle-twice") },
        Screen { steps: vec![click(300, 76), click(130, 223)], ..screen("toggle-then-done") },
        Screen { steps: vec![click(130, 223)], ..screen("done-unchanged") },
        Screen { steps: vec![click(300, 76), click(330, 223)], ..screen("toggle-then-cancel") },
        Screen { steps: vec![click(330, 223)], ..screen("cancel-unchanged") },
        Screen { steps: vec![key(ESCAPE)], ..screen("escape-unchanged") },
        Screen { steps: vec![click(300, 76), key(ESCAPE)], ..screen("escape-changed") },
    ]
}

/// A number box: typed into, out of bounds, and the invalid entry that a new search leaves behind.
fn number_rows() -> Vec<Screen> {
    let respawn = || vec![typed("respawn"), click(300, 76)];
    let with = |label, more: Vec<Value>| Screen { steps: [respawn(), more].concat(), ..screen(label) };
    let done = click(130, 223);
    vec![
        Screen { steps: vec![typed("respawn")], ..screen("number-shown") },
        with("number-clicked", vec![]),
        with("number-appended", vec![typed("5")]),
        with("number-appended-done", vec![typed("5"), done.clone()]),
        with("number-back-to-same-done", vec![typed("5"), key(259), done.clone()]),
        with("number-invalid", vec![typed("x")]),
        with("number-invalid-done", vec![typed("x"), done.clone()]),
        with("number-minus", vec![typed("-")]),
        with("number-empty", vec![key(259), key(259)]),
        with("number-invalid-tab", vec![typed("x"), key(TAB), key(TAB)]),
        with("number-invalid-fixed-done", vec![typed("x"), key(259), done.clone()]),
        with("number-invalid-escape", vec![typed("x"), key(ESCAPE)]),
        Screen {
            steps: [respawn(), vec![typed("x"), click(213, 25), key(259), done]].concat(),
            ..screen("number-invalid-stays-after-search")
        },
        Screen { steps: vec![click(300, 76), click(130, 223)], ..screen("toggle-done-sent") },
    ]
}

/// The focus by Tab, and Enter on it.
fn focus_rows() -> Vec<Screen> {
    let tabs = |count: usize| vec![key(TAB); count];
    let mut rows = Vec::new();
    for (label, count) in [("tab-1", 1), ("tab-2", 2), ("tab-3", 3), ("tab-4", 4), ("tab-5", 5)] {
        rows.push(Screen { steps: tabs(count), ..screen(label) });
    }
    rows.push(Screen { steps: vec![key(TAB), key(TAB), key(ENTER)], ..screen("tab-enter-cancel") });
    rows.push(Screen { steps: vec![key(TAB), key(ENTER)], ..screen("tab-enter-done") });
    rows.push(Screen { steps: vec![click(300, 76), key(TAB), key(ENTER)], ..screen("changed-tab-enter-done") });
    rows.push(Screen { steps: [tabs(4), vec![key(ENTER)]].concat(), ..screen("tab-enter-toggle") });
    let back = |count: usize| vec![json!({"do": "key", "key": TAB, "shift": true}); count];
    for (label, count) in [("shift-tab-1", 1), ("shift-tab-2", 2), ("shift-tab-3", 3)] {
        rows.push(Screen { steps: back(count), ..screen(label) });
    }
    rows.push(Screen { steps: [vec![key(TAB)], back(2)].concat(), ..screen("tab-shift-tab-2") });
    rows.extend(arrow_rows());
    rows
}

/// The focus by the arrow keys, from nothing, from each button, from the search box and inside
/// the list.
fn arrow_rows() -> Vec<Screen> {
    let search = click(213, 24);
    let keys = |codes: &[i64]| codes.iter().copied().map(key).collect::<Vec<_>>();
    let mut sequences: Vec<(&str, Vec<Value>)> = vec![
        ("arrow-down", keys(&[DOWN])),
        ("arrow-up", keys(&[UP])),
        ("arrow-down-2", keys(&[DOWN, DOWN])),
        ("arrow-down-3-up", keys(&[DOWN, DOWN, DOWN, UP])),
        ("arrow-down-up-up", keys(&[DOWN, DOWN, UP, UP])),
        ("arrow-list-right", keys(&[DOWN, RIGHT])),
        ("arrow-list-left", keys(&[DOWN, LEFT])),
        ("arrow-list-int-left", keys(&[DOWN, DOWN, LEFT])),
        ("arrow-list-int-right", keys(&[DOWN, DOWN, RIGHT])),
        ("arrow-list-3-left", keys(&[DOWN, DOWN, DOWN, LEFT])),
        ("arrow-list-3-right", keys(&[DOWN, DOWN, DOWN, RIGHT])),
        ("arrow-list-left-right", keys(&[DOWN, LEFT, RIGHT])),
        ("arrow-list-left-up", keys(&[DOWN, LEFT, UP])),
        ("arrow-list-left-down", keys(&[DOWN, LEFT, DOWN])),
        ("arrow-up-up", keys(&[UP, UP])),
        ("arrow-up-down", keys(&[UP, DOWN])),
        ("arrow-last-down", [&[TAB][..], &[UP, DOWN]].concat().iter().copied().map(key).collect()),
        ("arrow-last-left", [&[TAB][..], &[UP, LEFT]].concat().iter().copied().map(key).collect()),
        ("arrow-last-right", [&[TAB][..], &[UP, RIGHT]].concat().iter().copied().map(key).collect()),
        ("arrow-last-up", [&[TAB][..], &[UP, UP]].concat().iter().copied().map(key).collect()),
        ("arrow-left", keys(&[LEFT])),
        ("arrow-right", keys(&[RIGHT])),
    ];
    for (name, tabs) in [("done", 1), ("cancel", 2)] {
        for (dir, code) in [("down", DOWN), ("up", UP), ("left", LEFT), ("right", RIGHT)] {
            let mut steps = vec![key(TAB); tabs];
            steps.push(key(code));
            sequences.push((Box::leak(format!("arrow-{name}-{dir}").into_boxed_str()), steps));
        }
    }
    for (dir, code) in [("down", DOWN), ("up", UP), ("left", LEFT), ("right", RIGHT)] {
        sequences.push((Box::leak(format!("arrow-search-{dir}").into_boxed_str()), vec![search.clone(), key(code)]));
    }
    sequences.push(("arrow-search-down-down", vec![search.clone(), key(DOWN), key(DOWN)]));
    sequences.push(("arrow-search-down-enter", vec![search, key(DOWN), key(ENTER)]));
    let mut rows: Vec<Screen> = sequences.into_iter().map(|(label, steps)| Screen { steps, ..screen(label) }).collect();
    rows.extend(arrow_edge_rows(&keys));
    rows
}

/// The arrow keys at other sizes, deep in the list, over a filter and over an invalid number.
fn arrow_edge_rows(keys: &dyn Fn(&[i64]) -> Vec<Value>) -> Vec<Screen> {
    let mut rows = Vec::new();
    for (name, size) in [("large", (854, 480)), ("small", (320, 110))] {
        let labels = [
            (format!("arrow-{name}-down-2"), keys(&[DOWN, DOWN])),
            (format!("arrow-{name}-done-up-down"), keys(&[TAB, UP, DOWN])),
            (format!("arrow-{name}-up-left"), keys(&[TAB, UP, LEFT])),
        ];
        for (label, steps) in labels {
            let label = Box::leak(label.into_boxed_str());
            rows.push(Screen { label, size, steps, ..screen("") });
        }
    }
    rows.push(Screen { steps: keys(&[DOWN; 15]), ..screen("arrow-down-15") });
    rows.push(Screen { steps: keys(&[DOWN; 40]), ..screen("arrow-down-40") });
    rows.push(Screen { steps: [keys(&[DOWN; 15]), keys(&[UP; 4])].concat(), ..screen("arrow-down-15-up-4") });
    for (label, extra) in [
        ("arrow-filtered-down", vec![DOWN]),
        ("arrow-filtered-down-up", vec![DOWN, UP]),
        ("arrow-filtered-down-3", vec![DOWN, DOWN, DOWN]),
    ] {
        let steps = [vec![typed("respawn")], keys(&extra)].concat();
        rows.push(Screen { steps, ..screen(label) });
    }
    let invalid = vec![typed("respawn"), click(300, 76), typed("x")];
    for (label, extra) in [
        ("arrow-invalid-tab-left", vec![TAB, TAB, LEFT]),
        ("arrow-invalid-cancel-left", vec![TAB, TAB, TAB, LEFT]),
        ("arrow-invalid-down-left", vec![TAB, DOWN, LEFT]),
    ] {
        rows.push(Screen { steps: [invalid.clone(), keys(&extra)].concat(), ..screen(label) });
    }
    rows
}

pub fn game_rules_screen() -> Value {
    let rows: Vec<Value> = [shown_rows(), search_rows(), control_rows(), number_rows(), focus_rows()]
        .into_iter()
        .flatten()
        .map(Screen::row)
        .collect();
    gui("game-rules-screen", "game_rules_screen", rows)
}
