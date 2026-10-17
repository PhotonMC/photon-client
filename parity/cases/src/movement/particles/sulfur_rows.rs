//! The particle engine's rows of potent sulfur: the gas its ticker sends up through the water
//! above it, and the bubbles it animates.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::engine::*;
use super::events::*;
use crate::movement::world::{put, put_with};

type Cell = (i64, i64, i64);

/// A pool of water `depth` deep over stone, a potent sulfur block in the state given at its
/// bottom in the middle; `extra` replaces what is at a place.
fn pool(state: &str, depth: i64, extra: Vec<(Cell, Value)>) -> Vec<Value> {
    let mut blocks: BTreeMap<Cell, Value> = BTreeMap::new();
    for x in -6..=6_i64 {
        for z in -6..=6_i64 {
            blocks.insert((x, 63, z), put(x, 63, z, "minecraft:stone"));
            blocks.insert((x, 64, z), put(x, 64, z, "minecraft:stone"));
            if x.abs() <= 5 && z.abs() <= 5 {
                for y in 65..65 + depth {
                    blocks.insert((x, y, z), put_with(x, y, z, "minecraft:water", &[("level", "0")]));
                }
            }
        }
    }
    blocks.insert((0, 64, 0), put_with(0, 64, 0, "minecraft:potent_sulfur", &[("potent_sulfur_state", state)]));
    blocks.extend(extra);
    blocks.into_values().collect()
}

/// The row of a pool whose sulfur block ticks for thirty ticks (the clouds live twenty), the game
/// time held where the ticker runs, or not.
fn gas_row(label: &str, blocks: Vec<Value>, game_time: u64) -> Engine {
    engine_row(label, vec![])
        .blocks(blocks)
        .set("sulfurs", json!([{"x": 0, "y": 64, "z": 0}]))
        .set("game_time", json!(game_time))
        .set("tick_block_entities", json!(true))
        .ticks(30)
        .seeds(2000)
        .describe(12)
}

/// The gas over water of every depth, in each state a block can tick in on the client.
fn depth_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for depth in 0..=5 {
        rows.push(gas_row(&format!("sulfur-gas-depth-{depth}"), pool("wet", depth, vec![]), 40).row());
    }
    rows.push(gas_row("sulfur-gas-dormant", pool("dormant", 2, vec![]), 60).row());
    rows.push(gas_row("sulfur-gas-dry", pool("dry", 3, vec![]), 60).row());
    rows.push(gas_row("sulfur-gas-off-beat", pool("wet", 3, vec![]), 41).row());
    rows
}

/// What can stand in the water above the block: a solid cap, flowing water, a waterlogged fan, kelp,
/// and glass over half the pool, over its water and in it, and slabs in its top layer
/// that stop the gas on its way.
fn cover_rows() -> Vec<Value> {
    let fan = put_with(0, 66, 0, "minecraft:tube_coral_fan", &[("waterlogged", "true")]);
    let kelp = put_with(0, 66, 0, "minecraft:kelp", &[("age", "3")]);
    let flowing = put_with(0, 67, 0, "minecraft:water", &[("level", "3")]);
    let cap = put(0, 67, 0, "minecraft:stone");
    let slab = |y: i64| -> Vec<(Cell, Value)> {
        (-5..=5_i64)
            .flat_map(|x| {
                (1..=5_i64).map(move |z| {
                    (
                        (x, y, z),
                        put_with(x, y, z, "minecraft:stone_slab", &[("type", "bottom"), ("waterlogged", "true")]),
                    )
                })
            })
            .collect()
    };
    // Glass over half the pool: in the cells the gas would go to, and in the water below them.
    let glass = |y: i64| -> Vec<(Cell, Value)> {
        (-5..=5_i64).flat_map(|x| (1..=5_i64).map(move |z| ((x, y, z), put(x, y, z, "minecraft:glass")))).collect()
    };
    vec![
        gas_row("sulfur-gas-cap", pool("wet", 2, vec![((0, 67, 0), cap)]), 40).row(),
        gas_row("sulfur-gas-flowing", pool("wet", 2, vec![((0, 67, 0), flowing)]), 40).row(),
        gas_row("sulfur-gas-fan", pool("wet", 3, vec![((0, 66, 0), fan)]), 40).row(),
        gas_row("sulfur-gas-kelp", pool("wet", 3, vec![((0, 66, 0), kelp)]), 40).row(),
        gas_row("sulfur-gas-ceiling", pool("wet", 3, glass(68)), 40).row(),
        gas_row("sulfur-gas-sealed", pool("wet", 3, glass(67)), 40).row(),
        gas_row("sulfur-gas-slabs", pool("wet", 3, slab(67)), 40).row(),
    ]
}

/// The camera far from the block: the limiter lets none in.
fn far_row() -> Value {
    gas_row("sulfur-gas-far", pool("wet", 2, vec![]), 40).camera((40.0, 64.0, 0.0)).row()
}

/// Sulfur blocks in every state, with water above and without, picked at random over a few ticks.
fn bubble_row() -> Value {
    let states = ["dry", "wet", "dormant", "erupting", "continuous", "wet", "wet", "erupting"];
    let mut blocks: BTreeMap<Cell, Value> = BTreeMap::new();
    for x in -6..=6_i64 {
        for z in -3..=3_i64 {
            blocks.insert((x, 63, z), put(x, 63, z, "minecraft:stone"));
            blocks.insert((x, 64, z), put(x, 64, z, "minecraft:stone"));
        }
    }
    for (index, state) in states.iter().enumerate() {
        let x = index as i64 - 4;
        blocks.insert((x, 64, 0), put_with(x, 64, 0, "minecraft:potent_sulfur", &[("potent_sulfur_state", state)]));
        // The water over it: three deep; over the sixth a solid cap, over the seventh a flow, and
        // over the last only one.
        let depth = if index == 7 { 1 } else { 3 };
        for y in 65..65 + depth {
            blocks.insert((x, y, 0), put_with(x, y, 0, "minecraft:water", &[("level", "0")]));
        }
        match index {
            5 => {
                blocks.insert((x, 65, 0), put(x, 65, 0, "minecraft:stone"));
            }
            6 => {
                blocks.insert((x, 65, 0), put_with(x, 65, 0, "minecraft:water", &[("level", "3")]));
            }
            _ => {}
        }
    }
    engine_row(
        "animate-sulfur-bubbles",
        (0..12).map(|tick| animate_tick(tick, (0, 65, 0), 900, 700 + tick)).collect::<Vec<_>>(),
    )
    .blocks(blocks.into_values().collect())
    .ticks(12)
    .seeds(4000)
    .describe(12)
    .row()
}

pub fn rows() -> Vec<Value> {
    let mut rows = depth_rows();
    rows.extend(cover_rows());
    rows.push(far_row());
    rows.push(bubble_row());
    rows
}
