//! Cases for `client-world/entity-passengers`, `client-world/entity-passenger-position`, `client-world/entity-passenger-player-position`, `client-world/entity-passenger-boat-family-position` and more.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::values::*;

fn entity_passengers() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-passengers",
        "math",
        "client_world_entity_passengers",
        json!({
            "entities": [
                {"entity_id": 37, "type_id": 89},
                {"entity_id": 38, "type_id": 120},
                {"entity_id": 39, "type_id": 89},
                {"entity_id": 40, "type_id": 71},
                {"entity_id": 41, "type_id": 120},
                {"entity_id": 42, "type_id": 156},
            ],
            "initial_links": [
                {"vehicle_id": 39, "passenger_ids": [38]},
                {"vehicle_id": 37, "passenger_ids": [40]},
                {"vehicle_id": 40, "passenger_ids": [41]},
            ],
            "operations": [
                {
                    "label": "replacement-reparent-duplicate-unknown",
                    "vehicle_id": 37,
                    "passenger_ids": [38, 39, 999, 38, 42],
                },
                {"label": "cycle-rejected-after-reparent", "vehicle_id": 39, "passenger_ids": [37]},
                {"label": "empty-replacement", "vehicle_id": 37, "passenger_ids": []},
                {"label": "unknown-vehicle-ignored", "vehicle_id": 999, "passenger_ids": [38]},
                {"label": "nested-relation-order", "vehicle_id": 40, "passenger_ids": [37, 42]},
                {"label": "cycle-keeps-existing-links", "vehicle_id": 37, "passenger_ids": [38, 40]},
                {"label": "nested-replacement", "vehicle_id": 40, "passenger_ids": [41]},
            ],
        }),
    ));
    out
}

fn entity_passenger_position() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-passenger-position",
        "math",
        "client_world_entity_passenger_position",
        json!({
            "vehicle": {
                "entity_id": 37,
                "type_id": 89,
                "position": vector_bits(2.0, 2.0, 2.0),
                "rotation": {"y": f32_bits(90.0), "x": f32_bits(0.0)},
                "no_gravity": false,
            },
            "passengers": [
                {
                    "entity_id": 38,
                    "type_id": 120,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.125, 0.0, 0.0),
                    "rotation": {"y": f32_bits(10.0), "x": f32_bits(-15.0)},
                    "no_gravity": false,
                },
                {
                    "entity_id": 39,
                    "type_id": 120,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(-0.125, 0.0, 0.0),
                    "rotation": {"y": f32_bits(-20.0), "x": f32_bits(25.0)},
                    "no_gravity": false,
                },
            ],
            "initial_passenger_ids": [38],
            "operations": [
                {"action": "tick", "label": "single-seat", "ticks": 1},
                {"action": "set-passengers-and-tick", "label": "multi-seat", "passenger_ids": [38, 39], "ticks": 1},
            ],
        }),
    ));
    out
}

fn entity_passenger_player_position() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-passenger-player-position",
        "math",
        "client_world_entity_passenger_player_position",
        json!({
            "vehicle": {
                "entity_id": 37,
                "type_id": 89,
                "position": vector_bits(2.0, 2.0, 2.0),
                "rotation": {"y": f32_bits(90.0), "x": f32_bits(0.0)},
                "no_gravity": false,
            },
            "passenger": {
                "entity_id": 38,
                "type_id": 156,
                "position": vector_bits(2.0, 12.0, 2.0),
                "movement": vector_bits(0.125, 0.0, 0.0),
                "rotation": {"y": f32_bits(10.0), "x": f32_bits(-15.0)},
                "no_gravity": false,
            },
            "operations": [{"action": "tick", "label": "player-seat", "ticks": 1}],
        }),
    ));
    let boat_family_rows = vec![
        ("oak-boat", 37, 89, 38),
        ("oak-chest-boat", 39, 90, 40),
        ("bamboo-raft", 41, 9, 42),
        ("bamboo-chest-raft", 43, 8, 44),
    ];
    out.push(case(
        "client-world/entity-passenger-boat-family-position",
        "math",
        "client_world_entity_passenger_boat_family_position",
        json!({
            "rows": boat_family_rows.clone().iter().copied().map(|(label, vehicle_id, vehicle_type_id, passenger_id)| json!({"label": label, "vehicle": {"entity_id": vehicle_id, "type_id": vehicle_type_id, "position": vector_bits(2.0, 10.0, 2.0), "rotation": {"y": f32_bits(90.0), "x": f32_bits(0.0)}, "no_gravity": false}, "passenger": {"entity_id": passenger_id, "type_id": 120, "position": vector_bits(2.0, 12.0, 2.0), "movement": vector_bits(0.125, 0.0, 0.0), "rotation": {"y": f32_bits(10.0), "x": f32_bits(-15.0)}, "no_gravity": false}})).collect::<Vec<_>>(),
            "operations": [{"action": "tick", "label": "family-seat", "ticks": 1}],
        }),
    ));
    out
}

fn entity_passenger_rotation() -> Vec<Value> {
    let mut out = Vec::new();
    let rotation_rows = vec![("snowball-clamp", 45, 120, 46, 170.0), ("player-clamp", 47, 156, 48, 140.0)];
    out.push(case(
        "client-world/entity-passenger-rotation",
        "math",
        "client_world_entity_passenger_rotation",
        json!({
            "rows": rotation_rows.clone().iter().copied().map(|(label, vehicle_id, passenger_type_id, passenger_id, passenger_yaw)| json!({"label": label, "vehicle": {"entity_id": vehicle_id, "type_id": 89, "position": vector_bits(2.0, 2.0, 2.0), "rotation": {"y": f32_bits(0.0), "x": f32_bits(0.0)}, "no_gravity": true}, "passenger": {"entity_id": passenger_id, "type_id": passenger_type_id, "position": vector_bits(2.0, 12.0, 2.0), "movement": vector_bits(0.0, 0.0, 0.0), "rotation": {"y": f32_bits(passenger_yaw), "x": f32_bits(0.0)}, "no_gravity": true}})).collect::<Vec<_>>(),
            "operations": [{"action": "tick", "label": "clamp", "ticks": 1}],
        }),
    ));
    out.push(case(
        "client-world/entity-passenger-boat-turn-exemption",
        "math",
        "client_world_entity_passenger_boat_turn_exemption",
        json!({
            "vehicle": {
                "entity_id": 49,
                "type_id": 89,
                "position": vector_bits(2.0, 2.0, 2.0),
                "rotation": {"y": f32_bits(0.0), "x": f32_bits(0.0)},
                "no_gravity": true,
            },
            "passenger": {
                "entity_id": 50,
                "type_id": 17,
                "position": vector_bits(2.0, 12.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 0.0),
                "rotation": {"y": f32_bits(170.0), "x": f32_bits(0.0)},
                "no_gravity": true,
            },
            "operations": [{"action": "tick", "label": "breeze-exemption", "ticks": 1}],
        }),
    ));
    let animal_rotation_rows = vec![("even-animal", 51, 52, 53), ("odd-animal", 54, 55, 56)];
    out.push(case(
        "client-world/entity-passenger-animal-rotation",
        "math",
        "client_world_entity_passenger_animal_rotation",
        json!({
            "rows": animal_rotation_rows.clone().iter().copied().map(|(label, vehicle_id, animal_id, filler_id)| json!({"label": label, "vehicle": {"entity_id": vehicle_id, "type_id": 89, "position": vector_bits(2.0, 2.0, 2.0), "rotation": {"y": f32_bits(0.0), "x": f32_bits(0.0)}, "no_gravity": true}, "animal": {"entity_id": animal_id, "type_id": 30, "position": vector_bits(2.0, 12.0, 2.0), "movement": vector_bits(0.0, 0.0, 0.0), "rotation": {"y": f32_bits(170.0), "x": f32_bits(0.0)}, "no_gravity": true}, "filler": {"entity_id": filler_id, "type_id": 120, "position": vector_bits(2.0, 12.0, 2.0), "movement": vector_bits(0.0, 0.0, 0.0), "rotation": {"y": f32_bits(0.0), "x": f32_bits(0.0)}, "no_gravity": true}})).collect::<Vec<_>>(),
            "operations": [{"action": "tick", "label": "animal-capacity-offset", "ticks": 1}],
        }),
    ));
    out
}

fn entity_passenger_animal_position() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-passenger-animal-position",
        "math",
        "client_world_entity_passenger_animal_position",
        json!({
            "vehicle": {
                "entity_id": 57,
                "type_id": 89,
                "position": vector_bits(2.0, 2.0, 2.0),
                "rotation": {"y": f32_bits(0.0), "x": f32_bits(0.0)},
                "no_gravity": true,
            },
            "animal": {
                "entity_id": 58,
                "type_id": 30,
                "position": vector_bits(2.0, 12.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 0.0),
                "rotation": {"y": f32_bits(170.0), "x": f32_bits(0.0)},
                "no_gravity": true,
            },
            "operations": [{"action": "tick", "label": "single-animal-seat", "ticks": 1}],
        }),
    ));
    let turn_rows = vec![("player-callback", 59, 60, 156), ("breeze-callback", 61, 62, 17)];
    out.push(case(
        "client-world/entity-passenger-turn",
        "math",
        "client_world_entity_passenger_turn",
        json!({
            "rows": turn_rows.clone().iter().copied().map(|(label, vehicle_id, passenger_id, passenger_type_id)| json!({"label": label, "vehicle": {"entity_id": vehicle_id, "type_id": 89, "position": vector_bits(2.0, 2.0, 2.0), "rotation": {"y": f32_bits(0.0), "x": f32_bits(0.0)}, "no_gravity": true}, "passenger": {"entity_id": passenger_id, "type_id": passenger_type_id, "position": vector_bits(2.0, 2.0, 2.0), "movement": vector_bits(0.0, 0.0, 0.0), "rotation": {"y": f32_bits(168.75), "x": f32_bits(0.0)}, "no_gravity": true}})).collect::<Vec<_>>(),
            "operations": [{"action": "turn", "label": "direct-passenger-turn", "mouse_x": -100.0, "mouse_y": 0.0}],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        entity_passengers(),
        entity_passenger_position(),
        entity_passenger_player_position(),
        entity_passenger_rotation(),
        entity_passenger_animal_position(),
    ]
    .concat()
}
