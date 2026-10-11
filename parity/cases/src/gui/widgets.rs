//! The option widgets and the game mode switcher.

use serde_json::{Value, json};

use super::gui;

fn focus(input: &str) -> Value {
    json!({"do": "focus", "input": input})
}

fn mouse_focus() -> Value {
    focus("MOUSE")
}

fn click(x: i64, shift: bool) -> Value {
    json!({"do": "click", "x": x, "y": 60, "shift": shift})
}

fn key(name: &str, shift: bool) -> Value {
    json!({"do": "key", "key": name, "shift": shift})
}

fn drag(x: i64) -> Value {
    json!({"do": "drag", "x": x})
}

fn release(x: i64) -> Value {
    json!({"do": "release", "x": x})
}

fn scroll(dy: f64) -> Value {
    json!({"do": "scroll", "dy": dy})
}

fn widget(label: &str, kind: &str, value: Value, steps: Vec<Value>) -> Value {
    json!({"label": label, "kind": kind, "value": value, "width": 150, "active": true, "mouse": [0, 0], "steps": steps})
}

fn changed(mut row: Value, key: &str, value: Value) -> Value {
    row[key] = value;
    row
}

fn fov_rows() -> Vec<Value> {
    let fov = |label: &str, value: i64, steps: Vec<Value>| widget(label, "fov", json!(value), steps);
    vec![
        fov("fov-normal", 70, vec![]),
        fov("fov-quake-pro", 110, vec![]),
        fov("fov-lowest", 30, vec![]),
        fov("fov-90", 90, vec![]),
        changed(fov("fov-wide-bar", 90, vec![]), "width", json!(200)),
        changed(fov("fov-big-bar", 45, vec![]), "width", json!(310)),
        changed(fov("fov-inactive", 90, vec![]), "active", json!(false)),
        changed(fov("fov-hover-bar", 90, vec![]), "mouse", json!([120, 55])),
        changed(fov("fov-hover-handle", 90, vec![]), "mouse", json!([144, 55])),
        fov("fov-click", 70, vec![click(175, false)]),
        fov("fov-click-release", 70, vec![click(175, false), release(175)]),
        fov("fov-click-left-edge", 70, vec![click(100, false), release(100)]),
        fov("fov-click-right-edge", 70, vec![click(249, false), release(249)]),
        fov("fov-click-outside", 70, vec![click(300, false)]),
        changed(fov("fov-click-inactive", 70, vec![click(175, false)]), "active", json!(false)),
        fov("fov-drag", 70, vec![click(120, false), drag(160), drag(200)]),
        fov("fov-drag-release", 70, vec![click(120, false), drag(160), drag(200), release(200)]),
        fov("fov-drag-past-end", 70, vec![click(120, false), drag(900), release(900)]),
        fov("fov-drag-before-start", 90, vec![click(120, false), drag(-50), release(-50)]),
        fov("fov-focus-mouse", 70, vec![focus("MOUSE")]),
        fov("fov-focus-tab", 70, vec![focus("KEYBOARD_TAB")]),
        fov("fov-focus-arrow", 70, vec![focus("KEYBOARD_ARROW")]),
        fov("fov-focus-none", 70, vec![focus("NONE")]),
        fov("fov-keys-right", 70, vec![mouse_focus(), key("right", false), key("right", false), key("right", false)]),
        fov("fov-keys-left", 70, vec![mouse_focus(), key("left", false), key("left", false)]),
        fov("fov-keys-at-end", 110, vec![mouse_focus(), key("right", false)]),
        fov("fov-keys-at-start", 30, vec![mouse_focus(), key("left", false)]),
        fov("fov-keys-locked", 70, vec![focus("KEYBOARD_ARROW"), key("right", false)]),
        fov("fov-enter-unlocks", 70, vec![focus("KEYBOARD_ARROW"), key("enter", false), key("right", false)]),
        fov("fov-space-toggles", 70, vec![mouse_focus(), key("space", false), key("right", false)]),
        fov("fov-unfocus", 70, vec![mouse_focus(), json!({"do": "unfocus"})]),
    ]
}

fn share_rows() -> Vec<Value> {
    let share = |label: &str, kind: &str, value: f64, steps: Vec<Value>| widget(label, kind, json!(value), steps);
    let sensitivity = |label: &str, value: f64, steps: Vec<Value>| share(label, "sensitivity", value, steps);
    vec![
        sensitivity("sensitivity-half", 0.5, vec![]),
        sensitivity("sensitivity-yawn", 0.0, vec![]),
        sensitivity("sensitivity-hyperspeed", 1.0, vec![]),
        sensitivity("sensitivity-third", 0.33, vec![]),
        sensitivity("sensitivity-click", 0.5, vec![click(120, false), release(120)]),
        sensitivity("sensitivity-drag", 0.5, vec![click(120, false), drag(130)]),
        sensitivity("sensitivity-drag-to-ends", 0.5, vec![click(120, false), drag(-100), release(-100)]),
        sensitivity(
            "sensitivity-keys",
            0.5,
            vec![mouse_focus(), key("right", false), key("right", false), key("left", false)],
        ),
        sensitivity("sensitivity-keys-at-end", 1.0, vec![mouse_focus(), key("right", false)]),
        changed(
            sensitivity("sensitivity-wide-keys", 0.5, vec![mouse_focus(), key("right", false)]),
            "width",
            json!(310),
        ),
        share("chat-opacity-full", "chat_opacity", 1.0, vec![]),
        share("chat-opacity-none", "chat_opacity", 0.0, vec![]),
        share("chat-opacity-third", "chat_opacity", 0.33, vec![]),
        share("chat-opacity-click", "chat_opacity", 1.0, vec![click(130, false), release(130)]),
    ]
}

fn cycle_rows() -> Vec<Value> {
    let invert = |label: &str, value: bool, steps: Vec<Value>| widget(label, "invert_y", json!(value), steps);
    let visibility = |label: &str, value: i64, steps: Vec<Value>| widget(label, "chat_visibility", json!(value), steps);
    vec![
        invert("invert-off", false, vec![]),
        invert("invert-on", true, vec![]),
        changed(invert("invert-inactive", true, vec![]), "active", json!(false)),
        changed(invert("invert-hover", false, vec![]), "mouse", json!([150, 55])),
        invert("invert-click", false, vec![click(150, false)]),
        invert("invert-click-twice", false, vec![click(150, false), click(150, false)]),
        invert("invert-shift-click", false, vec![click(150, true)]),
        invert("invert-click-outside", false, vec![click(300, false)]),
        changed(invert("invert-click-inactive", false, vec![click(150, false)]), "active", json!(false)),
        invert("invert-scroll-up", false, vec![scroll(1.0)]),
        invert("invert-scroll-down", false, vec![scroll(-1.0)]),
        invert("invert-scroll-none", false, vec![scroll(0.0)]),
        invert("invert-enter-unfocused", false, vec![key("enter", false)]),
        invert("invert-enter-focused", false, vec![mouse_focus(), key("enter", false)]),
        invert("invert-space-shift", false, vec![mouse_focus(), key("space", true)]),
        invert("invert-focused", false, vec![mouse_focus()]),
        changed(invert("invert-wide", false, vec![]), "width", json!(310)),
        visibility("visibility-shown", 0, vec![]),
        visibility("visibility-commands", 1, vec![]),
        visibility("visibility-hidden", 2, vec![]),
        visibility("visibility-click", 0, vec![click(150, false)]),
        visibility("visibility-click-wraps", 2, vec![click(150, false)]),
        visibility("visibility-shift-click-wraps", 0, vec![click(150, true)]),
        visibility("visibility-scroll", 1, vec![scroll(1.0), scroll(1.0)]),
        visibility("visibility-scroll-down", 1, vec![scroll(-1.0), scroll(-1.0)]),
    ]
}

/// The widgets an option makes: the slider of a range (field of view), of a
/// share (sensitivity, chat opacity), and the cycle button of a boolean and
/// of a choice, set, focused, clicked, dragged, stepped and turned.
pub fn widgets() -> Value {
    gui("widgets", "widgets", [fov_rows(), share_rows(), cycle_rows()].concat())
}

/// A switcher frame: where the mouse is and whether the switch key is down.
type Step = ((i64, i64), bool);

fn switcher(
    label: &str,
    game_mode: &str,
    steps: &[Step],
    previous: Option<&str>,
    (width, height): (i64, i64),
) -> Value {
    let steps: Vec<Value> = steps
        .iter()
        .map(|((x, y), key)| if *key { json!({"mouse": [x, y], "key": true}) } else { json!({"mouse": [x, y]}) })
        .collect();
    let mut row = json!({
        "label": label, "width": width, "height": height, "game_mode": game_mode, "selected": 0, "state_id": 0,
        "blocks": [], "chunk_radius": 0, "clicks": [], "slots": [], "steps": steps,
    });
    if let Some(previous) = previous {
        row["previous"] = json!(previous);
    }
    row
}

/// `GameModeSwitcherScreen` (F3+F4) over the game's own client: where it
/// starts, the switch key, and the mouse picking a slot once it has moved.
pub fn game_mode_switcher() -> Value {
    let away = ((10, 10), false);
    let pressed = ((10, 10), true);
    let size = (427, 240);
    let rows = vec![
        switcher("from-creative", "creative", &[away], None, size),
        switcher("from-survival", "survival", &[away], None, size),
        switcher("previous-spectator", "survival", &[away], Some("spectator"), size),
        switcher("previous-adventure", "creative", &[away], Some("adventure"), size),
        switcher("switch-key-cycles", "survival", &[away, pressed, pressed, pressed, pressed], None, size),
        // The slots lie at x 154, 185, 216 and 247, 26 wide, from y 89.
        switcher(
            "mouse-picks-once-moved",
            "survival",
            &[((170, 100), false), ((170, 100), false), ((200, 100), false), ((260, 100), false), away],
            None,
            size,
        ),
        switcher(
            "mouse-waits-for-motion",
            "creative",
            &[((230, 100), false), ((230, 100), false), ((231, 100), false)],
            None,
            size,
        ),
        switcher(
            "switch-key-then-mouse",
            "creative",
            &[((200, 100), false), ((200, 100), true), ((200, 100), false), ((170, 95), false)],
            None,
            size,
        ),
        switcher("small-window", "survival", &[away, ((20, 30), true)], None, (200, 120)),
    ];
    gui("game-mode-switcher", "game_mode_switcher", rows)
}
