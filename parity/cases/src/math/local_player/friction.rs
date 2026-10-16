//! Cases for `player-movement/friction-influenced-speed`, `player-movement/modified-friction`, `player-movement/handle-relative-friction`.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::values::*;

fn friction_influenced_speed() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_friction_influenced_speed_cases = vec![
        ground_at_friction_threshold_keeps_speed(),
        ground_below_friction_threshold_keeps_speed(),
        ground_above_friction_threshold_scales_speed(),
        ground_just_above_friction_threshold_scales(),
        air_without_controlling_player_uses_default(),
        air_sprinting_without_flight_uses_sprint_label(),
        air_flying_uses_ability_speed_label(),
        air_flying_sprint_doubles_ability_speed_label(),
        air_flying_passenger_uses_grounded_fallback_label(),
        ground_scaled_speed_preserves_negative_zero(),
        nan_friction_fails_strict_comparison_and(),
    ];
    let local_player_friction_influenced_speed_defaults = friction_influenced_speed_on_ground();
    let mut local_player_friction_influenced_speed_inputs = vec![];
    for values in &local_player_friction_influenced_speed_cases {
        let input_values = merged(vec![local_player_friction_influenced_speed_defaults.clone(), values.clone()]);
        local_player_friction_influenced_speed_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/friction-influenced-speed",
        "math",
        "local_player_friction_influenced_speed",
        json!({"inputs": local_player_friction_influenced_speed_inputs.clone()}),
    ));
    let local_player_modified_friction_cases = [
        json!({"label": "unit-modifier-keeps-block-friction", "friction": f32_bits(0.8), "modifier": f32_bits(1.0)}),
        json!({"label": "zero-modifier-reaches-one", "friction": f32_bits(0.8), "modifier": f32_bits(0.0)}),
        json!({"label": "double-modifier-reduces-friction", "friction": f32_bits(0.8), "modifier": f32_bits(2.0)}),
        json!({"label": "negative-modifier-clamps-high", "friction": f32_bits(0.8), "modifier": f32_bits(-1.0)}),
        json!({"label": "large-modifier-clamps-low", "friction": f32_bits(0.2), "modifier": f32_bits(2.0)}),
        json!({"label": "zero-friction-with-unit-modifier", "friction": f32_bits(0.0), "modifier": f32_bits(1.0)}),
        json!({"label": "full-friction-ignores-modifier", "friction": f32_bits(1.0), "modifier": f32_bits(3.0)}),
        json!({"label": "nan-friction-propagates-through-clamp", "friction": "7fc01234", "modifier": f32_bits(1.0)}),
    ];
    let local_player_modified_friction_defaults = json!({"friction": f32_bits(0.6), "modifier": f32_bits(1.0)});
    let mut local_player_modified_friction_inputs = vec![];
    for values in &local_player_modified_friction_cases {
        let input_values = merged(vec![local_player_modified_friction_defaults.clone(), values.clone()]);
        local_player_modified_friction_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/modified-friction",
        "math",
        "local_player_modified_friction",
        json!({"inputs": local_player_modified_friction_inputs.clone()}),
    ));
    out
}

fn friction_influenced_speed_on_ground() -> Value {
    json!({
        "on_ground": true,
        "block_friction": f32_bits(0.6),
        "speed": f32_bits(0.1),
        "flying": false,
        "flying_speed": f32_bits(0.05),
        "sprinting": false,
        "passenger": false,
    })
}

fn air_flying_sprint_doubles_ability_speed_label() -> Value {
    json!({
        "label": "air-flying-sprint-doubles-ability-speed",
        "on_ground": false,
        "flying": true,
        "flying_speed": f32_bits(0.07),
        "sprinting": true,
        "passenger": false,
    })
}

fn air_flying_uses_ability_speed_label() -> Value {
    json!({
        "label": "air-flying-uses-ability-speed",
        "on_ground": false,
        "flying": true,
        "flying_speed": f32_bits(0.07),
        "sprinting": false,
        "passenger": false,
    })
}

fn air_flying_passenger_uses_grounded_fallback_label() -> Value {
    json!({
        "label": "air-flying-passenger-uses-grounded-fallback",
        "on_ground": false,
        "flying": true,
        "sprinting": true,
        "passenger": true,
    })
}

fn air_sprinting_without_flight_uses_sprint_label() -> Value {
    json!({
        "label": "air-sprinting-without-flight-uses-sprint-speed",
        "on_ground": false,
        "flying": false,
        "sprinting": true,
        "passenger": false,
    })
}

fn air_without_controlling_player_uses_default() -> Value {
    json!({
        "label": "air-without-controlling-player-uses-default-flying-speed",
        "on_ground": false,
        "flying": false,
        "sprinting": false,
        "passenger": false,
    })
}

fn nan_friction_fails_strict_comparison_and() -> Value {
    json!({
        "label": "nan-friction-fails-strict-comparison-and-keeps-speed",
        "on_ground": true,
        "block_friction": "7fc00001",
        "speed": f32_bits(0.25),
    })
}

fn ground_scaled_speed_preserves_negative_zero() -> Value {
    json!({
        "label": "ground-scaled-speed-preserves-negative-zero",
        "on_ground": true,
        "block_friction": f32_bits(0.8),
        "speed": "80000000",
    })
}

fn ground_just_above_friction_threshold_scales() -> Value {
    json!({
        "label": "ground-just-above-friction-threshold-scales-speed",
        "on_ground": true,
        "block_friction": f32_bits(0.6000001),
        "speed": f32_bits(0.125),
    })
}

fn ground_above_friction_threshold_scales_speed() -> Value {
    json!({
        "label": "ground-above-friction-threshold-scales-speed",
        "on_ground": true,
        "block_friction": f32_bits(0.8),
    })
}

fn ground_below_friction_threshold_keeps_speed() -> Value {
    json!({
        "label": "ground-below-friction-threshold-keeps-speed",
        "on_ground": true,
        "block_friction": f32_bits(0.5),
    })
}

fn ground_at_friction_threshold_keeps_speed() -> Value {
    json!({
        "label": "ground-at-friction-threshold-keeps-speed",
        "on_ground": true,
        "block_friction": f32_bits(0.6),
    })
}

fn handle_relative_friction() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_handle_relative_friction_cases = [
        json!({"label": "air-relative-movement-uses-player-flying-speed"}),
        json!({
            "label": "ground-friction-speed-enters-relative-movement",
            "on_ground": true,
            "block_friction": f32_bits(0.8),
            "speed": f32_bits(0.12),
        }),
        json!({
            "label": "climbable-clamps-delta-before-return",
            "input": vector_bits(0.0, 0.0, 0.0),
            "delta": vector_bits(0.4, -0.4, -0.4),
            "on_climbable": true,
        }),
        json!({
            "label": "horizontal-climbable-collision-sets-step-height",
            "on_climbable": true,
            "horizontal_collision": true,
        }),
        json!({"label": "jumping-climbable-sets-step-height", "on_climbable": true, "jumping": true}),
        json!({"label": "jumping-without-climbable-keeps-motion", "jumping": true}),
    ];
    let local_player_handle_relative_friction_defaults = json!({
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": f32_bits(37.5),
        "on_ground": false,
        "block_friction": f32_bits(0.6),
        "speed": f32_bits(0.1),
        "flying_speed": f32_bits(0.05),
        "flying": false,
        "sprinting": false,
        "passenger": false,
        "on_climbable": false,
        "scaffolding": false,
        "suppress_sliding_down_ladder": false,
        "horizontal_collision": false,
        "jumping": false,
    });
    let mut local_player_handle_relative_friction_inputs = vec![];
    for values in &local_player_handle_relative_friction_cases {
        let input_values = merged(vec![local_player_handle_relative_friction_defaults.clone(), values.clone()]);
        local_player_handle_relative_friction_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/handle-relative-friction",
        "math",
        "local_player_handle_relative_friction",
        json!({"inputs": local_player_handle_relative_friction_inputs.clone()}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [friction_influenced_speed(), handle_relative_friction()].concat()
}
