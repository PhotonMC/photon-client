//! Entity animation scenarios: aquatic.

use crate::movement::entities::prelude::*;

fn aquatic_1() -> Vec<Value> {
    vec![
        squid("squid-at-rest-in-water", 40, vec![]).row(),
        squid("squid-glides-in-water", 70, vec![motion(3, 0.12, -0.04, 0.07), motion(30, -0.1, 0.06, 0.02)]).row(),
        squid("squid-fast-tentacles", 60, vec![motion(3, 0.0, 0.1, 0.0)]).speed(0.28).row(),
        squid(
            "squid-beat-restarts",
            60,
            vec![motion(3, 0.1, 0.0, 0.1), json!({"tick": 25, "type": "entity_event", "event": 19})],
        )
        .row(),
        squid(
            "squid-follows-its-moves-packets",
            50,
            vec![
                motion(2, 0.1, 0.0, 0.0),
                json!({"tick": 4, "type": "move", "xa": 400, "ya": 0, "za": 200}),
                json!({"tick": 9, "type": "move", "xa": 300, "ya": 100, "za": 0}),
            ],
        )
        .row(),
        squid("squid-in-shallow-water", 50, vec![motion(3, 0.1, -0.03, 0.0)]).water("shallow").row(),
        squid("squid-out-of-water", 60, vec![motion(3, 0.1, -0.3, 0.0)]).water("none").row(),
        squid("glow-squid-in-water", 40, vec![motion(3, 0.05, 0.05, -0.1)]).entity("glow_squid").row(),
    ]
}

fn aquatic_2() -> Vec<Value> {
    vec![
        squid(
            "glow-squid-dark-ticks-run-out",
            30,
            vec![json!({"tick": 2, "type": "glow_squid_data", "dark_ticks": 12})],
        )
        .entity("glow_squid")
        .row(),
        // A fish is in water when the water reaches its box, which a salmon's size and a
        // pufferfish's puff state change: the water here lifts only the top of the larger ones.
        fish("cod-in-water", "cod", 10, vec![]).row(),
        fish("cod-out-of-water", "cod", 10, vec![]).water("none").row(),
        fish("tropical-fish-in-water", "tropical_fish", 10, vec![]).row(),
        fish("salmon-default-size", "salmon", 10, vec![]).water("shallow").y(63.5).row(),
        fish("salmon-small", "salmon", 10, vec![fish_data(2, "salmon", 0)]).water("shallow").y(63.5).row(),
        fish("salmon-large", "salmon", 10, vec![fish_data(2, "salmon", 2)]).water("shallow").y(63.5).row(),
        fish("salmon-size-past-the-last", "salmon", 10, vec![fish_data(2, "salmon", 7)]).water("shallow").y(63.5).row(),
    ]
}

fn aquatic_3() -> Vec<Value> {
    vec![
        fish("pufferfish-small", "pufferfish", 10, vec![]).water("shallow").y(63.4).row(),
        fish("pufferfish-mid", "pufferfish", 10, vec![fish_data(2, "pufferfish", 1)]).water("shallow").y(63.4).row(),
        fish("pufferfish-full", "pufferfish", 10, vec![fish_data(2, "pufferfish", 2)]).water("shallow").y(63.4).row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [aquatic_1(), aquatic_2(), aquatic_3()].concat()
}
