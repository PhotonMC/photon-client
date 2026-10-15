//! Cases for `player-movement/travel-flying`, `player-movement/travel-in-water`, `player-movement/travel-in-lava`, `player-movement/travel-in-air`.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

fn travel_flying() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_travel_flying_cases = [
        json!({"label": "air-branch-rotates-and-drags"}),
        json!({"label": "water-branch-uses-water-speed", "in_water": true, "water_speed": f32_bits(0.04)}),
        json!({"label": "lava-branch-uses-lava-speed", "in_lava": true, "lava_speed": f32_bits(0.06)}),
        json!({
            "label": "water-wins-over-lava",
            "in_water": true,
            "in_lava": true,
            "water_speed": f32_bits(0.04),
            "lava_speed": f32_bits(0.06),
        }),
        json!({
            "label": "long-input-normalizes-before-speed",
            "input": vector_bits(1.0, 2.0, 2.0),
            "delta": vector_bits(-0.25, 0.125, 0.5),
            "y_rot": f32_bits(-120.0),
            "air_speed": f32_bits(0.07),
        }),
        json!({
            "label": "subthreshold-input-skips-relative-movement",
            "input": vector_bits(0.0001, 0.0, 0.0),
            "delta": vector_bits(0.1, -0.2, 0.3),
        }),
        json!({
            "label": "zero-input-preserves-signed-motion",
            "input": vector_bits(-0.0, 0.0, -0.0),
            "delta": vector_bits(-0.0, 0.125, -0.25),
            "y_rot": f32_bits(90.0),
        }),
        json!({
            "label": "nonfinite-relative-delta-is-rejected",
            "input": {"x": "7ff8000000000001", "y": f64_bits(0.0), "z": f64_bits(0.0)},
            "delta": vector_bits(0.1, -0.2, 0.3),
        }),
    ];
    let local_player_travel_flying_defaults = json!({
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": f32_bits(37.5),
        "in_water": false,
        "in_lava": false,
        "water_speed": f32_bits(0.02),
        "lava_speed": f32_bits(0.03),
        "air_speed": f32_bits(0.04),
    });
    let mut local_player_travel_flying_inputs = vec![];
    for values in local_player_travel_flying_cases.iter() {
        let input_values = merged(vec![local_player_travel_flying_defaults.clone(), values.clone()]);
        local_player_travel_flying_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/travel-flying",
        "math",
        "local_player_travel_flying",
        json!({"inputs": local_player_travel_flying_inputs.clone()}),
    ));
    out
}

fn travel_in_water() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_travel_in_water_cases = [
        json!({"label": "plain-water-applies-drag-and-fluid-gravity"}),
        json!({"label": "sprinting-uses-sprint-water-slowdown-and-bypasses-gravity", "sprinting": true}),
        json!({
            "label": "ground-water-efficiency-blends-speed-and-drag",
            "water_efficiency": f32_bits(0.6),
            "speed": f32_bits(0.12),
        }),
        airborne_water_efficiency_is_halved(),
        dolphins_grace_overrides_computed_drag(),
        horizontal_climbable_sets_fluid_step(),
        free_horizontal_fluid_collision_jumps_out(),
        json!({
            "label": "negative-water-efficiency-is-ignored",
            "water_efficiency": f32_bits(-0.5),
            "speed": f32_bits(0.12),
        }),
    ];
    let local_player_travel_in_water_defaults = travel_in_water_input();
    let mut local_player_travel_in_water_inputs = vec![];
    for values in local_player_travel_in_water_cases.iter() {
        let input_values = merged(vec![local_player_travel_in_water_defaults.clone(), values.clone()]);
        local_player_travel_in_water_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/travel-in-water",
        "math",
        "local_player_travel_in_water",
        json!({"inputs": local_player_travel_in_water_inputs.clone()}),
    ));
    out
}

fn travel_in_water_input() -> Value {
    json!({
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": f32_bits(37.5),
        "sprinting": false,
        "water_slow_down": f32_bits(0.8),
        "water_efficiency": f32_bits(0.0),
        "on_ground": true,
        "speed": f32_bits(0.1),
        "dolphins_grace": false,
        "horizontal_collision": false,
        "on_climbable": false,
        "free_after_jump": false,
        "base_gravity": f64_bits(0.08),
        "is_falling": false,
        "old_y": f64_bits(10.0),
        "current_y": f64_bits(10.0),
    })
}

fn horizontal_climbable_sets_fluid_step() -> Value {
    json!({
        "label": "horizontal-climbable-sets-fluid-step",
        "horizontal_collision": true,
        "on_climbable": true,
        "base_gravity": f64_bits(0.08),
        "is_falling": true,
        "free_after_jump": false,
    })
}

fn free_horizontal_fluid_collision_jumps_out() -> Value {
    json!({
        "label": "free-horizontal-fluid-collision-jumps-out",
        "horizontal_collision": true,
        "base_gravity": f64_bits(0.08),
        "is_falling": true,
        "free_after_jump": true,
    })
}

fn dolphins_grace_overrides_computed_drag() -> Value {
    json!({
        "label": "dolphins-grace-overrides-computed-drag",
        "water_efficiency": f32_bits(0.6),
        "speed": f32_bits(0.12),
        "sprinting": true,
        "dolphins_grace": true,
    })
}

fn airborne_water_efficiency_is_halved() -> Value {
    json!({
        "label": "airborne-water-efficiency-is-halved",
        "water_efficiency": f32_bits(0.6),
        "speed": f32_bits(0.12),
        "on_ground": false,
    })
}

fn travel_in_lava() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_travel_in_lava_cases = [
        json!({"label": "shallow-lava-applies-fluid-drag-falling-and-gravity"}),
        json!({"label": "deep-lava-uses-half-scale-before-gravity", "shallow_lava": false}),
        json!({"label": "zero-gravity-skips-lava-gravity-step", "base_gravity": f64_bits(0.0)}),
        json!({"label": "sprinting-bypasses-falling-adjustment-but-keeps-gravity", "sprinting": true}),
        json!({
            "label": "horizontal-fluid-collision-without-free-space-keeps-motion",
            "horizontal_collision": true,
            "free_after_jump": false,
        }),
        json!({
            "label": "horizontal-fluid-collision-with-free-space-jumps-out",
            "horizontal_collision": true,
            "free_after_jump": true,
        }),
    ];
    let local_player_travel_in_lava_defaults = json!({
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": f32_bits(37.5),
        "sprinting": false,
        "shallow_lava": true,
        "horizontal_collision": false,
        "free_after_jump": false,
        "base_gravity": f64_bits(0.08),
        "is_falling": false,
        "old_y": f64_bits(10.0),
        "current_y": f64_bits(10.0),
    });
    let mut local_player_travel_in_lava_inputs = vec![];
    for values in local_player_travel_in_lava_cases.iter() {
        let input_values = merged(vec![local_player_travel_in_lava_defaults.clone(), values.clone()]);
        local_player_travel_in_lava_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/travel-in-lava",
        "math",
        "local_player_travel_in_lava",
        json!({"inputs": local_player_travel_in_lava_inputs.clone()}),
    ));
    out
}

fn travel_in_air() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_travel_in_air_cases = [
        json!({"label": "air-travel-applies-server-gravity-and-drag"}),
        json!({
            "label": "ground-travel-uses-below-block-friction",
            "on_ground": true,
            "below_block": "ice",
            "block_friction": f32_bits(0.98),
            "speed": f32_bits(0.12),
        }),
        json!({"label": "levitation-replaces-gravity-with-effect-target", "levitation_amplifier": 2}),
        json!({
            "label": "discard-friction-keeps-relative-horizontal-motion",
            "should_discard_friction": true,
            "air_drag_modifier": f32_bits(0.2),
        }),
        json!({
            "label": "omnidirectional-air-mover-uses-horizontal-drag-vertically",
            "omnidirectional_air_mover": true,
            "air_drag_modifier": f32_bits(0.2),
        }),
        json!({"label": "air-drag-attribute-modifies-horizontal-and-vertical", "air_drag_modifier": f32_bits(0.2)}),
        json!({
            "label": "client-unloaded-above-min-y-uses-fallback-vertical-speed",
            "client_side": true,
            "has_chunk_at": false,
        }),
        json!({
            "label": "client-unloaded-at-min-y-zeroes-vertical-speed",
            "client_side": true,
            "has_chunk_at": false,
            "current_y": f64_bits(-64.0),
        }),
    ];
    let local_player_travel_in_air_defaults = travel_in_air_input();
    let mut local_player_travel_in_air_inputs = vec![];
    for values in local_player_travel_in_air_cases.iter() {
        let input_values = merged(vec![local_player_travel_in_air_defaults.clone(), values.clone()]);
        local_player_travel_in_air_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/travel-in-air",
        "math",
        "local_player_travel_in_air",
        json!({"inputs": local_player_travel_in_air_inputs.clone()}),
    ));
    out
}

fn travel_in_air_input() -> Value {
    json!({
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": f32_bits(37.5),
        "on_ground": false,
        "below_block": "air",
        "block_friction": f32_bits(0.6),
        "friction_modifier": f32_bits(1.0),
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
        "levitation_amplifier": -1,
        "gravity": f64_bits(0.08),
        "should_discard_friction": false,
        "air_drag_modifier": f32_bits(0.0),
        "omnidirectional_air_mover": false,
        "client_side": false,
        "has_chunk_at": true,
        "min_y": -64,
        "current_y": f64_bits(10.0),
    })
}

pub fn cases() -> Vec<Value> {
    [travel_flying(), travel_in_water(), travel_in_lava(), travel_in_air()].concat()
}
