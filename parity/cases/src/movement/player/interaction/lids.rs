//! A chest the server opens: its lid rises with the level's block entities, which a frozen level
//! (`/tick freeze`) does not tick.

use serde_json::{Value, json};

use crate::movement::player::scene::{Scene, keys};
use crate::movement::world::{cat, floor, put};

/// The block event of a chest gaining a viewer.
fn open(tick: i64, at: (i64, i64, i64), viewers: i64) -> Value {
    json!({"tick": tick, "x": at.0, "y": at.1, "z": at.2, "b0": 1, "b1": viewers})
}

fn scene(label: &str, events: Vec<Value>) -> Scene {
    let at = (4, 64, 4);
    Scene::new(label, cat![floor(), vec![put(at.0, at.1, at.2, "minecraft:chest")]], keys(12, &[]))
        .start(4.5, 64.0, 7.5)
        .block_entity(at, "minecraft:chest")
        .block_events(events)
        .watch_lid(at)
}

pub fn rows() -> Vec<Value> {
    let at = (4, 64, 4);
    vec![
        // The lid rises while the level runs, and falls again when the viewer leaves.
        scene("opening", vec![open(0, at, 1)]).row(),
        scene("closing", vec![open(0, at, 1), open(6, at, 0)]).row(),
        // A frozen level leaves the lid where it was.
        scene("opening-frozen", vec![open(0, at, 1)]).frozen().row(),
        scene("closing-frozen", vec![open(0, at, 1), open(6, at, 0)]).frozen().row(),
    ]
}
