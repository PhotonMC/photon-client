//! Cases for `world/client-collision`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn indexed_entity_state() -> Value {
    case(
        "world/client-collision/indexed-entity-state",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "empty-index",
                    "test_area": box_bits(0, 0, 0, 1, 1, 1),
                    "source_entity_id": null,
                    "entities": [],
                    "placements": [],
                    "shape_states": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(40),
                        "absolute_max_size": 100,
                    },
                },
                {
                    "label": "indexed-boat-collision",
                    "test_area": box_bits(0, 0, 0, 1, 1, 1),
                    "source_entity_id": null,
                    "entities": [
                        {
                            "entity_id": 7,
                            "type_id": 0,
                            "position": vector_bits(0.5, 0, 0.5),
                            "can_be_collided_with": true,
                        },
                    ],
                    "placements": [],
                    "shape_states": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(40),
                        "absolute_max_size": 100,
                    },
                },
                indexed_boat_outside(),
                source_excluded(),
                indexed_player_no_collider(),
            ],
        }),
    )
}

fn indexed_player_no_collider() -> Value {
    json!({
        "label": "indexed-player-no-collider",
        "test_area": box_bits(0, 0, 0, 1, 2, 1),
        "source_entity_id": null,
        "entities": [
            {
                "entity_id": 8,
                "type_id": 156,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": false,
            },
        ],
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(40),
            "absolute_max_size": 100,
        },
    })
}

fn source_excluded() -> Value {
    json!({
        "label": "source-excluded",
        "test_area": box_bits(0, 0, 0, 1, 1, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 0,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": true,
            },
        ],
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(40),
            "absolute_max_size": 100,
        },
    })
}

fn indexed_boat_outside() -> Value {
    json!({
        "label": "indexed-boat-outside",
        "test_area": box_bits(0, 0, 0, 1, 1, 1),
        "source_entity_id": null,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 0,
                "position": vector_bits(3.5, 0, 3.5),
                "can_be_collided_with": true,
            },
        ],
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(40),
            "absolute_max_size": 100,
        },
    })
}

fn indexed_shulker_state() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "world/client-collision/indexed-shulker-state",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "shulker-override-collision",
                    "test_area": box_bits(0, 0, 0, 1, 1, 1),
                    "source_entity_id": null,
                    "entities": [
                        {
                            "entity_id": 9,
                            "type_id": 112,
                            "position": vector_bits(0.5, 0, 0.5),
                            "can_be_collided_with": false,
                        },
                    ],
                    "placements": [],
                    "shape_states": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(40),
                        "absolute_max_size": 100,
                    },
                },
                {
                    "label": "shulker-outside-query",
                    "test_area": box_bits(0, 0, 0, 1, 1, 1),
                    "source_entity_id": null,
                    "entities": [
                        {
                            "entity_id": 9,
                            "type_id": 112,
                            "position": vector_bits(2, 0, 2),
                            "can_be_collided_with": false,
                        },
                    ],
                    "placements": [],
                    "shape_states": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(40),
                        "absolute_max_size": 100,
                    },
                },
            ],
        }),
    ));
    out
}

fn source_vehicle_pushable() -> Value {
    case(
        "world/client-collision/source-vehicle-pushable",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "boat-source-pushable-player",
                    "test_area": box_bits(0, 0, 0, 1, 2, 1),
                    "source_entity_id": 7,
                    "entities": [
                        {
                            "entity_id": 7,
                            "type_id": 0,
                            "position": vector_bits(0.5, 0, 0.5),
                            "can_be_collided_with": true,
                        },
                        {
                            "entity_id": 8,
                            "type_id": 156,
                            "position": vector_bits(0.5, 0, 0.5),
                            "can_be_collided_with": false,
                        },
                    ],
                    "placements": [],
                    "shape_states": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(40),
                        "absolute_max_size": 100,
                    },
                },
                base_source_rejects_pushable_player(),
                minecart_source_pushable_player(),
                boat_source_pushable_minecart(),
            ],
        }),
    )
}

fn boat_source_pushable_minecart() -> Value {
    json!({
        "label": "boat-source-pushable-minecart",
        "test_area": box_bits(0, 0, 0, 1, 1, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 0,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": true,
            },
            {
                "entity_id": 8,
                "type_id": 85,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": false,
            },
        ],
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(40),
            "absolute_max_size": 100,
        },
    })
}

fn minecart_source_pushable_player() -> Value {
    json!({
        "label": "minecart-source-pushable-player",
        "test_area": box_bits(0, 0, 0, 1, 2, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 85,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": false,
            },
            {
                "entity_id": 8,
                "type_id": 156,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": false,
            },
        ],
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(40),
            "absolute_max_size": 100,
        },
    })
}

fn base_source_rejects_pushable_player() -> Value {
    json!({
        "label": "base-source-rejects-pushable-player",
        "test_area": box_bits(0, 0, 0, 1, 2, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 156,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": false,
            },
            {
                "entity_id": 8,
                "type_id": 156,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": false,
            },
        ],
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(40),
            "absolute_max_size": 100,
        },
    })
}

fn living_pushable_state() -> Value {
    case(
        "world/client-collision/living-pushable-state",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "alive-player-is-pushable",
                    "test_area": box_bits(0, 0, 0, 1, 2, 1),
                    "source_entity_id": 7,
                    "entities": [
                        {
                            "entity_id": 7,
                            "type_id": 0,
                            "position": vector_bits(0.5, 0, 0.5),
                            "can_be_collided_with": true,
                        },
                        {
                            "entity_id": 8,
                            "type_id": 156,
                            "position": vector_bits(0.5, 0, 0.5),
                            "alive": true,
                            "spectator": false,
                            "can_be_collided_with": false,
                        },
                    ],
                    "placements": [],
                    "shape_states": [],
                    "border": {
                        "center_x": f64_bits(0),
                        "center_z": f64_bits(0),
                        "size": f64_bits(40),
                        "absolute_max_size": 100,
                    },
                },
                spectator_player_is_not_pushable(),
                dead_player_is_not_pushable(),
            ],
        }),
    )
}

fn dead_player_is_not_pushable() -> Value {
    json!({
        "label": "dead-player-is-not-pushable",
        "test_area": box_bits(0, 0, 0, 1, 2, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 0,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": true,
            },
            {
                "entity_id": 8,
                "type_id": 156,
                "position": vector_bits(0.5, 0, 0.5),
                "alive": false,
                "spectator": false,
                "can_be_collided_with": false,
            },
        ],
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(40),
            "absolute_max_size": 100,
        },
    })
}

fn spectator_player_is_not_pushable() -> Value {
    json!({
        "label": "spectator-player-is-not-pushable",
        "test_area": box_bits(0, 0, 0, 1, 2, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 0,
                "position": vector_bits(0.5, 0, 0.5),
                "can_be_collided_with": true,
            },
            {
                "entity_id": 8,
                "type_id": 156,
                "position": vector_bits(0.5, 0, 0.5),
                "alive": true,
                "spectator": true,
                "can_be_collided_with": false,
            },
        ],
        "placements": [],
        "shape_states": [],
        "border": {
            "center_x": f64_bits(0),
            "center_z": f64_bits(0),
            "size": f64_bits(40),
            "absolute_max_size": 100,
        },
    })
}

pub fn cases() -> Vec<Value> {
    [
        vec![indexed_entity_state()],
        indexed_shulker_state(),
        vec![source_vehicle_pushable()],
        vec![living_pushable_state()],
    ]
    .concat()
}
