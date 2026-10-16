//! Cases for `player-movement/fluid-falling-adjustment`, `player-movement/climbable-movement`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn fluid_falling_adjustment() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_fluid_adjustment_cases = [
        json!({
            "label": "zero-gravity-preserves-movement",
            "base_gravity": f64_bits(0.0),
            "is_falling": true,
            "x": f64_bits(-1.25),
            "y": f64_bits(-0.005),
            "z": f64_bits(2.5),
        }),
        json!({"label": "sprinting-bypasses-gravity", "sprinting": true, "y": f64_bits(-0.2)}),
        json!({
            "label": "falling-near-custom-gravity-uses-clamp",
            "base_gravity": f64_bits(0.16),
            "is_falling": true,
            "y": f64_bits(0.01),
        }),
        json!({
            "label": "falling-outside-custom-gravity-window-uses-subtraction",
            "base_gravity": f64_bits(0.16),
            "is_falling": true,
            "y": f64_bits(0.014),
        }),
        json!({"label": "rising-uses-gravity-subtraction", "is_falling": false, "y": f64_bits(0.3)}),
        json!({"label": "default-gravity-exact-zero", "is_falling": true, "y": f64_bits(0.005)}),
        negative_zero_gravity_preserves_signed_zero(),
        json!({
            "label": "falling-custom-gravity-ordinary-subtraction",
            "base_gravity": f64_bits(0.16),
            "is_falling": true,
            "y": f64_bits(-0.2),
        }),
    ];
    let local_player_fluid_adjustment_defaults = fluid_falling_adjustment_base_gravity();
    let mut local_player_fluid_adjustment_inputs = vec![];
    for values in &local_player_fluid_adjustment_cases {
        let input_values = merged(vec![local_player_fluid_adjustment_defaults.clone(), values.clone()]);
        local_player_fluid_adjustment_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/fluid-falling-adjustment",
        "math",
        "local_player_fluid_falling_adjustment",
        json!({"inputs": local_player_fluid_adjustment_inputs.clone()}),
    ));
    out
}

fn fluid_falling_adjustment_base_gravity() -> Value {
    json!({
        "base_gravity": f64_bits(0.08),
        "is_falling": false,
        "sprinting": false,
        "x": f64_bits(0.125),
        "y": f64_bits(-0.05),
        "z": f64_bits(-0.25),
    })
}

fn negative_zero_gravity_preserves_signed_zero() -> Value {
    json!({
        "label": "negative-zero-gravity-preserves-signed-zero",
        "base_gravity": "8000000000000000",
        "is_falling": true,
        "x": "8000000000000000",
        "y": "8000000000000000",
        "z": "0000000000000000",
    })
}

fn climbable_movement() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_climbable_movement_cases = [
        json!({"label": "not-climbable-preserves-movement", "on_climbable": false}),
        json!({"label": "climbable-clamps-horizontal-and-downward", "on_climbable": true}),
        json!({
            "label": "suppressing-player-stops-ladder-slide",
            "on_climbable": true,
            "suppress_sliding_down_ladder": true,
            "x": f64_bits(0.1),
            "y": f64_bits(-0.1),
            "z": f64_bits(-0.1),
        }),
        json!({
            "label": "scaffolding-keeps-clamped-downward-motion",
            "on_climbable": true,
            "scaffolding": true,
            "suppress_sliding_down_ladder": true,
        }),
        json!({
            "label": "suppressing-player-keeps-upward-motion",
            "on_climbable": true,
            "suppress_sliding_down_ladder": true,
            "x": f64_bits(-0.1),
            "y": f64_bits(0.25),
            "z": f64_bits(0.1),
        }),
        json!({
            "label": "climbable-preserves-signed-zero",
            "on_climbable": true,
            "x": "8000000000000000",
            "y": "8000000000000000",
            "z": "0000000000000000",
        }),
    ];
    let local_player_climbable_movement_defaults = json!({
        "on_climbable": true,
        "scaffolding": false,
        "suppress_sliding_down_ladder": false,
        "x": f64_bits(0.4),
        "y": f64_bits(-0.4),
        "z": f64_bits(-0.4),
    });
    let mut local_player_climbable_movement_inputs = vec![];
    for values in &local_player_climbable_movement_cases {
        let input_values = merged(vec![local_player_climbable_movement_defaults.clone(), values.clone()]);
        local_player_climbable_movement_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/climbable-movement",
        "math",
        "local_player_climbable_movement",
        json!({"inputs": local_player_climbable_movement_inputs.clone()}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [fluid_falling_adjustment(), climbable_movement()].concat()
}
