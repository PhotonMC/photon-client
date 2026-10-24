//! The use key with a thrown item in hand: the stack is used up on the client, the projectile is
//! the server's.

use serde_json::Value;

use super::placing::uses;
use super::using;

/// One click, then three quiet ticks, with `item` in hand; `count` of them.
fn throw(label: &str, item: &'static str, count: i64) -> super::Interaction {
    let click = uses(1, None, &[]).clicks(1).go();
    let rest = uses(3, None, &[]).go();
    using(label, vec![], [click, rest].concat()).main_hand(item).count(count)
}

fn twice(label: &str, item: &'static str, count: i64) -> super::Interaction {
    let click = || uses(1, None, &[]).clicks(1).go();
    let wait = uses(6, None, &[]).go();
    using(label, vec![], [click(), wait, click(), uses(2, None, &[]).go()].concat()).main_hand(item).count(count)
}

/// A click, `quiet` ticks, then a click on each of three ticks: the cooldown's last ticks.
fn edge(label: &str, item: &'static str, quiet: usize) -> super::Interaction {
    let click = || uses(1, None, &[]).clicks(1).go();
    let ticks = [click(), uses(quiet, None, &[]).go(), click(), click(), click()].concat();
    using(label, vec![], ticks).main_hand(item).count(6)
}

pub fn rows() -> Vec<Value> {
    vec![
        throw("snowball", "minecraft:snowball", 4).row(),
        throw("egg", "minecraft:egg", 4).row(),
        throw("ender-pearl", "minecraft:ender_pearl", 2).row(),
        throw("experience-bottle", "minecraft:experience_bottle", 2).row(),
        throw("wind-charge", "minecraft:wind_charge", 2).row(),
        throw("splash-potion", "minecraft:splash_potion", 2).row(),
        throw("lingering-potion", "minecraft:lingering_potion", 2).row(),
        throw("last-snowball", "minecraft:snowball", 1).row(),
        throw("creative-snowball", "minecraft:snowball", 4).game_mode("creative").row(),
        twice("ender-pearl-twice", "minecraft:ender_pearl", 3).row(),
        edge("ender-pearl-cooldown-edge", "minecraft:ender_pearl", 18).row(),
        edge("wind-charge-cooldown-edge", "minecraft:wind_charge", 8).row(),
        twice("snowball-twice", "minecraft:snowball", 3).row(),
        // `use_remainder`: an emptied stack becomes it; one with items left keeps them (the
        // remainder goes to the inventory); creative has infinite materials.
        throw("snowball-remainder-last", "minecraft:snowball", 1).use_remainder("minecraft:stick").row(),
        throw("snowball-remainder-more", "minecraft:snowball", 3).use_remainder("minecraft:stick").row(),
        throw("snowball-remainder-creative", "minecraft:snowball", 1)
            .use_remainder("minecraft:stick")
            .game_mode("creative")
            .row(),
    ]
}
