//! The copper blocks' weathering and waxing, by axe and by honeycomb.

use serde_json::Value;

use super::item_on::{north, row};
use crate::movement::world::{put, put_with};

pub(super) fn copper_blocks() -> Vec<Value> {
    let stairs = |block: &str| {
        vec![put_with(
            0,
            64,
            2,
            block,
            &[("facing", "east"), ("half", "top"), ("shape", "straight"), ("waterlogged", "false")],
        )]
    };
    let plain = |block: &str| vec![put(0, 64, 2, block)];
    vec![
        row("item-on-copper-axe-scrape", plain("minecraft:oxidized_copper"), &north())
            .main_hand("minecraft:iron_axe")
            .row(),
        row("item-on-copper-axe-scrape-stairs", stairs("minecraft:weathered_cut_copper_stairs"), &north())
            .main_hand("minecraft:iron_axe")
            .row(),
        row("item-on-copper-axe-unaffected", plain("minecraft:copper_block"), &north())
            .main_hand("minecraft:iron_axe")
            .row(),
        row("item-on-copper-axe-wax-off", plain("minecraft:waxed_exposed_copper"), &north())
            .main_hand("minecraft:iron_axe")
            .row(),
        row("item-on-copper-axe-wax-off-stairs", stairs("minecraft:waxed_cut_copper_stairs"), &north())
            .main_hand("minecraft:iron_axe")
            .row(),
        row("item-on-copper-honeycomb", plain("minecraft:copper_block"), &north())
            .main_hand("minecraft:honeycomb")
            .count(3)
            .row(),
        row("item-on-copper-honeycomb-aged-stairs", stairs("minecraft:oxidized_cut_copper_stairs"), &north())
            .main_hand("minecraft:honeycomb")
            .count(1)
            .row(),
        row("item-on-copper-honeycomb-waxed", plain("minecraft:waxed_copper_block"), &north())
            .main_hand("minecraft:honeycomb")
            .count(3)
            .row(),
        row("item-on-copper-honeycomb-creative", plain("minecraft:copper_block"), &north())
            .main_hand("minecraft:honeycomb")
            .count(3)
            .game_mode("creative")
            .row(),
        row("item-on-copper-honeycomb-stone", plain("minecraft:stone"), &north())
            .main_hand("minecraft:honeycomb")
            .count(3)
            .row(),
    ]
}
