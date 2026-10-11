//! The particle engine of a level.

use serde_json::{Value, json};

use super::engine::*;
use super::events::*;
use super::layouts::*;
use crate::movement::world::cat;

fn fireworks_1() -> Vec<Value> {
    vec![
        // Fireworks (no trails: the sparks a trail leaves are not made with a seed to compare by): each
        // shape, the colours and fades, the starter's sounds near and far, the twinkle's wait.
        engine_row(
            "fireworks-small-ball",
            vec![fireworks(
                0,
                (0.0, 70.0, 0.0),
                vec![explosion("SMALL_BALL", vec![16711680], vec![], false)],
                (0.0, 0.0, 0.0),
            )],
        )
        .ticks(4)
        .seeds(120)
        .describe(8)
        .row(),
        engine_row(
            "fireworks-large-ball-fade",
            vec![fireworks(
                0,
                (0.0, 70.0, 0.0),
                vec![explosion("LARGE_BALL", vec![2985983, 16764928], vec![65280], false)],
                (0.0, 0.0, 0.0),
            )],
        )
        .ticks(4)
        .seeds(400)
        .describe(8)
        .row(),
        engine_row(
            "fireworks-star",
            vec![fireworks(
                0,
                (2.0, 70.0, 0.0),
                vec![explosion("STAR", vec![16777215, 8947848], vec![255], false)],
                (0.0, 0.0, 0.0),
            )],
        )
        .ticks(4)
        .seeds(140)
        .describe(8)
        .row(),
        engine_row(
            "fireworks-creeper",
            vec![fireworks(
                0,
                (0.0, 70.0, 3.0),
                vec![explosion("CREEPER", vec![43520], vec![17408], false)],
                (0.0, 0.0, 0.0),
            )],
        )
        .ticks(4)
        .seeds(280)
        .describe(8)
        .row(),
    ]
}

fn fireworks_2() -> Vec<Value> {
    vec![
        engine_row(
            "fireworks-burst",
            vec![fireworks(
                0,
                (0.0, 70.0, 0.0),
                vec![explosion("BURST", vec![16746496], vec![], false)],
                (0.1, 0.3, -0.05),
            )],
        )
        .ticks(4)
        .seeds(90)
        .describe(8)
        .row(),
        engine_row(
            "fireworks-no-colour",
            vec![fireworks(0, (0.0, 70.0, 0.0), vec![explosion("SMALL_BALL", vec![], vec![], false)], (0.0, 0.0, 0.0))],
        )
        .ticks(3)
        .seeds(110)
        .describe(4)
        .row(),
        engine_row(
            "fireworks-two-explosions",
            vec![fireworks(
                0,
                (0.0, 70.0, 0.0),
                vec![
                    explosion("SMALL_BALL", vec![16711680], vec![], false),
                    explosion("STAR", vec![65280], vec![], false),
                ],
                (0.0, 0.0, 0.0),
            )],
        )
        .ticks(6)
        .seeds(260)
        .describe(6)
        .row(),
        engine_row(
            "fireworks-three-large-blast",
            vec![fireworks(
                0,
                (0.0, 70.0, 0.0),
                vec![
                    explosion("SMALL_BALL", vec![1], vec![], false),
                    explosion("SMALL_BALL", vec![2], vec![], false),
                    explosion("SMALL_BALL", vec![3], vec![], false),
                ],
                (0.0, 0.0, 0.0),
            )],
        )
        .ticks(8)
        .seeds(330)
        .describe(4)
        .row(),
    ]
}

fn fireworks_3() -> Vec<Value> {
    vec![
        engine_row(
            "fireworks-twinkle-waits",
            vec![fireworks(
                0,
                (0.0, 70.0, 0.0),
                vec![explosion("SMALL_BALL", vec![16711935], vec![], true)],
                (0.0, 0.0, 0.0),
            )],
        )
        .ticks(24)
        .seeds(120)
        .describe(6)
        .row(),
        engine_row(
            "fireworks-far",
            vec![
                fireworks(
                    0,
                    (60.0, 70.0, 0.0),
                    vec![explosion("LARGE_BALL", vec![16777215], vec![], false)],
                    (0.0, 0.0, 0.0),
                ),
                fireworks(
                    1,
                    (0.0, 64.0, 15.0),
                    vec![explosion("SMALL_BALL", vec![1193046], vec![], false)],
                    (0.0, 0.0, 0.0),
                ),
            ],
        )
        .ticks(5)
        .seeds(520)
        .describe(4)
        .row(),
        engine_row(
            "fireworks-no-explosion",
            vec![
                fireworks(0, (0.0, 70.0, 0.0), vec![], (0.0, 0.0, 0.0)),
                fireworks(1, (1.0, 70.0, 1.0), vec![], (0.0, 0.0, 0.0)),
            ],
        )
        .ticks(3)
        .seeds(30)
        .describe(6)
        .row(),
        // The last level events that need no block entity: the dragon's breath (with and without its
        // fireball), the gateway and the growl, the sparks on a rod's axis and on the faces, wax and
        // scrapes, sculk charging (popped in a full and in a thin block, creeping over the faces of a
        // block and of a vein), the shrieker (dry and under water), an egg, a cobweb.
        engine_row(
            "level-event-dragon-breath",
            vec![level_event(0, 2006, (1, 64, 1), 0), level_event(1, 2006, (-2, 64, 2), 1)],
        )
        .ticks(5)
        .seeds(420)
        .describe(10)
        .row(),
    ]
}

fn fireworks_4() -> Vec<Value> {
    vec![
        engine_row(
            "level-event-end-sounds",
            vec![level_event(0, 3000, (2, 64, 2), 0), level_event(0, 3001, (-2, 64, -2), 0)],
        )
        .ticks(10)
        .seeds(60)
        .describe(4)
        .row(),
        engine_row(
            "level-event-block-faces",
            vec![
                level_event(0, 3002, (1, 64, 1), -1),
                level_event(0, 3003, (2, 64, 2), 0),
                level_event(1, 3004, (3, 64, 3), 0),
                level_event(1, 3005, (-1, 64, 3), 0),
                level_event(2, 3009, (-3, 64, 1), 0),
            ],
        )
        .ticks(5)
        .seeds(300)
        .describe(12)
        .row(),
        engine_row(
            "level-event-rod-sparks",
            vec![
                level_event(0, 3002, (1, 64, 1), 0),
                level_event(0, 3002, (2, 64, 2), 1),
                level_event(1, 3002, (-2, 64, 1), 2),
            ],
        )
        .ticks(4)
        .seeds(90)
        .describe(8)
        .row(),
        engine_row(
            "level-event-sculk",
            vec![
                level_event(0, 3006, (1, 64, 1), 0),
                level_event(0, 3006, (-1, 64, 1), 0),
                level_event(1, 3006, (3, 64, 3), 128),
                level_event(1, 3006, (-3, 64, 3), 197),
                level_event(2, 3006, (3, 64, -3), 127),
            ],
        )
        .blocks(cat![
            plane(63, "minecraft:stone", None, 8),
            vec![
                json!({"x": 1, "y": 64, "z": 1, "block": "minecraft:stone"}),
                json!({"x": -1, "y": 64, "z": 1, "block": "minecraft:stone_slab", "properties": {"type": "bottom"}})
            ]
        ])
        .ticks(5)
        .seeds(400)
        .describe(12)
        .row(),
    ]
}

fn fireworks_5() -> Vec<Value> {
    vec![
        engine_row("level-event-shriek-and-web", vec![level_event(0, 3007, (1, 64, 1), 0), level_event(0, 3007, (-1, 64, 1), 0), level_event(1, 3018, (2, 64, 2), 0)]).blocks(vec![json!({"x": 1, "y": 64, "z": 1, "block": "minecraft:sculk_shrieker", "properties": {"waterlogged": "false"}}), json!({"x": -1, "y": 64, "z": 1, "block": "minecraft:sculk_shrieker", "properties": {"waterlogged": "true"}})]).ticks(5).seeds(40).describe(12).row(),
        engine_row("level-event-trial-spawner", vec![level_event(0, 3011, (1, 64, 1), 0), level_event(0, 3012, (-1, 64, 1), 1), level_event(1, 3013, (2, 64, 2), 3), level_event(1, 3014, (-2, 64, 2), 0), level_event(2, 3017, (3, 64, 3), 0), level_event(2, 3019, (-3, 64, 3), 12), level_event(3, 3020, (2, 64, -2), 0), level_event(3, 3020, (-2, 64, -2), 1), level_event(4, 3021, (3, 64, -3), 0), level_event(4, 3011, (4, 64, 4), 7)]).ticks(8).seeds(1200).describe(14).row(),
        engine_row("global-level-events", vec![global_level_event(0, 1023, (50, 70, 0), 0), global_level_event(0, 1028, (0, 64, -80), 0), global_level_event(1, 1038, (-30, 100, 30), 0), global_level_event(1, 1000, (5, 5, 5), 0)]).ticks(3).seeds(2).describe(2).row(),
        // Past the reservoir's start a particle gets in by a falling chance, and a full group lets none in.
        engine_row("reservoir", vec![packet(0, "smoke", (0.0, 64.0, 0.0), (3.0, 1.0, 3.0), 0.02, 14000), packet(1, "smoke", (0.0, 64.0, 0.0), (3.0, 1.0, 3.0), 0.02, 14000)]).ticks(3).seeds(28000).describe(3).row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [fireworks_1(), fireworks_2(), fireworks_3(), fireworks_4(), fireworks_5()].concat()
}
