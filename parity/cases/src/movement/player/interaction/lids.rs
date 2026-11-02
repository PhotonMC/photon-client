//! Block animations the level's block entities carry on: a chest's lid, a bell's shake and a
//! powered dragon head. A frozen level (`/tick freeze`) does not tick them.

use serde_json::{Value, json};

use crate::movement::player::scene::{Scene, keys};
use crate::movement::world::{cat, floor, put, put_with};

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

/// The block event of a bell being struck from the north.
fn strike(tick: i64, at: (i64, i64, i64)) -> Value {
    json!({"tick": tick, "x": at.0, "y": at.1, "z": at.2, "b0": 1, "b1": 2})
}

fn bell_scene(label: &str) -> Scene {
    let at = (4, 64, 4);
    Scene::new(label, cat![floor(), vec![put(at.0, at.1, at.2, "minecraft:bell")]], keys(12, &[]))
        .start(4.5, 64.0, 7.5)
        .block_entity(at, "minecraft:bell")
        .block_events(vec![strike(0, at)])
        .watch_bell(at)
}

/// A dragon head with a signal; the event that follows does nothing, it only puts the level's
/// block entities on the clock.
fn head_scene(label: &str, powered: &str) -> Scene {
    let at = (4, 64, 4);
    let head = put_with(at.0, at.1, at.2, "minecraft:dragon_head", &[("powered", powered)]);
    Scene::new(label, cat![floor(), vec![head]], keys(12, &[]))
        .start(4.5, 64.0, 7.5)
        .block_entity(at, "minecraft:skull")
        .block_events(vec![json!({"tick": 0, "x": at.0, "y": at.1, "z": at.2, "b0": 99, "b1": 0})])
        .watch_skull(at)
}

pub fn animation_rows() -> Vec<Value> {
    vec![
        bell_scene("bell-struck").row(),
        bell_scene("bell-struck-frozen").frozen().row(),
        head_scene("head-powered", "true").row(),
        head_scene("head-powered-frozen", "true").frozen().row(),
        head_scene("head-unpowered", "false").row(),
    ]
}
