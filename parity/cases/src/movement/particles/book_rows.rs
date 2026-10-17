//! The particle engine's rows of an enchanting table's book: it turns to the player near it,
//! opens, and flips its pages, each tick the level ticks its block entities.

use serde_json::{Value, json};

use super::engine::*;
use crate::bits::f64_bits;
use crate::movement::world::put;

/// A player standing at a place, by the number that tells them apart.
fn player(number: u64, (x, y, z): (f64, f64, f64)) -> Value {
    json!({"uuid": [number, number, number, number], "x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

/// Tables in a row along the floor, each with a player placed to try one thing: near, exactly three
/// blocks off (which is far), none at all, straight along the west (the turn wraps), south of one,
/// and two players near one at different distances.
fn table_row(label: &str, seed: i64, ticks: i64) -> Value {
    let tables = [3, 10, 20, 30, 40, 50];
    let mut blocks: Vec<Value> = (-4..=60).map(|x| put(x, 63, 0, "minecraft:stone")).collect();
    let mut watch = Vec::new();
    let mut entities = Vec::new();
    for x in tables {
        blocks.push(put(x, 64, 0, "minecraft:enchanting_table"));
        watch.push(json!({"x": x, "y": 64, "z": 0}));
        entities.push(json!({"x": x, "y": 64, "z": 0, "type": "minecraft:enchanting_table"}));
    }
    let players = vec![
        player(1, (3.5, 64.0, 1.5)),
        player(2, (10.5, 64.5, 3.5)),
        player(4, (28.6, 64.0, 0.5)),
        player(5, (40.5, 64.0, 0.4)),
        player(6, (50.5, 64.0, 2.0)),
        player(7, (49.0, 64.0, 0.5)),
    ];
    engine_row(label, vec![])
        .blocks(blocks)
        .set("block_entities", json!(entities))
        .set("players", json!(players))
        .set("watch", json!(watch))
        .set("book_seed", json!(seed))
        .set("tick_block_entities", json!(true))
        .ticks(ticks)
        .seeds(8)
        .describe(0)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![table_row("enchanting-table-books", 4711, 90), table_row("enchanting-table-books-other-seed", 99, 70)]
}
