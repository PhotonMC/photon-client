//! The chat restrictions screen: each restriction with its button, the four permissions, and the
//! keys, clicks and wheel turns that move through it. A row has one restriction at most: the
//! reference lists several in the order of a set that is salted anew in every run.

use serde_json::{Value, json};

use super::gui;

const OPTIONS: &str = "CHAT_DISABLED_BY_OPTIONS";
const BOTH: &str = "CHAT_AND_COMMANDS_DISABLED_BY_OPTIONS";
const LAUNCHER: &str = "DISABLED_BY_LAUNCHER";
const PROFILE: &str = "DISABLED_BY_PROFILE";

struct Screen {
    label: &'static str,
    size: (i64, i64),
    restrictions: Vec<&'static str>,
    steps: Vec<Value>,
    mouse: (i64, i64),
    in_world: bool,
}

fn screen(label: &'static str, restrictions: &[&'static str]) -> Screen {
    Screen {
        label,
        size: (427, 240),
        restrictions: restrictions.to_vec(),
        steps: Vec::new(),
        mouse: (0, 0),
        in_world: false,
    }
}

impl Screen {
    fn row(self) -> Value {
        json!({
            "label": self.label, "width": self.size.0, "height": self.size.1,
            "restrictions": self.restrictions, "steps": self.steps,
            "mouse": [self.mouse.0, self.mouse.1], "in_world": self.in_world,
        })
    }
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn tabs(count: usize) -> Vec<Value> {
    vec![key("tab"); count]
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn scroll(x: i64, y: i64, amount: i64) -> Value {
    json!({"do": "scroll", "x": x, "y": y, "amount": amount})
}

/// What the screen shows for each set of restrictions.
fn shown_rows() -> Vec<Screen> {
    vec![
        screen("none", &[]),
        screen("chat-disabled", &[OPTIONS]),
        screen("chat-and-commands-disabled", &[BOTH]),
        screen("launcher", &[LAUNCHER]),
        screen("profile", &[PROFILE]),
        Screen { in_world: true, ..screen("in-world", &[OPTIONS]) },
        Screen { size: (854, 480), ..screen("large", &[PROFILE]) },
    ]
}

/// The focus moving by Tab: a text box, a button, Done, and round again.
fn focus_rows() -> Vec<Screen> {
    let mut rows = Vec::new();
    for count in 1..=5 {
        rows.push(Screen {
            steps: tabs(count),
            label: ["tab-1", "tab-2", "tab-3", "tab-4", "tab-5"][count - 1],
            ..screen("", &[BOTH])
        });
    }
    rows.push(Screen { steps: vec![key("shift+tab")], ..screen("shift-tab-first", &[OPTIONS]) });
    rows.push(Screen { steps: vec![key("tab"), key("shift+tab")], ..screen("tab-back", &[OPTIONS]) });
    rows.push(Screen { steps: tabs(2), ..screen("tab-none", &[]) });
    rows
}

/// A text box focused by the mouse, and what the buttons do when pressed.
fn press_rows() -> Vec<Screen> {
    let enter = |count: usize| [tabs(count), vec![key("enter")]].concat();
    vec![
        Screen { steps: enter(2), ..screen("enter-chat-settings", &[OPTIONS]) },
        Screen { steps: enter(2), ..screen("enter-account", &[PROFILE]) },
        Screen { steps: enter(4), ..screen("enter-done", &[OPTIONS]) },
        Screen { steps: [tabs(2), vec![key("space")]].concat(), ..screen("space-chat-settings", &[BOTH]) },
        Screen { steps: enter(2), ..screen("enter-launcher-has-no-button", &[LAUNCHER]) },
        Screen { steps: vec![key("escape")], ..screen("escape", &[OPTIONS]) },
        Screen { steps: vec![click(213, 223)], ..screen("click-done", &[OPTIONS]) },
        Screen { steps: vec![click(213, 223)], ..screen("click-done-none", &[]) },
        Screen { steps: vec![click(213, 80)], ..screen("click-text", &[OPTIONS]) },
    ]
}

/// A body taller than the box: the wheel moves it, and Tab brings a hidden part into view.
fn scroll_rows() -> Vec<Screen> {
    let small = (320, 110);
    let all = [PROFILE];
    vec![
        Screen { size: small, ..screen("small", &all) },
        Screen { size: small, steps: vec![scroll(160, 75, -3)], ..screen("small-scrolled", &all) },
        Screen { size: small, steps: vec![scroll(160, 75, -30)], ..screen("small-scrolled-end", &all) },
        Screen { size: small, steps: tabs(2), ..screen("small-tab-into-view", &all) },
        Screen { size: small, steps: tabs(3), ..screen("small-tab-to-done", &all) },
        Screen { size: small, steps: tabs(4), ..screen("small-tab-done", &all) },
        Screen { size: small, steps: vec![key("shift+tab")], ..screen("small-shift-tab", &all) },
    ]
}

pub fn restrictions_screen() -> Value {
    let rows: Vec<Value> =
        [shown_rows(), focus_rows(), press_rows(), scroll_rows()].into_iter().flatten().map(Screen::row).collect();
    gui("restrictions-screen", "restrictions_screen", rows)
}
