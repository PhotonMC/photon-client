//! Cases for `key-mapping/state`, `toggle-key-mapping/state`.

use serde_json::{Value, json};

use crate::case::case;

fn key_spec(kind: &str, value: i64) -> Value {
    json!({"type": kind, "value": value})
}

fn set_down(target: &str, value: bool) -> Value {
    json!({"op": "set_down", "target": target, "value": value})
}

fn set(key: &str, value: bool) -> Value {
    json!({"op": "set", "key": key, "value": value})
}

fn click(key: &str) -> Value {
    json!({"op": "click", "key": key})
}

fn consume_click(target: &str) -> Value {
    json!({"op": "consume_click", "target": target})
}

fn rebind_primary() -> Value {
    json!({"op": "set_key", "target": "primary", "key": "rebind"})
}

fn op(name: &str) -> Value {
    json!({"op": name})
}

fn should_restore_primary() -> Value {
    json!({"op": "should_restore", "target": "primary"})
}

fn key_event(key: i64, scancode: i64) -> Value {
    json!({"key": key, "scancode": scancode})
}

/// The three bound keys, the key event to feed them and the mouse button to press.
struct Bindings {
    primary: Value,
    secondary: Value,
    other: Value,
    key_event: Value,
    unknown_key_event: Value,
    mouse_button: i64,
}

impl Bindings {
    /// Bindings whose events use `base` as the key and `scancode` as the scancode.
    fn new(keys: [Value; 3], base: i64, scancode: i64, mouse_button: i64) -> Self {
        let [primary, secondary, other] = keys;
        Self {
            primary,
            secondary,
            other,
            key_event: key_event(base, scancode),
            unknown_key_event: key_event(-1, scancode),
            mouse_button,
        }
    }

    fn fields(self, actions: Vec<Value>) -> Value {
        json!({
            "primary": self.primary,
            "secondary": self.secondary,
            "other": self.other,
            "actions": actions,
            "key_event": self.key_event,
            "unknown_key_event": self.unknown_key_event,
            "mouse_button": self.mouse_button,
        })
    }
}

fn key_mapping(label: &str, rebind: Value, bindings: Bindings, actions: Vec<Value>) -> Value {
    let mut fields = bindings.fields(actions);
    fields["rebind"] = rebind;
    case(format!("key-mapping/state/{label}"), "math", "key_mapping", fields)
}

fn shared_press_release() -> Value {
    let base = 1000;
    let keys = [key_spec("keysym", base), key_spec("keysym", base), key_spec("keysym", base + 1)];
    let actions = vec![
        set_down("primary", true),
        set("primary", false),
        click("primary"),
        consume_click("primary"),
        consume_click("primary"),
        consume_click("secondary"),
        set_down("other", true),
        op("release_all"),
        consume_click("secondary"),
    ];
    let bindings = Bindings::new(keys, base, base + 10, base + 20);
    key_mapping("shared-press-release", key_spec("scancode", base + 2), bindings, actions)
}

fn repeated_clicks() -> Value {
    let base = 1100;
    let keys = [key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1)];
    let mut actions = vec![click("primary"), click("primary")];
    actions.extend(std::iter::repeat_n(consume_click("primary"), 3));
    actions.extend(std::iter::repeat_n(consume_click("secondary"), 3));
    let bindings = Bindings::new(keys, base, base + 10, base + 1);
    key_mapping("repeated-clicks", key_spec("scancode", base + 2), bindings, actions)
}

fn rebind_keeps_old_table() -> Value {
    let base = 1200;
    let keys = [key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 10)];
    let actions = vec![
        set_down("primary", true),
        rebind_primary(),
        set("primary_default", false),
        set("primary", true),
        click("primary"),
        click("primary_default"),
        consume_click("primary"),
        consume_click("secondary"),
        op("release_all"),
        rebind_primary(),
    ];
    let bindings = Bindings::new(keys, base, base + 20, base + 10);
    key_mapping("rebind-keeps-old-table", key_spec("scancode", base + 20), bindings, actions)
}

fn default_and_unbound() -> Value {
    let base = 1300;
    let keys = [key_spec("keysym", -1), key_spec("keysym", -1), key_spec("mouse", base)];
    let mut bindings = Bindings::new(keys, -1, base + 2, base);
    bindings.key_event = key_event(-1, base + 2);
    key_mapping("default-and-unbound", key_spec("keysym", base + 1), bindings, vec![rebind_primary()])
}

fn event_matching() -> Value {
    let base = 1400;
    let keys = [key_spec("keysym", base), key_spec("scancode", base + 1), key_spec("mouse", base + 2)];
    let bindings = Bindings::new(keys, base, base + 1, base + 2);
    key_mapping("event-matching", key_spec("keysym", base + 3), bindings, vec![])
}

fn known_key_names() -> Value {
    let keys = [key_spec("keysym", 87), key_spec("keysym", 65), key_spec("mouse", 0)];
    key_mapping("known-key-names", key_spec("keysym", 32), Bindings::new(keys, 87, 65, 0), vec![])
}

/// Which of the three bindings toggle, and which restore their state afterwards.
struct ToggleSettings {
    needs_toggle: [bool; 3],
    should_restore: [bool; 3],
}

fn toggle_key_mapping(label: &str, settings: ToggleSettings, bindings: Bindings, actions: Vec<Value>) -> Value {
    let mut fields = bindings.fields(actions);
    for (index, name) in ["primary", "secondary", "other"].into_iter().enumerate() {
        fields[format!("{name}_needs_toggle")] = json!(settings.needs_toggle[index]);
        fields[format!("{name}_should_restore")] = json!(settings.should_restore[index]);
    }
    case(format!("toggle-key-mapping/state/{label}"), "math", "toggle_key_mapping", fields)
}

/// The closing actions every toggle scenario ends with.
fn restore_sequence() -> Vec<Value> {
    vec![op("release_all"), op("restore_toggle_states"), should_restore_primary(), op("reset_toggle_keys")]
}

fn toggle_and_restore() -> Value {
    let base = 1500;
    let keys = [key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1)];
    let mut actions = vec![
        set_down("primary", true),
        set_down("primary", true),
        set("primary", true),
        click("primary"),
        consume_click("primary"),
    ];
    actions.extend(restore_sequence());
    let settings = ToggleSettings { needs_toggle: [true, true, false], should_restore: [true, false, true] };
    toggle_key_mapping("toggle-and-restore", settings, Bindings::new(keys, base, base + 10, base + 1), actions)
}

fn scancode_does_not_restore() -> Value {
    let base = 1600;
    let keys = [key_spec("scancode", base), key_spec("scancode", base), key_spec("keysym", base + 1)];
    let mut actions = vec![set_down("primary", true)];
    actions.extend(restore_sequence());
    let settings = ToggleSettings { needs_toggle: [true; 3], should_restore: [true; 3] };
    let mut bindings = Bindings::new(keys, -1, base, base + 1);
    bindings.key_event = key_event(-1, base);
    toggle_key_mapping("scancode-does-not-restore", settings, bindings, actions)
}

fn direct_set_mode() -> Value {
    let base = 1700;
    let keys = [key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1)];
    let mut actions = vec![set_down("primary", true), set_down("primary", false), set("primary", true)];
    actions.extend(restore_sequence());
    let settings = ToggleSettings { needs_toggle: [false; 3], should_restore: [true; 3] };
    toggle_key_mapping("direct-set-mode", settings, Bindings::new(keys, base, base + 10, base + 1), actions)
}

fn toggle_without_restore() -> Value {
    let base = 1800;
    let keys = [key_spec("keysym", base), key_spec("keysym", base + 1), key_spec("mouse", base + 2)];
    let mut actions = vec![set_down("primary", true)];
    actions.extend(restore_sequence());
    let settings = ToggleSettings { needs_toggle: [true; 3], should_restore: [false; 3] };
    toggle_key_mapping("toggle-without-restore", settings, Bindings::new(keys, base, base + 10, base + 2), actions)
}

pub fn cases() -> Vec<Value> {
    vec![
        shared_press_release(),
        repeated_clicks(),
        rebind_keeps_old_table(),
        default_and_unbound(),
        event_matching(),
        known_key_names(),
        toggle_and_restore(),
        scancode_does_not_restore(),
        direct_set_mode(),
        toggle_without_restore(),
    ]
}
