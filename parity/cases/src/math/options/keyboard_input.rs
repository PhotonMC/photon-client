//! Cases for `keyboard-input/tick`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

fn tick() -> Vec<Value> {
    let mut out = Vec::new();
    let keyboard_inputs = vec![
        ("empty", json!({})),
        ("forward", json!({"up": true})),
        ("backward", json!({"down": true})),
        ("left", json!({"left": true})),
        ("right", json!({"right": true})),
        ("forward-left", json!({"up": true, "left": true})),
        ("forward-right", json!({"up": true, "right": true})),
        ("backward-left", json!({"down": true, "left": true})),
        ("backward-right", json!({"down": true, "right": true})),
        ("forward-backward", json!({"up": true, "down": true})),
        ("left-right", json!({"left": true, "right": true})),
        ("all-movement", json!({"up": true, "down": true, "left": true, "right": true})),
        ("jump", json!({"jump": true})),
        ("shift", json!({"shift": true})),
        ("sprint", json!({"sprint": true})),
        ("movement-and-flags", json!({"up": true, "right": true, "jump": true, "shift": true, "sprint": true})),
    ];
    let keyboard_defaults = json!({
        "up": false,
        "down": false,
        "left": false,
        "right": false,
        "jump": false,
        "shift": false,
        "sprint": false,
    });
    for (label, values) in keyboard_inputs.iter() {
        let keys = merged(vec![keyboard_defaults.clone(), values.clone()]);
        out.push(case(format!("keyboard-input/tick/{}", label), "math", "keyboard_input", keys.clone()));
    }
    out
}

pub fn cases() -> Vec<Value> {
    tick()
}
