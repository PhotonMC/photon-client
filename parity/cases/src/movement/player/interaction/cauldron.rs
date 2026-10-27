//! The use key on cauldrons: their tables work on the server, so the client only learns whether the
//! stack was taken (a swing) or goes on to the hand and the item. A water potion also turns dirt
//! to mud.

use serde_json::Value;

use super::item_on::{north, row};
use super::placing::uses;
use super::{aim_at, using};
use crate::components::{BANNER_PATTERNS, DYED_COLOR, POTION_CONTENTS, patch, vi};
use crate::movement::world::{put, put_with};

/// Stacks the water cauldron cleans: dyed armor, patterned banners and coloured shulker boxes.
fn cauldron_cleaning() -> Vec<Value> {
    let water = |level: &str| vec![put_with(0, 64, 2, "minecraft:water_cauldron", &[("level", level)])];
    let empty = || vec![put(0, 64, 2, "minecraft:cauldron")];
    let held =
        |label: &str, world: Vec<Value>, item: &'static str| row(label, world, &north()).main_hand(item).count(1).row();
    let dyed = patch(&[(DYED_COLOR, "00a06540".to_owned())]);
    let pattern = patch(&[(BANNER_PATTERNS, format!("{}{}{}", vi(1), vi(1), vi(14)))]);
    vec![
        held("item-on-water-cauldron-iron-boots", water("2"), "minecraft:iron_boots"),
        held("item-on-cauldron-iron-boots", empty(), "minecraft:iron_boots"),
        held("item-on-water-cauldron-undyed-leather-boots", water("2"), "minecraft:leather_boots"),
        row("item-on-water-cauldron-dyed-leather-boots", water("2"), &north())
            .main_hand("minecraft:leather_boots")
            .count(1)
            .patch(dyed.clone())
            .row(),
        held("item-on-water-cauldron-undyed-horse-armor", water("2"), "minecraft:leather_horse_armor"),
        row("item-on-water-cauldron-dyed-horse-armor", water("2"), &north())
            .main_hand("minecraft:leather_horse_armor")
            .count(1)
            .patch(dyed.clone())
            .row(),
        row("item-on-cauldron-dyed-horse-armor", empty(), &north())
            .main_hand("minecraft:leather_horse_armor")
            .count(1)
            .patch(dyed)
            .row(),
        held("item-on-water-cauldron-plain-banner", water("2"), "minecraft:white_banner"),
        row("item-on-water-cauldron-banner", water("2"), &north())
            .main_hand("minecraft:white_banner")
            .count(1)
            .patch(pattern.clone())
            .row(),
        row("item-on-cauldron-banner", empty(), &north())
            .main_hand("minecraft:white_banner")
            .count(1)
            .patch(pattern)
            .row(),
        held("item-on-water-cauldron-shulker-box", water("2"), "minecraft:red_shulker_box"),
        held("item-on-water-cauldron-plain-shulker-box", water("2"), "minecraft:shulker_box"),
    ]
}

/// Potions: only water goes into a cauldron that has room.
fn cauldron_potions() -> Vec<Value> {
    let water = |level: &str| vec![put_with(0, 64, 2, "minecraft:water_cauldron", &[("level", level)])];
    let empty = || vec![put(0, 64, 2, "minecraft:cauldron")];
    let water_potion = patch(&[(POTION_CONTENTS, "0100000000".to_owned())]);
    let awkward = patch(&[(POTION_CONTENTS, "0103000000".to_owned())]);
    vec![
        row("item-on-cauldron-water-potion", empty(), &north())
            .main_hand("minecraft:potion")
            .count(1)
            .patch(water_potion.clone())
            .row(),
        row("item-on-cauldron-awkward-potion", empty(), &north())
            .main_hand("minecraft:potion")
            .count(1)
            .patch(awkward.clone())
            .row(),
        row("item-on-cauldron-plain-potion", empty(), &north()).main_hand("minecraft:potion").count(1).row(),
        row("item-on-water-cauldron-water-potion", water("1"), &north())
            .main_hand("minecraft:potion")
            .count(1)
            .patch(water_potion.clone())
            .row(),
        row("item-on-water-cauldron-water-potion-full", water("3"), &north())
            .main_hand("minecraft:potion")
            .count(1)
            .patch(water_potion)
            .row(),
        row("item-on-water-cauldron-awkward-potion", water("1"), &north())
            .main_hand("minecraft:potion")
            .count(1)
            .patch(awkward)
            .row(),
    ]
}

/// Cauldrons: the tables answer on the server, so the client only learns whether the stack was
/// taken (a swing) or goes on to the hand and the item.
fn cauldron_containers() -> Vec<Value> {
    let water = |level: &str| vec![put_with(0, 64, 2, "minecraft:water_cauldron", &[("level", level)])];
    let snow = |level: &str| vec![put_with(0, 64, 2, "minecraft:powder_snow_cauldron", &[("level", level)])];
    let empty = || vec![put(0, 64, 2, "minecraft:cauldron")];
    let lava = || vec![put(0, 64, 2, "minecraft:lava_cauldron")];
    let covered = |mut world: Vec<Value>| {
        world.push(put(0, 65, 2, "minecraft:water"));
        world
    };
    let held = |label: &str, world: Vec<Value>, item: &'static str, count: i64| {
        row(label, world, &north()).main_hand(item).count(count).row()
    };
    vec![
        held("item-on-cauldron-stone", empty(), "minecraft:stone", 4),
        row("item-on-cauldron-empty-hand", empty(), &north()).row(),
        held("item-on-cauldron-water-bucket", empty(), "minecraft:water_bucket", 1),
        held("item-on-cauldron-lava-bucket", empty(), "minecraft:lava_bucket", 1),
        held("item-on-cauldron-lava-bucket-covered", covered(empty()), "minecraft:lava_bucket", 1),
        held("item-on-cauldron-snow-bucket", empty(), "minecraft:powder_snow_bucket", 1),
        held("item-on-cauldron-snow-bucket-covered", covered(empty()), "minecraft:powder_snow_bucket", 1),
        held("item-on-cauldron-water-bucket-covered", covered(empty()), "minecraft:water_bucket", 1),
        held("item-on-cauldron-empty-bucket", empty(), "minecraft:bucket", 1),
        held("item-on-cauldron-glass-bottle", empty(), "minecraft:glass_bottle", 3),
        held("item-on-water-cauldron-stone", water("2"), "minecraft:stone", 4),
        held("item-on-water-cauldron-bucket-full", water("3"), "minecraft:bucket", 1),
        held("item-on-water-cauldron-bucket-partial", water("1"), "minecraft:bucket", 1),
        held("item-on-water-cauldron-bottle", water("1"), "minecraft:glass_bottle", 3),
        held("item-on-water-cauldron-water-bucket", water("1"), "minecraft:water_bucket", 1),
        held("item-on-water-cauldron-lava-bucket", water("1"), "minecraft:lava_bucket", 1),
        held("item-on-water-cauldron-lava-bucket-covered", covered(water("1")), "minecraft:lava_bucket", 1),
        held("item-on-lava-cauldron-bucket", lava(), "minecraft:bucket", 1),
        held("item-on-lava-cauldron-stone", lava(), "minecraft:stone", 4),
        held("item-on-lava-cauldron-water-bucket", lava(), "minecraft:water_bucket", 1),
        held("item-on-snow-cauldron-bucket-full", snow("3"), "minecraft:bucket", 1),
        held("item-on-snow-cauldron-bucket-partial", snow("2"), "minecraft:bucket", 1),
        held("item-on-snow-cauldron-glass-bottle", snow("2"), "minecraft:glass_bottle", 3),
    ]
}

pub fn rows() -> Vec<Value> {
    [cauldron_containers(), cauldron_potions(), cauldron_cleaning(), mud(), beehives()].concat()
}

/// `PotionItem.useOn`: a water potion poured on dirt, not from below.
fn mud() -> Vec<Value> {
    let water = patch(&[(POTION_CONTENTS, "0100000000".to_owned())]);
    let awkward = patch(&[(POTION_CONTENTS, "0103000000".to_owned())]);
    let below = aim_at((0, 64, 2), "down", (0.5, 64.0, 2.5));
    let poured = |label: &str, block: &str, patch: &str| {
        row(label, vec![put(0, 64, 2, block)], &north()).main_hand("minecraft:potion").count(2).patch(patch.to_owned())
    };
    vec![
        poured("item-on-dirt-water-potion", "minecraft:dirt", &water).row(),
        poured("item-on-coarse-dirt-water-potion", "minecraft:coarse_dirt", &water).row(),
        poured("item-on-rooted-dirt-water-potion", "minecraft:rooted_dirt", &water).row(),
        poured("item-on-dirt-awkward-potion", "minecraft:dirt", &awkward).row(),
        poured("item-on-stone-water-potion", "minecraft:stone", &water).row(),
        poured("item-on-dirt-water-potion-creative", "minecraft:dirt", &water).game_mode("creative").row(),
        row("item-on-dirt-water-potion-below", vec![put(0, 64, 2, "minecraft:dirt")], &below)
            .main_hand("minecraft:potion")
            .count(2)
            .patch(water)
            .row(),
    ]
}

/// `BeehiveBlock.useItemOn`: a full hive gives a glass bottle honey; the inventory shows where the
/// honey went.
fn beehives() -> Vec<Value> {
    let aim = north();
    let around = [(0, 64, 1), (0, 64, 2), (0, 65, 2), (0, 64, 3)];
    let ticks =
        [uses(1, Some(&aim), &around).clicks(1).inventory().go(), uses(3, Some(&aim), &around).inventory().go()]
            .concat();
    let hive =
        |block: &str, honey: &str| vec![put_with(0, 64, 2, block, &[("facing", "north"), ("honey_level", honey)])];
    let held = |label: &str, world: Vec<Value>, item: &'static str, count: i64| {
        using(label, world, ticks.clone()).main_hand(item).count(count)
    };
    vec![
        held("item-on-beehive-bottle", hive("minecraft:beehive", "5"), "minecraft:glass_bottle", 3).row(),
        held("item-on-beehive-last-bottle", hive("minecraft:beehive", "5"), "minecraft:glass_bottle", 1).row(),
        held("item-on-bee-nest-bottle", hive("minecraft:bee_nest", "5"), "minecraft:glass_bottle", 3).row(),
        held("item-on-beehive-bottle-creative", hive("minecraft:beehive", "5"), "minecraft:glass_bottle", 3)
            .game_mode("creative")
            .row(),
        held("item-on-beehive-bottle-partial", hive("minecraft:beehive", "4"), "minecraft:glass_bottle", 3).row(),
        held("item-on-beehive-bottle-with-entity", hive("minecraft:beehive", "5"), "minecraft:glass_bottle", 3)
            .block_entity((0, 64, 2), "minecraft:beehive")
            .row(),
        held("item-on-beehive-shears", hive("minecraft:beehive", "5"), "minecraft:shears", 1).row(),
        held("item-on-beehive-stone", hive("minecraft:beehive", "5"), "minecraft:stone", 4).row(),
        using("item-on-beehive-empty-hand", hive("minecraft:beehive", "5"), ticks.clone()).row(),
    ]
}
