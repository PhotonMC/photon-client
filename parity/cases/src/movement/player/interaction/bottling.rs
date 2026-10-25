//! The use key with a glass bottle in hand: the water it reaches fills the bottle on the client.

use serde_json::Value;

use super::placing::uses;
use super::{aim_at, using};
use crate::components::{POTION_CONTENTS, patch};
use crate::movement::world::{put, put_with};

pub fn rows() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3), (0, 64, 1)];
    let run = |aim: Option<&Value>| {
        [uses(1, aim, &around).clicks(1).inventory().go(), uses(3, aim, &around).inventory().go()].concat()
    };
    let pool = || vec![put(0, 64, 2, "minecraft:water")];
    let look = 39.0;
    let bottle = |label: &str, world: Vec<Value>, count: i64| {
        using(label, world, run(Some(&top_of))).main_hand("minecraft:glass_bottle").count(count).x_rot(look)
    };
    vec![
        bottle("bottle-on-water", pool(), 1).row(),
        bottle("bottle-stack-on-water", pool(), 3).row(),
        bottle("bottle-creative-on-water", pool(), 1).game_mode("creative").row(),
        // A creative player gets the bottle only when no stack of it is carried already: the water
        // potion (the first potion of the registry, no colour, effects or name) in the off hand.
        bottle("bottle-creative-already-held", pool(), 1)
            .game_mode("creative")
            .off_hand("minecraft:potion")
            .off_hand_patch(patch(&[(POTION_CONTENTS, "0100000000".to_owned())]))
            .row(),
        bottle("bottle-creative-other-potion-held", pool(), 1).game_mode("creative").off_hand("minecraft:potion").row(),
        bottle("bottle-adventure-on-water", pool(), 1).game_mode("adventure").row(),
        bottle("bottle-on-flowing-water", vec![put_with(0, 64, 2, "minecraft:water", &[("level", "3")])], 1).row(),
        bottle(
            "bottle-on-waterlogged-slab",
            vec![put_with(0, 64, 2, "minecraft:oak_slab", &[("type", "bottom"), ("waterlogged", "true")])],
            1,
        )
        .row(),
        bottle("bottle-on-seagrass", vec![put(0, 64, 2, "minecraft:seagrass")], 1).row(),
        bottle("bottle-on-bubble-column", vec![put_with(0, 64, 2, "minecraft:bubble_column", &[("drag", "true")])], 1)
            .row(),
        bottle("bottle-on-lava", vec![put(0, 64, 2, "minecraft:lava")], 1).row(),
        bottle("bottle-on-floor", vec![], 1).row(),
        using("bottle-at-sky", vec![], run(None)).main_hand("minecraft:glass_bottle").count(1).x_rot(-60.0).row(),
    ]
}
