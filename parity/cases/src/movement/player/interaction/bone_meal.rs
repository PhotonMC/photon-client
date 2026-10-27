//! Bone meal on blocks: the client swings when the block can be grown (`isValidBonemealTarget`)
//! and passes otherwise.

use serde_json::Value;

use super::item_on::{north, row};
use crate::movement::world::{put, put_with};

pub(super) fn rows() -> Vec<Value> {
    let meal =
        |label: &str, world: Vec<Value>| row(label, world, &north()).main_hand("minecraft:bone_meal").count(3).row();
    let crop = |block: &str, age: &str| vec![put_with(0, 64, 2, block, &[("age", age)])];
    let grass_under = |above: &str| vec![put(0, 64, 2, "minecraft:grass_block"), put(0, 65, 2, above)];
    vec![
        meal("bone-meal-stone", vec![put(0, 64, 2, "minecraft:stone")]),
        meal("bone-meal-wheat-young", crop("minecraft:wheat", "2")),
        meal("bone-meal-wheat-ripe", crop("minecraft:wheat", "7")),
        meal("bone-meal-carrot-young", crop("minecraft:carrots", "0")),
        meal("bone-meal-grass-block-open", grass_under("minecraft:air")),
        meal("bone-meal-grass-block-covered", grass_under("minecraft:stone")),
        meal("bone-meal-short-grass", vec![put(0, 64, 2, "minecraft:short_grass")]),
        meal("bone-meal-oak-sapling", vec![put_with(0, 64, 2, "minecraft:oak_sapling", &[("stage", "0")])]),
        meal("bone-meal-sweet-berry-young", crop("minecraft:sweet_berry_bush", "1")),
        meal("bone-meal-sweet-berry-ripe", crop("minecraft:sweet_berry_bush", "3")),
        meal("bone-meal-poppy", vec![put(0, 64, 2, "minecraft:poppy")]),
        meal("bone-meal-dandelion", vec![put(0, 64, 2, "minecraft:dandelion")]),
    ]
}
