//! Entity animation scenarios: animals.

use crate::movement::entities::prelude::*;

fn farm_1() -> Vec<Value> {
    vec![
        living("horse-quiet", "horse", 8, vec![json!({"tick": 0, "type": "seed", "seed": 11})])
            .extras(&["horse"])
            .row(),
        living(
            "horse-grazes",
            "horse",
            30,
            vec![json!({"tick": 0, "type": "seed", "seed": 11}), json!({"tick": 2, "type": "horse_data", "flags": 16})],
        )
        .extras(&["horse"])
        .row(),
        living(
            "horse-grazes-then-stops",
            "horse",
            30,
            vec![
                json!({"tick": 0, "type": "seed", "seed": 11}),
                json!({"tick": 2, "type": "horse_data", "flags": 16}),
                json!({"tick": 12, "type": "horse_data", "flags": 0}),
            ],
        )
        .extras(&["horse"])
        .row(),
        living(
            "horse-rears",
            "horse",
            30,
            vec![json!({"tick": 0, "type": "seed", "seed": 11}), json!({"tick": 2, "type": "horse_data", "flags": 32})],
        )
        .extras(&["horse"])
        .row(),
        living(
            "horse-rears-while-grazing",
            "donkey",
            30,
            vec![
                json!({"tick": 0, "type": "seed", "seed": 11}),
                json!({"tick": 2, "type": "horse_data", "flags": 16}),
                json!({"tick": 8, "type": "horse_data", "flags": 48}),
            ],
        )
        .extras(&["horse"])
        .row(),
    ]
}

fn farm_2() -> Vec<Value> {
    vec![
        living(
            "horse-opens-its-mouth",
            "horse",
            50,
            vec![json!({"tick": 0, "type": "seed", "seed": 11}), json!({"tick": 2, "type": "horse_data", "flags": 64})],
        )
        .extras(&["horse"])
        .row(),
        living("horse-swishes-its-tail", "horse", 600, vec![json!({"tick": 0, "type": "seed", "seed": 3})])
            .extras(&["horse"])
            .row(),
        living(
            "copper-golem-gets-an-item",
            "copper_golem",
            40,
            vec![
                json!({"tick": 0, "type": "copper_golem_idle", "seed": 5}),
                json!({"tick": 6, "type": "copper_golem_data", "state": 1}),
            ],
        )
        .extras(&["copper_golem"])
        .row(),
        living(
            "copper-golem-gets-nothing",
            "copper_golem",
            40,
            vec![json!({"tick": 6, "type": "copper_golem_data", "state": 2})],
        )
        .extras(&["copper_golem"])
        .row(),
        living(
            "copper-golem-drops-an-item",
            "copper_golem",
            40,
            vec![json!({"tick": 6, "type": "copper_golem_data", "state": 3})],
        )
        .extras(&["copper_golem"])
        .row(),
    ]
}

fn farm_3() -> Vec<Value> {
    vec![
        living(
            "copper-golem-drops-nothing",
            "copper_golem",
            40,
            vec![json!({"tick": 6, "type": "copper_golem_data", "state": 4})],
        )
        .extras(&["copper_golem"])
        .row(),
        living(
            "copper-golem-switches-between-chest-actions",
            "copper_golem",
            60,
            vec![
                json!({"tick": 6, "type": "copper_golem_data", "state": 1}),
                json!({"tick": 20, "type": "copper_golem_data", "state": 3}),
                json!({"tick": 33, "type": "copper_golem_data", "state": 0}),
            ],
        )
        .extras(&["copper_golem"])
        .row(),
        living(
            "copper-golem-spins-again-after-a-chest",
            "copper_golem",
            330,
            vec![
                json!({"tick": 0, "type": "copper_golem_idle", "seed": 9}),
                json!({"tick": 6, "type": "copper_golem_data", "state": 2}),
                json!({"tick": 20, "type": "copper_golem_data", "state": 0}),
            ],
        )
        .extras(&["copper_golem"])
        .row(),
        living(
            "copper-golem-without-ai-does-not-animate",
            "copper_golem",
            300,
            vec![
                json!({"tick": 0, "type": "copper_golem_idle", "seed": 5}),
                json!({"tick": 0, "type": "mob_flags", "flags": 1}),
                json!({"tick": 6, "type": "copper_golem_data", "state": 1}),
            ],
        )
        .extras(&["copper_golem"])
        .row(),
        living(
            "copper-golem-weathers",
            "copper_golem",
            8,
            vec![json!({"tick": 2, "type": "copper_golem_data", "weather": 2})],
        )
        .extras(&["copper_golem"])
        .row(),
    ]
}

fn farm_4() -> Vec<Value> {
    vec![
        living(
            "copper-golem-oxidizes-while-working",
            "copper_golem",
            12,
            vec![json!({"tick": 2, "type": "copper_golem_data", "weather": 3, "state": 4})],
        )
        .extras(&["copper_golem"])
        .row(),
        living(
            "rabbit-quiet",
            "rabbit",
            40,
            vec![json!({"tick": 0, "type": "rabbit_idle", "timeout": 400, "seed": 5})],
        )
        .extras(&["rabbit"])
        .row(),
        living(
            "rabbit-hops",
            "rabbit",
            60,
            vec![
                json!({"tick": 0, "type": "rabbit_idle", "timeout": 400, "seed": 5}),
                json!({"tick": 3, "type": "entity_event", "event": 1}),
            ],
        )
        .extras(&["rabbit"])
        .row(),
        living(
            "rabbit-hops-twice",
            "rabbit",
            70,
            vec![
                json!({"tick": 0, "type": "rabbit_idle", "timeout": 400, "seed": 5}),
                json!({"tick": 3, "type": "entity_event", "event": 1}),
                json!({"tick": 11, "type": "entity_event", "event": 1}),
            ],
        )
        .extras(&["rabbit"])
        .row(),
        living(
            "rabbit-tilts-its-head",
            "rabbit",
            60,
            vec![json!({"tick": 0, "type": "rabbit_idle", "timeout": 12, "seed": 77})],
        )
        .extras(&["rabbit"])
        .row(),
    ]
}

fn farm_5() -> Vec<Value> {
    vec![
        living(
            "rabbit-tilts-then-hops",
            "rabbit",
            80,
            vec![
                json!({"tick": 0, "type": "rabbit_idle", "timeout": 12, "seed": 77}),
                json!({"tick": 20, "type": "entity_event", "event": 1}),
            ],
        )
        .extras(&["rabbit"])
        .row(),
        living("goat-quiet", "goat", 6, vec![]).extras(&["goat"]).row(),
        living(
            "goat-rams",
            "goat",
            40,
            vec![
                json!({"tick": 2, "type": "entity_event", "event": 58}),
                json!({"tick": 18, "type": "entity_event", "event": 59}),
            ],
        )
        .extras(&["goat"])
        .row(),
        living(
            "goat-rams-and-stops-early",
            "goat",
            30,
            vec![
                json!({"tick": 1, "type": "entity_event", "event": 58}),
                json!({"tick": 7, "type": "entity_event", "event": 59}),
            ],
        )
        .extras(&["goat"])
        .row(),
        living("iron-golem-quiet", "iron_golem", 6, vec![]).extras(&["iron_golem"]).row(),
    ]
}

fn farm_6() -> Vec<Value> {
    vec![
        living("iron-golem-attacks", "iron_golem", 20, vec![json!({"tick": 2, "type": "entity_event", "event": 4})])
            .extras(&["iron_golem"])
            .row(),
        living(
            "iron-golem-offers-a-flower",
            "iron_golem",
            30,
            vec![json!({"tick": 1, "type": "entity_event", "event": 11})],
        )
        .extras(&["iron_golem"])
        .row(),
        living(
            "iron-golem-offers-then-stops",
            "iron_golem",
            30,
            vec![
                json!({"tick": 1, "type": "entity_event", "event": 11}),
                json!({"tick": 12, "type": "entity_event", "event": 34}),
            ],
        )
        .extras(&["iron_golem"])
        .row(),
        living(
            "iron-golem-attacks-again",
            "iron_golem",
            30,
            vec![
                json!({"tick": 1, "type": "entity_event", "event": 4}),
                json!({"tick": 6, "type": "entity_event", "event": 4}),
            ],
        )
        .extras(&["iron_golem"])
        .row(),
        living("chicken-in-the-air", "chicken", 24, vec![]).extras(&["chicken"]).row(),
    ]
}

fn farm_7() -> Vec<Value> {
    vec![
        living("chicken-lands", "chicken", 24, vec![on_ground(6, true)]).extras(&["chicken"]).row(),
        living(
            "chicken-lands-and-takes-off",
            "chicken",
            40,
            vec![on_ground(3, true), on_ground(12, false), on_ground(20, true)],
        )
        .extras(&["chicken"])
        .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [farm_1(), farm_2(), farm_3(), farm_4(), farm_5(), farm_6(), farm_7()].concat()
}
