//! The faces of the end portal and end gateway cubes that are drawn: the portal's top and bottom, and
//! the gateway's, hidden by the blocks beside it (`shouldRenderFace`), read from the real block entities.

use serde_json::{Value, json};

use super::engine::*;
use crate::movement::world::{put, put_with};

fn portal_row() -> Value {
    let mut blocks = vec![
        // A gateway alone, one with stone over and under it, one among other shapes, and two side by side.
        put(0, 64, 0, "minecraft:end_gateway"),
        put(4, 64, 0, "minecraft:end_gateway"),
        put(4, 65, 0, "minecraft:stone"),
        put(4, 63, 0, "minecraft:stone"),
        put(8, 64, 0, "minecraft:end_gateway"),
        put(9, 64, 0, "minecraft:glass"),
        put_with(7, 64, 0, "minecraft:stone_slab", &[("type", "bottom")]),
        put_with(
            8,
            64,
            -1,
            "minecraft:oak_leaves",
            &[("persistent", "true"), ("distance", "1"), ("waterlogged", "false")],
        ),
        put(8, 64, 1, "minecraft:water"),
        put(12, 64, 0, "minecraft:end_gateway"),
        put(13, 64, 0, "minecraft:end_gateway"),
        put_with(12, 65, 0, "minecraft:stone_slab", &[("type", "top")]),
        // A portal alone, and a sheet of portal with stone over one of it.
        put(17, 64, 0, "minecraft:end_portal"),
        put(20, 64, 0, "minecraft:end_portal"),
        put(21, 64, 0, "minecraft:end_portal"),
        put(22, 64, 0, "minecraft:end_portal"),
        put(21, 65, 0, "minecraft:stone"),
    ];
    blocks.push(put(25, 64, 0, "minecraft:stone"));
    let at = [
        (0, "minecraft:end_gateway"),
        (4, "minecraft:end_gateway"),
        (8, "minecraft:end_gateway"),
        (12, "minecraft:end_gateway"),
        (13, "minecraft:end_gateway"),
        (17, "minecraft:end_portal"),
        (20, "minecraft:end_portal"),
        (21, "minecraft:end_portal"),
        (22, "minecraft:end_portal"),
    ];
    let places: Vec<Value> = at.iter().map(|(x, kind)| json!({"x": x, "y": 64, "z": 0, "type": kind})).collect();
    let watch: Vec<Value> = at.iter().map(|(x, _)| json!({"x": x, "y": 64, "z": 0})).collect();
    engine_row("portal-faces", vec![])
        .blocks(blocks)
        .set("block_entities", json!(places))
        .set("watch", json!(watch))
        .ticks(2)
        .describe(2)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![portal_row()]
}
