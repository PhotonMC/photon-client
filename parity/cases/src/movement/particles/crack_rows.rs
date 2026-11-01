//! The particle engine's rows of the cracks other players leave on blocks: one that is not updated
//! for more than 400 game ticks goes at the next twentieth tick, and a new stage refreshes it.

use serde_json::{Value, json};

use super::engine::*;

fn time(tick: i64, time: i64) -> Value {
    json!({"tick": tick, "type": "game_time", "time": time})
}

fn crack(tick: i64, id: i64, x: i64, stage: i64) -> Value {
    json!({"tick": tick, "type": "destroy_progress", "id": id, "x": x, "y": 64, "z": 1, "stage": stage})
}

/// Two cracks at game time 100; one is refreshed at 501, a stage out of range clears an entry, and a
/// crack still there at exactly 400 ticks old stays until the next check after that.
pub fn rows() -> Vec<Value> {
    vec![row(false), row(true), flash_row(false), flash_row(true)]
}

/// A sky flash of three ticks counts down on a level that runs and on a frozen one alike.
fn flash_row(frozen: bool) -> Value {
    let label = if frozen { "sky-flash-frozen" } else { "sky-flash" };
    let flash = json!({"tick": 0, "type": "sky_flash", "time": 3});
    engine_row(label, vec![flash]).set("watch_flash", json!(true)).set("frozen", json!(frozen)).ticks(5).row()
}

/// The same script on a level that runs, and on one that is frozen (its cracks never expire).
fn row(frozen: bool) -> Value {
    let events = vec![
        time(0, 100),
        crack(0, 7, 1, 3),
        crack(0, 8, 2, 5),
        time(1, 480),
        time(2, 500),
        time(3, 501),
        crack(3, 8, 2, 6),
        time(4, 520),
        crack(5, 9, 3, 0),
        crack(5, 8, 2, 12),
        time(6, 901),
        time(7, 920),
        time(8, 940),
    ];
    let label = if frozen { "crack-expiry-frozen" } else { "crack-expiry" };
    engine_row(label, events)
        .set("expire_cracks", json!(true))
        .set("watch_cracks", json!(true))
        .set("frozen", json!(frozen))
        .ticks(9)
        .row()
}
