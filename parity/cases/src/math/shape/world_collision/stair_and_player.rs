//! Cases for `world/client-collision`.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::values::*;

fn stair_neighbor_shape_updates() -> Value {
    case(
        "world/client-collision/stair-neighbor-shape-updates",
        "math",
        "shape",
        json!({
            "kind": "client_world_stair_neighbor_collision",
            "inputs": [
                {
                    "label": "behind-outer-left",
                    "target": {"x": 0, "y": 0, "z": 0},
                    "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                    "placements": [
                        {"x": 0, "y": 0, "z": 0, "state_id": 3918},
                        {"x": 0, "y": 0, "z": -1, "state_id": 3958},
                    ],
                },
                {
                    "label": "behind-outer-right",
                    "target": {"x": 0, "y": 0, "z": 0},
                    "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                    "placements": [
                        {"x": 0, "y": 0, "z": 0, "state_id": 3918},
                        {"x": 0, "y": 0, "z": -1, "state_id": 3978},
                    ],
                },
                {
                    "label": "front-inner-left",
                    "target": {"x": 0, "y": 0, "z": 0},
                    "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                    "placements": [
                        {"x": 0, "y": 0, "z": 0, "state_id": 3918},
                        {"x": 0, "y": 0, "z": 1, "state_id": 3958},
                    ],
                },
                {
                    "label": "front-inner-right",
                    "target": {"x": 0, "y": 0, "z": 0},
                    "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                    "placements": [
                        {"x": 0, "y": 0, "z": 0, "state_id": 3918},
                        {"x": 0, "y": 0, "z": 1, "state_id": 3978},
                    ],
                },
                half_mismatch_keeps_straight(),
                can_take_shape_blocked(),
                waterlogged_outer_left(),
            ],
        }),
    )
}

fn can_take_shape_blocked() -> Value {
    json!({
        "label": "can-take-shape-blocked",
        "target": {"x": 0, "y": 0, "z": 0},
        "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
        "placements": [
            {"x": 0, "y": 0, "z": 0, "state_id": 3918},
            {"x": 0, "y": 0, "z": -1, "state_id": 3958},
            {"x": 1, "y": 0, "z": 0, "state_id": 3918},
        ],
    })
}

fn waterlogged_outer_left() -> Value {
    json!({
        "label": "waterlogged-outer-left",
        "target": {"x": 0, "y": 0, "z": 0},
        "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
        "placements": [
            {"x": 0, "y": 0, "z": 0, "state_id": 3917},
            {"x": 0, "y": 0, "z": -1, "state_id": 3958},
        ],
    })
}

fn half_mismatch_keeps_straight() -> Value {
    json!({
        "label": "half-mismatch-keeps-straight",
        "target": {"x": 0, "y": 0, "z": 0},
        "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
        "placements": [
            {"x": 0, "y": 0, "z": 0, "state_id": 3918},
            {"x": 0, "y": 0, "z": -1, "state_id": 3908},
        ],
    })
}

fn local_player_minor_horizontal() -> Value {
    case(
        "world/client-collision/local-player-minor-horizontal",
        "math",
        "shape",
        json!({
            "kind": "local_player_minor_horizontal_collision",
            "inputs": [
                {
                    "label": "aligned-forward",
                    "y_rot": f32_bits(0),
                    "xxa": f32_bits(0),
                    "zza": f32_bits(1),
                    "movement": vector_bits(0, 0, 1),
                },
                {
                    "label": "aligned-yaw-90",
                    "y_rot": f32_bits(90),
                    "xxa": f32_bits(0),
                    "zza": f32_bits(1),
                    "movement": vector_bits(-1, 0, 0),
                },
                {
                    "label": "aligned-strafe-yaw-45",
                    "y_rot": f32_bits(45),
                    "xxa": f32_bits(1),
                    "zza": f32_bits(0),
                    "movement": vector_bits(0.7071067811865476, 0, 0.7071067811865476),
                },
                {
                    "label": "seven-point-nine-degrees",
                    "y_rot": f32_bits(0),
                    "xxa": f32_bits(0),
                    "zza": f32_bits(1),
                    "movement": vector_bits(0.13744454603714665, 0, 0.9905094632383088),
                },
                {
                    "label": "eight-degree-threshold",
                    "y_rot": f32_bits(0),
                    "xxa": f32_bits(0),
                    "zza": f32_bits(1),
                    "movement": vector_bits(0.13917310096006544, 0, 0.9902680687415704),
                },
                eight_point_one_degrees(),
                reverse_movement(),
                zero_input(),
                zero_movement(),
                subthreshold_input(),
            ],
        }),
    )
}

fn subthreshold_input() -> Value {
    json!({
        "label": "subthreshold-input",
        "y_rot": f32_bits(0),
        "xxa": f32_bits(0),
        "zza": f32_bits(0.001),
        "movement": vector_bits(0, 0, 1),
    })
}

fn zero_movement() -> Value {
    json!({
        "label": "zero-movement",
        "y_rot": f32_bits(0),
        "xxa": f32_bits(0),
        "zza": f32_bits(1),
        "movement": vector_bits(0, 0, 0),
    })
}

fn zero_input() -> Value {
    json!({
        "label": "zero-input",
        "y_rot": f32_bits(0),
        "xxa": f32_bits(0),
        "zza": f32_bits(0),
        "movement": vector_bits(0, 0, 1),
    })
}

fn reverse_movement() -> Value {
    json!({
        "label": "reverse-movement",
        "y_rot": f32_bits(0),
        "xxa": f32_bits(0),
        "zza": f32_bits(1),
        "movement": vector_bits(0, 0, -1),
    })
}

fn eight_point_one_degrees() -> Value {
    json!({
        "label": "eight-point-one-degrees",
        "y_rot": f32_bits(0),
        "xxa": f32_bits(0),
        "zza": f32_bits(1),
        "movement": vector_bits(0.14090123193758267, 0, 0.9900236577165575),
    })
}

pub fn cases() -> Vec<Value> {
    [vec![stair_neighbor_shape_updates()], vec![local_player_minor_horizontal()]].concat()
}
