//! The poplar leaves, which only 26.3 has: the leaf each lets fall, and the rustle they make now
//! and then beside a natural log.

use serde_json::{Value, json};

use super::engine::*;
use super::events::*;
use super::layouts::plane;
use crate::movement::world::{cat, put_with};

const LEAVES: [&str; 3] =
    ["minecraft:red_poplar_leaves", "minecraft:orange_poplar_leaves", "minecraft:yellow_poplar_leaves"];

/// What stands at a place of the plane: patches of one poplar leaf each, here and there a log
/// (some natural, some not), a stone, or nothing.
fn cell(x: i64, z: i64) -> Value {
    let kind = (x * x * 5 + z * z * 3 + x * z * 7 + x * 11 + z * 13).rem_euclid(17);
    let patch = (x.div_euclid(5) + z.div_euclid(5)).rem_euclid(3) as usize;
    let leaves = [("distance", "1"), ("persistent", "true"), ("waterlogged", "false")];
    match kind {
        0 | 4 => put_with(x, 70, z, "minecraft:oak_log", &[("axis", "y")]),
        1 => put_with(x, 70, z, "minecraft:birch_log", &[("axis", "y")]),
        2 => put_with(x, 70, z, "minecraft:crimson_stem", &[("axis", "y")]),
        3 => put_with(x, 70, z, "minecraft:stone", &[]),
        _ => put_with(x, 70, z, LEAVES[patch], &leaves),
    }
}

/// A plane of those cells with a roof of stone under some of them, so that some leaves have room
/// to let a leaf fall and some do not.
fn leaf_plane() -> Vec<Value> {
    let cells = (-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| cell(x, z))).collect::<Vec<_>>();
    let roof = (-15_i64..16)
        .flat_map(|x| (-15_i64..16).map(move |z| (x, z)))
        .filter(|&(x, z)| (x * z).rem_euclid(4) == 0)
        .map(|(x, z)| put_with(x, 69, z, "minecraft:stone", &[]))
        .collect::<Vec<_>>();
    cat![cells, roof, plane(60, "minecraft:stone", None, 15)]
}

pub fn rows() -> Vec<Value> {
    vec![
        // Each kind of leaf falls and swirls like a pale oak leaf, until the ground or a wall.
        engine_row(
            "poplar-leaves-add",
            [(0, "red_poplar_leaves"), (0, "orange_poplar_leaves"), (1, "yellow_poplar_leaves")]
                .into_iter()
                .enumerate()
                .map(|(index, (tick, name))| add(tick, name, (0.5 + index as f64, 66.5, 0.5)))
                .collect::<Vec<_>>(),
        )
        .blocks(plane(63, "minecraft:stone", None, 6))
        .ticks(60)
        .seeds(10)
        .describe(8)
        .row(),
        // Picked at random, many times a tick so that the rare rustle comes up: a leaf by the
        // block's chance, and the rustle when the sides hold one natural log and three leaves of
        // the same kind (its draw comes between the rain's and the leaf's).
        engine_row(
            "animate-poplar-leaves",
            (0_i64..8).map(|tick| animate_tick(tick, (0, 70, 0), 200000, 500 + tick)).collect::<Vec<_>>(),
        )
        .blocks(leaf_plane())
        .set("vanilla_tags", json!(true))
        .ticks(60)
        .seeds(4000)
        .describe(12)
        .row(),
    ]
}
