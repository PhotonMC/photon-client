//! The use key on blocks that answer to the stack in the hand (`useItemOn`): the result the client
//! sees, told by whether a block goes down beside the face, and the state the click sets.

use serde_json::Value;

use super::placing::uses;
use super::{aim_at, using};
use crate::movement::world::{put, put_with};

/// A use of the block at (0, 64, 2) aimed at, with the stack the row holds.
fn row(label: &str, world: Vec<Value>, aim: &Value) -> super::Interaction {
    let around = [(0, 64, 1), (0, 64, 2), (0, 65, 2), (0, 64, 3)];
    let ticks = [uses(1, Some(aim), &around).clicks(1).go(), uses(3, Some(aim), &around).go()].concat();
    using(label, world, ticks)
}

fn north() -> Value {
    aim_at((0, 64, 2), "north", (0.5, 64.5, 2.0))
}

fn on_top() -> Value {
    aim_at((0, 64, 2), "up", (0.5, 65.0, 2.5))
}

fn simple() -> Vec<Value> {
    let note = || {
        vec![put_with(0, 64, 2, "minecraft:note_block", &[("instrument", "harp"), ("note", "0"), ("powered", "false")])]
    };
    let tnt = || vec![put_with(0, 64, 2, "minecraft:tnt", &[("unstable", "false")])];
    let pumpkin = || vec![put(0, 64, 2, "minecraft:pumpkin")];
    let vault = |state: &str| {
        vec![put_with(
            0,
            64,
            2,
            "minecraft:vault",
            &[("facing", "north"), ("ominous", "false"), ("vault_state", state)],
        )]
    };
    vec![
        row("item-on-note-block-top-instrument", note(), &on_top()).main_hand("minecraft:zombie_head").count(2).row(),
        row("item-on-note-block-side-instrument", note(), &north()).main_hand("minecraft:zombie_head").count(2).row(),
        row("item-on-note-block-stone", note(), &north()).main_hand("minecraft:stone").count(4).row(),
        row("item-on-pumpkin-shears", pumpkin(), &north()).main_hand("minecraft:shears").row(),
        row("item-on-pumpkin-stone", pumpkin(), &north()).main_hand("minecraft:stone").count(4).row(),
        row("item-on-tnt-flint", tnt(), &north()).main_hand("minecraft:flint_and_steel").row(),
        row("item-on-tnt-fire-charge", tnt(), &north()).main_hand("minecraft:fire_charge").count(3).row(),
        row("item-on-tnt-stone", tnt(), &north()).main_hand("minecraft:stone").count(4).row(),
        row("item-on-vault-active-stone", vault("active"), &north()).main_hand("minecraft:stone").count(4).row(),
        row("item-on-vault-inactive-stone", vault("inactive"), &north()).main_hand("minecraft:stone").count(4).row(),
        row("item-on-vault-active-empty", vault("active"), &north()).row(),
    ]
}

fn lit_candles() -> Vec<Value> {
    let candle = |lit: &str| {
        vec![put_with(0, 64, 2, "minecraft:candle", &[("candles", "1"), ("lit", lit), ("waterlogged", "false")])]
    };
    let cake = |lit: &str| vec![put_with(0, 64, 2, "minecraft:white_candle_cake", &[("lit", lit)])];
    let candle_top = aim_at((0, 64, 2), "up", (0.5, 64.375, 2.5));
    let high = aim_at((0, 64, 2), "north", (0.5, 64.7, 2.4375));
    let low = aim_at((0, 64, 2), "north", (0.5, 64.3, 2.0625));
    vec![
        row("item-on-candle-lit-empty", candle("true"), &candle_top).row(),
        row("item-on-candle-unlit-empty", candle("false"), &candle_top).row(),
        row("item-on-candle-lit-stone", candle("true"), &candle_top).main_hand("minecraft:stone").count(4).row(),
        row("item-on-candle-lit-adventure", candle("true"), &candle_top).game_mode("adventure").row(),
        row("item-on-candle-lit-flint", candle("true"), &candle_top).main_hand("minecraft:flint_and_steel").row(),
        row("item-on-candle-cake-lit-high", cake("true"), &high).row(),
        row("item-on-candle-cake-lit-low", cake("true"), &low).row(),
        row("item-on-candle-cake-unlit-high", cake("false"), &high).row(),
        row("item-on-candle-cake-lit-flint", cake("true"), &high).main_hand("minecraft:flint_and_steel").row(),
    ]
}

fn anchors_and_statues() -> Vec<Value> {
    let anchor = |charges: &str| vec![put_with(0, 64, 2, "minecraft:respawn_anchor", &[("charges", charges)])];
    let statue = |pose: &str| {
        vec![put_with(
            0,
            64,
            2,
            "minecraft:waxed_copper_golem_statue",
            &[("copper_golem_pose", pose), ("facing", "north"), ("waterlogged", "false")],
        )]
    };
    vec![
        row("item-on-anchor-charges", anchor("1"), &north()).main_hand("minecraft:glowstone").count(3).row(),
        row("item-on-anchor-full", anchor("4"), &north()).main_hand("minecraft:glowstone").count(3).row(),
        row("item-on-anchor-full-offhand", anchor("4"), &north())
            .main_hand("minecraft:stone")
            .count(3)
            .off_hand("minecraft:glowstone")
            .row(),
        row("item-on-anchor-offhand-charges", anchor("2"), &north())
            .main_hand("minecraft:stone")
            .count(3)
            .off_hand("minecraft:glowstone")
            .row(),
        row("item-on-anchor-stone", anchor("2"), &north()).main_hand("minecraft:stone").count(3).row(),
        row("item-on-statue-pose-standing", statue("standing"), &north()).main_hand("minecraft:stone").count(3).row(),
        row("item-on-statue-pose-star", statue("star"), &north()).main_hand("minecraft:stone").count(3).row(),
        row("item-on-statue-empty", statue("running"), &north()).row(),
    ]
}

fn records_and_books() -> Vec<Value> {
    let jukebox = |has: &str| vec![put_with(0, 64, 2, "minecraft:jukebox", &[("has_record", has)])];
    let lectern = |has: &str| {
        vec![put_with(0, 64, 2, "minecraft:lectern", &[("facing", "north"), ("has_book", has), ("powered", "false")])]
    };
    let (at, kind) = ((0, 64, 2), "minecraft:jukebox");
    vec![
        row("item-on-jukebox-record", jukebox("false"), &north())
            .main_hand("minecraft:music_disc_13")
            .block_entity(at, kind)
            .row(),
        row("item-on-jukebox-stone", jukebox("false"), &north())
            .main_hand("minecraft:stone")
            .count(4)
            .block_entity(at, kind)
            .row(),
        row("item-on-jukebox-full-record", jukebox("true"), &north())
            .main_hand("minecraft:music_disc_13")
            .block_entity(at, kind)
            .row(),
        row("item-on-jukebox-full-stone", jukebox("true"), &north())
            .main_hand("minecraft:stone")
            .count(4)
            .block_entity(at, kind)
            .row(),
        row("item-on-jukebox-full-empty", jukebox("true"), &north()).block_entity(at, kind).row(),
        row("item-on-lectern-book", lectern("false"), &north()).main_hand("minecraft:writable_book").row(),
        row("item-on-lectern-stone", lectern("false"), &north()).main_hand("minecraft:stone").count(4).row(),
        row("item-on-lectern-empty", lectern("false"), &north()).row(),
        row("item-on-lectern-full-book", lectern("true"), &north()).main_hand("minecraft:writable_book").row(),
    ]
}

fn cakes() -> Vec<Value> {
    let cake = |bites: &str| vec![put_with(0, 64, 2, "minecraft:cake", &[("bites", bites)])];
    let low = aim_at((0, 64, 2), "north", (0.5, 64.3, 2.0));
    vec![
        row("item-on-cake-empty-full", cake("0"), &north()).row(),
        row("item-on-cake-stone-full", cake("0"), &north()).main_hand("minecraft:stone").count(4).row(),
        row("item-on-cake-creative-eats", cake("2"), &north()).game_mode("creative").row(),
        row("item-on-cake-creative-last-bite", cake("6"), &north()).game_mode("creative").row(),
        row("item-on-cake-candle", cake("0"), &north()).main_hand("minecraft:red_candle").count(2).row(),
        row("item-on-cake-plain-candle", cake("0"), &north()).main_hand("minecraft:candle").count(2).row(),
        row("item-on-cake-candle-bitten", cake("1"), &north()).main_hand("minecraft:red_candle").count(2).row(),
        row("item-on-cake-candle-creative", cake("0"), &north())
            .main_hand("minecraft:red_candle")
            .count(2)
            .game_mode("creative")
            .row(),
        row(
            "item-on-candle-cake-creative-eats",
            vec![put_with(0, 64, 2, "minecraft:white_candle_cake", &[("lit", "false")])],
            &low,
        )
        .game_mode("creative")
        .row(),
    ]
}

/// Blocks that answer from their block entity, which the reference always has and the client has
/// when the server sent it.
fn with_block_entities() -> Vec<Value> {
    let at = (0, 64, 2);
    let statue = |pose: &str| {
        vec![put_with(
            0,
            64,
            2,
            "minecraft:copper_golem_statue",
            &[("copper_golem_pose", pose), ("facing", "north"), ("waterlogged", "false")],
        )]
    };
    let pot = || {
        vec![put_with(
            0,
            64,
            2,
            "minecraft:decorated_pot",
            &[("cracked", "false"), ("facing", "north"), ("waterlogged", "false")],
        )]
    };
    let statue_kind = "minecraft:copper_golem_statue";
    let pot_kind = "minecraft:decorated_pot";
    vec![
        row("item-on-weathering-statue-stone", statue("sitting"), &north())
            .main_hand("minecraft:stone")
            .count(3)
            .block_entity(at, statue_kind)
            .row(),
        row("item-on-weathering-statue-star", statue("star"), &north()).block_entity(at, statue_kind).row(),
        row("item-on-pot-stone", pot(), &north())
            .main_hand("minecraft:stone")
            .count(3)
            .block_entity(at, pot_kind)
            .row(),
        row("item-on-pot-empty", pot(), &north()).block_entity(at, pot_kind).row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [simple(), lit_candles(), anchors_and_statues(), records_and_books(), cakes(), with_block_entities()].concat()
}
