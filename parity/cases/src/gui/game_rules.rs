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
