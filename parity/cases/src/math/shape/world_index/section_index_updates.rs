//! Cases for `world/client-collision`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn entity_section_index_updates() -> Value {
    case(
        "world/client-collision/entity-section-index-updates",
        "math",
        "shape",
        json!({
            "kind": "client_world_entity_section_index",
            "inputs": [
                {
                    "label": "move-across-horizontal-and-vertical-sections",
                    "entities": [
                        {
                            "entity_id": 21,
                            "type_id": 0,
                            "position": vector_bits(0.5, 0, 0.5),
                            "can_be_collided_with": true,
                        },
                    ],
                    "events": [
                        {"op": "query", "label": "initial-section", "test_area": box_bits(-1, 0, -1, 2, 1, 2)},
                        {"op": "move", "entity_id": 21, "position": vector_bits(1.5, 0, 0.5)},
                        {"op": "query", "label": "same-section-move", "test_area": box_bits(0, 0, 0, 3, 1, 2)},
                        {"op": "move", "entity_id": 21, "position": vector_bits(16.5, 0, 0.5)},
                        {"op": "query", "label": "old-horizontal-section", "test_area": box_bits(-1, 0, -1, 3, 1, 2)},
                        {"op": "query", "label": "new-horizontal-section", "test_area": box_bits(16, 0, 0, 18, 1, 2)},
                        {"op": "move", "entity_id": 21, "position": vector_bits(16.5, 16, 0.5)},
                        {"op": "query", "label": "new-vertical-section", "test_area": box_bits(16, 16, 0, 18, 18, 2)},
                        {"op": "remove", "entity_id": 21},
                        {"op": "query", "label": "removed-section", "test_area": box_bits(16, 16, 0, 18, 18, 2)},
                    ],
                    "placements": [],
                    "shape_states": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(1000),
                        "absolute_max_size": 100000,
                    },
                },
                section_removal_keeps_neighbor(),
            ],
        }),
    )
}

fn section_removal_keeps_neighbor() -> Value {
    json!({
        "label": "section-removal-keeps-neighbor",
        "entities": [
            {
                "entity_id": 22,
                "type_id": 0,
                "position": vector_bits(-0.5, -16, -0.5),
                "can_be_collided_with": true,
            },
            {
                "entity_id": 23,
                "type_id": 0,
                "position": vector_bits(31.5, 0, 0.5),
                "can_be_collided_with": true,
            },
        ],
        "events": [
            {"op": "query", "label": "two-sections", "test_area": box_bits(-2, -17, -2, 1, -15, 1)},
            {"op": "remove", "entity_id": 22},
            {
                "op": "query",
                "label": "neighbor-section-remains",
                "test_area": box_bits(30, -1, -1, 33, 1, 2),
            },
            {"op": "remove", "entity_id": 23},
            {"op": "query", "label": "all-sections-removed", "test_area": box_bits(30, -1, -1, 33, 1, 2)},
        ],
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

pub fn cases() -> Vec<Value> {
    [vec![entity_section_index_updates()]].concat()
}
