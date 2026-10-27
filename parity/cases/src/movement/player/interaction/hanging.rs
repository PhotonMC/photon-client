//! Hanging signs under a click: the editing and the chaining of another hanging sign.

use serde_json::Value;

use super::aim_at;
use super::item_on::{north, row, row_around};
use crate::movement::world::put_with;

fn below() -> Value {
    aim_at((0, 64, 2), "down", (0.5, 64.0, 2.5))
}

pub(super) fn hanging_signs() -> Vec<Value> {
    [editing(), chaining()].concat()
}

fn hanging() -> Vec<Value> {
    vec![put_with(
        0,
        64,
        2,
        "minecraft:oak_hanging_sign",
        &[("attached", "false"), ("rotation", "0"), ("waterlogged", "false")],
    )]
}

fn wall() -> Vec<Value> {
    vec![put_with(0, 64, 2, "minecraft:oak_wall_hanging_sign", &[("facing", "north"), ("waterlogged", "false")])]
}

fn editing() -> Vec<Value> {
    let at = (0, 64, 2);
    let kind = "minecraft:hanging_sign";
    vec![
        row("item-on-hanging-sign-stone", hanging(), &north())
            .main_hand("minecraft:stone")
            .count(4)
            .block_entity(at, kind)
            .row(),
        row("item-on-hanging-sign-dye", hanging(), &north())
            .main_hand("minecraft:black_dye")
            .count(4)
            .block_entity(at, kind)
            .row(),
        row("item-on-wall-hanging-sign-empty", wall(), &north()).block_entity(at, kind).row(),
        row("item-on-wall-hanging-sign-stone", wall(), &north())
            .main_hand("minecraft:stone")
            .count(4)
            .block_entity(at, kind)
            .row(),
    ]
}

fn chaining() -> Vec<Value> {
    let at = (0, 64, 2);
    let kind = "minecraft:hanging_sign";
    vec![
        row_around(
            "item-on-hanging-sign-chain",
            hanging(),
            &below(),
            &[(0, 63, 2), (0, 64, 2), (0, 64, 1), (0, 64, 3)],
        )
        .main_hand("minecraft:oak_hanging_sign")
        .count(4)
        .block_entity(at, kind)
        .row(),
        row("item-on-hanging-sign-chain-side", hanging(), &north())
            .main_hand("minecraft:oak_hanging_sign")
            .count(4)
            .block_entity(at, kind)
            .row(),
        row_around(
            "item-on-hanging-sign-chain-stone",
            hanging(),
            &below(),
            &[(0, 63, 2), (0, 64, 2), (0, 64, 1), (0, 64, 3)],
        )
        .main_hand("minecraft:stone")
        .count(4)
        .block_entity(at, kind)
        .row(),
        row("item-on-wall-hanging-sign-chain-side", wall(), &aim_at((0, 64, 2), "east", (1.0, 64.5, 2.5)))
            .main_hand("minecraft:oak_hanging_sign")
            .count(4)
            .block_entity(at, kind)
            .row(),
        row("item-on-wall-hanging-sign-chain-front", wall(), &north())
            .main_hand("minecraft:oak_hanging_sign")
            .count(4)
            .block_entity(at, kind)
            .row(),
    ]
}
