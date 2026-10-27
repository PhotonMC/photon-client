//! Bone meal on blocks: the client swings when the block can be grown (`isValidBonemealTarget`)
//! and passes otherwise.

use serde_json::Value;

use super::item_on::{north, row};
use super::placing::uses;
use super::using;
use crate::movement::world::{put, put_with};

/// Bone meal used while sneaking, which skips the block's own use: a berried vine then reaches
/// the item.
fn sneaking(label: &str, world: Vec<Value>) -> Value {
    let around = [(0, 64, 1), (0, 64, 2), (0, 65, 2), (0, 64, 3)];
    let aim = north();
    // The crouch takes hold a tick after its key goes down, so the click comes after two ticks of it.
    let ticks = [
        uses(2, None, &around).held(&["sneak"]).go(),
        uses(1, Some(&aim), &around).clicks(1).held(&["sneak"]).go(),
        uses(3, Some(&aim), &around).held(&["sneak"]).go(),
    ]
    .concat();
    using(label, world, ticks).main_hand("minecraft:bone_meal").count(3).row()
}

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
    let mut rows = plants(meal, &at);
    rows.extend(columns(meal, &below));
    rows.extend(vec![
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
        meal("bone-meal-mangrove-leaves-open", below("minecraft:mangrove_leaves", "minecraft:air")),
        meal("bone-meal-mangrove-leaves-covered", below("minecraft:mangrove_leaves", "minecraft:stone")),
        meal("bone-meal-crimson-fungus-nylium", below("minecraft:crimson_fungus", "minecraft:crimson_nylium")),
        meal("bone-meal-crimson-fungus-warped", below("minecraft:crimson_fungus", "minecraft:warped_nylium")),
        meal("bone-meal-warped-fungus-nylium", below("minecraft:warped_fungus", "minecraft:warped_nylium")),
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
    ]);
    rows
}

/// Growing plants: the head or the body needs room to grow into.
fn plants(meal: &dyn Fn(&str, Vec<Value>) -> Value, at: &dyn Fn(&str) -> Vec<Value>) -> Vec<Value> {
    vec![
        meal(
            "bone-meal-kelp-open",
            vec![put(0, 63, 2, "minecraft:sand"), put(0, 64, 2, "minecraft:kelp"), put(0, 65, 2, "minecraft:water")],
        ),
        meal("bone-meal-kelp-blocked", vec![put(0, 64, 2, "minecraft:kelp"), put(0, 65, 2, "minecraft:stone")]),
        meal(
            "bone-meal-kelp-plant-open",
            vec![
                put(0, 64, 2, "minecraft:kelp_plant"),
                put(0, 65, 2, "minecraft:kelp"),
                put(0, 66, 2, "minecraft:water"),
            ],
        ),
        meal(
            "bone-meal-kelp-plant-headless",
            vec![put(0, 64, 2, "minecraft:kelp_plant"), put(0, 65, 2, "minecraft:water")],
        ),
        meal("bone-meal-twisting-open", at("minecraft:twisting_vines")),
        meal(
            "bone-meal-twisting-blocked",
            vec![put(0, 64, 2, "minecraft:twisting_vines"), put(0, 65, 2, "minecraft:stone")],
        ),
        meal(
            "bone-meal-twisting-plant",
            vec![put(0, 64, 2, "minecraft:twisting_vines_plant"), put(0, 65, 2, "minecraft:twisting_vines")],
        ),
        meal(
            "bone-meal-weeping-plant",
            vec![put(0, 64, 2, "minecraft:weeping_vines_plant"), put(0, 63, 2, "minecraft:weeping_vines")],
        ),
        sneaking(
            "bone-meal-sneaking-cave-vines-berries",
            vec![put_with(0, 64, 2, "minecraft:cave_vines", &[("berries", "true")])],
        ),
        sneaking(
            "bone-meal-sneaking-cave-vines-bare",
            vec![put_with(0, 64, 2, "minecraft:cave_vines", &[("berries", "false")])],
        ),
        meal("bone-meal-cave-vines-berries", vec![put_with(0, 64, 2, "minecraft:cave_vines", &[("berries", "true")])]),
        meal(
            "bone-meal-cave-vines-bare",
            vec![put_with(0, 64, 2, "minecraft:cave_vines_plant", &[("berries", "false")])],
        ),
    ]
}

/// Plants that grow along a column: the big dripleaf with its stem, and pale hanging moss.
fn columns(meal: &dyn Fn(&str, Vec<Value>) -> Value, below: &dyn Fn(&str, &str) -> Vec<Value>) -> Vec<Value> {
    let at = |block: &str| vec![put(0, 64, 2, block)];
    vec![
        meal("bone-meal-big-dripleaf-open", at("minecraft:big_dripleaf")),
        meal(
            "bone-meal-big-dripleaf-blocked",
            vec![put(0, 64, 2, "minecraft:big_dripleaf"), put(0, 65, 2, "minecraft:stone")],
        ),
        meal(
            "bone-meal-big-dripleaf-water",
            vec![put(0, 64, 2, "minecraft:big_dripleaf"), put(0, 65, 2, "minecraft:water")],
        ),
        meal(
            "bone-meal-big-dripleaf-small-above",
            vec![
                put(0, 64, 2, "minecraft:big_dripleaf"),
                put_with(0, 65, 2, "minecraft:small_dripleaf", &[("half", "lower")]),
            ],
        ),
        meal(
            "bone-meal-dripleaf-stem-open",
            vec![put(0, 64, 2, "minecraft:big_dripleaf_stem"), put(0, 65, 2, "minecraft:big_dripleaf")],
        ),
        meal(
            "bone-meal-dripleaf-stem-blocked",
            vec![
                put(0, 64, 2, "minecraft:big_dripleaf_stem"),
                put(0, 65, 2, "minecraft:big_dripleaf"),
                put(0, 66, 2, "minecraft:stone"),
            ],
        ),
        meal("bone-meal-dripleaf-stem-headless", vec![put(0, 64, 2, "minecraft:big_dripleaf_stem")]),
        meal("bone-meal-hanging-moss-open", below("minecraft:pale_hanging_moss", "minecraft:air")),
        meal("bone-meal-hanging-moss-blocked", below("minecraft:pale_hanging_moss", "minecraft:stone")),
        meal(
            "bone-meal-hanging-moss-chain",
            vec![
                put(0, 65, 2, "minecraft:pale_hanging_moss"),
                put(0, 64, 2, "minecraft:pale_hanging_moss"),
                put(0, 63, 2, "minecraft:air"),
            ],
        ),
    ]
}
