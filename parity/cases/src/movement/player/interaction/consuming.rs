//! The server's entity event 9 with a consumable in use: the client finishes the use on its own,
//! so the stack loses one, a `use_remainder` replaces an emptied stack and a `use_cooldown`
//! begins, all before the server's slot update.

use serde_json::{Value, json};

use super::placing::uses;
use super::using;

/// Eat or drink `item` (`count` of them): the click on the first tick, the event on the sixth.
fn consume(label: &str, item: &'static str, count: i64) -> super::Interaction {
    let mut ticks = uses(8, None, &[]).clicks(1).down().go();
    ticks[5]["finish_use"] = json!(true);
    using(label, vec![], ticks).main_hand(item).count(count)
}

pub fn rows() -> Vec<Value> {
    vec![
        consume("eat-golden-apple", "minecraft:golden_apple", 3).row(),
        consume("eat-last-golden-apple", "minecraft:golden_apple", 1).row(),
        consume("eat-creative", "minecraft:golden_apple", 3).game_mode("creative").row(),
        consume("drink-milk", "minecraft:milk_bucket", 1).row(),
        consume("drink-honey", "minecraft:honey_bottle", 2).row(),
        consume("drink-honey-last", "minecraft:honey_bottle", 1).row(),
        consume("eat-suspicious-stew", "minecraft:suspicious_stew", 1).row(),
        consume("eat-chorus-fruit", "minecraft:chorus_fruit", 2).row(),
        consume("eat-with-cooldown", "minecraft:golden_apple", 3).use_cooldown(2.0).row(),
        consume("eat-with-remainder", "minecraft:golden_apple", 1).use_remainder("minecraft:stick").row(),
        consume("eat-with-remainder-more", "minecraft:golden_apple", 2).use_remainder("minecraft:stick").row(),
        consume("eat-not-hungry", "minecraft:apple", 3).row(),
    ]
}
