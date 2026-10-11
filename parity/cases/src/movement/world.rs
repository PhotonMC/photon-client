//! Blocks placed in a scene: single blocks, floors, columns and slabs.

use serde_json::{Map, Value, json};

/// The concatenation of block lists.
macro_rules! cat {
    ($($part:expr),* $(,)?) => { [$($part),*].concat() };
}
pub(crate) use cat;

/// A block at a point.
pub fn put(x: i64, y: i64, z: i64, block: &str) -> Value {
    json!({"x": x, "y": y, "z": z, "block": block})
}

/// A block at a point with block state `properties`.
pub fn put_with(x: i64, y: i64, z: i64, block: &str, properties: &[(&str, &str)]) -> Value {
    let mut placed = put(x, y, z, block);
    placed["properties"] = properties_of(properties);
    placed
}

fn properties_of(properties: &[(&str, &str)]) -> Value {
    Value::Object(properties.iter().map(|(name, value)| ((*name).to_owned(), json!(value))).collect::<Map<_, _>>())
}

/// A square floor of `block` at height `y`, from `low` to `high` on both
/// axes.
pub fn floor_of(y: i64, block: &str, low: i64, high: i64) -> Vec<Value> {
    (low..=high).flat_map(|x| (low..=high).map(move |z| (x, z))).map(|(x, z)| put(x, y, z, block)).collect()
}

/// The player scenes' stone floor.
pub fn floor() -> Vec<Value> {
    floor_of(63, "minecraft:stone", -16, 31)
}

/// The player scenes' floor of another block.
pub fn floor_block(block: &str) -> Vec<Value> {
    floor_of(63, block, -16, 31)
}

/// A column of stone from `low` to `high`.
pub fn column(x: i64, z: i64, low: i64, high: i64) -> Vec<Value> {
    column_of(x, z, low, high, "minecraft:stone")
}

/// A column of `block` from `low` to `high`.
pub fn column_of(x: i64, z: i64, low: i64, high: i64, block: &str) -> Vec<Value> {
    (low..=high).map(|y| put(x, y, z, block)).collect()
}

/// A column of `block` with `properties`.
pub fn column_with(x: i64, z: i64, low: i64, high: i64, block: &str, properties: &[(&str, &str)]) -> Vec<Value> {
    (low..=high).map(|y| put_with(x, y, z, block, properties)).collect()
}

/// A box of stone: `x`, `z` and `y` ranges, each from low to high.
pub fn slab(x: (i64, i64), z: (i64, i64), y: (i64, i64)) -> Vec<Value> {
    slab_of(x, z, y, "minecraft:stone")
}

/// A box of `block`.
pub fn slab_of(x: (i64, i64), z: (i64, i64), y: (i64, i64), block: &str) -> Vec<Value> {
    (x.0..=x.1)
        .flat_map(|x| (z.0..=z.1).map(move |z| (x, z)))
        .flat_map(|(x, z)| (y.0..=y.1).map(move |y| (x, y, z)))
        .map(|(x, y, z)| put(x, y, z, block))
        .collect()
}
