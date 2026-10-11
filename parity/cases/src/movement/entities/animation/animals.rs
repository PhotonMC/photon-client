//! Entity animation scenarios: animals.

use crate::movement::entities::prelude::*;

fn animals_1() -> Vec<Value> {
    vec![
        living("zombie-swings", "zombie", 14, vec![animate(1, 0)]).extras(swing()).row(),
        living("zombie-swings-the-off-hand", "zombie", 14, vec![animate(1, 3)]).extras(swing()).row(),
        living("zombie-swings-again-too-early", "zombie", 14, vec![animate(1, 0), animate(2, 3)]).extras(swing()).row(),
        living("zombie-swings-again-halfway", "zombie", 16, vec![animate(1, 0), animate(5, 3)]).extras(swing()).row(),
        living("skeleton-swings", "skeleton", 14, vec![animate(2, 0)]).extras(swing()).row(),
        living("piglin-swings", "piglin", 14, vec![animate(2, 0)]).extras(swing()).row(),
        living("pig-is-told-to-swing", "pig", 14, vec![animate(1, 0)]).extras(swing()).row(),
        living("quiet-zombie", "zombie", 4, vec![]).extras(swing()).row(),
    ]
}

fn animals_2() -> Vec<Value> {
    vec![
        living("sheep-quiet", "sheep", 6, vec![]).extras(&["sheep"]).row(),
        living("sheep-eats", "sheep", 46, vec![json!({"tick": 1, "type": "entity_event", "event": 10})])
            .extras(&["sheep"])
            .row(),
        living(
            "sheep-eats-twice",
            "sheep",
            60,
            vec![
                json!({"tick": 1, "type": "entity_event", "event": 10}),
                json!({"tick": 30, "type": "entity_event", "event": 10}),
            ],
        )
        .extras(&["sheep"])
        .row(),
        living("creeper-quiet", "creeper", 6, vec![]).extras(&["creeper"]).row(),
        living("creeper-fuse", "creeper", 40, vec![json!({"tick": 2, "type": "creeper_data", "swell_dir": 1})])
            .extras(&["creeper"])
            .row(),
        living(
            "creeper-fuse-put-out",
            "creeper",
            40,
            vec![
                json!({"tick": 2, "type": "creeper_data", "swell_dir": 1}),
                json!({"tick": 14, "type": "creeper_data", "swell_dir": -1}),
            ],
        )
        .extras(&["creeper"])
        .row(),
        living("creeper-ignited", "creeper", 40, vec![json!({"tick": 3, "type": "creeper_data", "ignited": true})])
            .extras(&["creeper"])
            .row(),
        living(
            "creeper-dies-while-the-fuse-burns",
            "creeper",
            30,
            vec![json!({"tick": 2, "type": "creeper_data", "swell_dir": 1}), json!({"tick": 12, "type": "death"})],
        )
        .extras(&["creeper"])
        .row(),
    ]
}

fn animals_3() -> Vec<Value> {
    vec![
        living("attributes-pig", "pig", 3, vec![]).extras(&["attributes"]).row(),
        living("attributes-iron-golem", "iron_golem", 3, vec![]).extras(&["attributes"]).row(),
        living("attributes-changed", "iron_golem", 6, vec![json!({"tick": 1, "type": "attributes", "attributes": [{"name": "minecraft:max_health", "base": f64_bits(40.0), "modifiers": [{"id": "minecraft:test_add", "amount": f64_bits(5.0), "operation": 0}, {"id": "minecraft:test_total", "amount": f64_bits(0.5), "operation": 2}]}, {"name": "minecraft:scale", "base": f64_bits(2.0), "modifiers": [{"id": "minecraft:test_base", "amount": f64_bits(0.25), "operation": 1}]}]})]).extras(&["attributes"]).row(),
        living("attributes-out-of-range", "cow", 6, vec![json!({"tick": 1, "type": "attributes", "attributes": [{"name": "minecraft:max_health", "base": f64_bits(5000.0), "modifiers": []}, {"name": "minecraft:scale", "base": f64_bits(0.01), "modifiers": []}]})]).extras(&["attributes"]).row(),
        living("attributes-one-the-type-lacks", "pig", 6, vec![json!({"tick": 1, "type": "attributes", "attributes": [{"name": "minecraft:flying_speed", "base": f64_bits(3.0), "modifiers": []}, {"name": "minecraft:max_health", "base": f64_bits(12.0), "modifiers": []}]})]).extras(&["attributes"]).row(),
        living("camel-quiet", "camel", 100, vec![json!({"tick": 0, "type": "game_time", "time": 1000}), json!({"tick": 0, "type": "seed", "seed": 5})]).extras(&["camel"]).row(),
        living("camel-sits", "camel", 110, vec![json!({"tick": 0, "type": "game_time", "time": 1000}), json!({"tick": 0, "type": "seed", "seed": 5}), json!({"tick": 3, "type": "camel_data", "last_pose": -990, "pose": "SITTING"}), json!({"tick": 50, "type": "game_time", "time": 1050})]).extras(&["camel"]).row(),
        living("camel-stands-up", "camel", 110, vec![json!({"tick": 0, "type": "game_time", "time": 1100}), json!({"tick": 0, "type": "seed", "seed": 5}), json!({"tick": 0, "type": "camel_data", "last_pose": -1000, "pose": "SITTING"}), json!({"tick": 60, "type": "game_time", "time": 1160}), json!({"tick": 60, "type": "camel_data", "last_pose": 1160, "pose": "STANDING"})]).extras(&["camel"]).row(),
    ]
}

fn animals_4() -> Vec<Value> {
    vec![
        living(
            "camel-dashes",
            "camel",
            40,
            vec![
                json!({"tick": 0, "type": "game_time", "time": 1000}),
                json!({"tick": 0, "type": "seed", "seed": 5}),
                json!({"tick": 2, "type": "camel_data", "dash": true}),
            ],
        )
        .extras(&["camel"])
        .row(),
        living(
            "camel-dashes-until-it-lands",
            "camel",
            40,
            vec![
                json!({"tick": 0, "type": "game_time", "time": 1000}),
                json!({"tick": 0, "type": "seed", "seed": 5}),
                json!({"tick": 2, "type": "camel_data", "dash": true}),
                json!({"tick": 10, "type": "move", "xa": 0, "ya": 0, "za": 0, "on_ground": true}),
                json!({"tick": 20, "type": "camel_data", "dash": true}),
            ],
        )
        .extras(&["camel"])
        .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [animals_1(), animals_2(), animals_3(), animals_4()].concat()
}
