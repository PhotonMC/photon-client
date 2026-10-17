//! The particle engine's rows of what the environment attributes say: the creaking heart that
//! speaks while the creaking is about (the overworld's clock), and the particles a biome lets
//! drift.

use serde_json::{Value, json};

use super::engine::*;
use super::events::*;
use crate::movement::world::{put, put_with};

fn clock(tick: i64, ticks: i64) -> Value {
    json!({"tick": tick, "type": "clock", "ticks": ticks})
}

/// A block of the lattice: a heart where the coordinates add up to an even number, in a state
/// that depends on where it is, and a log where they do not: the ones the tag lists, and now and
/// again a block that is no such log.
fn lattice_block(x: i64, y: i64, z: i64) -> Value {
    if (x + y + z).rem_euclid(2) == 0 {
        let state = ["awake", "dormant", "uprooted"][(x * 3 + y * 5 + z * 7).rem_euclid(3) as usize];
        return put_with(x, y, z, "minecraft:creaking_heart", &[("axis", "y"), ("creaking_heart_state", state)]);
    }
    let log = match (x * 11 + y * 13 + z * 17).rem_euclid(9) {
        0 => "minecraft:oak_log",
        1 => "minecraft:pale_oak_wood",
        2 => "minecraft:stripped_pale_oak_log",
        3 => "minecraft:stripped_pale_oak_wood",
        _ => "minecraft:pale_oak_log",
    };
    put(x, y, z, log)
}

/// A lattice of creaking hearts and pale oak logs: each heart inside is held by logs on all six
/// sides, the ones on its edge are not.
fn heart_lattice() -> Vec<Value> {
    let mut blocks = Vec::new();
    for x in -4..=4_i64 {
        for y in 60..=68_i64 {
            blocks.extend((-4..=4_i64).map(|z| lattice_block(x, y, z)));
        }
    }
    blocks
}

/// The hearts picked at random over ticks that straddle the edges of the clock's flag (it is up
/// from 12600 to 23400 of the day), and a wrap of the day.
fn creaking_row() -> Value {
    let clocks = [12_599, 12_600, 18_000, 23_400, 23_401, 24_000 + 12_600, 48_000 + 23_401];
    let mut events = Vec::new();
    for (tick, ticks) in clocks.into_iter().enumerate() {
        let tick = tick as i64;
        events.push(clock(tick, ticks));
        events.push(animate_tick(tick, (0, 64, 0), 12000, 1100 + tick));
    }
    engine_row("animate-creaking-heart", events)
        .blocks(heart_lattice())
        .set("clock", json!(0))
        .ticks(clocks.len() as i64)
        .seeds(400)
        .describe(8)
        .row()
}

/// A floor of stone and one of grass, with posts above them, in a biome that lets particles
/// drift: they come from the air above, never from the full blocks.
fn drifting_row(label: &str, biome: &str, seed: i64) -> Value {
    let mut blocks = Vec::new();
    for x in -6..=6_i64 {
        for z in -6..=6_i64 {
            blocks.push(put(x, 63, z, "minecraft:stone"));
            if (x + z).rem_euclid(5) == 0 {
                blocks.push(put(x, 64, z, "minecraft:stone"));
            } else if (x * z).rem_euclid(7) == 3 {
                blocks.push(put_with(
                    x,
                    64,
                    z,
                    "minecraft:stone_slab",
                    &[("type", "bottom"), ("waterlogged", "false")],
                ));
            }
        }
    }
    engine_row(label, (0..6).map(|tick| animate_tick(tick, (0, 64, 0), 2500, seed + tick)).collect::<Vec<_>>())
        .blocks(blocks)
        .set("biome", json!(biome))
        .ticks(6)
        .seeds(20000)
        .describe(10)
        .row()
}

/// Firefly bushes over a floor, some in the open, some under a roof of stone and some under
/// leaves, some beside a torch: where the sky lets light in, where it does not, and where a block
/// lights it.
fn firefly_scene() -> Vec<Value> {
    let mut blocks = Vec::new();
    for x in -8..=8_i64 {
        for z in -8..=8_i64 {
            blocks.push(put(x, 63, z, "minecraft:stone"));
            if (x + z).rem_euclid(3) == 0 && (x * 5 + z * 3).rem_euclid(4) != 0 {
                blocks.push(put(x, 64, z, "minecraft:firefly_bush"));
            }
            if x < -2 && z < 3 {
                blocks.push(put(x, 70, z, "minecraft:stone"));
            } else if x > 2 && z > -3 {
                blocks.push(put_with(
                    x,
                    70,
                    z,
                    "minecraft:oak_leaves",
                    &[("persistent", "true"), ("distance", "1"), ("waterlogged", "false")],
                ));
            }
            if (x * 7 + z * 11).rem_euclid(13) == 0 {
                blocks.push(put_with(x, 65, z, "minecraft:torch", &[]));
            }
        }
    }
    blocks
}

/// The bushes picked at random over a day: noon, the evening the clock's flag comes up in, the
/// middle of the night and the morning it goes down.
fn firefly_row() -> Value {
    let clocks = [6000, 11_867, 12_000, 12_150, 12_300, 12_600, 13_000, 18_000, 22_000, 23_401, 36_000];
    let mut events = Vec::new();
    for (tick, ticks) in clocks.into_iter().enumerate() {
        let tick = tick as i64;
        events.push(clock(tick, ticks));
        events.push(animate_tick(tick, (0, 64, 0), 4000, 3100 + tick));
    }
    engine_row("animate-firefly-bush", events)
        .blocks(firefly_scene())
        .set("clock", json!(0))
        .set("light", json!(true))
        .ticks(clocks.len() as i64)
        .seeds(20000)
        .describe(8)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![
        creaking_row(),
        firefly_row(),
        drifting_row("animate-biome-crimson-forest", "minecraft:crimson_forest", 2100),
        drifting_row("animate-biome-warped-forest", "minecraft:warped_forest", 2200),
        drifting_row("animate-biome-soul-sand-valley", "minecraft:soul_sand_valley", 2300),
        drifting_row("animate-biome-basalt-deltas", "minecraft:basalt_deltas", 2400),
        drifting_row("animate-biome-plains", "minecraft:plains", 2500),
    ]
}
