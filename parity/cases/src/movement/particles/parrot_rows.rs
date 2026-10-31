//! The particle engine's rows of the parrots' draws from the level's random source: each parrot's
//! every tick, and now and then the look for a mob to imitate, which a vault's idle smoke (a draw of
//! its own each tick) shows the shifts of.

use serde_json::{Value, json};

use super::engine::*;
use super::events::level_event;
use crate::bits::f64_bits;
use crate::movement::world::{floor_of, put_with};

fn parrot_row(label: &str, mobs: &[(&str, f64)], frozen: bool) -> Value {
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
        .set("frozen", json!(frozen))
        .ticks(900)
        .seeds(2500)
        .describe(8)
        .row()
}

/// Parrots dancing to a record whose jukebox is then taken away: each parrot's own tick notices and
/// stops, unless the level is frozen and the parrots do not tick.
fn jukebox_gone_row(label: &str, frozen: bool) -> Value {
    let at = |x: f64, z: f64| json!({"x": f64_bits(x), "y": f64_bits(64.0), "z": f64_bits(z), "silent": true});
    let mut blocks = floor_of(63, "minecraft:stone", -6, 6);
    blocks.push(put_with(0, 64, 0, "minecraft:jukebox", &[]));
    let events = vec![
        level_event(0, 1010, (0, 64, 0), 2),
        json!({"tick": 2, "type": "set_block", "x": 0, "y": 64, "z": 0, "state": {"block": "minecraft:air"}}),
    ];
    engine_row(label, events)
        .blocks(blocks)
        .set("parrots", json!([at(0.5, 0.5), at(2.5, 0.5)]))
        .set("tick_entities", json!(true))
        .set("frozen", json!(frozen))
        .ticks(5)
        .seeds(8)
        .describe(8)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![
        parrot_row("parrot-draws", &[], false),
        parrot_row("parrot-draws-near-mobs", &[("minecraft:creeper", 1.5), ("minecraft:zombie", -1.5)], false),
        // `/tick freeze`: neither the parrots nor the vault tick, so the draws stand still.
        parrot_row("parrot-frozen", &[("minecraft:creeper", 1.5)], true),
        jukebox_gone_row("parrots-lose-jukebox", false),
        jukebox_gone_row("parrots-lose-jukebox-frozen", true),
    ]
}
