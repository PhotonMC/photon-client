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
    let spawners: [Spawner; 8] = [
        (0, Some("minecraft:zombie"), None, None),
        (3, None, None, None),
        (6, Some("minecraft:zombie"), None, Some(0)),
        (9, Some("minecraft:zombie"), Some(2), None),
        (12, Some("minecraft:skeleton"), Some(20), Some(5)),
        (30, Some("minecraft:zombie"), None, None),
        (15, Some("minecraft:zombie"), None, Some(-1)),
        (18, Some("minecraft:zombie"), Some(-1), Some(200)),
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

pub fn rows() -> Vec<Value> {
    vec![spawner_row("spawner-ticks")]
}
