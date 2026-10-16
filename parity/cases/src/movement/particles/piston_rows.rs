//! The particle engine's rows of pistons: what a block event does to the blocks around a piston.

use serde_json::{Value, json};

use super::engine::*;
use super::events::*;
use crate::movement::world::put_with;

/// A box of the level the row compares block by block.
type Region = [i64; 6];

/// A piston, sticky or not, of a facing, at a place.
fn piston(at: (i64, i64, i64), facing: &str, (sticky, extended): (bool, bool)) -> Value {
    let block = if sticky { "minecraft:sticky_piston" } else { "minecraft:piston" };
    let extended = if extended { "true" } else { "false" };
    put_with(at.0, at.1, at.2, block, &[("facing", facing), ("extended", extended)])
}

/// A piston's head of a facing, at a place.
fn head(at: (i64, i64, i64), facing: &str, sticky: bool) -> Value {
    let kind = if sticky { "sticky" } else { "normal" };
    put_with(at.0, at.1, at.2, "minecraft:piston_head", &[("facing", facing), ("type", kind), ("short", "false")])
}

/// A block with no properties, at a place.
fn block(at: (i64, i64, i64), name: &str) -> Value {
    put_with(at.0, at.1, at.2, name, &[])
}

/// A row of pistons: the blocks, the events and the box that is compared. A smoke event after the
/// pistons' draws from the level's random source, which a missing draw of the pistons' would move.
fn scene(label: &str, (blocks, mut events): (Vec<Value>, Vec<Value>), region: Region, ticks: i64) -> Value {
    events.push(level_event(ticks - 1, 2000, (5, 64, 5), 0));
    engine_row(label, events).blocks(blocks).set("region", json!(region)).ticks(ticks).seeds(600).describe(8).row()
}

/// Pistons pushing and pulling along x: a line of stone, the limit of twelve, obsidian, slime and
/// honey holding on to what is beside them, and sticky pistons that draw back or let go.
fn along_x() -> Value {
    let stone = |x: i64, z: i64| block((x, 64, z), "minecraft:stone");
    let line = |z: i64, count: i64| -> Vec<Value> { (1..=count).map(|x| stone(x, z)).collect() };
    let rows: Vec<Vec<Value>> = vec![
        [vec![piston((0, 64, 0), "east", (false, false))], line(0, 1)].concat(),
        [vec![piston((0, 64, 4), "east", (false, false))], line(4, 12)].concat(),
        [vec![piston((0, 64, 8), "east", (false, false))], line(8, 13)].concat(),
        vec![piston((0, 64, 12), "east", (false, false)), block((1, 64, 12), "minecraft:obsidian")],
        vec![piston((0, 64, 16), "east", (false, false)), stone(2, 16)],
        vec![
            piston((0, 64, 24), "east", (false, false)),
            block((1, 64, 24), "minecraft:slime_block"),
            stone(1, 25),
            block((1, 64, 23), "minecraft:slime_block"),
            block((1, 64, 22), "minecraft:honey_block"),
            stone(2, 23),
        ],
        vec![piston((0, 64, 28), "east", (true, true)), head((1, 64, 28), "east", true), stone(2, 28)],
        vec![piston((0, 64, 32), "east", (true, true)), head((1, 64, 32), "east", true), stone(2, 32)],
        vec![piston((0, 64, 36), "east", (false, true)), head((1, 64, 36), "east", false), stone(2, 36)],
    ];
    let events = vec![
        block_event(0, (0, 64, 0), 0, 5),
        block_event(0, (0, 64, 4), 0, 5),
        block_event(0, (0, 64, 8), 0, 5),
        block_event(0, (0, 64, 12), 0, 5),
        block_event(0, (0, 64, 16), 0, 5),
        block_event(0, (0, 64, 24), 0, 5),
        block_event(0, (0, 64, 28), 1, 5),
        block_event(0, (0, 64, 32), 2, 5),
        block_event(0, (0, 64, 36), 1, 5),
    ];
    scene("pistons-push-and-pull", (rows.concat(), events), [-1, 63, -1, 16, 66, 40], 3)
}

/// Pistons up and down, and at the ends of the level, where nothing is pushed out of it.
fn along_y() -> Value {
    let blocks = vec![
        piston((0, 64, 0), "up", (false, false)),
        block((0, 65, 0), "minecraft:stone"),
        block((0, 66, 0), "minecraft:stone"),
        piston((0, 67, 4), "down", (false, false)),
        block((0, 66, 4), "minecraft:stone"),
        block((0, 65, 4), "minecraft:stone"),
        piston((0, 64, 8), "up", (true, true)),
        head((0, 65, 8), "up", true),
        block((0, 66, 8), "minecraft:stone"),
        piston((0, 318, 12), "up", (false, false)),
        block((0, 319, 12), "minecraft:stone"),
        piston((0, -63, 16), "down", (false, false)),
        block((0, -64, 16), "minecraft:stone"),
    ];
    let events = vec![
        block_event(0, (0, 64, 0), 0, 1),
        block_event(0, (0, 67, 4), 0, 0),
        block_event(0, (0, 64, 8), 1, 1),
        block_event(0, (0, 318, 12), 0, 1),
        block_event(0, (0, -63, 16), 0, 0),
    ];
    scene("pistons-up-and-down", (blocks, events), [-1, -65, -1, 3, 320, 18], 3)
}

/// A piston that is sent a second event before the first is done: its arm and what it pushed are
/// still on their way, and settle at once.
fn quick_pulses() -> Value {
    let rows: Vec<Vec<Value>> = vec![
        vec![piston((0, 64, 0), "east", (true, false)), block((1, 64, 0), "minecraft:stone")],
        vec![piston((0, 64, 4), "east", (true, false)), block((1, 64, 4), "minecraft:stone")],
        vec![piston((0, 64, 8), "east", (false, false)), block((1, 64, 8), "minecraft:stone")],
        vec![
            piston((0, 64, 12), "east", (true, false)),
            block((1, 64, 12), "minecraft:stone"),
            block((2, 64, 12), "minecraft:stone"),
        ],
    ];
    let events = vec![
        block_event(0, (0, 64, 0), 0, 5),
        block_event(1, (0, 64, 0), 1, 5),
        block_event(0, (0, 64, 4), 0, 5),
        block_event(1, (0, 64, 4), 2, 5),
        block_event(0, (0, 64, 8), 0, 5),
        block_event(1, (0, 64, 8), 1, 5),
        block_event(0, (0, 64, 12), 0, 5),
        block_event(1, (0, 64, 12), 1, 5),
    ];
    scene("pistons-quick-pulses", (rows.concat(), events), [-1, 63, -1, 6, 66, 14], 3)
}

pub fn rows() -> Vec<Value> {
    vec![along_x(), along_y(), quick_pulses()]
}
