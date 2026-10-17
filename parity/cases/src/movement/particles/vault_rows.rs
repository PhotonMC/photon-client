//! The particle engine's rows of a vault's client tick: its display item's spin, the smoke and flame
//! it lets off, its ambient sound and the particles it sends to the players connected to it.

use serde_json::{Value, json};

use super::engine::*;
use crate::bits::f64_bits;
use crate::movement::world::put_with;

/// A vault of a row: how it faces, whether it is ominous, its display item and who is connected.
type Vault<'a> = (&'a str, &'a str, Option<&'a str>, Vec<[i64; 4]>);

fn vault_row(label: &str, game_time: u64) -> Value {
    let near = [1, 2, 3, 4];
    let far = [5, 6, 7, 8];
    let vaults: [Vault; 5] = [
        ("north", "false", None, vec![]),
        ("south", "false", Some("minecraft:diamond"), vec![near]),
        ("east", "true", Some("minecraft:emerald"), vec![]),
        ("west", "false", Some("minecraft:diamond"), vec![far]),
        ("north", "true", None, vec![near, far]),
    ];
    let mut blocks = Vec::new();
    let mut places = Vec::new();
    let mut shared = Vec::new();
    for (index, (facing, ominous, item, players)) in vaults.iter().enumerate() {
        let x = index as i64 * 3;
        blocks.push(put_with(
            x,
            64,
            0,
            "minecraft:vault",
            &[("facing", facing), ("ominous", ominous), ("vault_state", "active")],
        ));
        places.push(json!({"x": x, "y": 64, "z": 0}));
        let mut vault = json!({"x": x, "y": 64, "z": 0, "players": players});
        if let Some(item) = item {
            vault["item"] = json!(item);
        }
        shared.push(vault);
    }
    let players = vec![
        json!({"uuid": near, "x": f64_bits(1.5), "y": f64_bits(64.0), "z": f64_bits(2.5)}),
        json!({"uuid": far, "x": f64_bits(40.25), "y": f64_bits(64.0), "z": f64_bits(-40.75)}),
    ];
    engine_row(label, vec![])
        .blocks(blocks)
        .set("vaults", json!(shared))
        .set("players", json!(players))
        .set("watch", json!(places))
        .set("game_time", json!(game_time))
        .set("tick_block_entities", json!(true))
        .ticks(60)
        .seeds(3000)
        .describe(6)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![vault_row("vault-ticks", 160), vault_row("vault-ticks-off-beat", 161)]
}
