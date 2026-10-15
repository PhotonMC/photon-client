//! Cases for `world/client-collision`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn living_pushable_climbable() -> Value {
    case(
        "world/client-collision/living-pushable-climbable",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "non-climbable-player-is-pushable",
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
                            "on_climbable": false,
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
                climbable_player_is_not_pushable(),
            ],
        }),
    )
}

fn climbable_player_is_not_pushable() -> Value {
    json!({
        "label": "climbable-player-is-not-pushable",
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
                "on_climbable": true,
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

fn living_pushable_flying() -> Value {
    case(
        "world/client-collision/living-pushable-flying",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "climbable-grounded-player-is-not-pushable",
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
                            "on_climbable": true,
                            "flying": false,
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
                climbable_flying_player_is_pushable(),
            ],
        }),
    )
}

fn climbable_flying_player_is_pushable() -> Value {
    json!({
        "label": "climbable-flying-player-is-pushable",
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
                "on_climbable": true,
                "flying": true,
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

fn living_pushable_trapdoor_context() -> Value {
    case(
        "world/client-collision/living-pushable-trapdoor-context",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "open-matching-ladder-player-is-not-pushable",
                    "test_area": box_bits(0, 0, 0, 1, 3, 1),
                    "source_entity_id": 7,
                    "entities": [
                        {
                            "entity_id": 7,
                            "type_id": 0,
                            "position": vector_bits(0.5, 1, 0.5),
                            "can_be_collided_with": true,
                        },
                        {
                            "entity_id": 8,
                            "type_id": 156,
                            "position": vector_bits(0.5, 1, 0.5),
                            "alive": true,
                            "spectator": false,
                            "flying": false,
                            "climbable_context": {
                                "kind": "trapdoor_ladder",
                                "trapdoor_open": true,
                                "trapdoor_facing": "north",
                                "ladder_facing": "north",
                            },
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
                closed_trapdoor_player_is_pushable(),
                open_mismatched_ladder_player_is_pushable(),
                open_without_ladder_player_is_pushable(),
            ],
        }),
    )
}

fn open_without_ladder_player_is_pushable() -> Value {
    json!({
        "label": "open-without-ladder-player-is-pushable",
        "test_area": box_bits(0, 0, 0, 1, 3, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 0,
                "position": vector_bits(0.5, 1, 0.5),
                "can_be_collided_with": true,
            },
            {
                "entity_id": 8,
                "type_id": 156,
                "position": vector_bits(0.5, 1, 0.5),
                "alive": true,
                "spectator": false,
                "flying": false,
                "climbable_context": {
                    "kind": "trapdoor_ladder",
                    "trapdoor_open": true,
                    "trapdoor_facing": "north",
                    "ladder_facing": "north",
                    "below_is_ladder": false,
                },
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

fn open_mismatched_ladder_player_is_pushable() -> Value {
    json!({
        "label": "open-mismatched-ladder-player-is-pushable",
        "test_area": box_bits(0, 0, 0, 1, 3, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 0,
                "position": vector_bits(0.5, 1, 0.5),
                "can_be_collided_with": true,
            },
            {
                "entity_id": 8,
                "type_id": 156,
                "position": vector_bits(0.5, 1, 0.5),
                "alive": true,
                "spectator": false,
                "flying": false,
                "climbable_context": {
                    "kind": "trapdoor_ladder",
                    "trapdoor_open": true,
                    "trapdoor_facing": "north",
                    "ladder_facing": "south",
                },
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

fn closed_trapdoor_player_is_pushable() -> Value {
    json!({
        "label": "closed-trapdoor-player-is-pushable",
        "test_area": box_bits(0, 0, 0, 1, 3, 1),
        "source_entity_id": 7,
        "entities": [
            {
                "entity_id": 7,
                "type_id": 0,
                "position": vector_bits(0.5, 1, 0.5),
                "can_be_collided_with": true,
            },
            {
                "entity_id": 8,
                "type_id": 156,
                "position": vector_bits(0.5, 1, 0.5),
                "alive": true,
                "spectator": false,
                "flying": false,
                "climbable_context": {
                    "kind": "trapdoor_ladder",
                    "trapdoor_open": false,
                    "trapdoor_facing": "north",
                    "ladder_facing": "north",
                },
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
    [vec![living_pushable_climbable()], vec![living_pushable_flying()], vec![living_pushable_trapdoor_context()]]
        .concat()
}
