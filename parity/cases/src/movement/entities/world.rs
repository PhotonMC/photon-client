//! The blocks around an entity scene: floors, pools and the walls items run
//! into.

use serde_json::Value;

use crate::movement::world::{put, put_with};

/// A stone floor, 8 blocks either way at height 63.
pub fn floor() -> Vec<Value> {
    floor_from("minecraft:stone", -8, 8)
}

/// A floor of `block`.
pub fn floor_of(block: &str) -> Vec<Value> {
    floor_from(block, -8, 8)
}

/// A floor of `block` from `x_low` to `high` on x and from -8 to `high` on z.
pub fn floor_from(block: &str, x_low: i64, high: i64) -> Vec<Value> {
    (x_low..=high).flat_map(|x| (-8..=high).map(move |z| put(x, 63, z, block))).collect()
}

/// A pool of `block` `depth` blocks deep from `low` to `high` on both axes.
pub fn pool(block: &str, depth: i64, low: i64, high: i64) -> Vec<Value> {
    pool_where(block, depth, low, high, &[])
}

/// A pool with block state `properties`.
pub fn pool_where(block: &str, depth: i64, low: i64, high: i64, properties: &[(&str, &str)]) -> Vec<Value> {
    let mut blocks = Vec::new();
    for x in low..=high {
        for z in low..=high {
            for y in 64..64 + depth {
                blocks.push(if properties.is_empty() {
                    put(x, y, z, block)
                } else {
                    put_with(x, y, z, block, properties)
                });
            }
        }
    }
    blocks
}

/// A fluid block of the given `depth` level.
pub fn level(x: i64, y: i64, z: i64, depth: i64, block: &str) -> Value {
    put_with(x, y, z, block, &[("level", &depth.to_string())])
}

/// A wall of stone at x = 3.
pub fn wall() -> Vec<Value> {
    (64..67).flat_map(|y| (-8..9).map(move |z| put(3, y, z, "minecraft:stone"))).collect()
}

/// A stone ceiling over the floor.
pub fn ceiling() -> Vec<Value> {
    (-8..9).flat_map(|x| (-8..9).map(move |z| put(x, 66, z, "minecraft:stone"))).collect()
}

/// A stream: a source with the water flowing away over a floor, each block
/// one level lower than the one before.
pub fn stream() -> Vec<Value> {
    let mut blocks = vec![level(0, 64, 0, 0, "minecraft:water")];
    blocks.extend((1..8).map(|x| level(x, 64, 0, x, "minecraft:water")));
    blocks
}
