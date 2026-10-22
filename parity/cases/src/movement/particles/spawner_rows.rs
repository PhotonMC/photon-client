//! The particle engine's rows of a spawner's client tick (`BaseSpawner.clientTick`): the delay that
//! runs down and the spin that quickens with it while a player is near and the spawner shows a
//! mob, the old spin that catches up with none, and the smoke and flame it lets off.

use serde_json::{Value, json};

use super::engine::*;
use crate::bits::f64_bits;
use crate::movement::world::put_with;

/// A spawner of a row: where, what it shows, how far a player reaches it and its delay.
type Spawner = (i64, Option<&'static str>, Option<i64>, Option<i64>);

fn spawner_row(label: &str) -> Value {
    // The two at negative x are never drawn, so they make no mob and do not tick.
    let spawners: [Spawner; 10] = [
        (0, Some("minecraft:zombie"), None, None),
        (3, None, None, None),
        (6, Some("minecraft:zombie"), None, Some(0)),
        (9, Some("minecraft:zombie"), Some(2), None),
        (12, Some("minecraft:skeleton"), Some(20), Some(5)),
        (30, Some("minecraft:zombie"), None, None),
        (15, Some("minecraft:zombie"), None, Some(-1)),
        (18, Some("minecraft:zombie"), Some(-1), Some(200)),
        (-3, Some("minecraft:zombie"), None, None),
        (-6, Some("minecraft:skeleton"), None, Some(5)),
    ];
    let mut blocks = Vec::new();
    let mut places = Vec::new();
    let mut described = Vec::new();
    for (x, entity, range, delay) in spawners {
        blocks.push(put_with(x, 64, 0, "minecraft:spawner", &[]));
        places.push(json!({"x": x, "y": 64, "z": 0}));
        let mut spawner = json!({"x": x, "y": 64, "z": 0});
        if let Some(entity) = entity {
            spawner["entity"] = json!(entity);
        }
        if x < 0 {
            spawner["unseen"] = json!(true);
        }
        if let Some(range) = range {
            spawner["range"] = json!(range);
        }
        if let Some(delay) = delay {
            spawner["delay"] = json!(delay);
        }
        described.push(spawner);
    }
    let players = vec![json!({"uuid": [1, 2, 3, 4], "x": f64_bits(1.5), "y": f64_bits(64.0), "z": f64_bits(2.5)})];
    engine_row(label, vec![])
        .blocks(blocks)
        .set("spawners", json!(described))
        .set("players", json!(players))
        .set("watch", json!(places))
        .set("tick_block_entities", json!(true))
        .ticks(60)
        .seeds(3000)
        .describe(6)
        .row()
}

/// Trial spawners in each state, with a mob to show and the next spawn near and far, and one that
/// names no mob.
fn trial_row(label: &str) -> Value {
    let states = [
        "inactive",
        "waiting_for_players",
        "active",
        "waiting_for_reward_ejection",
        "ejecting_reward",
        "cooldown",
        "active",
        "waiting_for_players",
    ];
    let mut blocks = Vec::new();
    let mut places = Vec::new();
    let mut described = Vec::new();
    for (index, state) in states.into_iter().enumerate() {
        let x = 3 * index as i64;
        let ominous = if index >= 6 { "true" } else { "false" };
        let props = [("trial_spawner_state", state), ("ominous", ominous)];
        blocks.push(put_with(x, 64, 0, "minecraft:trial_spawner", &props));
        places.push(json!({"x": x, "y": 64, "z": 0}));
        let mut spawner = json!({"x": x, "y": 64, "z": 0, "trial": true, "next": 40 * index as i64});
        if index != 7 {
            spawner["entity"] = json!("minecraft:zombie");
        }
        described.push(spawner);
    }
    engine_row(label, vec![])
        .blocks(blocks)
        .set("spawners", json!(described))
        .set("watch", json!(places))
        .set("tick_block_entities", json!(true))
        .ticks(60)
        .seeds(3000)
        .describe(6)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![spawner_row("spawner-ticks"), trial_row("trial-spawner-ticks")]
}
