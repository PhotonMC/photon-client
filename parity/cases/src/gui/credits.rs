//! The credits screen.

use serde_json::{Value, json};

use super::gui;

const SPACE: i64 = 32;
const CTRL_LEFT: i64 = 341;
const CTRL_RIGHT: i64 = 345;
const UP: i64 = 265;

fn key(code: i64, down: bool) -> Value {
    json!({"code": code, "down": down})
}

fn pressed(code: i64) -> Value {
    key(code, true)
}

/// One credits scenario.
struct Credits {
    label: String,
    poem: bool,
    size: (i64, i64),
    scroll: f64,
    keys: Vec<Value>,
    partial: f64,
    player: &'static str,
}

fn row(label: String, poem: bool) -> Credits {
    Credits { label, poem, size: (427, 240), scroll: 0.0, keys: vec![], partial: 0.0, player: "Steve" }
}

impl Credits {
    fn row(self) -> Value {
        json!({
            "label": self.label, "poem": self.poem, "width": self.size.0, "height": self.size.1,
            "scroll": self.scroll, "keys": self.keys, "partial": self.partial, "player": self.player,
        })
    }
}

/// The credits roll and the end poem: scrolled, the speed keys (Space,
/// Ctrl, Up) pressed and released, and a roll that has run past its end.
pub fn credits_screen() -> Value {
    let mut rows = Vec::new();
    for poem in [false, true] {
        let tag = if poem { "poem" } else { "credits" };
        let new = |name: &str| row(format!("{tag}-{name}"), poem);
        let held = |name: &str, keys: Vec<Value>, partial: f64| Credits { scroll: 300.0, keys, partial, ..new(name) };
        rows.extend([
            new("start"),
            Credits { scroll: 60.0, ..new("early") },
            Credits { scroll: 900.5, ..new("middle") },
            Credits { scroll: 2500.0, ..new("deep") },
            Credits { size: (320, 240), scroll: 700.0, ..new("small") },
            Credits { size: (854, 480), scroll: 700.0, ..new("wide") },
            held("space", vec![pressed(SPACE)], 1.0),
            held("space-ctrl", vec![pressed(SPACE), pressed(CTRL_LEFT)], 1.0),
            held("both-ctrl", vec![pressed(SPACE), pressed(CTRL_LEFT), pressed(CTRL_RIGHT)], 1.0),
            held("ctrl-alone", vec![pressed(CTRL_LEFT)], 1.0),
            held("up", vec![pressed(UP)], 1.0),
            held("up-space", vec![pressed(UP), pressed(SPACE)], 2.0),
            held("released", vec![pressed(SPACE), pressed(CTRL_LEFT), key(SPACE, false), key(CTRL_LEFT, false)], 1.0),
            held("up-released", vec![pressed(UP), key(UP, false)], 1.0),
            Credits { scroll: 0.0, ..held("up-at-top", vec![pressed(UP)], 1.0) },
            Credits { scroll: 10.0, partial: 0.37, ..new("fraction") },
            Credits { scroll: 1.0e6, ..new("past-end") },
        ]);
    }
    rows.push(Credits { scroll: 400.0, player: "Alexandra_the_Great", ..row("poem-name".to_owned(), true) });
    gui("credits-screen", "credits_screen", rows.into_iter().map(Credits::row).collect())
}
