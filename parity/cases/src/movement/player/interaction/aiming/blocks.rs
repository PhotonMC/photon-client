//! The look at blocks whose shape depends on who asks or whose interaction shape names the face:
//! scaffolding held in the hand, and the hollow blocks one looks into from above.

use serde_json::Value;

use super::{aims, facing};
use crate::movement::world::{put, put_with};

pub(super) fn rows() -> Vec<Value> {
    let once = || [aims(1).use_clicks(1).go(), aims(3).go()].concat();
    let at = |label: &str, block: Value| facing(label, vec![], once()).blocks(vec![block]);
    let scaffold = || put_with(0, 65, 3, "minecraft:scaffolding", &[("bottom", "false"), ("distance", "0")]);
    let scaffolding = "minecraft:scaffolding";
    vec![
        // Held, scaffolding is a whole cube to the look; with anything else the look passes
        // through the gaps of its frame.
        at("look-at-scaffolding-holding-it", scaffold()).main_hand(scaffolding).row(),
        at("look-at-scaffolding-holding-stone", scaffold()).main_hand("minecraft:stone").row(),
        at("look-at-scaffolding-empty-hand", scaffold()).row(),
        // Looking down into a hollow block, the face is the one of its inside, which is met first.
        at("look-into-cauldron", put(0, 64, 2, "minecraft:cauldron")).x_rot(20.0).row(),
        at("look-into-water-cauldron", put_with(0, 64, 2, "minecraft:water_cauldron", &[("level", "2")]))
            .x_rot(20.0)
            .row(),
        at("look-into-composter", put_with(0, 64, 2, "minecraft:composter", &[("level", "3")])).x_rot(20.0).row(),
        at("look-into-hopper", put_with(0, 64, 2, "minecraft:hopper", &[("facing", "down")])).x_rot(25.0).row(),
        at("look-into-sideways-hopper", put_with(0, 64, 2, "minecraft:hopper", &[("facing", "east")]))
            .x_rot(25.0)
            .row(),
    ]
}
