//! Cases for `world/client-collision`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn no_collision_sources() -> Value {
    case(
        "world/client-collision/no-collision-sources",
        "math",
        "shape",
        json!({
            "kind": "client_world_collision",
            "inputs": [
                {
                    "label": "empty-null-source",
                    "test_area": box_bits(0, 0, 0, 1, 1, 1),
                    "source_present": false,
                    "placements": [],
                    "shape_states": [],
                    "candidates": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(4),
                        "absolute_max_size": 100,
                    },
                },
                {
                    "label": "block-source",
                    "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                    "source_present": false,
                    "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"}],
                    "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}],
                    "candidates": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(20),
                        "absolute_max_size": 100,
                    },
                },
                entity_source(),
                border_source(),
                far_outside_source(),
            ],
        }),
    )
}

fn entity_source() -> Value {
    json!({
        "label": "entity-source",
        "test_area": box_bits(0, 0, 0, 1, 2, 1),
        "source_present": true,
        "source_box": box_bits(0, 0, 0, 1, 2, 1),
        "source_position": vector_bits(0.5, 1, 0.5),
        "placements": [],
        "shape_states": [],
        "candidates": [
            {"is_source": true, "box": box_bits(0, 0, 0, 1, 2, 1), "can_be_collided_with": true},
            {"box": box_bits(0.5, 0, 0.5, 1.5, 2, 1.5), "can_be_collided_with": true},
        ],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(20),
            "absolute_max_size": 100,
        },
    })
}

fn far_outside_source() -> Value {
    json!({
        "label": "far-outside-source",
        "test_area": box_bits(10, 0, 0, 11, 2, 1),
        "source_present": true,
        "source_box": box_bits(10, 0, 0, 11, 2, 1),
        "source_position": vector_bits(10.5, 1, 0.5),
        "placements": [],
        "shape_states": [],
        "candidates": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(4),
            "absolute_max_size": 100,
        },
    })
}

fn border_source() -> Value {
    json!({
        "label": "border-source",
        "test_area": box_bits(1.5, 0, 0, 2.5, 2, 1),
        "source_present": true,
        "source_box": box_bits(1.5, 0, 0, 2.5, 2, 1),
        "source_position": vector_bits(1.75, 1, 0.5),
        "placements": [],
        "shape_states": [],
        "candidates": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(4),
            "absolute_max_size": 100,
        },
    })
}

fn happy_ghast_collider_override() -> Value {
    case(
        "world/client-collision/happy-ghast-collider-override",
        "math",
        "shape",
        json!({
            "kind": "client_world_happy_ghast_collision",
            "inputs": [
                {
                    "label": "adult-still-timeout",
                    "entities": [
                        {
                            "entity_id": 31,
                            "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": false,
                            "alive": true,
                            "still_timeout": true,
                            "is_vehicle": false,
                        },
                    ],
                    "source_entity_id": null,
                    "test_area": box_bits(-3, -1, -3, 3, 5, 3),
                    "placements": [],
                    "shape_states": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(1000),
                        "absolute_max_size": 100000,
                    },
                },
                adult_free_no_source_collision(),
                baby_still_timeout_rejected(),
                dead_adult_still_timeout_rejected(),
                client_player_above_bounding_box(),
                client_player_below_bounding_box(),
                happy_ghast_vehicle_collides_with_happy(),
                happy_ghast_source_does_not_match_vehicle(),
                source_boat_pushes_live_happy_ghast(),
            ],
        }),
    )
}

fn happy_ghast_source_does_not_match_vehicle() -> Value {
    json!({
        "label": "happy-ghast-source-does-not-match-vehicle-branch",
        "entities": [
            {
                "entity_id": 40,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": true,
                "still_timeout": false,
                "is_vehicle": false,
            },
            {
                "entity_id": 41,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": true,
                "still_timeout": false,
                "is_vehicle": false,
            },
        ],
        "source_entity_id": 40,
        "test_area": box_bits(-3, -1, -3, 3, 5, 3),
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(1000),
            "absolute_max_size": 100000,
        },
    })
}

fn happy_ghast_vehicle_collides_with_happy() -> Value {
    json!({
        "label": "happy-ghast-vehicle-collides-with-happy-ghast",
        "entities": [
            {
                "entity_id": 38,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": true,
                "still_timeout": false,
                "is_vehicle": false,
            },
            {
                "entity_id": 39,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": true,
                "still_timeout": false,
                "is_vehicle": true,
            },
        ],
        "source_entity_id": 38,
        "test_area": box_bits(-3, -1, -3, 3, 5, 3),
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(1000),
            "absolute_max_size": 100000,
        },
    })
}

fn source_boat_pushes_live_happy_ghast() -> Value {
    json!({
        "label": "source-boat-pushes-live-happy-ghast",
        "entities": [
            {
                "entity_id": 43,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": true,
                "still_timeout": false,
                "is_vehicle": false,
            },
            {"entity_id": 44, "type_id": 0, "position": vector_bits(0, 0, 0)},
        ],
        "source_entity_id": 44,
        "test_area": box_bits(-3, -1, -3, 3, 5, 3),
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(1000),
            "absolute_max_size": 100000,
        },
    })
}

fn client_player_below_bounding_box() -> Value {
    json!({
        "label": "client-player-below-bounding-box",
        "entities": [
            {
                "entity_id": 36,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": true,
                "still_timeout": false,
                "is_vehicle": false,
            },
            {"entity_id": 37, "type_id": 156, "position": vector_bits(0, 3.999, 0)},
        ],
        "source_entity_id": 37,
        "test_area": box_bits(-3, -1, -3, 3, 5, 3),
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(1000),
            "absolute_max_size": 100000,
        },
    })
}

fn client_player_above_bounding_box() -> Value {
    json!({
        "label": "client-player-above-bounding-box",
        "entities": [
            {
                "entity_id": 34,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": true,
                "still_timeout": false,
                "is_vehicle": false,
            },
            {"entity_id": 35, "type_id": 156, "position": vector_bits(0, 4, 0)},
        ],
        "source_entity_id": 35,
        "test_area": box_bits(-3, -1, -3, 3, 5, 3),
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(1000),
            "absolute_max_size": 100000,
        },
    })
}

fn dead_adult_still_timeout_rejected() -> Value {
    json!({
        "label": "dead-adult-still-timeout-rejected",
        "entities": [
            {
                "entity_id": 42,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": false,
                "still_timeout": true,
                "is_vehicle": false,
            },
        ],
        "source_entity_id": null,
        "test_area": box_bits(-3, -1, -3, 3, 5, 3),
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(1000),
            "absolute_max_size": 100000,
        },
    })
}

fn baby_still_timeout_rejected() -> Value {
    json!({
        "label": "baby-still-timeout-rejected",
        "entities": [
            {
                "entity_id": 33,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": true,
                "alive": true,
                "still_timeout": true,
                "is_vehicle": false,
            },
        ],
        "source_entity_id": null,
        "test_area": box_bits(-1, -1, -1, 2, 2, 2),
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(1000),
            "absolute_max_size": 100000,
        },
    })
}

fn adult_free_no_source_collision() -> Value {
    json!({
        "label": "adult-free-no-source-collision",
        "entities": [
            {
                "entity_id": 32,
                "type_id": 58,
                "position": vector_bits(0, 0, 0),
                "baby": false,
                "alive": true,
                "still_timeout": false,
                "is_vehicle": false,
            },
        ],
        "source_entity_id": null,
        "test_area": box_bits(-3, -1, -3, 3, 5, 3),
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(1000),
            "absolute_max_size": 100000,
        },
    })
}

fn registry_state_context() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "world/client-collision/registry-state-context",
        "math",
        "shape",
        json!({
            "kind": "client_world_registry_collision",
            "state_ids": [0, 1, 13331, 13333, 13335, 99999],
            "test_area": box_bits(-0.25, -0.25, -0.25, 4.75, 1.25, 1.25),
            "placements": [
                {"x": 0, "y": 0, "z": 0, "state_id": 1},
                {"x": 1, "y": 0, "z": 0, "state_id": 13333},
                {"x": 2, "y": 0, "z": 0, "state_id": 13331},
                {"x": 3, "y": 0, "z": 0, "state_id": 13335},
                {"x": 4, "y": 0, "z": 0, "state_id": 99999},
            ],
        }),
    ));
    out.push(case(
        "world/client-collision/stair-state-shapes",
        "math",
        "shape",
        json!({
            "kind": "client_world_registry_collision",
            "state_ids": [3908, 3918, 3920, 3926, 99999],
            "test_area": box_bits(-0.25, -0.25, -0.25, 4.75, 1.25, 1.25),
            "placements": [
                {"x": 0, "y": 0, "z": 0, "state_id": 3908},
                {"x": 1, "y": 0, "z": 0, "state_id": 3918},
                {"x": 2, "y": 0, "z": 0, "state_id": 3920},
                {"x": 3, "y": 0, "z": 0, "state_id": 3926},
                {"x": 4, "y": 0, "z": 0, "state_id": 99999},
            ],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [vec![no_collision_sources()], vec![happy_ghast_collider_override()], registry_state_context()].concat()
}
