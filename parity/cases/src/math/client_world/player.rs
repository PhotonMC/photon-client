//! Cases for `client-world/player-attribute-application-and-effective-value`, `client-world/player-attribute-movement-speed-binding`, `client-world/player-attribute-water-travel-binding`, `client-world/player-sprint-transition-attribute-binding`.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

fn player_attribute_application_and_effective() -> Value {
    case(
        "client-world/player-attribute-application-and-effective-value",
        "math",
        "client_world_player_attributes",
        json!({
            "player_id": 42,
            "operations": [
                initial_snapshot(),
                movement_speed_replacement(),
                {
                    "action": "apply",
                    "label": "wrong-player-ignored",
                    "player_id": 99,
                    "attributes": [
                        {
                            "attribute_id": 26,
                            "base": f64_bits(2.0),
                            "modifiers": [attribute_modifier("p08:ignored", 1.0, 0)],
                        },
                    ],
                    "queries": [
                        {
                            "label": "state-after-wrong-player",
                            "attribute_id": 26,
                            "min": f64_bits(0.0),
                            "max": f64_bits(1024.0),
                        },
                    ],
                },
            ],
        }),
    )
}

fn initial_snapshot() -> Value {
    json!({
        "action": "apply",
        "label": "initial-snapshot",
        "player_id": 42,
        "attributes": [
            {
                "attribute_id": 26,
                "base": f64_bits(0.25),
                "modifiers": [
                    attribute_modifier("p08:oracle_add", 0.1, 0),
                    attribute_modifier("p08:oracle_base", 0.2, 1),
                    attribute_modifier("p08:oracle_total", 0.5, 2),
                ],
            },
            {
                "attribute_id": 23,
                "base": f64_bits(40.0),
                "modifiers": [attribute_modifier("p08:health_add", 5.0, 0)],
            },
        ],
        "queries": [
            {
                "label": "movement-speed-effective",
                "attribute_id": 26,
                "min": f64_bits(0.0),
                "max": f64_bits(1024.0),
            },
            {
                "label": "max-health-effective",
                "attribute_id": 23,
                "min": f64_bits(1.0),
                "max": f64_bits(1024.0),
            },
        ],
    })
}

fn movement_speed_replacement() -> Value {
    json!({
        "action": "apply",
        "label": "movement-speed-replacement",
        "player_id": 42,
        "attributes": [
            {
                "attribute_id": 26,
                "base": f64_bits(0.5),
                "modifiers": [attribute_modifier("p08:replacement", -0.1, 0)],
            },
        ],
        "queries": [
            {
                "label": "replacement-clears-old-modifiers",
                "attribute_id": 26,
                "min": f64_bits(0.0),
                "max": f64_bits(1024.0),
            },
            {
                "label": "unmentioned-attribute-preserved",
                "attribute_id": 23,
                "min": f64_bits(1.0),
                "max": f64_bits(1024.0),
            },
        ],
    })
}

fn player_attribute_movement_speed_binding() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/player-attribute-movement-speed-binding",
        "math",
        "client_world_player_movement_speed",
        json!({
            "player_id": 42,
            "operations": [
                {
                    "label": "default-player-speed",
                    "player_id": 42,
                    "attributes": [],
                    "on_ground": true,
                    "block_friction": f32_bits(0.6),
                },
                {
                    "label": "modified-attribute-ground-speed",
                    "player_id": 42,
                    "attributes": [
                        {
                            "attribute_id": 26,
                            "base": f64_bits(0.25),
                            "modifiers": [
                                attribute_modifier("p10:movement_add", 0.1, 0),
                                attribute_modifier("p10:movement_base", 0.2, 1),
                                attribute_modifier("p10:movement_total", 0.5, 2),
                            ],
                        },
                    ],
                    "on_ground": true,
                    "block_friction": f32_bits(0.8),
                },
                {
                    "label": "replacement-attribute-ground-speed",
                    "player_id": 42,
                    "attributes": [
                        {
                            "attribute_id": 26,
                            "base": f64_bits(0.5),
                            "modifiers": [attribute_modifier("p10:movement_replacement", -0.1, 0)],
                        },
                    ],
                    "on_ground": true,
                    "block_friction": f32_bits(0.6),
                },
                {
                    "label": "wrong-player-preserves-speed",
                    "player_id": 99,
                    "attributes": [{"attribute_id": 26, "base": f64_bits(2.0), "modifiers": []}],
                    "on_ground": true,
                    "block_friction": f32_bits(0.6),
                },
            ],
        }),
    ));
    out
}

fn player_attribute_water_travel_binding() -> Vec<Value> {
    let mut out = Vec::new();
    let player_attribute_water_defaults = player_attribute_water_travel_binding_input();
    let player_attribute_water_operations = vec![
        json!({"label": "default-speed-enters-water-efficiency", "player_id": 42, "attributes": []}),
        json!({
            "label": "modified-speed-enters-water-efficiency",
            "player_id": 42,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": f64_bits(0.25),
                    "modifiers": [
                        attribute_modifier("p10:water_add", 0.1, 0),
                        attribute_modifier("p10:water_base", 0.2, 1),
                        attribute_modifier("p10:water_total", 0.5, 2),
                    ],
                },
            ],
        }),
        json!({
            "label": "replacement-speed-enters-water-efficiency",
            "player_id": 42,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": f64_bits(0.5),
                    "modifiers": [attribute_modifier("p10:water_replacement", -0.1, 0)],
                },
            ],
        }),
        json!({
            "label": "wrong-player-preserves-water-speed",
            "player_id": 99,
            "attributes": [{"attribute_id": 26, "base": f64_bits(2.0), "modifiers": []}],
        }),
    ];
    out.push(case(
        "client-world/player-attribute-water-travel-binding",
        "math",
        "client_world_player_attribute_water",
        json!({
            "player_id": 42,
            "operations": player_attribute_water_operations.clone().iter().cloned().map(|operation| merged(vec![player_attribute_water_defaults.clone(), operation])).collect::<Vec<_>>(),
        }),
    ));
    out
}

fn player_attribute_water_travel_binding_input() -> Value {
    json!({
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": f32_bits(37.5),
        "sprinting": false,
        "water_slow_down": f32_bits(0.8),
        "water_efficiency": f32_bits(0.6),
        "on_ground": true,
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

fn player_sprint_transition_attribute_binding() -> Vec<Value> {
    let mut out = Vec::new();
    let player_sprint_attribute_operations = vec![
        json!({
            "label": "default-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": true,
            "attributes": [],
        }),
        json!({
            "label": "repeated-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": true,
            "attributes": [],
        }),
        json!({
            "label": "custom-speed-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": true,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": f64_bits(0.25),
                    "modifiers": [attribute_modifier("p10:sprint_custom", 0.2, 2)],
                },
            ],
        }),
        json!({
            "label": "stop-sprinting-removes-modifier",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": false,
            "attributes": [],
        }),
        json!({
            "label": "wrong-player-preserves-stopped-state",
            "player_id": 99,
            "action": "set_sprinting",
            "sprinting": true,
            "attributes": [{"attribute_id": 26, "base": f64_bits(2.0), "modifiers": []}],
        }),
    ];
    out.push(case(
        "client-world/player-sprint-transition-attribute-binding",
        "math",
        "client_world_player_sprint_attribute",
        json!({"player_id": 42, "operations": player_sprint_attribute_operations.clone()}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        vec![player_attribute_application_and_effective()],
        player_attribute_movement_speed_binding(),
        player_attribute_water_travel_binding(),
        player_sprint_transition_attribute_binding(),
    ]
    .concat()
}
