//! A dropped item ticked in a scene (`ItemEntity.tick` on the client).

use crate::movement::entities::prelude::*;

/// A stone step at one side of the floor.
fn step() -> Vec<Value> {
    (2..9).flat_map(|x| (-8..9).map(move |z| put(x, 64, z, "minecraft:stone"))).collect()
}

/// A waterfall: water falling down from a source.
fn waterfall() -> Vec<Value> {
    cat![
        vec![level(0, 67, 0, 0, "minecraft:water")],
        (0..3).map(|y| level(0, 64 + y, 0, 8, "minecraft:water")).collect::<Vec<_>>()
    ]
}

fn item_1() -> Vec<Value> {
    vec![
        // A block broken a little above the floor: the drop's own random motion.
        row("block-drop", floor(), (0.5, 65.2, 0.5), (0.05, 0.2, -0.03), 80).row(),
        row("block-drop-other-way", floor(), (0.5, 65.2, 0.5), (-0.08, 0.2, 0.09), 80).row(),
        row("dropped-high", floor(), (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), 80).row(),
        row("dropped-high-sideways", floor(), (0.5, 70.0, 0.5), (0.2, 0.0, -0.15), 100).row(),
    ]
}

fn item_2() -> Vec<Value> {
    // Resting on the floor, still: one tick in four moves it, by the id.
    [20, 21, 22, 23, -3]
        .into_iter()
        .map(|entity_id| {
            row(format!("resting-id-{}", entity_id), floor(), (0.5, 64.0, 0.5), (0.0, 0.0, 0.0), 24)
                .on_ground(true)
                .entity_id(entity_id)
                .row()
        })
        .collect::<Vec<_>>()
}

fn item_3() -> Vec<Value> {
    vec![
        row("sliding-on-stone", floor(), (0.5, 64.0, 0.5), (0.2, 0.0, 0.0), 60).on_ground(true).row(),
        row("sliding-on-ice", floor_of("minecraft:ice"), (0.5, 64.0, 0.5), (0.2, 0.0, 0.1), 80).on_ground(true).row(),
        row("sliding-on-packed-ice", floor_of("minecraft:packed_ice"), (0.5, 64.0, 0.5), (0.2, 0.0, 0.0), 80)
            .on_ground(true)
            .row(),
        row("sliding-on-slime", floor_of("minecraft:slime_block"), (0.5, 64.0, 0.5), (0.2, 0.0, 0.0), 60)
            .on_ground(true)
            .row(),
        row("falling-onto-slime", floor_of("minecraft:slime_block"), (0.5, 68.0, 0.5), (0.0, 0.0, 0.0), 80).row(),
        row("sliding-on-soul-sand", floor_of("minecraft:soul_sand"), (0.5, 64.0, 0.5), (0.2, 0.0, 0.0), 60)
            .on_ground(true)
            .row(),
    ]
}

fn item_4() -> Vec<Value> {
    vec![
        row("falling-onto-soul-sand", floor_of("minecraft:soul_sand"), (0.5, 66.0, 0.5), (0.1, 0.0, 0.0), 60).row(),
        row("sliding-on-honey", floor_of("minecraft:honey_block"), (0.5, 64.0, 0.5), (0.2, 0.0, 0.0), 60)
            .on_ground(true)
            .row(),
        // Obstacles.
        row("into-a-wall", cat![floor(), wall()], (0.5, 64.0, 0.5), (0.4, 0.0, 0.0), 40).on_ground(true).row(),
        row("falling-along-a-wall", cat![floor(), wall()], (2.8, 68.0, 0.5), (0.3, 0.0, 0.0), 60).row(),
        row("into-a-ceiling", cat![floor(), ceiling()], (0.5, 64.0, 0.5), (0.0, 0.6, 0.0), 40).row(),
        row("no-step-up", cat![floor(), step()], (0.5, 64.0, 0.5), (0.3, 0.0, 0.0), 40).on_ground(true).row(),
    ]
}

fn item_5() -> Vec<Value> {
    vec![
        // The edge between two floors, where the block below is picked by
        // the supporting block, and not the item's own column.
        row("edge-of-ice", cat![floor(), floor_from("minecraft:ice", 1, 8)], (0.9, 64.0, 0.5), (0.0, 0.0, 0.0), 40)
            .on_ground(true)
            .row(),
        row(
            "landing-on-the-edge-of-ice",
            cat![floor(), floor_from("minecraft:ice", 1, 8)],
            (0.9, 66.0, 0.5),
            (0.05, 0.0, 0.0),
            60,
        )
        .row(),
        row(
            "sliding-off-ice-onto-stone",
            cat![floor(), floor_from("minecraft:ice", -8, 0)],
            (-0.5, 64.0, 0.5),
            (0.3, 0.0, 0.0),
            60,
        )
        .on_ground(true)
        .row(),
        // Shapes that are not whole blocks.
        row(
            "landing-on-a-slab",
            cat![vec![json!({"x": 0, "y": 64, "z": 0, "block": "minecraft:stone_slab"})], floor()],
            (0.5, 67.0, 0.5),
            (0.0, 0.0, 0.0),
            60,
        )
        .row(),
        row(
            "landing-on-a-fence",
            cat![vec![json!({"x": 0, "y": 64, "z": 0, "block": "minecraft:oak_fence"})], floor()],
            (0.5, 67.0, 0.5),
            (0.0, 0.0, 0.0),
            60,
        )
        .row(),
        row(
            "landing-on-a-carpet",
            cat![vec![json!({"x": 0, "y": 64, "z": 0, "block": "minecraft:white_carpet"})], floor()],
            (0.5, 67.0, 0.5),
            (0.0, 0.0, 0.0),
            60,
        )
        .row(),
    ]
}

fn item_6() -> Vec<Value> {
    vec![
        // A box the floor does not reach.
        row("falling-out-of-the-world", vec![], (0.5, 10.0, 0.5), (0.0, 0.0, 0.0), 30).row(),
        row(
            "falling-into-water",
            cat![floor(), pool("minecraft:water", 3, -3, 3)],
            (0.5, 70.0, 0.5),
            (0.0, 0.0, 0.0),
            120,
        )
        .row(),
        row(
            "falling-into-water-sideways",
            cat![floor(), pool("minecraft:water", 3, -3, 3)],
            (-2.5, 70.0, 0.5),
            (0.15, 0.0, 0.05),
            120,
        )
        .row(),
        row(
            "sunk-in-deep-water",
            cat![floor(), pool("minecraft:water", 5, -3, 3)],
            (0.5, 64.2, 0.5),
            (0.0, 0.0, 0.0),
            100,
        )
        .row(),
        row(
            "lying-in-shallow-water",
            cat![floor(), pool("minecraft:water", 1, -3, 3)],
            (0.5, 64.0, 0.5),
            (0.0, 0.0, 0.0),
            80,
        )
        .on_ground(true)
        .row(),
        row(
            "thrown-up-through-water",
            cat![floor(), pool("minecraft:water", 3, -3, 3)],
            (0.5, 64.5, 0.5),
            (0.0, 0.5, 0.0),
            100,
        )
        .row(),
    ]
}

fn item_7() -> Vec<Value> {
    vec![
        row(
            "water-over-ice",
            cat![floor_of("minecraft:ice"), pool("minecraft:water", 2, -3, 3)],
            (0.5, 66.0, 0.5),
            (0.1, 0.0, 0.0),
            80,
        )
        .row(),
        row(
            "water-over-slime",
            cat![floor_of("minecraft:slime_block"), pool("minecraft:water", 2, -3, 3)],
            (0.5, 66.0, 0.5),
            (0.0, 0.0, 0.0),
            80,
        )
        .row(),
        row(
            "water-surface-eighth",
            cat![floor(), pool_where("minecraft:water", 1, -3, 3, &[("level", "7")])],
            (0.5, 64.0, 0.5),
            (0.0, 0.0, 0.0),
            60,
        )
        .on_ground(true)
        .row(),
        row(
            "water-surface-sixth",
            cat![floor(), pool_where("minecraft:water", 1, -3, 3, &[("level", "6")])],
            (0.5, 64.0, 0.5),
            (0.0, 0.0, 0.0),
            60,
        )
        .on_ground(true)
        .row(),
        row("in-a-stream", cat![floor(), stream()], (1.5, 64.0, 0.5), (0.0, 0.0, 0.0), 100).on_ground(true).row(),
        row("dropped-into-a-stream", cat![floor(), stream()], (2.5, 67.0, 0.5), (0.0, 0.0, 0.0), 100).row(),
    ]
}

fn item_8() -> Vec<Value> {
    vec![
        row("at-the-stream-end", cat![floor(), stream()], (7.5, 64.0, 0.5), (0.0, 0.0, 0.0), 80).on_ground(true).row(),
        row("in-a-waterfall", cat![floor(), waterfall()], (0.5, 66.8, 0.5), (0.0, 0.0, 0.0), 80).row(),
        row(
            "falling-into-lava",
            cat![floor(), pool("minecraft:lava", 3, -3, 3)],
            (0.5, 70.0, 0.5),
            (0.0, 0.0, 0.0),
            120,
        )
        .row(),
        row(
            "lying-in-shallow-lava",
            cat![floor(), pool("minecraft:lava", 1, -3, 3)],
            (0.5, 64.0, 0.5),
            (0.0, 0.0, 0.0),
            80,
        )
        .on_ground(true)
        .row(),
        row(
            "lava-flow",
            cat![
                floor(),
                vec![level(0, 64, 0, 0, "minecraft:lava")],
                (1_i64..4).map(|x| level(x, 64, 0, x, "minecraft:lava")).collect::<Vec<_>>()
            ],
            (1.5, 64.0, 0.5),
            (0.0, 0.0, 0.0),
            80,
        )
        .on_ground(true)
        .row(),
        row(
            "between-water-and-lava",
            cat![floor(), pool("minecraft:water", 3, -3, 0), pool("minecraft:lava", 3, 1, 3)],
            (0.5, 66.0, 0.5),
            (0.0, 0.0, 0.0),
            100,
        )
        .row(),
    ]
}

fn item_9() -> Vec<Value> {
    vec![
        row(
            "into-a-wall-under-water",
            cat![floor(), pool("minecraft:water", 3, -3, 3), wall()],
            (0.5, 65.0, 0.5),
            (0.3, 0.0, 0.0),
            60,
        )
        .row(),
        row("moves-count-from-the-last-packet", floor(), (0.5, 66.0, 0.5), (0.1, 0.1, 0.0), 40)
            .events(vec![
                move_item(5, 400, 1200, 0),
                move_item(10, 800, -300, 40),
                move_ground(15, 0, 0, 0),
                move_ground(25, -200, 0, 100),
            ])
            .row(),
        row("sync-then-moves", floor(), (0.5, 66.0, 0.5), (0.1, 0.1, 0.0), 50)
            .events(vec![
                sync(8, (1.25, 64.0, 0.75), (0.0, 0.0, 0.0), true),
                move_ground(12, 100, 0, -100),
                move_ground(30, 0, 8, 0),
            ])
            .row(),
        row("motion-packets", floor(), (0.5, 65.0, 0.5), (0.0, 0.0, 0.0), 60)
            .on_ground(true)
            .events(vec![
                motion_event(3, (0.15, 0.3, 0.0)),
                move_item(10, 1000, 2000, 0),
                motion_event(20, (0.0, 0.2, 0.1)),
                sync(40, (3.0, 64.0, 0.5), (0.0, 0.0, 0.0), true),
            ])
            .row(),
        row("sync-while-resting", floor(), (0.5, 64.0, 0.5), (0.0, 0.0, 0.0), 40)
            .on_ground(true)
            .events(vec![
                sync(10, (0.5, 64.0, 0.5), (0.0, 0.0, 0.0), true),
                sync(30, (0.62, 64.0, 0.5), (0.0, 0.0, 0.0), true),
            ])
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [item_1(), item_2(), item_3(), item_4(), item_5(), item_6(), item_7(), item_8(), item_9()].concat()
}
