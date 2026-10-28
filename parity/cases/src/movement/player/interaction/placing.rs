//! The use key as `handleKeybinds` spends it, into the reference's own
//! game mode and block items. The aim is named, with where on the face it
//! lands.

use serde_json::{Value, json};

use super::{aim_at, using};
use crate::components::{CAN_PLACE_ON, adventure_blocks, patch};
use crate::movement::player::scene::keys;
use crate::movement::world::{put, put_with};

/// Ticks of the use key, aimed at a block.
pub(super) struct Uses {
    count: usize,
    aim: Option<Value>,
    clicks: i64,
    down: bool,
    watch: Vec<(i64, i64, i64)>,
    held: &'static [&'static str],
    inventory: bool,
}

pub(super) fn uses(count: usize, aim: Option<&Value>, watch: &[(i64, i64, i64)]) -> Uses {
    Uses { count, aim: aim.cloned(), clicks: 0, down: false, watch: watch.to_vec(), held: &[], inventory: false }
}

impl Uses {
    pub(super) fn clicks(self, clicks: i64) -> Self {
        Self { clicks, ..self }
    }

    pub(super) fn down(self) -> Self {
        Self { down: true, ..self }
    }

    /// Asks the oracles for the inventory's slots in every tick.
    pub(super) fn inventory(self) -> Self {
        Self { inventory: true, ..self }
    }

    pub(super) fn held(self, held: &'static [&'static str]) -> Self {
        Self { held, ..self }
    }

    pub(super) fn go(self) -> Vec<Value> {
        let mut ticks = keys(self.count, self.held);
        let watch: Vec<Value> = self.watch.iter().map(|(x, y, z)| json!({"x": x, "y": y, "z": z})).collect();
        for (index, tick) in ticks.iter_mut().enumerate() {
            let mut attack = json!({
                "clicks": 0, "down": false,
                "use": {"clicks": if index == 0 { self.clicks } else { 0 }, "down": self.down},
                "watch": watch,
            });
            if self.inventory {
                attack["inventory"] = json!(true);
            }
            if let Some(aim) = &self.aim {
                attack["target"] = aim.clone();
            }
            tick["attack"] = attack;
        }
        ticks
    }
}

fn using_rows_1() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3)];
    let click = |aim: &Value, watch: &[(i64, i64, i64)]| uses(1, Some(aim), watch).clicks(1).go();
    let rest = |aim: &Value| uses(3, Some(aim), &around).go();
    let start = |aim: &Value| click(aim, &around);
    vec![
        using("stone-on-floor", vec![], [start(&top_of), uses(12, Some(&top_of), &around).down().go()].concat())
            .main_hand("minecraft:stone")
            .count(4)
            .row(),
        using("empty-hand-on-stone", vec![], [start(&top_of), rest(&top_of)].concat()).row(),
        using(
            "use-in-air",
            vec![],
            [uses(1, None, &around).clicks(1).go(), uses(6, None, &around).down().go()].concat(),
        )
        .main_hand("minecraft:stone")
        .count(4)
        .row(),
        using(
            "slab-bottom-then-double",
            vec![],
            [
                start(&top_of),
                uses(5, Some(&top_of), &around).go(),
                start(&aim_at((0, 64, 2), "up", (0.5, 64.5, 2.5))),
                rest(&top_of),
            ]
            .concat(),
        )
        .main_hand("minecraft:oak_slab")
        .count(4)
        .row(),
    ]
}

fn using_rows_2() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3)];
    let side_high = aim_at((0, 64, 3), "north", (0.5, 64.75, 3.0));
    let side_low = aim_at((0, 64, 3), "north", (0.5, 64.25, 3.0));
    let click = |aim: &Value, watch: &[(i64, i64, i64)]| uses(1, Some(aim), watch).clicks(1).go();
    let rest = |aim: &Value| uses(3, Some(aim), &around).go();
    let start = |aim: &Value| click(aim, &around);
    let wider = |extra: (i64, i64, i64)| [&around[..], &[extra]].concat();
    let stone_wall = || vec![put(0, 64, 3, "minecraft:stone")];
    vec![
        using(
            "slab-on-side-halves",
            stone_wall(),
            [
                start(&side_high),
                uses(5, Some(&side_low), &around).go(),
                click(&aim_at((0, 64, 3), "west", (0.0, 64.25, 3.5)), &wider((-1, 64, 3))),
            ]
            .concat(),
        )
        .main_hand("minecraft:stone_slab")
        .count(4)
        .row(),
        using(
            "log-axis-from-face",
            stone_wall(),
            [
                start(&side_high),
                uses(5, Some(&top_of), &around).go(),
                click(&aim_at((0, 64, 3), "east", (1.0, 64.5, 3.5)), &wider((1, 64, 3))),
            ]
            .concat(),
        )
        .main_hand("minecraft:oak_log")
        .count(4)
        .row(),
        using(
            "stairs-meet-stairs",
            vec![put_with(
                1,
                64,
                2,
                "minecraft:oak_stairs",
                &[("facing", "east"), ("half", "bottom"), ("shape", "straight")],
            )],
            [start(&top_of), rest(&top_of)].concat(),
        )
        .main_hand("minecraft:oak_stairs")
        .count(4)
        .y_rot(90.0)
        .row(),
    ]
}

fn using_rows_3() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3)];
    let side_high = aim_at((0, 64, 3), "north", (0.5, 64.75, 3.0));
    let pool_under: Vec<Value> = (-2..3).flat_map(|x| (1..5).map(move |z| put(x, 64, z, "minecraft:water"))).collect();
    let click = |aim: &Value, watch: &[(i64, i64, i64)]| uses(1, Some(aim), watch).clicks(1).go();
    let rest = |aim: &Value| uses(3, Some(aim), &around).go();
    let start = |aim: &Value| click(aim, &around);
    let stone_wall = || vec![put(0, 64, 3, "minecraft:stone")];
    vec![
        using("stairs-top-half-facing", stone_wall(), [start(&side_high), rest(&top_of)].concat())
            .main_hand("minecraft:stone_stairs")
            .count(4)
            .y_rot(-45.0)
            .row(),
        using(
            "slab-into-water",
            pool_under,
            [start(&aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5))), rest(&top_of)].concat(),
        )
        .main_hand("minecraft:oak_slab")
        .count(4)
        .row(),
        using(
            "place-where-standing",
            vec![],
            [
                click(&aim_at((0, 63, 0), "up", (0.5, 64.0, 0.5)), &[(0, 64, 0)]),
                uses(3, Some(&top_of), &[(0, 64, 0)]).go(),
            ]
            .concat(),
        )
        .main_hand("minecraft:stone")
        .count(4)
        .row(),
        using("creative-keeps-count", vec![], [start(&top_of), rest(&top_of)].concat())
            .main_hand("minecraft:stone")
            .count(4)
            .game_mode("creative")
            .row(),
    ]
}

fn using_rows_4() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3)];
    let click = |aim: &Value, watch: &[(i64, i64, i64)]| uses(1, Some(aim), watch).clicks(1).go();
    let rest = |aim: &Value| uses(3, Some(aim), &around).go();
    let start = |aim: &Value| click(aim, &around);
    let table = || vec![put(0, 64, 2, "minecraft:crafting_table")];
    let on_table = aim_at((0, 64, 2), "up", (0.5, 65.0, 2.5));
    vec![
        using("crafting-table-use", table(), [start(&on_table), rest(&top_of)].concat())
            .main_hand("minecraft:stone")
            .count(4)
            .row(),
        using(
            "sneak-place-on-crafting-table",
            table(),
            [
                // The crouch takes hold a tick after its key goes down.
                uses(2, None, &around).held(&["sneak"]).go(),
                uses(1, Some(&on_table), &around).clicks(1).held(&["sneak"]).go(),
                uses(3, Some(&top_of), &around).held(&["sneak"]).go(),
            ]
            .concat(),
        )
        .main_hand("minecraft:stone")
        .count(4)
        .row(),
        using("off-hand-places", vec![], [start(&top_of), rest(&top_of)].concat())
            .off_hand("minecraft:cobblestone")
            .row(),
        using("adventure-cannot-place", vec![], [start(&top_of), rest(&top_of)].concat())
            .main_hand("minecraft:stone")
            .count(4)
            .game_mode("adventure")
            .row(),
        using("adventure-can-place-on-stone", vec![], [start(&top_of), rest(&top_of)].concat())
            .main_hand("minecraft:stone")
            .count(4)
            .game_mode("adventure")
            .patch(patch(&[(CAN_PLACE_ON, adventure_blocks(&[1]))]))
            .row(),
        using("adventure-can-place-on-dirt", vec![], [start(&top_of), rest(&top_of)].concat())
            .main_hand("minecraft:stone")
            .count(4)
            .game_mode("adventure")
            .patch(patch(&[(CAN_PLACE_ON, adventure_blocks(&[9]))]))
            .row(),
        using("place-last-block", vec![], [start(&top_of), rest(&top_of)].concat())
            .main_hand("minecraft:stone")
            .count(1)
            .row(),
    ]
}

fn bucket_rows() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3), (0, 64, 1)];
    let pool = || vec![put(0, 64, 2, "minecraft:water")];
    let lava = || vec![put(0, 64, 2, "minecraft:lava")];
    let run = |aim: &Value| {
        [uses(1, Some(aim), &around).clicks(1).inventory().go(), uses(3, Some(aim), &around).inventory().go()].concat()
    };
    let look = 39.0;
    vec![
        using("water-bucket-on-floor", vec![], run(&top_of))
            .main_hand("minecraft:water_bucket")
            .count(1)
            .x_rot(look)
            .row(),
        using("water-bucket-creative", vec![], run(&top_of))
            .main_hand("minecraft:water_bucket")
            .count(1)
            .game_mode("creative")
            .x_rot(look)
            .row(),
        using("lava-bucket-on-floor", vec![], run(&top_of))
            .main_hand("minecraft:lava_bucket")
            .count(1)
            .x_rot(look)
            .row(),
        using("empty-bucket-on-water", pool(), run(&top_of)).main_hand("minecraft:bucket").count(1).x_rot(look).row(),
        using("empty-bucket-on-lava", lava(), run(&top_of)).main_hand("minecraft:bucket").count(1).x_rot(look).row(),
        using("empty-bucket-stack-on-water", pool(), run(&top_of))
            .main_hand("minecraft:bucket")
            .count(3)
            .x_rot(look)
            .row(),
        using("empty-bucket-creative", pool(), run(&top_of))
            .main_hand("minecraft:bucket")
            .count(1)
            .game_mode("creative")
            .x_rot(look)
            .row(),
        using("water-bucket-on-water", pool(), run(&top_of))
            .main_hand("minecraft:water_bucket")
            .count(1)
            .x_rot(look)
            .row(),
        using(
            "water-bucket-into-slab",
            vec![put_with(0, 64, 2, "minecraft:oak_slab", &[("type", "bottom"), ("waterlogged", "false")])],
            run(&top_of),
        )
        .main_hand("minecraft:water_bucket")
        .count(1)
        .x_rot(look)
        .row(),
    ]
}

fn bucket_rows_2() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3), (0, 64, 1)];
    let run = |aim: &Value| {
        [uses(1, Some(aim), &around).clicks(1).inventory().go(), uses(3, Some(aim), &around).inventory().go()].concat()
    };
    let look = 39.0;
    let sneak_watch: Vec<(i64, i64, i64)> = (63..67).flat_map(|y| (-1..4).map(move |z| (0, y, z))).collect();
    vec![
        using("water-bucket-stack", vec![], run(&top_of))
            .main_hand("minecraft:water_bucket")
            .count(2)
            .x_rot(look)
            .row(),
        using(
            "empty-bucket-on-waterlogged-slab",
            vec![put_with(0, 64, 2, "minecraft:oak_slab", &[("type", "bottom"), ("waterlogged", "true")])],
            run(&top_of),
        )
        .main_hand("minecraft:bucket")
        .count(1)
        .x_rot(look)
        .row(),
        using(
            "empty-bucket-on-flowing-water",
            vec![put_with(0, 64, 2, "minecraft:water", &[("level", "3")])],
            run(&top_of),
        )
        .main_hand("minecraft:bucket")
        .count(1)
        .x_rot(look)
        .row(),
        using(
            "sneak-water-bucket-into-slab",
            vec![put_with(0, 64, 2, "minecraft:oak_slab", &[("type", "bottom"), ("waterlogged", "false")])],
            [
                uses(3, Some(&top_of), &sneak_watch).held(&["sneak"]).go(),
                uses(1, Some(&top_of), &sneak_watch).clicks(1).held(&["sneak"]).go(),
                uses(3, Some(&top_of), &sneak_watch).held(&["sneak"]).go(),
            ]
            .concat(),
        )
        .main_hand("minecraft:water_bucket")
        .count(1)
        .x_rot(look)
        .row(),
        using("cod-bucket-on-floor", vec![], run(&top_of)).main_hand("minecraft:cod_bucket").count(1).x_rot(look).row(),
        using("sulfur-cube-bucket-on-water", vec![put(0, 64, 2, "minecraft:water")], run(&top_of))
            .main_hand("minecraft:sulfur_cube_bucket")
            .count(1)
            .x_rot(look)
            .row(),
    ]
}

fn bucket_rows_3() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3), (0, 64, 1)];
    let run = |aim: &Value| {
        [uses(1, Some(aim), &around).clicks(1).inventory().go(), uses(3, Some(aim), &around).inventory().go()].concat()
    };
    let look = 39.0;
    vec![
        using(
            "water-bucket-into-lit-candle",
            vec![put_with(
                0,
                64,
                2,
                "minecraft:candle",
                &[("candles", "1"), ("lit", "true"), ("waterlogged", "false")],
            )],
            run(&top_of),
        )
        .main_hand("minecraft:water_bucket")
        .count(1)
        .x_rot(look)
        .row(),
        using(
            "water-bucket-into-campfire",
            vec![put_with(0, 64, 2, "minecraft:campfire", &[("lit", "true"), ("waterlogged", "false")])],
            run(&top_of),
        )
        .main_hand("minecraft:water_bucket")
        .count(1)
        .x_rot(look)
        .row(),
        using("water-bucket-adventure", vec![], run(&top_of))
            .main_hand("minecraft:water_bucket")
            .count(1)
            .game_mode("adventure")
            .x_rot(look)
            .row(),
        using("water-bucket-adventure-can-place-on", vec![], run(&top_of))
            .main_hand("minecraft:water_bucket")
            .count(1)
            .game_mode("adventure")
            .patch(patch(&[(CAN_PLACE_ON, adventure_blocks(&[1]))]))
            .x_rot(look)
            .row(),
        using("water-bucket-adventure-other-block", vec![], run(&top_of))
            .main_hand("minecraft:water_bucket")
            .count(1)
            .game_mode("adventure")
            .patch(patch(&[(CAN_PLACE_ON, adventure_blocks(&[9]))]))
            .x_rot(look)
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [using_rows_1(), using_rows_2(), using_rows_3(), using_rows_4(), bucket_rows(), bucket_rows_2(), bucket_rows_3()]
        .concat()
}
