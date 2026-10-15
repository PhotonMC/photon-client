//! Cases for `shape/player-pose-selection`.

use serde_json::{Value, json};

use crate::case::case;

fn priority_fit_fallbacks() -> Value {
    case(
        "shape/player-pose-selection/priority-fit-fallbacks",
        "math",
        "shape",
        json!({
            "kind": "player_pose_selection",
            "inputs": [
                {
                    "label": "sleeping-priority",
                    "sleeping": true,
                    "swimming": true,
                    "fall_flying": true,
                    "auto_spin_attack": true,
                    "shift_key_down": true,
                    "flying": false,
                    "spectator": false,
                    "passenger": false,
                    "swimming_fits": true,
                    "desired_fits": true,
                    "crouching_fits": true,
                },
                {
                    "label": "swimming-priority",
                    "sleeping": false,
                    "swimming": true,
                    "fall_flying": true,
                    "auto_spin_attack": true,
                    "shift_key_down": true,
                    "flying": false,
                    "spectator": false,
                    "passenger": false,
                    "swimming_fits": true,
                    "desired_fits": true,
                    "crouching_fits": true,
                },
                fall_flying_priority(),
                spin_attack_priority(),
                crouching(),
                flying_clears_crouch(),
                standing_default(),
                swimming_fit_gate(),
                fallback_to_crouching(),
                fallback_to_swimming(),
                spectator_bypass(),
                passenger_bypass(),
            ],
        }),
    )
}

fn passenger_bypass() -> Value {
    json!({
        "label": "passenger-bypass",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": false,
        "shift_key_down": false,
        "flying": false,
        "spectator": false,
        "passenger": true,
        "swimming_fits": true,
        "desired_fits": false,
        "crouching_fits": false,
    })
}

fn spectator_bypass() -> Value {
    json!({
        "label": "spectator-bypass",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": false,
        "shift_key_down": false,
        "flying": false,
        "spectator": true,
        "passenger": false,
        "swimming_fits": true,
        "desired_fits": false,
        "crouching_fits": false,
    })
}

fn fallback_to_swimming() -> Value {
    json!({
        "label": "fallback-to-swimming",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": false,
        "shift_key_down": false,
        "flying": false,
        "spectator": false,
        "passenger": false,
        "swimming_fits": true,
        "desired_fits": false,
        "crouching_fits": false,
    })
}

fn fallback_to_crouching() -> Value {
    json!({
        "label": "fallback-to-crouching",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": false,
        "shift_key_down": false,
        "flying": false,
        "spectator": false,
        "passenger": false,
        "swimming_fits": true,
        "desired_fits": false,
        "crouching_fits": true,
    })
}

fn swimming_fit_gate() -> Value {
    json!({
        "label": "swimming-fit-gate",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": false,
        "shift_key_down": false,
        "flying": false,
        "spectator": false,
        "passenger": false,
        "swimming_fits": false,
        "desired_fits": false,
        "crouching_fits": false,
    })
}

fn standing_default() -> Value {
    json!({
        "label": "standing-default",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": false,
        "shift_key_down": false,
        "flying": false,
        "spectator": false,
        "passenger": false,
        "swimming_fits": true,
        "desired_fits": true,
        "crouching_fits": true,
    })
}

fn flying_clears_crouch() -> Value {
    json!({
        "label": "flying-clears-crouch",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": false,
        "shift_key_down": true,
        "flying": true,
        "spectator": false,
        "passenger": false,
        "swimming_fits": true,
        "desired_fits": true,
        "crouching_fits": true,
    })
}

fn crouching() -> Value {
    json!({
        "label": "crouching",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": false,
        "shift_key_down": true,
        "flying": false,
        "spectator": false,
        "passenger": false,
        "swimming_fits": true,
        "desired_fits": true,
        "crouching_fits": true,
    })
}

fn spin_attack_priority() -> Value {
    json!({
        "label": "spin-attack-priority",
        "sleeping": false,
        "swimming": false,
        "fall_flying": false,
        "auto_spin_attack": true,
        "shift_key_down": false,
        "flying": false,
        "spectator": false,
        "passenger": false,
        "swimming_fits": true,
        "desired_fits": true,
        "crouching_fits": true,
    })
}

fn fall_flying_priority() -> Value {
    json!({
        "label": "fall-flying-priority",
        "sleeping": false,
        "swimming": false,
        "fall_flying": true,
        "auto_spin_attack": true,
        "shift_key_down": false,
        "flying": false,
        "spectator": false,
        "passenger": false,
        "swimming_fits": true,
        "desired_fits": true,
        "crouching_fits": true,
    })
}

pub fn cases() -> Vec<Value> {
    [vec![priority_fit_fallbacks()]].concat()
}
