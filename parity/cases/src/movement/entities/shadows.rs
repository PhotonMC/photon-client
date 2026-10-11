//! An entity's shadow: the block squares under it that are lit and whole enough, how dark each is, and the quad drawn for each.

use crate::movement::entities::prelude::*;

fn ground() -> Vec<Value> {
    floor()
}

/// A stone roof over the floor at height `y`.
fn roof_at(y: i64) -> Vec<Value> {
    (-8..9).flat_map(|x| (-8..9).map(move |z| put(x, y, z, "minecraft:stone"))).collect()
}

fn roof() -> Vec<Value> {
    roof_at(67)
}

/// The block states a shadow falls on, one under the entity.
const UNDER: [(&str, &[(&str, &str)]); 27] = [
    ("minecraft:stone_slab", &[("type", "bottom")]),
    ("minecraft:stone_slab", &[("type", "top")]),
    ("minecraft:stone_slab", &[("type", "double")]),
    ("minecraft:oak_stairs", &[]),
    ("minecraft:white_carpet", &[]),
    ("minecraft:oak_fence", &[]),
    ("minecraft:glass", &[]),
    ("minecraft:ice", &[]),
    ("minecraft:soul_sand", &[]),
    ("minecraft:dirt_path", &[]),
    ("minecraft:farmland", &[]),
    ("minecraft:honey_block", &[]),
    ("minecraft:barrier", &[]),
    ("minecraft:water", &[]),
    ("minecraft:lava", &[]),
    ("minecraft:snow", &[("layers", "1")]),
    ("minecraft:snow", &[("layers", "8")]),
    ("minecraft:cauldron", &[]),
    ("minecraft:cactus", &[]),
    ("minecraft:chest", &[]),
    ("minecraft:oak_leaves", &[]),
    ("minecraft:light", &[]),
    ("minecraft:oak_trapdoor", &[("open", "false"), ("half", "bottom")]),
    ("minecraft:oak_door", &[]),
    ("minecraft:piston", &[]),
    ("minecraft:iron_bars", &[]),
    ("minecraft:bedrock", &[]),
];

fn under_blocks() -> Vec<Value> {
    UNDER
        .iter()
        .map(|(block, properties)| {
            let name = block.split(':').nth(1).unwrap_or(block);
            let values: Vec<&str> = properties.iter().map(|(_, value)| *value).collect();
            let label =
                if values.is_empty() { format!("on-{name}") } else { format!("on-{name}-{}", values.join("-")) };
            let under =
                if properties.is_empty() { put(0, 63, 0, block) } else { put_with(0, 63, 0, block, properties) };
            shadow(label, cat![ground(), vec![under]]).row()
        })
        .collect()
}

fn shadow_1() -> Vec<Value> {
    vec![
        shadow("plain-floor", ground()).row(),
        shadow("small-radius", ground()).radius(0.15).strength(0.75).row(),
        shadow("wide-radius", ground()).radius(2.6).row(),
        shadow("player", ground()).radius(0.5).row(),
        shadow("at-a-block-corner", ground()).position(0.02, 64.0, 0.98).row(),
        shadow("on-the-border", ground()).position(0.0, 64.0, 0.0).radius(1.0).row(),
    ]
}

fn shadow_2() -> Vec<Value> {
    vec![
        shadow("raised-half", ground()).position(0.5, 64.5, 0.5).row(),
        shadow("raised-one", ground()).position(0.5, 65.0, 0.5).row(),
        shadow("raised-two", ground()).position(0.5, 66.0, 0.5).row(),
        shadow("raised-too-high", ground()).position(0.5, 70.0, 0.5).row(),
        shadow("raised-but-weak", ground()).position(0.5, 65.0, 0.5).strength(0.6).row(),
        shadow("far-from-the-camera-64", ground()).distance_sq(64.0).row(),
    ]
}

fn shadow_3() -> Vec<Value> {
    vec![
        shadow("far-from-the-camera-200", ground()).distance_sq(200.0).row(),
        shadow("far-from-the-camera-255", ground()).distance_sq(255.0).row(),
        shadow("far-from-the-camera-256", ground()).distance_sq(256.0).row(),
        shadow("far-from-the-camera-300", ground()).distance_sq(300.0).row(),
        shadow("invisible", ground()).invisible(true).row(),
        shadow("option-off", ground()).option(false).row(),
    ]
}

fn shadow_4() -> Vec<Value> {
    vec![
        shadow("under-a-roof", cat![ground(), roof()]).position(0.5, 64.0, 0.5).row(),
        shadow("under-a-high-roof", cat![ground(), roof_at(80)]).row(),
        shadow("by-a-torch", cat![ground(), vec![json!({"x": 1, "y": 64, "z": 0, "block": "minecraft:torch"})]]).row(),
        shadow("by-a-lantern", cat![ground(), vec![json!({"x": 0, "y": 64, "z": 1, "block": "minecraft:lantern"})]])
            .row(),
        shadow("over-nothing", vec![]).position(0.5, 64.0, 0.5).row(),
        shadow("far-below-the-ground", ground()).position(0.5, 40.0, 0.5).row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [shadow_1(), shadow_2(), shadow_3(), shadow_4(), under_blocks()].concat()
}
