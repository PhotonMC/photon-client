//! Cases for `world/client-collision`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn living_pushable_climbable_tag() -> Value {
    case(
        "world/client-collision/living-pushable-climbable-tag",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "ladder-state-is-climbable",
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
                            "climbable_context": {"kind": "climbable_tag", "state_id": 5720},
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
                waterlogged_ladder_state_is_climbable(),
                stone_state_is_not_climbable(),
                air_state_is_not_climbable(),
            ],
        }),
    )
}

fn air_state_is_not_climbable() -> Value {
    json!({
        "label": "air-state-is-not-climbable",
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
                "climbable_context": {"kind": "climbable_tag", "state_id": 0},
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

fn stone_state_is_not_climbable() -> Value {
    json!({
        "label": "stone-state-is-not-climbable",
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
                "climbable_context": {"kind": "climbable_tag", "state_id": 1},
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

fn waterlogged_ladder_state_is_climbable() -> Value {
    json!({
        "label": "waterlogged-ladder-state-is-climbable",
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
                "climbable_context": {"kind": "climbable_tag", "state_id": 5719},
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

fn living_pushable_glide_through() -> Value {
    case(
        "world/client-collision/living-pushable-glide-through",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": [
                {
                    "label": "vine-grounded-player-is-not-pushable",
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
                            "climbable_context": {"kind": "glide_through_tag", "state_id": 8389, "fall_flying": false},
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
                vine_fall_flying_player_is_pushable(),
                stone_fall_flying_player_is_pushable(),
            ],
        }),
    )
}

fn stone_fall_flying_player_is_pushable() -> Value {
    json!({
        "label": "stone-fall-flying-player-is-pushable",
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
                "climbable_context": {"kind": "glide_through_tag", "state_id": 1, "fall_flying": true},
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

fn vine_fall_flying_player_is_pushable() -> Value {
    json!({
        "label": "vine-fall-flying-player-is-pushable",
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
                "climbable_context": {"kind": "glide_through_tag", "state_id": 8389, "fall_flying": true},
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

fn living_pushable_update_tags_packet() -> Value {
    case(
        "world/client-collision/living-pushable-update-tags-packet",
        "math",
        "shape",
        json!({
            "kind": "client_world_indexed_collision",
            "inputs": living_pushable_update_tags_packet_inputs(),
        }),
    )
}

fn packet_ladder_grounded_player_is_not() -> Value {
    json!({
        "label": "packet-ladder-grounded-player-is-not-pushable",
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
                    "kind": "update_tags_packet",
                    "state_id": 5720,
                    "fall_flying": false,
                    "block_tags": {
                        "minecraft:climbable": [221, 366],
                        "minecraft:can_glide_through": [366],
                    },
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

fn packet_vine_fall_flying_player_is_pushable() -> Value {
    json!({
        "label": "packet-vine-fall-flying-player-is-pushable",
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
                    "kind": "update_tags_packet",
                    "state_id": 8389,
                    "fall_flying": true,
                    "block_tags": {
                        "minecraft:climbable": [221, 366],
                        "minecraft:can_glide_through": [366],
                    },
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

fn living_pushable_update_tags_packet_inputs() -> Value {
    json!([
        {
            "label": "packet-vine-grounded-player-is-not-pushable",
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
                        "kind": "update_tags_packet",
                        "state_id": 8389,
                        "fall_flying": false,
                        "block_tags": {
                            "minecraft:climbable": [221, 366],
                            "minecraft:can_glide_through": [366],
                        },
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
        packet_vine_fall_flying_player_is_pushable(),
        packet_ladder_grounded_player_is_not(),
    ])
}

pub fn cases() -> Vec<Value> {
    [
        vec![living_pushable_climbable_tag()],
        vec![living_pushable_glide_through()],
        vec![living_pushable_update_tags_packet()],
    ]
    .concat()
}
