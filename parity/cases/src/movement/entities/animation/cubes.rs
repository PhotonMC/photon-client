//! Entity animation scenarios: cubes.

use crate::movement::entities::prelude::*;

fn cubes_1() -> Vec<Value> {
    vec![
        // A squid beats its tentacles, and turns with its motion while it is in water; out of it
        // the tentacles flap and it tips head down. Its motion fades (a fiftieth a tick, and
        // under three thousandths is dropped) unless a position interpolation runs.
        // A slime or a magma cube squashes as it lands and stretches as it leaves the ground, its
        // target fading faster for the slime; each time its size is said it turns its body to where
        // the head looks (the same size again too) and scales its box.
        living("slime-lands", "slime", 24, vec![on_ground(2, true)]).extras(&["cube"]).row(),
        living("slime-hops", "slime", 44, hops()).extras(&["cube"]).row(),
        living("magma-cube-hops", "magma_cube", 44, hops()).extras(&["cube"]).row(),
        living("slime-grows-and-turns", "slime", 30, vec![head_turn(2, 64), cube_size(4, 4), on_ground(6, true)])
            .extras(&["cube"])
            .row(),
        living(
            "slime-size-said-again",
            "slime",
            36,
            vec![
                head_turn(1, -50),
                cube_size(2, 4),
                head_turn(8, 30),
                cube_size(10, 4),
                head_turn(14, 100),
                cube_size(18, 2),
            ],
        )
        .extras(&["cube"])
        .row(),
        living(
            "magma-cube-size-three",
            "magma_cube",
            20,
            vec![head_turn(1, 90), cube_size(3, 3), on_ground(5, true), on_ground(12, false)],
        )
        .extras(&["cube"])
        .row(),
    ]
}

fn cubes_2() -> Vec<Value> {
    vec![
        // A shulker's lid moves a twentieth a tick towards the peek (hundredths) the data says, and
        // its body stays facing south wherever its head turns.
        living("shulker-opens", "shulker", 40, vec![shulker_data(2, json!({"peek": 100}))]).extras(&["shulker"]).row(),
        living("shulker-opens-part-way", "shulker", 30, vec![shulker_data(1, json!({"peek": 30}))])
            .extras(&["shulker"])
            .row(),
        living(
            "shulker-opens-and-closes",
            "shulker",
            70,
            vec![
                shulker_data(1, json!({"peek": 100})),
                shulker_data(26, json!({"peek": 0})),
                shulker_data(45, json!({"peek": 60})),
            ],
        )
        .extras(&["shulker"])
        .row(),
        living("shulker-on-a-wall-in-a-colour", "shulker", 12, vec![shulker_data(1, json!({"face": 4, "color": 14}))])
            .extras(&["shulker"])
            .row(),
        living("shulker-no-colour", "shulker", 6, vec![shulker_data(1, json!({"color": 16}))])
            .extras(&["shulker"])
            .row(),
        living(
            "shulker-looks-around",
            "shulker",
            30,
            vec![head_turn(2, 64), head_turn(9, -100), shulker_data(12, json!({"peek": 40}))],
        )
        .extras(&["shulker"])
        .row(),
    ]
}

fn cubes_3() -> Vec<Value> {
    vec![
        // A shulker is held to the middle of its block (a half block up is the next one), closes its
        // lid when it comes to another block and, if it is not riding, slides there from the old
        // one over six ticks; its box follows the lid and the side it clings to.
        shulker_at("shulker-held-to-its-block", 8, vec![], 3.3, 64.7, 5.8).row(),
        shulker_at("shulker-held-low", 8, vec![shulker_data(1, json!({"peek": 60}))], 3.9, 64.2, 5.1).row(),
        shulker_at(
            "shulker-on-the-ceiling",
            30,
            vec![shulker_data(1, json!({"face": 1, "peek": 100}))],
            3.5,
            64.0,
            5.5,
        )
        .row(),
        shulker_at("shulker-on-a-wall", 30, vec![shulker_data(1, json!({"face": 2, "peek": 70}))], 3.5, 64.0, 5.5)
            .row(),
        shulker_at(
            "shulker-slides-to-the-next-block",
            36,
            vec![shulker_data(1, json!({"peek": 100})), move_by(14, 12288, 0, 0)],
            3.5,
            64.0,
            5.5,
        )
        .row(),
        shulker_at(
            "shulker-shifts-within-its-block",
            24,
            vec![shulker_data(1, json!({"peek": 100})), move_by(10, 1200, 0, 900)],
            3.5,
            64.0,
            5.5,
        )
        .row(),
    ]
}

fn cubes_4() -> Vec<Value> {
    vec![
        shulker_at(
            "shulker-moves-again-while-sliding",
            40,
            vec![
                shulker_data(1, json!({"peek": 100})),
                move_by(10, 8192, 0, 0),
                move_by(15, 0, 4096, 0),
                move_by(18, -8192, 0, 0),
            ],
            3.5,
            64.0,
            5.5,
        )
        .row(),
        shulker_at(
            "shulker-synced-far-away",
            30,
            vec![shulker_data(1, json!({"peek": 50})), sync_to(8, 6.2, 66.9, 4.1)],
            3.5,
            64.0,
            5.5,
        )
        .row(),
        shulker_at(
            "shulker-rides-and-steps-off",
            40,
            vec![shulker_data(1, json!({"peek": 100})), passengers(6, 41, &[40]), passengers(24, 41, &[])],
            3.5,
            64.0,
            5.5,
        )
        .vehicle(true)
        .row(),
        // A sulfur cube squashes like the others and, once primed, runs its fuse down to zero and
        // stays there; its small body (a baby's) and a size said again.
        living("sulfur-cube-hops", "sulfur_cube", 44, hops()).extras(&["cube"]).row(),
        living(
            "sulfur-cube-primed",
            "sulfur_cube",
            40,
            vec![json!({"tick": 2, "type": "sulfur_fuse", "fuse": 12}), cube_size(3, 2)],
        )
        .extras(&["cube"])
        .row(),
        living(
            "sulfur-cube-small",
            "sulfur_cube",
            30,
            vec![json!({"tick": 1, "type": "baby_data", "baby": true}), cube_size(2, 1), on_ground(4, true)],
        )
        .extras(&["cube"])
        .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [cubes_1(), cubes_2(), cubes_3(), cubes_4()].concat()
}
