//! Cases for `player-movement/update-fall-flying-movement`, `player-movement/travel-dispatch`.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

fn update_fall_flying_movement() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_update_fall_flying_cases = [
        json!({"label": "neutral-glide-applies-lift-and-drag"}),
        json!({
            "label": "nose-down-converts-descending-speed",
            "look": vector_bits(0.0, -0.5, 0.8660254037844386),
            "x_rot": f32_bits(30.0),
            "delta": vector_bits(0.2, -0.4, 0.3),
        }),
        json!({
            "label": "nose-up-adds-lift",
            "look": vector_bits(0.0, 0.5, 0.8660254037844386),
            "x_rot": f32_bits(-30.0),
            "delta": vector_bits(0.3, 0.1, 0.2),
        }),
        json!({
            "label": "vertical-look-skips-horizontal-conversion",
            "look": vector_bits(0.0, 1.0, 0.0),
            "x_rot": f32_bits(15.0),
        }),
        json!({
            "label": "zero-gravity-keeps-lift-free-motion",
            "gravity": f64_bits(0.0),
            "look": vector_bits(-0.6, 0.0, 0.8),
            "x_rot": f32_bits(0.0),
        }),
        json!({
            "label": "negative-look-direction-aligns-horizontal-motion",
            "look": vector_bits(-0.6, 0.0, 0.8),
            "x_rot": f32_bits(0.0),
            "delta": vector_bits(0.35, -0.1, -0.25),
        }),
    ];
    let local_player_update_fall_flying_defaults = json!({
        "look": vector_bits(0.0, 0.0, 1.0),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "x_rot": f32_bits(0.0),
        "gravity": f64_bits(0.08),
    });
    let mut local_player_update_fall_flying_inputs = vec![];
    for values in &local_player_update_fall_flying_cases {
        let input_values = merged(vec![local_player_update_fall_flying_defaults.clone(), values.clone()]);
        local_player_update_fall_flying_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/update-fall-flying-movement",
        "math",
        "local_player_update_fall_flying_movement",
        json!({"inputs": local_player_update_fall_flying_inputs.clone()}),
    ));
    out
}

fn travel_dispatch() -> Vec<Value> {
    let mut out = Vec::new();
    let local_player_travel_dispatch_cases = vec![
        json!({"label": "air-is-default-when-no-fluid-or-glide"}),
        json!({"label": "water-fluid-branch", "in_water": true}),
        json!({"label": "lava-fluid-branch", "in_lava": true, "shallow_lava": false}),
        json!({
            "label": "water-wins-over-lava-in-fluid-branch",
            "in_water": true,
            "in_lava": true,
            "water_efficiency": f32_bits(0.4),
        }),
        json!({"label": "unaffected-water-falls-through-to-air", "in_water": true, "affected_by_fluids": false}),
        json!({"label": "standing-on-lava-falls-through-to-air", "in_lava": true, "can_stand_on_fluid": true}),
        json!({
            "label": "fall-flying-uses-glide-branch",
            "fall_flying": true,
            "look": vector_bits(0.0, -0.35, 0.9367496997597597),
            "x_rot": f32_bits(20.0),
        }),
        json!({
            "label": "fall-flying-on-climbable-falls-back-to-air",
            "fall_flying": true,
            "on_climbable": true,
            "on_ground": true,
        }),
        json!({"label": "fluid-precedes-fall-flying", "in_water": true, "fall_flying": true}),
    ];
    let local_player_travel_dispatch_defaults = travel_dispatch_input();
    let mut local_player_travel_dispatch_inputs = vec![];
    for values in &local_player_travel_dispatch_cases {
        let input_values = merged(vec![local_player_travel_dispatch_defaults.clone(), values.clone()]);
        local_player_travel_dispatch_inputs.push(input_values.clone());
    }
    out.push(case(
        "player-movement/travel-dispatch",
        "math",
        "local_player_travel_dispatch",
        json!({"inputs": local_player_travel_dispatch_inputs.clone()}),
    ));
    out
}

fn travel_dispatch_input() -> Value {
    json!({
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": f32_bits(37.5),
        "x_rot": f32_bits(0.0),
        "look": vector_bits(0.0, 0.0, 1.0),
        "in_water": false,
        "in_lava": false,
        "affected_by_fluids": true,
        "can_stand_on_fluid": false,
        "fall_flying": false,
        "on_climbable": false,
        "sprinting": false,
        "water_slow_down": f32_bits(0.8),
        "water_efficiency": f32_bits(0.0),
        "on_ground": false,
        "block_friction": f32_bits(0.6),
        "speed": f32_bits(0.1),
        "dolphins_grace": false,
        "horizontal_collision": false,
        "free_after_jump": false,
        "shallow_lava": true,
        "flying_speed": f32_bits(0.05),
        "flying": false,
        "passenger": false,
        "scaffolding": false,
        "suppress_sliding_down_ladder": false,
        "jumping": false,
        "levitation_amplifier": -1,
        "gravity": f64_bits(0.08),
        "client_side": true,
        "has_chunk_at": true,
        "current_y": f64_bits(10.0),
        "min_y": -64,
        "should_discard_friction": false,
        "friction_modifier": f32_bits(1.0),
        "air_drag_modifier": f32_bits(0.0),
        "omnidirectional_air_mover": false,
        "below_block": "air",
    })
}

pub fn cases() -> Vec<Value> {
    [update_fall_flying_movement(), travel_dispatch()].concat()
}
