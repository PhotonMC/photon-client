//! A geyser of potent sulfur sending the player up: the block entity's launch ticker runs after
//! the player's own tick, so the push shows in the next tick's fall.

use serde_json::{Value, json};

use crate::movement::player::scene::{Scene, keys};
use crate::movement::world::{cat, floor, put, put_with};

/// A potent sulfur block of a state at the foot of `water` cells of water.
fn geyser(at: (i64, i64, i64), state: &str, water: i64) -> Vec<Value> {
    let mut blocks = vec![put_with(at.0, at.1, at.2, "minecraft:potent_sulfur", &[("potent_sulfur_state", state)])];
    blocks.extend((1..=water).map(|up| put(at.0, at.1 + up, at.2, "minecraft:water")));
    blocks
}

/// The block event `onPlace` sends; it also puts the level's block entities on the clock.
fn erupt(at: (i64, i64, i64)) -> Value {
    json!({"tick": 0, "x": at.0, "y": at.1, "z": at.2, "b0": 0, "b1": 0})
}

fn scene(label: &str, blocks: Vec<Value>, at: (i64, i64, i64), start: (f64, f64, f64)) -> Scene {
    Scene::new(label, cat![floor(), blocks], keys(12, &[]))
        .start(start.0, start.1, start.2)
        .block_entity(at, "minecraft:potent_sulfur")
        .block_events(vec![erupt(at)])
}

pub fn rows() -> Vec<Value> {
    let at = (4, 64, 4);
    vec![
        // Falling through the plume of one block of water.
        scene("erupting", geyser(at, "erupting", 1), at, (4.5, 68.0, 4.5)).row(),
        scene("continuous", geyser(at, "continuous", 1), at, (4.5, 68.0, 4.5)).row(),
        // Three blocks of water make the plume taller and the speed it gives way to higher.
        scene("deep", geyser(at, "continuous", 3), at, (4.5, 80.0, 4.5)).row(),
        // The states that do not erupt leave the player alone.
        scene("wet", geyser(at, "wet", 1), at, (4.5, 68.0, 4.5)).row(),
        scene("dormant", geyser(at, "dormant", 1), at, (4.5, 68.0, 4.5)).row(),
        // Beside the column.
        scene("beside", geyser(at, "continuous", 1), at, (6.5, 68.0, 4.5)).row(),
        // A spectator passes through.
        scene("spectator", geyser(at, "continuous", 1), at, (4.5, 68.0, 4.5)).game_mode("spectator").row(),
        // A block over the water stops the gas, and with it the geyser.
        scene(
            "capped",
            cat![geyser(at, "continuous", 1), vec![put(4, 66, 4, "minecraft:stone")]],
            at,
            (4.5, 68.0, 4.5),
        )
        .row(),
        // A creative player is pushed until the double tap lifts it into flight, then left alone.
        Scene::new(
            "flying",
            cat![floor(), geyser(at, "continuous", 1)],
            [keys(1, &["jump"]), keys(1, &[]), keys(1, &["jump"]), keys(9, &[])].concat(),
        )
        .start(4.5, 68.0, 4.5)
        .game_mode("creative")
        .block_entity(at, "minecraft:potent_sulfur")
        .block_events(vec![erupt(at)])
        .row(),
        // A slab in the column cuts the force short: the player on it is above the reach.
        scene(
            "cut-short",
            cat![geyser(at, "continuous", 1), vec![put_with(4, 70, 4, "minecraft:stone_slab", &[("type", "bottom")])]],
            at,
            (4.5, 70.5, 4.5),
        )
        .row(),
        // Falling in from above, slowly enough that the fall distance is held down.
        scene("fall-held", geyser(at, "continuous", 1), at, (4.5, 73.5, 4.5)).row(),
        // Standing on the ground of the plume (the player stands up in the water, then is lifted).
        scene("standing", geyser(at, "continuous", 1), at, (4.5, 66.0, 4.5)).row(),
    ]
}
