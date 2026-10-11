//! Entity animation scenarios: objects.

use crate::movement::entities::prelude::*;

fn objects_1() -> Vec<Value> {
    vec![
        // A boat's hit (its time and damage) runs down a tick at a time; a bubble column shakes it
        // (not under water) by a sine of its age, building up and dying away; whether it is under
        // water is whether the water reaches a thousandth over its box.
        fish("boat-floats", "oak_boat", 20, vec![]).tag("boat").row(),
        fish(
            "boat-is-hit",
            "oak_boat",
            30,
            vec![json!({"tick": 3, "type": "boat_data", "hurt": 10, "hurt_dir": -1, "damage": 25.0})],
        )
        .tag("boat")
        .row(),
        fish(
            "boat-shaken-by-bubbles",
            "oak_boat",
            40,
            vec![
                json!({"tick": 3, "type": "boat_data", "bubble_time": 60}),
                json!({"tick": 22, "type": "boat_data", "bubble_time": 0}),
            ],
        )
        .water("none")
        .tag("boat")
        .row(),
        fish("boat-under-water", "oak_chest_boat", 10, vec![]).tag("boat").row(),
        fish("boat-in-shallow-water", "birch_boat", 10, vec![]).water("shallow").y(63.5).tag("boat").row(),
        fish("boat-above-the-water", "bamboo_raft", 10, vec![]).water("shallow").y(64.6).tag("boat").row(),
    ]
}

fn objects_2() -> Vec<Value> {
    vec![
        fish("boat-on-dry-land", "spruce_boat", 10, vec![]).water("none").tag("boat").row(),
        // An arrow starts to shake when the server says it is in the ground (not on its first tick,
        // not while it shakes already), and the shake runs down by a tick; a potion's colour above
        // nothing makes it tipped.
        living("arrow-flies", "arrow", 12, vec![]).extras(&["arrow"]).player_x(30.5).row(),
        living("arrow-sticks", "arrow", 20, vec![json!({"tick": 3, "type": "arrow_data", "in_ground": true})])
            .extras(&["arrow"])
            .player_x(30.5)
            .row(),
        living(
            "arrow-sticks-and-sticks-again",
            "arrow",
            24,
            vec![
                json!({"tick": 3, "type": "arrow_data", "in_ground": true}),
                json!({"tick": 6, "type": "arrow_data", "in_ground": true}),
                json!({"tick": 14, "type": "arrow_data", "in_ground": false}),
                json!({"tick": 16, "type": "arrow_data", "in_ground": true}),
            ],
        )
        .extras(&["arrow"])
        .player_x(30.5)
        .row(),
        living(
            "arrow-sticks-from-the-start",
            "arrow",
            14,
            vec![json!({"tick": 0, "type": "arrow_data", "in_ground": true})],
        )
        .extras(&["arrow"])
        .player_x(30.5)
        .row(),
        living(
            "arrow-is-tipped",
            "arrow",
            8,
            vec![
                json!({"tick": 2, "type": "arrow_data", "color": 16253176}),
                json!({"tick": 5, "type": "arrow_data", "color": -1}),
            ],
        )
        .extras(&["arrow"])
        .player_x(30.5)
        .row(),
    ]
}

fn objects_3() -> Vec<Value> {
    vec![
        living(
            "spectral-arrow-sticks",
            "spectral_arrow",
            14,
            vec![json!({"tick": 3, "type": "arrow_data", "in_ground": true})],
        )
        .extras(&["arrow"])
        .player_x(30.5)
        .row(),
        // An end crystal holds the block its beam reaches and whether it shows its base; a dragon
        // looks for the nearest crystal whose box meets its own made 32 blocks wider (one tick in
        // ten, so long enough in these rows), and lets go of one that is gone or out of reach.
        living("crystal-defaults", "end_crystal", 6, vec![]).extras(&["end_crystal"]).player_x(30.5).row(),
        living(
            "crystal-beam-and-base",
            "end_crystal",
            10,
            vec![json!({"tick": 2, "type": "crystal_data", "target": [3, 70, -4], "bottom": false})],
        )
        .extras(&["end_crystal"])
        .player_x(30.5)
        .row(),
        living(
            "crystal-loses-its-beam",
            "end_crystal",
            10,
            vec![
                json!({"tick": 2, "type": "crystal_data", "target": [3, 70, -4]}),
                json!({"tick": 6, "type": "crystal_data", "no_target": true, "bottom": true}),
            ],
        )
        .extras(&["end_crystal"])
        .player_x(30.5)
        .row(),
        living(
            "dragon-finds-a-crystal",
            "ender_dragon",
            220,
            vec![json!({"tick": 2, "type": "dragon_data", "phase": 0})],
        )
        .extras(&["ender_dragon"])
        .player_x(30.5)
        .beside("end_crystal", 14.5, 64.0, 3.5)
        .row(),
        living(
            "dragon-does-not-find-a-far-crystal",
            "ender_dragon",
            220,
            vec![json!({"tick": 2, "type": "dragon_data", "phase": 0})],
        )
        .extras(&["ender_dragon"])
        .player_x(30.5)
        .beside("end_crystal", 90.5, 64.0, 3.5)
        .row(),
    ]
}

fn objects_4() -> Vec<Value> {
    vec![
        living("dragon-finds-a-crystal-just-in-reach", "ender_dragon", 220, vec![])
            .extras(&["ender_dragon"])
            .player_x(30.5)
            .beside("end_crystal", 39.9, 64.0, 0.5)
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [objects_1(), objects_2(), objects_3(), objects_4()].concat()
}
