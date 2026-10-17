//! The particle engine's rows of blocks that animate on their own: what each shows (or sounds) as
//! the blocks about the player are picked at random.

use serde_json::Value;

use super::engine::*;
use super::events::*;
use crate::movement::world::put_with;

/// A block with its properties, at a place.
fn at(pos: (i64, i64, i64), block: &str, properties: &[(&str, &str)]) -> Value {
    put_with(pos.0, pos.1, pos.2, block, properties)
}

/// What stands in a column of the plane: a block at height 64 with what it needs below and above.
type Column = &'static [(i64, &'static str, &'static [(&'static str, &'static str)])];

/// The columns of the plane, in the order they are dealt out.
const COLUMNS: [Column; 17] = [
    &[(64, "minecraft:wither_rose", &[]), (63, "minecraft:grass_block", &[])],
    &[(64, "minecraft:end_portal", &[]), (63, "minecraft:stone", &[])],
    &[(64, "minecraft:dried_ghast", &[("waterlogged", "false")]), (63, "minecraft:soul_sand", &[])],
    &[(64, "minecraft:dried_ghast", &[("waterlogged", "true")]), (63, "minecraft:stone", &[])],
    &[(64, "minecraft:suspicious_sand", &[])],
    &[(64, "minecraft:suspicious_gravel", &[]), (63, "minecraft:stone", &[])],
    &[(64, "minecraft:beehive", &[("honey_level", "5"), ("facing", "north")])],
    &[(64, "minecraft:bee_nest", &[("honey_level", "5"), ("facing", "south")]), (63, "minecraft:stone", &[])],
    &[(64, "minecraft:pale_hanging_moss", &[("tip", "true")]), (65, "minecraft:pale_oak_log", &[])],
    &[(64, "minecraft:pale_hanging_moss", &[("tip", "true")]), (65, "minecraft:stone", &[])],
    &[(64, "minecraft:open_eyeblossom", &[]), (63, "minecraft:pale_moss_block", &[])],
    &[(64, "minecraft:closed_eyeblossom", &[]), (63, "minecraft:pale_moss_block", &[])],
    &[(64, "minecraft:dead_bush", &[]), (63, "minecraft:sand", &[]), (62, "minecraft:sand", &[])],
    &[(64, "minecraft:dead_bush", &[]), (63, "minecraft:red_sand", &[]), (62, "minecraft:red_sand", &[])],
    &[
        (64, "minecraft:dead_bush", &[]),
        (63, "minecraft:white_terracotta", &[]),
        (62, "minecraft:white_terracotta", &[]),
    ],
    &[(64, "minecraft:short_dry_grass", &[]), (63, "minecraft:sand", &[]), (62, "minecraft:sand", &[])],
    &[(64, "minecraft:torch", &[]), (63, "minecraft:stone", &[])],
];

/// A plane of those columns, one for each place by a fixed shuffle, so that the same kind is not
/// side by side.
fn plane() -> Vec<Value> {
    let mut blocks = Vec::new();
    for x in -15_i64..16 {
        for z in -15_i64..16 {
            let index = (x * 7 + z * 13).rem_euclid(COLUMNS.len() as i64) as usize;
            for (y, block, properties) in COLUMNS[index] {
                blocks.push(at((x, *y, z), block, properties));
            }
        }
    }
    blocks
}

/// One enchanting table at a place with bookshelves about it: all of the west, north and east
/// rings, none of the south, and the one in the middle of the north behind a block that lets no
/// power through.
fn enchanting_table((tx, tz): (i64, i64)) -> Vec<Value> {
    let ring: Vec<(i64, i64)> = (-2_i64..=2)
        .flat_map(|z| (-2_i64..=2).map(move |x| (x, z)))
        .filter(|(x, z)| (x.abs() == 2 || z.abs() == 2) && *z != 2)
        .collect();
    let mut blocks = vec![at((tx, 64, tz), "minecraft:enchanting_table", &[])];
    for y in [64, 65] {
        blocks.extend(ring.iter().map(|(x, z)| at((tx + x, y, tz + z), "minecraft:bookshelf", &[])));
    }
    blocks.push(at((tx, 64, tz - 1), "minecraft:stone", &[]));
    blocks
}

/// Enchanting tables across the plane, each with its bookshelves.
fn enchanting_tables() -> Vec<Value> {
    let places = [-14_i64, -7, 0, 7, 14];
    places.iter().flat_map(|tx| places.iter().map(move |tz| (*tx, *tz))).flat_map(enchanting_table).collect()
}

pub fn rows() -> Vec<Value> {
    let seeds = |tick: i64| 400 + tick;
    vec![
        engine_row(
            "animate-ambient",
            (0..6).map(|tick| animate_tick(tick, (0, 64, 0), 500, seeds(tick))).collect::<Vec<_>>(),
        )
        .blocks(plane())
        .ticks(6)
        .seeds(4000)
        .describe(10)
        .row(),
        engine_row(
            "animate-enchanting-table",
            (0..4).map(|tick| animate_tick(tick, (0, 64, 0), 600, seeds(tick) + 20)).collect::<Vec<_>>(),
        )
        .blocks(enchanting_tables())
        .ticks(4)
        .seeds(4000)
        .describe(10)
        .row(),
    ]
}
