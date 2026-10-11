//! The key binds screen (`KeyBindsScreen`): its list of categories and key
//! mappings.

use serde_json::{Value, json};

use super::gui;

/// One key binds scenario.
struct Binds {
    label: &'static str,
    framebuffer: (i64, i64),
    gui_scale: i64,
    in_world: bool,
    mouse: (i64, i64),
    steps: Vec<Value>,
    bindings: Option<Value>,
    ms: i64,
}

fn row(label: &'static str) -> Binds {
    Binds {
        label,
        framebuffer: (854, 480),
        gui_scale: 2,
        in_world: true,
        mouse: (0, 0),
        steps: vec![],
        bindings: None,
        ms: 0,
    }
}

impl Binds {
    fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "framebuffer": [self.framebuffer.0, self.framebuffer.1], "gui_scale": self.gui_scale,
            "in_world": self.in_world, "mouse": [self.mouse.0, self.mouse.1], "steps": self.steps, "ms": self.ms,
        });
        if let Some(bindings) = self.bindings {
            row["bindings"] = bindings;
        }
        row
    }
}

fn click(x: i64, y: i64, button: i64) -> Value {
    json!({"do": "click", "x": x, "y": y, "button": button})
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn wheel(dy: i64) -> Value {
    json!({"do": "scroll", "x": 200, "y": 100, "dy": dy, "ctrl": false})
}

fn frame() -> Value {
    json!({"do": "frame", "ms": 0})
}

/// The key binds screen in several sizes, scrolled, with keys bound away
/// from their defaults and onto each other, a mapping waiting for its key
/// and given one (a key, Escape, a mouse button), the row and footer
/// resets, hovers with the tooltip of a clash, and the keyboard.
pub fn key_binds_screen() -> Value {
    // At 854x480 and scale 2 the second entry's change button is at
    // 251..326, its reset at 331..381, both 55..75; the footer's Reset Keys
    // at 59..209 and Done at 217..367, 213..233.
    let clash = || Some(json!({"key.jump": "key.keyboard.w"}));
    let with = |label: &'static str, steps: Vec<Value>| Binds { steps, ..row(label) };
    let bound = |label: &'static str, bindings: Value| Binds { bindings: Some(bindings), ..row(label) };
    let at = |label: &'static str, x: i64, y: i64| Binds { mouse: (x, y), ..row(label) };
    let select = || click(280, 65, 0);
    let tabs = |count: usize| vec![key("tab"); count];
    let rows = vec![
        row("in-world"),
        Binds { in_world: false, ..row("out-of-world") },
        Binds { framebuffer: (640, 480), ..row("small") },
        Binds { framebuffer: (1280, 720), ..row("large") },
        Binds { framebuffer: (854, 240), ..row("short") },
        Binds { framebuffer: (640, 480), gui_scale: 3, ..row("tiny") },
        with("scrolled", vec![wheel(-3)]),
        with("scrolled-end", vec![wheel(-100)]),
        bound("rebound", json!({"key.forward": "key.keyboard.up", "key.attack": "key.mouse.4"})),
        bound("unbound", json!({"key.forward": "key.keyboard.unknown"})),
        Binds { bindings: clash(), ..row("clash") },
        bound("clash-three", json!({"key.jump": "key.keyboard.w", "key.sneak": "key.keyboard.w"})),
        bound("clash-mouse", json!({"key.pickItem": "key.mouse.left"})),
        at("hover-change", 280, 65),
        at("hover-reset", 350, 65),
        Binds { bindings: Some(json!({"key.forward": "key.keyboard.up"})), ..at("hover-reset-active", 350, 65) },
        at("hover-category", 213, 40),
        at("hover-reset-keys", 130, 222),
        Binds { bindings: clash(), steps: vec![frame()], ms: 10, ..at("tip-clash", 280, 65) },
        with("select", vec![select()]),
        with("select-twice", vec![select(), select()]),
        with("assign-key", vec![select(), key("code:70")]),
        with("assign-escape", vec![select(), key("escape")]),
        with("assign-mouse", vec![select(), click(10, 10, 1)]),
        with("assign-clash", vec![select(), key("code:65")]),
        with("assign-same", vec![select(), key("code:87")]),
        Binds {
            bindings: Some(json!({"key.forward": "key.keyboard.up"})),
            ..with("reset-row", vec![click(350, 65, 0)])
        },
        Binds {
            bindings: Some(json!({"key.forward": "key.keyboard.up", "key.jump": "key.keyboard.w"})),
            ..with("reset-all", vec![click(130, 222, 0)])
        },
        with("tab-first", tabs(1)),
        with("tab-second", tabs(2)),
        with("tab-back", vec![key("shift+tab")]),
        with("arrow-down", vec![key("tab"), key("down"), key("down")]),
        with("arrow-right", vec![key("tab"), key("down"), key("right")]),
        Binds { framebuffer: (854, 240), ..with("arrow-into-footer", vec![key("shift+tab"), key("up")]) },
        with("tab-select", vec![key("tab"), key("tab"), key("enter"), key("code:71")]),
    ];
    gui("key-binds-screen", "key_binds_screen", rows.into_iter().map(Binds::row).collect())
}
