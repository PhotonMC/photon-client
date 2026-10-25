//! The use key on saddleable mobs: riding a saddled pig or strider, and saddling one.

use serde_json::Value;

use super::herding::{flagged_animal, sneaking_click};
use super::{aims, facing};

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in ["pig", "strider"] {
        rows.extend(kind_rows(kind));
    }
    rows
}

fn kind_rows(kind: &str) -> Vec<Value> {
    let once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let sneaking = sneaking_click;
    let food = if kind == "pig" { "minecraft:carrot" } else { "minecraft:warped_fungus" };
    let label = |what: &str| format!("use-{what}-{kind}");
    let click = |what: &str, flags: &[&str], item: &'static str, clicks: Vec<Value>| {
        facing(&label(what), vec![flagged_animal(kind, flags)], clicks).main_hand(item).count(2).x_rot(30.0).row()
    };
    let saddle = "minecraft:saddle";
    vec![
        click("saddled-empty", &["saddled"], "minecraft:stone", once()),
        click("saddled-food", &["saddled"], food, once()),
        click("saddled-sneaking", &["saddled"], "minecraft:stone", sneaking()),
        click("saddle-on", &[], saddle, once()),
        click("saddle-on-baby", &["baby"], saddle, once()),
        click("saddle-on-saddled", &["saddled"], saddle, sneaking()),
        click("stone-on-bare", &[], "minecraft:stone", once()),
        click("horse-armor-on-bare", &[], "minecraft:iron_horse_armor", once()),
        click("wolf-armor-on-bare", &[], "minecraft:wolf_armor", once()),
        click("food-on-bare", &[], food, once()),
    ]
}
