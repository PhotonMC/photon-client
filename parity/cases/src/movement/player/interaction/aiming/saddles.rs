//! The use key on saddleable mobs: riding a saddled pig or strider, and saddling one.

use serde_json::{Value, json};

use super::herding::{flagged_animal, sneaking_click};
use super::{aims, facing};

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in ["pig", "strider"] {
        rows.extend(kind_rows(kind));
    }
    rows.extend(other_rows());
    rows
}

/// Items that equip on interact, on mobs that may or may not wear them.
fn other_rows() -> Vec<Value> {
    let once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let click = |label: &str, kind: &str, flags: &[&str], item: &'static str| {
        facing(label, vec![flagged_animal(kind, flags)], once()).main_hand(item).count(2).x_rot(30.0).row()
    };
    [
        vec![
            click("use-saddle-on-zombie", "zombie", &[], "minecraft:saddle"),
            click("use-saddle-on-cow", "cow", &[], "minecraft:saddle"),
            click("use-harness-on-pig", "pig", &[], "minecraft:red_harness"),
            click("use-harness-on-happy-ghast", "happy_ghast", &[], "minecraft:red_harness"),
            click("use-harness-on-baby-happy-ghast", "happy_ghast", &["baby"], "minecraft:red_harness"),
            click("use-saddle-on-baby-camel", "camel", &["baby"], "minecraft:saddle"),
            click("use-saddle-on-untamed-donkey", "donkey", &[], "minecraft:saddle"),
            click("use-saddle-on-tamed-donkey", "donkey", &["tamed"], "minecraft:saddle"),
            click("use-saddle-on-tamed-horse", "horse", &["tamed"], "minecraft:saddle"),
            click("use-saddle-on-untamed-horse", "horse", &[], "minecraft:saddle"),
            click("use-saddle-on-nautilus", "nautilus", &[], "minecraft:saddle"),
            click("use-armor-on-nautilus", "nautilus", &[], "minecraft:iron_nautilus_armor"),
            click("use-saddle-on-zombie-nautilus", "zombie_nautilus", &[], "minecraft:saddle"),
            click("use-saddle-on-camel", "camel", &[], "minecraft:saddle"),
            click("use-saddle-on-saddled-camel", "camel", &["saddled"], "minecraft:saddle"),
        ],
        shearing_rows(),
    ]
    .concat()
}

/// Shears on mobs that wear something shearable, clicked twice: the first click takes the saddle
/// or armor off, so the second no longer rides or shears.
fn shearing_rows() -> Vec<Value> {
    let twice = || {
        [
            aims(1).use_clicks(1).inventory().go(),
            aims(2).inventory().go(),
            aims(1).use_clicks(1).inventory().go(),
            aims(8).inventory().go(),
        ]
        .concat()
    };
    let shears = |label: &str, kind: &str, flags: &[&str], extra: &[(&str, Value)]| {
        let mut entity = flagged_animal(kind, flags);
        for (key, value) in extra {
            entity[*key] = value.clone();
        }
        let mut row = facing(label, vec![entity], twice()).main_hand("minecraft:shears").count(2).x_rot(30.0).row();
        row["player_uuid"] = json!(true);
        row
    };
    vec![
        shears("shear-saddled-pig", "pig", &["saddled"], &[]),
        shears("shear-saddled-strider", "strider", &["saddled"], &[]),
        shears("shear-saddled-horse", "horse", &["tamed", "saddled"], &[]),
        shears("shear-armored-horse", "horse", &["tamed"], &[("armored", json!("minecraft:iron_horse_armor"))]),
        shears("shear-saddled-camel", "camel", &["saddled"], &[]),
        shears("shear-saddled-nautilus", "nautilus", &["saddled"], &[]),
        shears("shear-owned-armored-wolf", "wolf", &[], &[("tame", json!(true)), ("armored", json!(true))]),
        shears("shear-other-armored-wolf", "wolf", &[], &[("tame", json!("other")), ("armored", json!(true))]),
        shears("shear-wild-saddled-pig", "pig", &["saddled", "baby"], &[]),
    ]
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
