//! The use key on villagers, wandering traders and zombie villagers: the client answers the
//! click with a swing and the server does the trading.

use serde_json::Value;

use super::herding::flagged_animal;
use super::{aims, facing};
use crate::movement::player::interaction::Interaction;

pub(super) fn rows() -> Vec<Value> {
    [villager_rows(), camel_rows()].concat()
}

fn villager_rows() -> Vec<Value> {
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

/// Camels have their own order: sneaking first, then food, then the age lock, and riding an
/// adult only consumes the click.
fn camel_rows() -> Vec<Value> {
    let at = |label: &str, kind: &str, flags: &[&str]| -> Interaction {
        let once = [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
        facing(label, vec![flagged_animal(kind, flags)], once).x_rot(30.0)
    };
    let sneaking = [
        aims(2).held(&["sneak"]).inventory().go(),
        aims(1).use_clicks(1).held(&["sneak"]).inventory().go(),
        aims(3).held(&["sneak"]).inventory().go(),
    ]
    .concat();
    let (golden, stone, cactus) = ("minecraft:golden_dandelion", "minecraft:stone", "minecraft:cactus");
    vec![
        at("use-camel-empty-handed", "camel", &[]).row(),
        at("use-camel-with-stone", "camel", &[]).main_hand(stone).count(2).row(),
        at("use-camel-with-cactus", "camel", &[]).main_hand(cactus).count(2).row(),
        at("use-baby-camel-with-cactus", "camel", &["baby"]).main_hand(cactus).count(2).row(),
        at("use-locked-baby-camel-with-cactus", "camel", &["baby", "age_locked"]).main_hand(cactus).count(2).row(),
        at("use-baby-camel-with-stone", "camel", &["baby"]).main_hand(stone).count(2).row(),
        at("use-baby-camel-with-golden", "camel", &["baby"]).main_hand(golden).count(2).row(),
        at("use-baby-camel-cactus-and-golden-off-hand", "camel", &["baby"])
            .main_hand(cactus)
            .off_hand(golden)
            .count(2)
            .row(),
        at("use-camel-husk-with-rabbit-foot", "camel_husk", &[]).main_hand("minecraft:rabbit_foot").count(2).row(),
        at("use-camel-husk-with-cactus", "camel_husk", &[]).main_hand(cactus).count(2).row(),
        facing("use-camel-sneaking", vec![flagged_animal("camel", &[])], sneaking.clone())
            .x_rot(30.0)
            .main_hand(stone)
            .count(2)
            .row(),
        facing("use-baby-camel-sneaking", vec![flagged_animal("camel", &["baby"])], sneaking)
            .x_rot(30.0)
            .main_hand(stone)
            .count(2)
            .row(),
    ]
}
