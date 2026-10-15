//! Cases for `client-world/player-sneaking-speed-input-binding`, `client-world/player-keyboard-input-binding`, `client-world/player-tick-state`.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

fn player_sneaking_speed_input_binding() -> Vec<Value> {
    let mut out = Vec::new();
    let player_sneaking_input_defaults = player_sneaking_speed_input_binding_action();
    let player_sneaking_input_operations = vec![
        json!({"label": "default-sneaking-speed", "attributes": []}),
        json!({
            "label": "custom-sneaking-speed",
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": f64_bits(0.5),
                    "modifiers": [attribute_modifier("p10:sneaking_total", 0.5, 2)],
                },
            ],
        }),
        json!({
            "label": "item-and-sneaking-attribute-order",
            "move_x": f32_bits(1.0),
            "move_y": f32_bits(1.0),
            "using_item": true,
            "item_use_speed_multiplier": f32_bits(0.5),
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": f64_bits(0.4),
                    "modifiers": [attribute_modifier("p10:sneaking_add", 0.1, 0)],
                },
            ],
        }),
        json!({
            "label": "non-slow-input-ignores-sneaking-speed",
            "moving_slowly": false,
            "attributes": [{"attribute_id": 31, "base": f64_bits(0.9), "modifiers": []}],
        }),
        json!({
            "label": "wrong-player-preserves-sneaking-speed",
            "player_id": 99,
            "attributes": [{"attribute_id": 31, "base": f64_bits(0.8), "modifiers": []}],
        }),
    ];
    out.push(case(
        "client-world/player-sneaking-speed-input-binding",
        "math",
        "client_world_player_sneaking_input",
        json!({
            "player_id": 42,
            "operations": player_sneaking_input_operations.clone().iter().cloned().map(|operation| merged(vec![player_sneaking_input_defaults.clone(), operation])).collect::<Vec<_>>(),
        }),
    ));
    out
}

fn player_sneaking_speed_input_binding_action() -> Value {
    json!({
        "action": "apply_input",
        "player_id": 42,
        "move_x": f32_bits(0.0),
        "move_y": f32_bits(1.0),
        "jump": false,
        "xxa": f32_bits(0.0),
        "zza": f32_bits(0.0),
        "jumping": false,
        "x_bob": f32_bits(0.0),
        "y_bob": f32_bits(0.0),
        "x_bob_o": f32_bits(0.0),
        "y_bob_o": f32_bits(0.0),
        "x_rot": f32_bits(0.0),
        "y_rot": f32_bits(0.0),
        "controlled_camera": true,
        "using_item": false,
        "passenger": false,
        "moving_slowly": true,
        "item_use_speed_multiplier": f32_bits(0.2),
    })
}

fn player_keyboard_input_binding() -> Vec<Value> {
    let mut out = Vec::new();
    let player_keyboard_input_defaults = player_keyboard_input_binding_action();
    let player_keyboard_input_operations = vec![
        json!({"label": "forward-key-reaches-local-input", "up": true}),
        diagonal_keys_use_sneaking_attribute(),
        opposing_keys_zero_movement(),
        item_and_sneaking_key_order(),
        json!({
            "label": "wrong-player-preserves-key-attribute",
            "player_id": 99,
            "up": true,
            "moving_slowly": true,
            "attributes": [{"attribute_id": 31, "base": f64_bits(0.9), "modifiers": []}],
        }),
    ];
    out.push(case(
        "client-world/player-keyboard-input-binding",
        "math",
        "client_world_player_keyboard_input",
        json!({
            "player_id": 42,
            "operations": player_keyboard_input_operations.clone().iter().cloned().map(|operation| merged(vec![player_keyboard_input_defaults.clone(), operation])).collect::<Vec<_>>(),
        }),
    ));
    out
}

fn player_keyboard_input_binding_action() -> Value {
    json!({
        "action": "apply_keyboard_input",
        "player_id": 42,
        "up": false,
        "down": false,
        "left": false,
        "right": false,
        "jump": false,
        "shift": false,
        "sprint": false,
        "controlled_camera": true,
        "using_item": false,
        "passenger": false,
        "moving_slowly": false,
        "item_use_speed_multiplier": f32_bits(0.2),
        "attributes": [],
    })
}

fn item_and_sneaking_key_order() -> Value {
    json!({
        "label": "item-and-sneaking-key-order",
        "up": true,
        "right": true,
        "using_item": true,
        "item_use_speed_multiplier": f32_bits(0.5),
        "moving_slowly": true,
        "attributes": [
            {
                "attribute_id": 31,
                "base": f64_bits(0.4),
                "modifiers": [attribute_modifier("p10:keyboard_sneaking_add", 0.1, 0)],
            },
        ],
    })
}

fn diagonal_keys_use_sneaking_attribute() -> Value {
    json!({
        "label": "diagonal-keys-use-sneaking-attribute",
        "up": true,
        "left": true,
        "jump": true,
        "moving_slowly": true,
        "attributes": [
            {
                "attribute_id": 31,
                "base": f64_bits(0.4),
                "modifiers": [attribute_modifier("p10:keyboard_sneaking_total", 0.5, 2)],
            },
        ],
    })
}

fn opposing_keys_zero_movement() -> Value {
    json!({
        "label": "opposing-keys-zero-movement",
        "up": true,
        "down": true,
        "left": true,
        "right": true,
        "jump": true,
        "shift": true,
        "sprint": true,
        "moving_slowly": true,
    })
}

fn player_tick_state() -> Vec<Value> {
    let mut out = Vec::new();
    let player_tick_defaults = json!({
        "action": "tick",
        "position": vector_bits(10.0, 64.0, -2.0),
        "rotation": {"y": f32_bits(350.0), "x": f32_bits(-20.0)},
        "ticks": 1,
        "head_target": null,
        "effects": [],
    });
    let player_tick_operations = vec![
        json!({"label": "head-target-and-effect-start", "head_target": f32_bits(135.0)}),
        json!({
            "label": "position-and-rotation-update-before-tick",
            "position": vector_bits(14.0, 68.0, 6.0),
            "rotation": {"y": f32_bits(10.0), "x": f32_bits(20.0)},
            "ticks": 0,
        }),
        json!({
            "label": "head-target-reached-after-position-tick",
            "position": vector_bits(14.0, 68.0, 6.0),
            "rotation": {"y": f32_bits(10.0), "x": f32_bits(20.0)},
            "ticks": 2,
        }),
        json!({
            "label": "effect-replacement-before-tick",
            "effects": [{"effect_id": 18, "amplifier": 1, "duration_ticks": 2, "flags": 15}],
        }),
    ];
    out.push(case(
        "client-world/player-tick-state",
        "math",
        "client_world_player_tick",
        json!({
            "player_id": 42,
            "initial_position": vector_bits(10.0, 64.0, -2.0),
            "initial_rotation": {"y": f32_bits(350.0), "x": f32_bits(-20.0)},
            "initial_head": f32_bits(350.0),
            "initial_effects": [
                {"effect_id": 18, "amplifier": 0, "duration_ticks": 3, "flags": 8},
                {"effect_id": 19, "amplifier": 1, "duration_ticks": -1, "flags": 0},
            ],
            "operations": player_tick_operations.clone().iter().cloned().map(|operation| merged(vec![player_tick_defaults.clone(), operation])).collect::<Vec<_>>(),
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [player_sneaking_speed_input_binding(), player_keyboard_input_binding(), player_tick_state()].concat()
}
