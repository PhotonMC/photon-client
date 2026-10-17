//! The particle engine's rows of the parrots' draws from the level's random source: each parrot's
//! every tick, and now and then the look for a mob to imitate, which a vault's idle smoke (a draw of
//! its own each tick) shows the shifts of.

use serde_json::{Value, json};

use super::engine::*;
use crate::bits::f64_bits;
use crate::movement::world::{floor_of, put_with};

fn parrot_row(label: &str, mobs: &[(&str, f64)]) -> Value {
    // The parrots are silent: a loud one's ambient sounds are drawn from its own random source, which the
    // reference does not seed, so the reference's rows would differ from run to run.
    let at = |x: f64, z: f64| json!({"x": f64_bits(x), "y": f64_bits(64.0), "z": f64_bits(z), "silent": true});
    let mut blocks = floor_of(63, "minecraft:stone", -12, 12);
    blocks.push(put_with(
        0,
        64,
        5,
        "minecraft:vault",
        &[("facing", "north"), ("ominous", "false"), ("vault_state", "active")],
    ));
    let mobs: Vec<Value> = mobs
        .iter()
        .map(|(kind, x)| json!({"type": kind, "x": f64_bits(*x), "y": f64_bits(64.0), "z": f64_bits(0.5), "silent": true}))
        .collect();
    engine_row(label, vec![])
        .blocks(blocks)
        .set("parrots", json!([at(-3.5, 0.5), at(3.5, 0.5), at(0.5, -4.5)]))
        .set("mobs", json!(mobs))
        .set("block_entities", json!([{"x": 0, "y": 64, "z": 5, "type": "minecraft:vault"}]))
        .set("vaults", json!([{"x": 0, "y": 64, "z": 5, "players": []}]))
        .set("game_time", json!(161))
        .set("tick_entities", json!(true))
        .set("tick_block_entities", json!(true))
        .ticks(900)
        .seeds(2500)
        .describe(8)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![
        parrot_row("parrot-draws", &[]),
        parrot_row("parrot-draws-near-mobs", &[("minecraft:creeper", 1.5), ("minecraft:zombie", -1.5)]),
    ]
}
