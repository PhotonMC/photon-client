//! Cases for `player-input/modify`, `player-input/apply`, `player-sprint/can-start`.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::values::*;

fn modify() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_input_cases = vec![
        json!({"label": "zero-preserves-signed-zero", "x": "80000000", "y": "00000000"}),
        json!({"label": "axis-applies-base-factor", "x": f32_bits(0.0), "y": f32_bits(1.0)}),
        json!({"label": "diagonal-clamps-to-unit-square", "x": f32_bits(1.0), "y": f32_bits(1.0)}),
        json!({"label": "short-vector-keeps-scaled-length", "x": f32_bits(0.25), "y": f32_bits(-0.5)}),
        json!({
            "label": "item-use-multiplier",
            "x": f32_bits(1.0),
            "y": f32_bits(0.0),
            "using_item": true,
            "item_use_speed_multiplier": f32_bits(0.2),
        }),
        item_use_bypassed_in_passenger(),
        json!({
            "label": "sneaking-attribute-factor",
            "x": f32_bits(0.0),
            "y": f32_bits(1.0),
            "moving_slowly": true,
            "sneaking_movement_factor": f32_bits(0.3),
        }),
        item_and_sneaking_order(),
        json!({
            "label": "zero-item-factor-reaches-square-zero",
            "x": f32_bits(0.25),
            "y": f32_bits(-0.5),
            "using_item": true,
            "item_use_speed_multiplier": f32_bits(0.0),
        }),
        json!({"label": "non-unit-vector-clamps", "x": f32_bits(2.0), "y": f32_bits(1.0)}),
    ];
    let local_player_input_defaults = json!({
        "using_item": false,
        "passenger": false,
        "moving_slowly": false,
        "item_use_speed_multiplier": f32_bits(0.2),
        "sneaking_movement_factor": f32_bits(0.3),
    });
    let mut local_player_input_inputs = vec![];
    for values in &local_player_input_cases {
        let input_values = merged(vec![local_player_input_defaults.clone(), values.clone()]);
        local_player_input_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-input/modify",
        "math",
        "local_player_input",
        json!({"inputs": local_player_input_inputs.clone()}),
    ));
    out
}

fn item_and_sneaking_order() -> Value {
    json!({
        "label": "item-and-sneaking-order",
        "x": f32_bits(1.0),
        "y": f32_bits(1.0),
        "using_item": true,
        "item_use_speed_multiplier": f32_bits(0.5),
        "moving_slowly": true,
        "sneaking_movement_factor": f32_bits(0.3),
    })
}

fn item_use_bypassed_in_passenger() -> Value {
    json!({
        "label": "item-use-bypassed-in-passenger",
        "x": f32_bits(1.0),
        "y": f32_bits(0.0),
        "using_item": true,
        "passenger": true,
        "item_use_speed_multiplier": f32_bits(0.2),
    })
}

fn apply() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_apply_input_cases = [
        controlled_axis_updates_impulses_and_bob(),
        controlled_diagonal_sets_jumping(),
        controlled_zero_preserves_signed_zero_label(),
        uncontrolled_camera_uses_base_input_label(),
        controlled_overwrites_existing_impulses_label(),
    ];
    let local_player_apply_input_defaults = apply_move_x();
    let mut local_player_apply_input_inputs = vec![];
    for values in &local_player_apply_input_cases {
        let input_values = merged(vec![local_player_apply_input_defaults.clone(), values.clone()]);
        local_player_apply_input_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-input/apply",
        "math",
        "local_player_apply_input",
        json!({"inputs": local_player_apply_input_inputs.clone()}),
    ));
    out
}

fn controlled_overwrites_existing_impulses_label() -> Value {
    json!({
        "label": "controlled-overwrites-existing-impulses",
        "move_x": f32_bits(-1.0),
        "move_y": f32_bits(0.0),
        "jump": false,
        "xxa": f32_bits(2.0),
        "zza": f32_bits(3.0),
        "jumping": true,
        "x_bob": f32_bits(0.0),
        "y_bob": f32_bits(0.0),
        "x_bob_o": f32_bits(1.0),
        "y_bob_o": f32_bits(1.0),
        "x_rot": f32_bits(0.0),
        "y_rot": f32_bits(0.0),
        "controlled_camera": true,
    })
}

fn uncontrolled_camera_uses_base_input_label() -> Value {
    json!({
        "label": "uncontrolled-camera-uses-base-input",
        "move_x": f32_bits(0.0),
        "move_y": f32_bits(1.0),
        "jump": true,
        "xxa": f32_bits(0.5),
        "zza": f32_bits(-0.25),
        "jumping": true,
        "x_bob": f32_bits(1.0),
        "y_bob": f32_bits(2.0),
        "x_bob_o": f32_bits(3.0),
        "y_bob_o": f32_bits(4.0),
        "x_rot": f32_bits(10.0),
        "y_rot": f32_bits(20.0),
        "controlled_camera": false,
    })
}

fn controlled_zero_preserves_signed_zero_label() -> Value {
    json!({
        "label": "controlled-zero-preserves-signed-zero",
        "move_x": "80000000",
        "move_y": "00000000",
        "jump": true,
        "xxa": f32_bits(1.0),
        "zza": f32_bits(-1.0),
        "jumping": false,
        "x_bob": "80000000",
        "y_bob": "00000000",
        "x_bob_o": "80000000",
        "y_bob_o": "00000000",
        "x_rot": f32_bits(0.0),
        "y_rot": f32_bits(-0.0),
        "controlled_camera": true,
    })
}

fn controlled_diagonal_sets_jumping() -> Value {
    json!({
        "label": "controlled-diagonal-sets-jumping",
        "move_x": f32_bits(0.6),
        "move_y": f32_bits(0.8),
        "jump": true,
        "xxa": f32_bits(-1.0),
        "zza": f32_bits(1.0),
        "jumping": false,
        "x_bob": f32_bits(-2.0),
        "y_bob": f32_bits(4.0),
        "x_bob_o": f32_bits(-3.0),
        "y_bob_o": f32_bits(5.0),
        "x_rot": f32_bits(-30.0),
        "y_rot": f32_bits(60.0),
        "controlled_camera": true,
    })
}

fn controlled_axis_updates_impulses_and_bob() -> Value {
    json!({
        "label": "controlled-axis-updates-impulses-and-bob",
        "move_x": f32_bits(0.0),
        "move_y": f32_bits(1.0),
        "jump": false,
        "xxa": f32_bits(0.25),
        "zza": f32_bits(-0.5),
        "jumping": true,
        "x_bob": f32_bits(1.0),
        "y_bob": f32_bits(2.0),
        "x_bob_o": f32_bits(3.0),
        "y_bob_o": f32_bits(4.0),
        "x_rot": f32_bits(10.0),
        "y_rot": f32_bits(20.0),
        "controlled_camera": true,
    })
}

fn apply_move_x() -> Value {
    json!({
        "move_x": f32_bits(0.0),
        "move_y": f32_bits(0.0),
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
    })
}

fn can_start() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_sprint_gate_cases = vec![
        json!({"label": "eligible-ground-forward", "move_y": f32_bits(1.0)}),
        json!({"label": "already-sprinting-blocks-start", "sprinting": true}),
        json!({"label": "forward-threshold-rejects-zero", "move_y": f32_bits(0.0)}),
        json!({"label": "forward-threshold-rejects-small", "move_y": f32_bits(1e-05)}),
        json!({"label": "mobility-restricted-blocks-start", "mobility_restricted": true}),
        json!({"label": "insufficient-food-blocks-start", "enough_food": false}),
        json!({"label": "mayfly-allows-insufficient-food", "enough_food": false, "mayfly": true}),
        json!({"label": "shallow-water-blocks-ground-start", "shallow_water": true}),
        json!({"label": "flying-allows-shallow-water", "shallow_water": true, "flying": true}),
        json!({"label": "item-use-blocks-sprint", "using_item": true}),
        json!({"label": "fall-flying-blocks-out-of-water", "fall_flying": true}),
        json!({"label": "fall-flying-allows-underwater", "fall_flying": true, "underwater": true}),
        json!({"label": "moving-slowly-blocks-out-of-water", "moving_slowly": true}),
        json!({"label": "moving-slowly-allows-underwater", "moving_slowly": true, "underwater": true}),
    ];
    let local_player_sprint_gate_defaults = json!({
        "move_y": f32_bits(1.0),
        "sprinting": false,
        "mobility_restricted": false,
        "enough_food": true,
        "mayfly": false,
        "shallow_water": false,
        "flying": false,
        "using_item": false,
        "fall_flying": false,
        "underwater": false,
        "moving_slowly": false,
    });
    let mut local_player_sprint_gate_inputs = vec![];
    for values in &local_player_sprint_gate_cases {
        let input_values = merged(vec![local_player_sprint_gate_defaults.clone(), values.clone()]);
        local_player_sprint_gate_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-sprint/can-start",
        "math",
        "local_player_can_start_sprinting",
        json!({"inputs": local_player_sprint_gate_inputs.clone()}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [modify(), apply(), can_start()].concat()
}
