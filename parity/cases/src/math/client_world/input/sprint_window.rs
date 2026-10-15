//! Cases for `client-world/player-sprint-trigger-window`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

const KEYS: [&str; 7] = ["forward", "backward", "left", "right", "jump", "shift", "sprint"];

/// The key states of one tick: every key up except the ones named in `down`.
fn sprint_key_presses(down: &[&str]) -> Value {
    KEYS.iter().map(|key| ((*key).to_owned(), Value::Bool(down.contains(key)))).collect()
}

fn player_sprint_trigger_window() -> Vec<Value> {
    let mut out = Vec::new();
    let sprint_trigger_conditions = json!({
        "mobility_restricted": false,
        "enough_food": true,
        "mayfly": false,
        "shallow_water": false,
        "flying": false,
        "slow_due_to_using_item": false,
        "fall_flying": false,
        "underwater": false,
        "moving_slowly": false,
    });
    let sprint_trigger_sequences = vec![
        double_tap_within_window(),
        window_expiry_and_sprint_key_label(),
        shift_resets_window_label(),
        backward_resets_window_label(),
    ];
    out.push(case(
        "client-world/player-sprint-trigger-window",
        "math",
        "client_world_player_sprint_trigger",
        json!({
            "player_id": 42,
            "sprint_window": 7,
            "conditions": sprint_trigger_conditions.clone(),
            "sequences": sprint_trigger_sequences.clone(),
        }),
    ));
    out
}

fn shift_resets_window_label() -> Value {
    json!({
        "label": "shift-resets-window",
        "operations": [
            {
                "label": "first-forward-opens-window",
                "key_presses": sprint_key_presses(&["forward"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
            {
                "label": "shift-press-is-current-only",
                "key_presses": sprint_key_presses(&["shift"]),
                "move_vector": vector2_bits(0.0, 0.0),
                "ticks": 1,
            },
            {
                "label": "previous-shift-clears-window",
                "key_presses": sprint_key_presses(&[]),
                "move_vector": vector2_bits(0.0, 0.0),
                "ticks": 1,
            },
            {
                "label": "forward-after-shift-opens-new-window",
                "key_presses": sprint_key_presses(&["forward"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
        ],
    })
}

fn window_expiry_and_sprint_key_label() -> Value {
    json!({
        "label": "window-expiry-and-sprint-key",
        "operations": [
            {
                "label": "first-forward-opens-window",
                "key_presses": sprint_key_presses(&["forward"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
            {
                "label": "release-expires-window",
                "key_presses": sprint_key_presses(&[]),
                "move_vector": vector2_bits(0.0, 0.0),
                "ticks": 7,
            },
            {
                "label": "forward-after-expiry-reopens-window",
                "key_presses": sprint_key_presses(&["forward"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
            {
                "label": "sprint-key-starts-immediately",
                "key_presses": sprint_key_presses(&["forward", "sprint"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
        ],
    })
}

fn backward_resets_window_label() -> Value {
    json!({
        "label": "backward-resets-window",
        "operations": [
            {
                "label": "first-forward-opens-window",
                "key_presses": sprint_key_presses(&["forward"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
            {
                "label": "backward-clears-window",
                "key_presses": sprint_key_presses(&["backward"]),
                "move_vector": vector2_bits(0.0, -1.0),
                "ticks": 1,
            },
            {
                "label": "forward-after-backward-opens-new-window",
                "key_presses": sprint_key_presses(&["forward"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
        ],
    })
}

fn double_tap_within_window() -> Value {
    json!({
        "label": "double-tap-within-window",
        "operations": [
            {
                "label": "first-forward-opens-window",
                "key_presses": sprint_key_presses(&["forward"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
            {
                "label": "release-keeps-window",
                "key_presses": sprint_key_presses(&[]),
                "move_vector": vector2_bits(0.0, 0.0),
                "ticks": 1,
            },
            {
                "label": "second-forward-starts-sprint",
                "key_presses": sprint_key_presses(&["forward"]),
                "move_vector": vector2_bits(0.0, 1.0),
                "ticks": 1,
            },
        ],
    })
}

pub fn cases() -> Vec<Value> {
    [player_sprint_trigger_window()].concat()
}
