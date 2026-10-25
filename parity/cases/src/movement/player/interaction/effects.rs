//! The effects and attributes that move the player: a potion's speed comes as the server's
//! attribute modifiers, while jump boost, levitation, slow falling and dolphin's grace are read
//! from the effects themselves.

use serde_json::Value;

use super::breaking;
use crate::movement::player::scene::keys;
use crate::movement::world::{cat, column, put};

/// `MobEffect`'s modifier for an amplifier: the effect's float amount times the level.
fn effect_amount(amount: f32, amplifier: i64) -> f64 {
    f64::from(amount) * (amplifier + 1) as f64
}

fn pool() -> Vec<Value> {
    (-6..7)
        .flat_map(|x| (-6..7).map(move |z| (x, z)))
        .flat_map(|(x, z)| (64..67).map(move |y| (x, y, z)))
        .map(|(x, y, z)| put(x, y, z, "minecraft:water"))
        .collect()
}

/// A step one block high across the way ahead.
fn ledge() -> Vec<Value> {
    (-3..4).flat_map(|x| column(x, 3, 64, 64)).collect()
}

pub fn rows() -> Vec<Value> {
    [speed_rows(), vertical_rows(), other_rows()].concat()
}

/// The movement speed: a potion's modifiers and the sprinting one, from the server and the client.
fn speed_rows() -> Vec<Value> {
    let speed = |amplifier| ("minecraft:effect.speed", effect_amount(0.2, amplifier), 2);
    let slowness = ("minecraft:effect.slowness", effect_amount(-0.15, 0), 2);
    let sprinting = ("minecraft:sprinting", f64::from(0.3_f32), 2);
    let walk_speed = f64::from(0.1_f32);
    vec![
        breaking("speed-ii-walk", vec![], keys(15, &["forward"]))
            .attribute("minecraft:movement_speed", walk_speed, &[speed(1)])
            .row(),
        breaking("speed-ii-sprint", vec![], keys(15, &["forward", "sprint"]))
            .attribute("minecraft:movement_speed", walk_speed, &[speed(1)])
            .row(),
        breaking("slowness-walk", vec![], keys(15, &["forward"]))
            .attribute("minecraft:movement_speed", walk_speed, &[slowness])
            .row(),
        // The server's sprinting modifier stays on until the client's own sprint changes it.
        breaking("server-sprint-modifier-walk", vec![], keys(15, &["forward"]))
            .attribute("minecraft:movement_speed", walk_speed, &[sprinting])
            .row(),
        breaking(
            "server-sprint-modifier-then-sprint",
            vec![],
            [keys(5, &["forward"]), keys(10, &["forward", "sprint"]), keys(6, &[])].concat(),
        )
        .attribute("minecraft:movement_speed", walk_speed, &[speed(0), sprinting])
        .row(),
    ]
}

/// Jumps and falls: jump boost, levitation, slow falling and the attributes they read.
fn vertical_rows() -> Vec<Value> {
    vec![
        breaking("jump-boost-ii-jump", vec![], keys(20, &["jump"])).effects(&[("minecraft:jump_boost", 1)]).row(),
        breaking("jump-boost-sprint-jump", vec![], keys(20, &["forward", "sprint", "jump"]))
            .effects(&[("minecraft:jump_boost", 0)])
            .row(),
        breaking("jump-strength-base", vec![], keys(20, &["jump"]))
            .attribute("minecraft:jump_strength", 0.6, &[])
            .row(),
        breaking("levitation-i", vec![], keys(25, &[])).effects(&[("minecraft:levitation", 0)]).row(),
        breaking("levitation-iii-forward", vec![], keys(20, &["forward"]))
            .effects(&[("minecraft:levitation", 2)])
            .row(),
        breaking("slow-falling-drop", vec![], keys(30, &[]))
            .start(0.5, 72.0, 0.5)
            .effects(&[("minecraft:slow_falling", 0)])
            .row(),
        breaking("slow-falling-jump", vec![], keys(25, &["jump"])).effects(&[("minecraft:slow_falling", 0)]).row(),
        breaking("low-gravity-jump", vec![], keys(30, &["jump"])).attribute("minecraft:gravity", 0.02, &[]).row(),
    ]
}

/// Swimming, steps, sneaking and drag.
fn other_rows() -> Vec<Value> {
    vec![
        breaking("dolphins-grace-swim", pool(), keys(20, &["forward"]))
            .start(0.5, 64.5, 0.5)
            .effects(&[("minecraft:dolphins_grace", 0)])
            .row(),
        breaking("dolphins-grace-sprint-swim", pool(), keys(24, &["forward", "sprint"]))
            .start(0.5, 64.5, 0.5)
            .effects(&[("minecraft:dolphins_grace", 0)])
            .row(),
        breaking("depth-strider-walk", pool(), keys(20, &["forward"]))
            .start(0.5, 64.0, 0.5)
            .attribute(
                "minecraft:water_movement_efficiency",
                0.0,
                &[("minecraft:enchantment.depth_strider", f64::from(0.333_333_34_f32) * 3.0, 0)],
            )
            .row(),
        breaking("step-height-ledge", cat![ledge()], keys(20, &["forward"]))
            .attribute("minecraft:step_height", 1.0, &[])
            .row(),
        breaking("sneaking-speed-walk", vec![], keys(15, &["forward", "sneak"]))
            .attribute("minecraft:sneaking_speed", 0.3, &[("minecraft:enchantment.swift_sneak", 0.45, 0)])
            .row(),
        breaking("air-drag-modifier-jump", vec![], keys(20, &["forward", "jump"]))
            .attribute("minecraft:air_drag_modifier", 0.5, &[])
            .row(),
    ]
}
