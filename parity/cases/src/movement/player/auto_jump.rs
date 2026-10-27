//! Auto-jump: the player walks at a step, and the reference decides whether to jump.

use serde_json::Value;

use super::scene::{Scene, keys, turned};
use crate::movement::world::{cat, column, column_with, floor, put};

/// A full block step across the way.
pub(in crate::movement::player) fn step() -> Vec<Value> {
    (-3..4).flat_map(|x| (3..8).flat_map(move |z| column(x, z, 64, 64))).collect()
}

/// A wall two blocks high across the way.
pub(in crate::movement::player) fn two_high() -> Vec<Value> {
    let upper = (-3..4).flat_map(|x| (3..8).map(move |z| put(x, 65, z, "minecraft:stone")));
    cat![step(), upper.collect::<Vec<_>>()]
}

/// A step of `block` with `properties`, one block high, across the way.
fn step_of(block: &str, properties: &[(&str, &str)]) -> Vec<Value> {
    (-3..4).flat_map(|x| (3..8).flat_map(move |z| column_with(x, z, 64, 64, block, properties))).collect()
}

fn walk(label: &str, blocks: Vec<Value>, held: &[&str]) -> Scene {
    Scene::new(label, cat![floor(), blocks], keys(20, held))
}

/// A walk with the player looking along `y_rot` on every tick.
fn turned_walk(label: &str, blocks: Vec<Value>, held: &[&str], y_rot: f64) -> Scene {
    Scene::new(label, cat![floor(), blocks], turned(keys(20, held), Some(y_rot), None)).y_rot(y_rot)
}

pub fn rows() -> Vec<Value> {
    let ahead = &["forward"];
    vec![
        walk("auto-jump-top-slab", step_of("minecraft:stone_slab", &[("type", "top")]), ahead).auto_jump().row(),
        walk("auto-jump-bottom-slab", step_of("minecraft:stone_slab", &[("type", "bottom")]), ahead).auto_jump().row(),
        walk("auto-jump-two-high", two_high(), ahead).auto_jump().row(),
        walk("auto-jump-fence", step_of("minecraft:oak_fence", &[]), ahead).auto_jump().row(),
        walk("auto-jump-stairs", step_of("minecraft:oak_stairs", &[("facing", "south"), ("half", "bottom")]), ahead)
            .auto_jump()
            .row(),
        walk(
            "auto-jump-ceiling",
            cat![step(), (0..4).map(|z| put(0, 66, z, "minecraft:stone")).collect::<Vec<_>>()],
            ahead,
        )
        .auto_jump()
        .row(),
        walk("auto-jump-sneaking", step(), &["forward", "sneak"]).auto_jump().row(),
        turned_walk("auto-jump-backing", step(), &["backward"], 180.0).auto_jump().row(),
        turned_walk("auto-jump-strafing", step(), &["left"], 90.0).auto_jump().row(),
        walk("auto-jump-off", step(), ahead).row(),
        walk(
            "auto-jump-honey",
            cat![
                step(),
                (-1..2).flat_map(|x| (1..4).map(move |z| put(x, 63, z, "minecraft:honey_block"))).collect::<Vec<_>>()
            ],
            ahead,
        )
        .auto_jump()
        .row(),
    ]
}
