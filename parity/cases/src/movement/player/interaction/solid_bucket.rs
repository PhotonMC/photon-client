//! The powder snow bucket, a block item that leaves an empty bucket in the hand.

use serde_json::Value;

use super::placing::uses;
use super::{aim_at, using};
use crate::movement::world::put;

pub fn rows() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3), (0, 64, 1)];
    let run = || {
        [uses(1, Some(&top_of), &around).clicks(1).inventory().go(), uses(3, Some(&top_of), &around).inventory().go()]
            .concat()
    };
    let bucket = "minecraft:powder_snow_bucket";
    vec![
        using("powder-snow-bucket-floor", vec![], run()).main_hand(bucket).count(1).x_rot(39.0).row(),
        using("powder-snow-bucket-floor-creative", vec![], run())
            .main_hand(bucket)
            .count(1)
            .x_rot(39.0)
            .game_mode("creative")
            .row(),
        using("powder-snow-bucket-blocked", vec![put(0, 64, 2, "minecraft:stone")], run())
            .main_hand(bucket)
            .count(1)
            .x_rot(39.0)
            .row(),
    ]
}
