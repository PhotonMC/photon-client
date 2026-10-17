//! The particle engine's rows of a beacon's beam: the column above it worked out ten blocks a tick,
//! its colours blended, its stops, the pyramid under it counted once in four seconds, and the beam
//! shown once the scan has reached the top and the pyramid is whole in at least one layer.

use serde_json::{Value, json};

use super::engine::*;
use crate::movement::world::{put, put_with};

fn stained(color: &str, x: i64, y: i64) -> Value {
    put(x, y, 0, &format!("minecraft:{color}_stained_glass"))
}

/// The layers of iron under a beacon, the nearest layer first: `layers` of them, each a square
/// wider by a block on every side.
fn pyramid(x: i64, y: i64, layers: i64) -> Vec<Value> {
    let mut blocks = Vec::new();
    for step in 1..=layers {
        for dx in -step..=step {
            for dz in -step..=step {
                blocks.push(put(x + dx, y - step, dz, "minecraft:iron_block"));
            }
        }
    }
    blocks
}

/// The beacons of a row, each with a column over it to try one thing, and the places that set a
/// block partway through a scan.
fn columns(pyramids: bool) -> (Vec<Value>, Vec<Value>) {
    let mut blocks: Vec<Value> = Vec::new();
    let mut places = Vec::new();
    // The beacon, and the iron it stands on.
    let mut beacon = |x: i64, y: i64, layers: i64| {
        blocks.push(put(x, y - 1, 0, "minecraft:stone"));
        if pyramids {
            blocks.extend(pyramid(x, y, layers));
        }
        blocks.push(put(x, y, 0, "minecraft:beacon"));
        places.push(json!({"x": x, "y": y, "z": 0, "type": "minecraft:beacon"}));
    };
    beacon(-14, 64, 4);
    beacon(-10, 64, 4);
    beacon(-6, 64, 4);
    beacon(-2, 64, 2);
    beacon(2, -10, 1);
    beacon(6, 64, 3);
    beacon(10, 64, 4);
    beacon(14, 64, 0);
    beacon(18, 64, 4);
    // Red, red, blue, blue, white, then air.
    for (y, color) in [(65, "red"), (66, "red"), (67, "blue"), (68, "blue"), (69, "white")] {
        blocks.push(stained(color, -10, y));
    }
    // Capped by stone far over it.
    blocks.push(put(-6, 65, 0, "minecraft:cyan_stained_glass"));
    blocks.push(put(-6, 75, 0, "minecraft:stone"));
    // Bedrock, leaves (a block of light 1) and glass let the beam through.
    blocks.push(put(-2, 65, 0, "minecraft:bedrock"));
    let leaves = [("persistent", "true"), ("distance", "1"), ("waterlogged", "false")];
    blocks.push(put_with(-2, 66, 0, "minecraft:oak_leaves", &leaves));
    blocks.push(put(-2, 67, 0, "minecraft:glass"));
    blocks.push(put(-2, 68, 0, "minecraft:orange_stained_glass"));
    // Deep down, where the scan first looks a long way up.
    for y in -9..=-5 {
        blocks.push(stained(if y % 2 == 0 { "lime" } else { "pink" }, 2, y));
    }
    // A tall column that takes several ticks to scan.
    for y in 65..=94 {
        let color = ["red", "red", "yellow", "yellow", "yellow", "gray"][(y % 6) as usize];
        blocks.push(stained(color, 6, y));
    }
    // A column a block is set in as the scan is part of the way up.
    for y in 65..=80 {
        blocks.push(stained("magenta", 10, y));
    }
    // Panes, and a pane of another colour between two of one.
    for (y, color) in [(65, "purple"), (66, "purple"), (67, "green"), (68, "purple")] {
        blocks.push(put(14, y, 0, &format!("minecraft:{color}_stained_glass_pane")));
    }
    // A column that is solid at once.
    blocks.push(put(18, 65, 0, "minecraft:stone"));
    blocks.push(put(18, 66, 0, "minecraft:red_stained_glass"));
    (blocks, places)
}

fn beacon_row(label: &str, game_time: u64, pyramids: bool) -> Value {
    let (blocks, places) = columns(pyramids);
    let set = |tick: i64, x: i64, y: i64, block: &str| json!({"tick": tick, "type": "set_block", "x": x, "y": y, "z": 0, "state": {"block": block}});
    let events = vec![
        set(1, 10, 75, "minecraft:stone"),
        set(6, 6, 70, "minecraft:blue_stained_glass"),
        set(9, -10, 69, "minecraft:air"),
    ];
    let watch: Vec<Value> = places.iter().map(|place| json!({"x": place["x"], "y": place["y"], "z": 0})).collect();
    engine_row(label, events)
        .blocks(blocks)
        .set("block_entities", json!(places))
        .set("watch", json!(watch))
        .set("game_time", json!(game_time))
        .set("tick_block_entities", json!(true))
        .ticks(24)
        .seeds(8)
        .describe(0)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![
        beacon_row("beacon-beams", 160, true),
        beacon_row("beacon-beams-off-beat", 161, true),
        beacon_row("beacon-beams-no-base", 160, false),
    ]
}
