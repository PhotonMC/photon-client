//! The use key on mobs a water bucket picks up: fish and axolotls (`Bucketable`), and what the
//! axolotl does with the food it is given.

use serde_json::Value;

use super::herding::flagged_animal;
use super::{aims, facing};
use crate::movement::player::interaction::Interaction;

pub(super) fn rows() -> Vec<Value> {
    let at = |label: &str, kind: &str, flags: &[&str]| -> Interaction {
        let once = [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
        facing(label, vec![flagged_animal(kind, flags)], once).x_rot(30.0)
    };
    let water = "minecraft:water_bucket";
    vec![
        at("pick-up-cod", "cod", &[]).main_hand(water).x_rot(33.0).row(),
        at("pick-up-cod-in-creative", "cod", &[]).main_hand(water).game_mode("creative").x_rot(33.0).row(),
        at("pick-up-salmon", "salmon", &[]).main_hand(water).row(),
        at("pick-up-pufferfish", "pufferfish", &[]).main_hand(water).x_rot(33.0).row(),
        at("pick-up-tropical-fish", "tropical_fish", &[]).main_hand(water).row(),
        at("pick-up-axolotl", "axolotl", &[]).main_hand(water).row(),
        at("feed-axolotl-tropical-fish-bucket", "axolotl", &[]).main_hand("minecraft:tropical_fish_bucket").row(),
        at("feed-baby-axolotl-tropical-fish-bucket", "axolotl", &["baby"])
            .main_hand("minecraft:tropical_fish_bucket")
            .x_rot(33.0)
            .row(),
        at("feed-baby-axolotl-in-creative", "axolotl", &["baby"])
            .main_hand("minecraft:tropical_fish_bucket")
            .game_mode("creative")
            .x_rot(33.0)
            .row(),
        at("feed-baby-axolotl-creative-with-water-bucket", "axolotl", &["baby"])
            .main_hand("minecraft:tropical_fish_bucket")
            .off_hand("minecraft:water_bucket")
            .game_mode("creative")
            .x_rot(33.0)
            .row(),
        at("pick-up-cod-creative-with-cod-bucket", "cod", &[])
            .main_hand(water)
            .off_hand("minecraft:cod_bucket")
            .game_mode("creative")
            .x_rot(33.0)
            .row(),
        at("use-baby-axolotl-with-golden", "axolotl", &["baby"])
            .main_hand("minecraft:golden_dandelion")
            .count(2)
            .x_rot(33.0)
            .row(),
    ]
}
