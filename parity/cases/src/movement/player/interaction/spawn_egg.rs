//! Spawn eggs: an egg of a type that may not spawn in peaceful fails (`EntityType.canSpawn`); the
//! reference's level is peaceful.

use serde_json::Value;

use super::item_on::{north, row};
use super::placing::uses;
use super::using;
use crate::movement::world::put;

pub(super) fn rows() -> Vec<Value> {
    let around = [(0, 64, 1), (0, 64, 2), (0, 65, 2), (0, 64, 3)];
    let mut rows = Vec::new();
    for egg in ["pig", "creeper", "blaze", "villager", "ghast", "slime", "wither", "bat"] {
        let item = format!("minecraft:{egg}_spawn_egg");
        let item: &'static str = Box::leak(item.into_boxed_str());
        rows.push(
            row(&format!("spawn-egg-{egg}-on-block"), vec![put(0, 64, 2, "minecraft:stone")], &north())
                .main_hand(item)
                .count(3)
                .row(),
        );
        let water_ticks = [uses(1, None, &around).clicks(1).go(), uses(3, None, &around).go()].concat();
        rows.push(
            using(&format!("spawn-egg-{egg}-over-water"), vec![put(0, 64, 2, "minecraft:water")], water_ticks)
                .main_hand(item)
                .count(3)
                .x_rot(39.0)
                .row(),
        );
    }
    rows
}
