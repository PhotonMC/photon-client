//! A cushion used on a block (`CushionItem.useOn`): only a top face takes it, and only where
//! something holds it from below, the blocks around it are not all suffocating and its resting
//! sliver is not shut in. A cauldron, hopper or composter measures the click against its collision
//! shape first. The newest version has the item.

use serde_json::Value;

use super::aim_at;
use super::item_on::{north, row};
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

pub(super) fn rows() -> Vec<Value> {
    if crate::components::is_26_2() {
        return Vec::new();
    }
    [faces(), surfaces(), covered(), special()].concat()
}
