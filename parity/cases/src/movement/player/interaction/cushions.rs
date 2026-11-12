//! A cushion used on a block (`CushionItem.useOn`): only a top face takes it, and only where
//! something holds it from below, the blocks around it are not all suffocating and its resting
//! sliver is not shut in. A cauldron, hopper or composter measures the click against its collision
//! shape first. The newest version has the item.

use serde_json::{Value, json};

use super::aim_at;
use super::item_on::{north, row};
use super::placing::uses;
use crate::movement::world::{put, put_with};

const CUSHION: &str = "minecraft:white_cushion";

fn cushion(label: &str, world: Vec<Value>, aim: &Value) -> super::Interaction {
    row(label, world, aim).main_hand(CUSHION).count(3)
}

fn top(y: f64) -> Value {
    aim_at((0, 64, 2), "up", (0.5, y, 2.5))
}

fn faces() -> Vec<Value> {
    let stone = || vec![put(0, 64, 2, "minecraft:stone")];
    let down = aim_at((0, 64, 2), "down", (0.5, 64.0, 2.5));
    let east = aim_at((0, 64, 2), "east", (1.0, 64.5, 2.5));
    vec![
        cushion("cushion-on-top", stone(), &top(65.0)).row(),
        cushion("cushion-on-top-creative", stone(), &top(65.0)).game_mode("creative").row(),
        cushion("cushion-on-top-adventure", stone(), &top(65.0)).game_mode("adventure").row(),
        cushion("cushion-on-side", stone(), &north()).row(),
        cushion("cushion-on-east-side", stone(), &east).row(),
        cushion("cushion-on-bottom", stone(), &down).row(),
    ]
}

fn surfaces() -> Vec<Value> {
    let on = |block: &str| vec![put(0, 64, 2, block)];
    let slab = vec![put_with(0, 64, 2, "minecraft:stone_slab", &[("type", "bottom"), ("waterlogged", "false")])];
    vec![
        cushion("cushion-on-slab-top", slab.clone(), &top(64.5)).row(),
        // The click is inside the slab: the sliver it rests on is shut in.
        cushion("cushion-in-slab", slab, &top(64.25)).row(),
        cushion("cushion-on-carpet", on("minecraft:white_carpet"), &top(64.0625)).row(),
        cushion("cushion-on-glass", on("minecraft:glass"), &top(65.0)).row(),
        cushion("cushion-on-fence", on("minecraft:oak_fence"), &top(65.0)).row(),
        cushion("cushion-on-soul-sand", on("minecraft:soul_sand"), &top(64.875)).row(),
        cushion("cushion-on-air-below", vec![], &top(65.0)).row(),
        // A click above the stone leaves a gap under the box.
        cushion("cushion-above-stone", on("minecraft:stone"), &top(65.5)).row(),
        // A click within the stone: the sliver is shut in by the stone itself.
        cushion("cushion-inside-stone", on("minecraft:stone"), &top(64.75)).row(),
        cushion("cushion-on-short-grass", on("minecraft:short_grass"), &top(64.5)).row(),
    ]
}

fn covered() -> Vec<Value> {
    let stone = |above: &str| vec![put(0, 64, 2, "minecraft:stone"), put(0, 65, 2, above)];
    vec![
        cushion("cushion-under-stone", stone("minecraft:stone"), &top(65.0)).row(),
        cushion("cushion-under-glass", stone("minecraft:glass"), &top(65.0)).row(),
        cushion("cushion-under-slab", stone("minecraft:stone_slab"), &top(65.0)).row(),
        cushion("cushion-under-carpet", stone("minecraft:white_carpet"), &top(65.0)).row(),
        cushion("cushion-under-fence", stone("minecraft:oak_fence"), &top(65.0)).row(),
        cushion("cushion-under-leaves", stone("minecraft:oak_leaves"), &top(65.0)).row(),
    ]
}

/// The blocks of the tag, whose collision shape the click is measured against.
fn special() -> Vec<Value> {
    let cauldron = || vec![put(0, 64, 2, "minecraft:cauldron")];
    let hopper = || vec![put_with(0, 64, 2, "minecraft:hopper", &[("enabled", "true"), ("facing", "down")])];
    let composter = || vec![put_with(0, 64, 2, "minecraft:composter", &[("level", "0")])];
    let rim = |y: f64| aim_at((0, 64, 2), "up", (0.5, y, 2.0625));
    vec![
        cushion("cushion-on-cauldron-centre", cauldron(), &top(65.0)).row(),
        cushion("cushion-on-cauldron-rim", cauldron(), &rim(65.0)).row(),
        cushion("cushion-on-cauldron-rim-inside", cauldron(), &rim(64.9)).row(),
        cushion("cushion-on-cauldron-floor", cauldron(), &top(64.25)).row(),
        cushion("cushion-on-hopper-centre", hopper(), &top(65.0)).row(),
        cushion("cushion-on-hopper-rim", hopper(), &rim(65.0)).row(),
        cushion("cushion-on-hopper-rim-inside", hopper(), &rim(64.9)).row(),
        cushion("cushion-on-composter-centre", composter(), &top(65.0)).row(),
        cushion("cushion-on-composter-rim", composter(), &rim(65.0)).row(),
        cushion("cushion-on-composter-rim-inside", composter(), &rim(64.9)).row(),
        cushion("cushion-on-lava-cauldron", vec![put(0, 64, 2, "minecraft:lava_cauldron")], &rim(64.9)).row(),
    ]
}

/// Scaffolding, which the collision walk with no entity asking treats as above and not
/// descending: its stable shape whatever its state, a slab on top of four posts.
fn scaffolding() -> Vec<Value> {
    let block = |bottom: &str, distance: &str| {
        vec![put_with(
            0,
            64,
            2,
            "minecraft:scaffolding",
            &[("bottom", bottom), ("distance", distance), ("waterlogged", "false")],
        )]
    };
    vec![
        cushion("cushion-on-scaffolding", block("false", "0"), &top(65.0)).row(),
        // Inside the top slab the sliver is shut in, in a stable block and an unstable bottom one.
        cushion("cushion-in-scaffolding-top", block("false", "0"), &top(64.9)).row(),
        cushion("cushion-in-unstable-scaffolding-top", block("true", "3"), &top(64.9)).row(),
        // Between the posts, below the slab: nothing shuts it in from above, the posts are beside it.
        cushion("cushion-in-scaffolding-posts", block("false", "0"), &top(64.5)).row(),
    ]
}

/// A moving piston block, whose collision shape is what its block entity fills at the progress it
/// has made: a stone pushed up from the cell the piston head is in, clicked on the head a few
/// ticks into the move, so the cushion would go in the cell the stone is passing.
fn moving_piston() -> Vec<Value> {
    let world = || {
        vec![
            put_with(0, 63, 2, "minecraft:piston", &[("facing", "up"), ("extended", "false")]),
            put(0, 64, 2, "minecraft:stone"),
        ]
    };
    let event = json!({"tick": 0, "x": 0, "y": 63, "z": 2, "b0": 0, "b1": 1});
    let row_at = |label: &str, wait: usize| {
        let aim = aim_at((0, 64, 2), "up", (0.5, 65.0, 2.5));
        let around = [(0, 64, 2), (0, 65, 2), (0, 66, 2)];
        let click = uses(1, Some(&aim), &around).clicks(1).go();
        let ticks = [uses(wait, Some(&aim), &around).go(), click, uses(3, Some(&aim), &around).go()].concat();
        let mut row = super::using(label, world(), ticks).main_hand(CUSHION).count(3).row();
        row["block_events"] = json!([event.clone()]);
        row
    };
    vec![
        row_at("cushion-over-moving-stone-1", 1),
        row_at("cushion-over-moving-stone-2", 2),
        row_at("cushion-over-moving-stone-3", 3),
    ]
}

pub(super) fn rows() -> Vec<Value> {
    if crate::components::is_26_2() {
        return Vec::new();
    }
    [faces(), surfaces(), covered(), special(), scaffolding(), moving_piston()].concat()
}
