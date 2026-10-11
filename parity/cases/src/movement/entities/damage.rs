//! A living entity hurt and killed by the packets that do it, ticked on.

use crate::movement::entities::prelude::*;

fn damage_1() -> Vec<Value> {
    vec![
        living("quiet", "pig", 6, vec![]).row(),
        living("damage", "pig", 16, vec![json!({"tick": 2, "type": "damage"})]).row(),
        living("hurt-animation", "cow", 16, vec![json!({"tick": 2, "type": "hurt"})]).row(),
        living("hurt-twice", "pig", 24, vec![json!({"tick": 1, "type": "damage"}), json!({"tick": 6, "type": "hurt"})])
            .row(),
        living("death", "pig", 30, vec![json!({"tick": 2, "type": "death"})]).row(),
        living(
            "damage-then-death",
            "zombie",
            36,
            vec![json!({"tick": 2, "type": "damage"}), json!({"tick": 3, "type": "death"})],
        )
        .row(),
        living(
            "death-then-damage",
            "cow",
            30,
            vec![json!({"tick": 2, "type": "death"}), json!({"tick": 5, "type": "damage"})],
        )
        .row(),
        living(
            "damage-while-walking",
            "pig",
            20,
            vec![
                json!({"tick": 1, "type": "move", "xa": 800, "ya": 0, "za": 0}),
                json!({"tick": 3, "type": "damage"}),
                json!({"tick": 4, "type": "move", "xa": 800, "ya": 0, "za": 0}),
            ],
        )
        .row(),
    ]
}

fn damage_2() -> Vec<Value> {
    vec![
        living("spider-dies", "spider", 30, vec![json!({"tick": 1, "type": "death"})]).row(),
        living("death-with-the-player-far-off", "pig", 30, vec![json!({"tick": 2, "type": "death"})])
            .player_x(200.5)
            .row(),
        living("death-at-the-edge-of-simulation", "pig", 30, vec![json!({"tick": 2, "type": "death"})])
            .player_x(136.5)
            .row(),
        living("death-just-past-simulation", "pig", 30, vec![json!({"tick": 2, "type": "death"})])
            .player_x(144.5)
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [damage_1(), damage_2()].concat()
}
