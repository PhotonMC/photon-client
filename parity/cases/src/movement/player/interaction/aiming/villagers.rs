//! The use key on villagers, wandering traders and zombie villagers: the client answers the
//! click with a swing and the server does the trading.

use serde_json::Value;

use super::herding::flagged_animal;
use super::{aims, facing};
use crate::movement::player::interaction::Interaction;

pub(super) fn rows() -> Vec<Value> {
    let at = |label: &str, kind: &str, flags: &[&str]| -> Interaction {
        let once = [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
        facing(label, vec![flagged_animal(kind, flags)], once).x_rot(30.0)
    };
    let golden = "minecraft:golden_dandelion";
    let stone = "minecraft:stone";
    vec![
        at("use-villager-empty-handed", "villager", &[]).row(),
        at("use-villager-with-stone", "villager", &[]).main_hand(stone).count(2).row(),
        at("use-villager-with-bread", "villager", &[]).main_hand("minecraft:bread").count(2).row(),
        at("use-baby-villager-with-stone", "villager", &["baby"]).main_hand(stone).count(2).row(),
        at("use-baby-villager-with-golden", "villager", &["baby"]).main_hand(golden).count(2).row(),
        at("use-wandering-trader-with-stone", "wandering_trader", &[]).main_hand(stone).count(2).row(),
        at("use-wandering-trader-with-bread", "wandering_trader", &[]).main_hand("minecraft:bread").count(2).row(),
        at("use-baby-wandering-trader-with-stone", "wandering_trader", &["baby"]).main_hand(stone).count(2).row(),
        at("use-baby-wandering-trader-with-golden", "wandering_trader", &["baby"]).main_hand(golden).count(2).row(),
        at("use-zombie-villager-with-golden-apple", "zombie_villager", &[])
            .main_hand("minecraft:golden_apple")
            .count(2)
            .row(),
        at("use-zombie-villager-with-stone", "zombie_villager", &[]).main_hand(stone).count(2).row(),
    ]
}
