//! The attack key as `handleKeybinds` spends it, into the reference's own
//! game mode. The target is named, not picked. Each tick reports the game
//! mode's destroy state, the crack stage the level keeps, the prediction
//! sequence and the watched block states.

use serde_json::{Value, json};

use super::{breaking, place};
use crate::movement::player::scene::keys;
use crate::movement::world::{cat, put, put_with};

/// Ticks of the attack key held, clicked or released.
struct Hits {
    count: usize,
    aim: Option<Value>,
    down: bool,
    clicks: i64,
    server: Option<Vec<Value>>,
    held: &'static [&'static str],
    watch: Vec<Value>,
}

fn target() -> Value {
    json!({"x": 0, "y": 64, "z": 2, "face": "north"})
}

fn second() -> Value {
    json!({"x": 1, "y": 64, "z": 2, "face": "north"})
}

fn hits(count: usize) -> Hits {
    let watch = [target(), second()].iter().map(|w| json!({"x": w["x"], "y": w["y"], "z": w["z"]})).collect();
    Hits { count, aim: Some(target()), down: true, clicks: 0, server: None, held: &[], watch }
}

impl Hits {
    fn clicks(self, clicks: i64) -> Self {
        Self { clicks, ..self }
    }

    fn aim(self, aim: Option<Value>) -> Self {
        Self { aim, ..self }
    }

    fn released(self) -> Self {
        Self { down: false, ..self }
    }

    fn server(self, server: Vec<Value>) -> Self {
        Self { server: Some(server), ..self }
    }

    fn held(self, held: &'static [&'static str]) -> Self {
        Self { held, ..self }
    }

    fn go(self) -> Vec<Value> {
        let mut ticks = keys(self.count, self.held);
        for (index, tick) in ticks.iter_mut().enumerate() {
            let mut attack = json!({
                "clicks": if index == 0 { self.clicks } else { 0 }, "down": self.down, "watch": self.watch,
            });
            if let Some(aim) = &self.aim {
                attack["target"] = aim.clone();
            }
            if let (Some(server), 0) = (&self.server, index) {
                attack["server"] = json!(server);
            }
            tick["attack"] = attack;
        }
        ticks
    }
}

fn breaking_rows_1() -> Vec<Value> {
    let air = || json!({"x": 0, "y": 64, "z": 2, "block": "minecraft:air"});
    let dirt = || vec![place(&target(), "minecraft:dirt")];
    let stone = || vec![place(&target(), "minecraft:stone")];
    let first_click = || hits(1).clicks(1).go();
    let hold = |count: usize| hits(count).go();
    let two_dirt = || cat![dirt(), vec![place(&second(), "minecraft:dirt")]];
    vec![
        breaking(
            "dirt-bare-hand",
            dirt(),
            [first_click(), hold(19), hits(1).server(vec![air(), json!({"ack": 2})]).go(), hold(3)].concat(),
        )
        .row(),
        breaking("stone-wooden-pickaxe", stone(), [first_click(), hold(26)].concat())
            .main_hand("minecraft:wooden_pickaxe")
            .row(),
        breaking("stone-wooden-shovel", stone(), [first_click(), hold(12)].concat())
            .main_hand("minecraft:wooden_shovel")
            .row(),
        breaking("stone-release", stone(), [first_click(), hold(9), hits(3).released().go()].concat()).row(),
        breaking("switch-target", two_dirt(), [first_click(), hold(5), hits(20).aim(Some(second())).go()].concat())
            .row(),
        breaking(
            "dirt-rejected",
            dirt(),
            [
                first_click(),
                hold(16),
                hits(1).server(vec![place(&target(), "minecraft:dirt"), json!({"ack": 2})]).go(),
                hold(4),
            ]
            .concat(),
        )
        .row(),
        breaking("dirt-haste", dirt(), [first_click(), hold(12)].concat()).effects(&[("minecraft:haste", 1)]).row(),
        breaking("dirt-mining-fatigue", dirt(), [first_click(), hold(12)].concat())
            .effects(&[("minecraft:mining_fatigue", 0)])
            .row(),
        breaking(
            "dirt-jumping",
            dirt(),
            [hits(1).clicks(1).held(&["jump"]).go(), hits(20).held(&["jump"]).go()].concat(),
        )
        .row(),
    ]
}

fn breaking_rows_2() -> Vec<Value> {
    let dirt = || vec![place(&target(), "minecraft:dirt")];
    let pool: Vec<Value> = (-3..4)
        .flat_map(|x| (-3..2).map(move |z| (x, z)))
        .flat_map(|(x, z)| (64..67).map(move |y| (x, y, z)))
        .map(|(x, y, z)| put(x, y, z, "minecraft:water"))
        .collect();
    let first_click = || hits(1).clicks(1).go();
    let hold = |count: usize| hits(count).go();
    let two_dirt = || cat![dirt(), vec![place(&second(), "minecraft:dirt")]];
    vec![
        breaking("dirt-underwater", cat![dirt(), pool], [first_click(), hold(20)].concat()).start(0.5, 64.0, 0.5).row(),
        breaking(
            "waterlogged-slab-axe",
            vec![put_with(0, 64, 2, "minecraft:oak_slab", &[("waterlogged", "true")])],
            [first_click(), hold(34)].concat(),
        )
        .main_hand("minecraft:wooden_axe")
        .row(),
        breaking(
            "miss-then-hold",
            dirt(),
            [
                hits(1).aim(None).clicks(1).go(),
                hits(3).aim(None).clicks(1).go(),
                hits(8).clicks(1).go(),
                hits(2).released().go(),
                first_click(),
                hold(3),
            ]
            .concat(),
        )
        .row(),
        breaking("creative-hold", two_dirt(), [first_click(), hold(3), hits(8).aim(Some(second())).go()].concat())
            .game_mode("creative")
            .row(),
        breaking("creative-sword", dirt(), [first_click(), hold(6)].concat())
            .main_hand("minecraft:wooden_sword")
            .game_mode("creative")
            .row(),
        breaking("adventure-bare-hand", dirt(), [first_click(), hold(4)].concat()).game_mode("adventure").row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [breaking_rows_1(), breaking_rows_2()].concat()
}
