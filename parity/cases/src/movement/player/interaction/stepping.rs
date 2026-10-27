//! A block that acts on being stepped on (`Block.stepOn`): a redstone ore lights up under a
//! player who is not sneaking.

use serde_json::{Value, json};

use crate::movement::player::scene::{Scene, keys};
use crate::movement::world::{cat, floor, put, put_with};

/// The ticks, each watching the block at `at`.
fn watching(ticks: Vec<Value>, at: (i64, i64, i64)) -> Vec<Value> {
    ticks
        .into_iter()
        .map(|mut tick| {
            tick["watch"] = json!([{"x": at.0, "y": at.1, "z": at.2}]);
            tick
        })
        .collect()
}

/// An unlit ore of a kind in the floor.
fn ore(at: (i64, i64, i64), block: &str) -> Value {
    put_with(at.0, at.1, at.2, block, &[("lit", "false")])
}

fn scene(label: &str, blocks: Vec<Value>, held: &[&str], start: (f64, f64, f64)) -> Scene {
    let at = (4, 63, 4);
    Scene::new(label, cat![floor(), blocks], watching(keys(6, held), at)).start(start.0, start.1, start.2)
}

pub fn rows() -> Vec<Value> {
    let at = (4, 63, 4);
    vec![
        // Standing on it.
        scene("stand", vec![ore(at, "minecraft:redstone_ore")], &[], (4.5, 64.0, 4.5)).row(),
        scene("stand-deepslate", vec![ore(at, "minecraft:deepslate_redstone_ore")], &[], (4.5, 64.0, 4.5)).row(),
        // Sneaking on it leaves it be.
        scene("sneak", vec![ore(at, "minecraft:redstone_ore")], &["sneak"], (4.5, 64.0, 4.5)).row(),
        // Falling toward it without landing.
        scene("fall-short", vec![ore(at, "minecraft:redstone_ore")], &[], (4.5, 67.0, 4.5)).row(),
        // A carpet on it: the step reaches through the carpet to the block that holds it up.
        scene(
            "under-carpet",
            vec![ore(at, "minecraft:redstone_ore"), put(4, 64, 4, "minecraft:white_carpet")],
            &[],
            (4.5, 64.0625, 4.5),
        )
        .row(),
    ]
}
