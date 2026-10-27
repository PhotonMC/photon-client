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
    .into_iter()
    .chain(more(&meal, &crop))
    .collect()
}

/// Blocks beyond the first rows: ages at the limit, water beside a face, and the blocks whose
/// target depends on what is above or below.
fn more(meal: &dyn Fn(&str, Vec<Value>) -> Value, crop: &dyn Fn(&str, &str) -> Vec<Value>) -> Vec<Value> {
    let water = |level: &str| {
        vec![put(0, 64, 2, "minecraft:stone"), put_with(0, 64, 1, "minecraft:water", &[("level", level)])]
    };
    let at = |block: &str| vec![put(0, 64, 2, block)];
    let below = |block: &str, under: &str| vec![put(0, 63, 2, under), put(0, 64, 2, block)];
    let pickle = |wet: &str, under: &str| {
        vec![put(0, 63, 2, under), put_with(0, 64, 2, "minecraft:sea_pickle", &[("waterlogged", wet)])]
    };
    vec![
        meal("bone-meal-water-source", water("0")),
        meal("bone-meal-water-flowing", water("3")),
        meal("bone-meal-water-falling", water("8")),
        meal(
            "bone-meal-water-slab",
            vec![
                put_with(0, 64, 2, "minecraft:stone_slab", &[("type", "bottom")]),
                put_with(0, 64, 1, "minecraft:water", &[("level", "0")]),
            ],
        ),
        meal("bone-meal-bamboo-sapling", at("minecraft:bamboo_sapling")),
        meal("bone-meal-torchflower-0", crop("minecraft:torchflower_crop", "0")),
        meal("bone-meal-torchflower-1", crop("minecraft:torchflower_crop", "1")),
        meal("bone-meal-beetroot-2", crop("minecraft:beetroots", "2")),
        meal("bone-meal-beetroot-3", crop("minecraft:beetroots", "3")),
        meal("bone-meal-stem-7", crop("minecraft:melon_stem", "7")),
        meal("bone-meal-stem-6", crop("minecraft:melon_stem", "6")),
        meal("bone-meal-cocoa-1", crop("minecraft:cocoa", "1")),
        meal("bone-meal-cocoa-2", crop("minecraft:cocoa", "2")),
        meal("bone-meal-tall-flower", vec![put_with(0, 64, 2, "minecraft:sunflower", &[("half", "lower")])]),
        meal("bone-meal-tall-grass", vec![put_with(0, 64, 2, "minecraft:tall_grass", &[("half", "lower")])]),
        meal("bone-meal-brown-mushroom", at("minecraft:brown_mushroom")),
        meal("bone-meal-azalea", at("minecraft:azalea")),
        meal("bone-meal-pink-petals", at("minecraft:pink_petals")),
        meal("bone-meal-seagrass-wet", vec![put(0, 64, 2, "minecraft:seagrass"), put(0, 65, 2, "minecraft:water")]),
        meal("bone-meal-seagrass-dry", at("minecraft:seagrass")),
        meal("bone-meal-sea-pickle-live", pickle("true", "minecraft:tube_coral_block")),
        meal("bone-meal-sea-pickle-dead", pickle("true", "minecraft:dead_tube_coral_block")),
        meal("bone-meal-sea-pickle-dry", pickle("false", "minecraft:tube_coral_block")),
        meal("bone-meal-rooted-dirt-air", below("minecraft:rooted_dirt", "minecraft:air")),
        meal("bone-meal-rooted-dirt-covered", below("minecraft:rooted_dirt", "minecraft:stone")),
        meal("bone-meal-moss-open", at("minecraft:moss_block")),
        meal("bone-meal-moss-covered", vec![put(0, 64, 2, "minecraft:moss_block"), put(0, 65, 2, "minecraft:stone")]),
        meal("bone-meal-nylium-open", at("minecraft:crimson_nylium")),
        meal(
            "bone-meal-nylium-covered",
            vec![put(0, 64, 2, "minecraft:warped_nylium"), put(0, 65, 2, "minecraft:stone")],
        ),
    ]
}
