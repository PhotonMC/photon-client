//! Cases for `client-world/entity-snowball-tick`, `client-world/entity-arrow-tick`, `client-world/entity-experience-orb-tick`, `client-world/entity-primed-tnt-tick` and more.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::values::*;

fn entity_snowball_tick() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-snowball-tick",
        "math",
        "client_world_entity_snowball_tick",
        json!({
            "entities": [
                {
                    "entity_id": 37,
                    "position": vector_bits(2.0, 1.0, 2.0),
                    "movement": vector_bits(0.25, 0.5, 0.25),
                    "rotation": {"y": f32_bits(10.0), "x": f32_bits(-20.0)},
                    "no_gravity": false,
                },
                {
                    "entity_id": 38,
                    "position": vector_bits(4.0, 1.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 1.0),
                    "rotation": {"y": f32_bits(355.0), "x": f32_bits(30.0)},
                    "no_gravity": true,
                },
            ],
            "operations": [
                {"action": "tick", "label": "first-air-tick", "ticks": 1},
                {"action": "tick", "label": "second-air-tick", "ticks": 1},
            ],
        }),
    ));
    out
}

fn entity_arrow_tick() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-arrow-tick",
        "math",
        "client_world_entity_arrow_tick",
        json!({
            "entities": [
                {
                    "entity_id": 37,
                    "position": vector_bits(2.0, 1.0, 2.0),
                    "movement": vector_bits(0.25, 0.5, 0.25),
                    "rotation": {"y": f32_bits(10.0), "x": f32_bits(-20.0)},
                    "no_gravity": false,
                },
                {
                    "entity_id": 38,
                    "position": vector_bits(4.0, 1.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 1.0),
                    "rotation": {"y": f32_bits(355.0), "x": f32_bits(30.0)},
                    "no_gravity": true,
                },
            ],
            "operations": [
                {"action": "tick", "label": "first-air-tick", "ticks": 1},
                {"action": "tick", "label": "second-air-tick", "ticks": 1},
            ],
        }),
    ));
    out
}

fn entity_experience_orb_tick() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-experience-orb-tick",
        "math",
        "client_world_entity_experience_orb_tick",
        json!({
            "entities": [
                {
                    "entity_id": 37,
                    "position": vector_bits(2.0, 10.0, 2.0),
                    "movement": vector_bits(0.125, 0.25, -0.125),
                    "rotation": {"y": f32_bits(10.0), "x": f32_bits(20.0)},
                    "no_gravity": false,
                },
                {
                    "entity_id": 38,
                    "position": vector_bits(4.0, 10.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 1.0),
                    "rotation": {"y": f32_bits(45.0), "x": f32_bits(30.0)},
                    "no_gravity": true,
                },
            ],
            "operations": [
                {"action": "tick", "label": "first-client-tick", "ticks": 1},
                {"action": "tick", "label": "second-client-tick", "ticks": 1},
            ],
        }),
    ));
    out
}

fn entity_primed_tnt_tick() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-primed-tnt-tick",
        "math",
        "client_world_entity_primed_tnt_tick",
        json!({
            "entities": [
                {
                    "entity_id": 37,
                    "position": vector_bits(2.0, 10.0, 2.0),
                    "movement": vector_bits(0.125, 0.25, -0.125),
                    "rotation": {"y": f32_bits(10.0), "x": f32_bits(20.0)},
                    "no_gravity": false,
                },
                {
                    "entity_id": 38,
                    "position": vector_bits(4.0, 10.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 1.0),
                    "rotation": {"y": f32_bits(45.0), "x": f32_bits(30.0)},
                    "no_gravity": true,
                },
            ],
            "operations": [
                {"action": "tick", "label": "first-air-tick", "ticks": 1},
                {"action": "tick", "label": "second-air-tick", "ticks": 1},
            ],
        }),
    ));
    out
}

fn entity_falling_block_tick() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-falling-block-tick",
        "math",
        "client_world_entity_falling_block_tick",
        json!({
            "entities": [
                {
                    "entity_id": 37,
                    "position": vector_bits(2.0, 10.0, 2.0),
                    "movement": vector_bits(0.125, 0.25, -0.125),
                    "rotation": {"y": f32_bits(10.0), "x": f32_bits(20.0)},
                    "no_gravity": false,
                },
                {
                    "entity_id": 38,
                    "position": vector_bits(4.0, 10.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 1.0),
                    "rotation": {"y": f32_bits(45.0), "x": f32_bits(30.0)},
                    "no_gravity": true,
                },
            ],
            "operations": [
                {"action": "tick", "label": "first-air-tick", "ticks": 1},
                {"action": "tick", "label": "second-air-tick", "ticks": 1},
            ],
        }),
    ));
    out
}

fn entity_firework_tick() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-firework-tick",
        "math",
        "client_world_entity_firework_tick",
        json!({
            "entities": [
                {
                    "entity_id": 37,
                    "position": vector_bits(2.0, 10.0, 2.0),
                    "movement": vector_bits(0.25, 0.5, 0.125),
                    "rotation": {"y": f32_bits(355.0), "x": f32_bits(-20.0)},
                    "no_gravity": false,
                },
            ],
            "operations": [
                {"action": "tick", "label": "first-air-tick", "ticks": 1},
                {"action": "tick", "label": "second-air-tick", "ticks": 1},
            ],
        }),
    ));
    out.push(case(
        "client-world/entity-boat-air-tick",
        "math",
        "client_world_entity_boat_air_tick",
        json!({
            "player_id": 38,
            "boat": {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.25, 0.5, -0.75),
                "rotation": {"y": f32_bits(10.0), "x": f32_bits(-20.0)},
                "no_gravity": false,
            },
            "input": {"left": false, "right": true, "forward": true, "backward": false},
            "operations": [
                {"action": "tick", "label": "air-gravity-control", "ticks": 1},
                {"action": "set_no_gravity", "label": "enable-no-gravity", "value": true},
                {"action": "tick", "label": "air-no-gravity-control", "ticks": 1},
            ],
        }),
    ));
    out
}

fn entity_boat_remote_tick() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-boat-remote-tick",
        "math",
        "client_world_entity_boat_remote_tick",
        json!({
            "passenger": {"entity_id": 38, "type_id": 151},
            "boat": {
                "entity_id": 37,
                "position": vector_bits(2.0, 12.0, 2.0),
                "movement": vector_bits(0.25, 0.5, -0.75),
                "rotation": {"y": f32_bits(10.0), "x": f32_bits(-20.0)},
                "no_gravity": false,
            },
            "input": {"left": false, "right": true, "forward": true, "backward": false},
            "operations": [
                {"action": "tick", "label": "remote-first-tick", "ticks": 1},
                {"action": "tick", "label": "remote-second-tick", "ticks": 1},
            ],
        }),
    ));
    out.push(case(
        "client-world/entity-ticking-eligibility",
        "math",
        "client_world_entity_ticking_eligibility",
        json!({
            "player": {"entity_id": 42, "position": vector_bits(10.0, 64.0, -2.0)},
            "remote": {"entity_id": 37, "type_id": 71, "position": vector_bits(1.25, 80.0, -35.5)},
            "operations": [
                {"action": "tick", "label": "loaded-first-tick", "ticks": 1},
                {"action": "unload_chunk", "label": "remote-chunk-unloaded", "chunk_x": 0, "chunk_z": -3},
                {"action": "tick", "label": "unloaded-tick", "ticks": 1},
                {"action": "load_chunk", "label": "remote-chunk-reloaded", "chunk_x": 0, "chunk_z": -3},
                {"action": "tick", "label": "reloaded-tick", "ticks": 1},
            ],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        entity_snowball_tick(),
        entity_arrow_tick(),
        entity_experience_orb_tick(),
        entity_primed_tnt_tick(),
        entity_falling_block_tick(),
        entity_firework_tick(),
        entity_boat_remote_tick(),
    ]
    .concat()
}
