//! The use key on wolves and cats: taming, feeding, collar dyes, wolf armor and sitting.

use serde_json::{Value, json};

use super::herding::{flagged_animal, sneaking_click};
use super::{aims, facing};

pub(super) fn rows() -> Vec<Value> {
    [wolf_rows(), cat_rows()].concat()
}

/// A pet at the usual spot, with the pet flags given as key and value.
fn pet(kind: &str, flags: &[&str], extra: &[(&str, Value)]) -> Value {
    let mut entity = flagged_animal(kind, flags);
    for (key, value) in extra {
        entity[*key] = value.clone();
    }
    entity
}

/// One click on the pet with an item; the row names the player's uuid, which owners refer to.
fn click(label: &str, entity: Value, item: &'static str, clicks: Vec<Value>) -> Value {
    let mut row = facing(label, vec![entity], clicks).main_hand(item).count(2).x_rot(30.0).row();
    row["player_uuid"] = json!(true);
    row
}

fn once() -> Vec<Value> {
    [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat()
}

fn wolf_rows() -> Vec<Value> {
    let owned = |extra: &[(&str, Value)]| {
        let mut flags = vec![("tame", json!(true))];
        flags.extend_from_slice(extra);
        pet("wolf", &[], &flags)
    };
    let other = || pet("wolf", &[], &[("tame", json!("other"))]);
    vec![
        click("use-wild-wolf-with-bone", pet("wolf", &[], &[]), "minecraft:bone", once()),
        click("use-wild-wolf-with-beef", pet("wolf", &[], &[]), "minecraft:beef", once()),
        click("use-wild-baby-wolf-with-beef", pet("wolf", &["baby"], &[]), "minecraft:beef", once()),
        click("use-owned-hurt-wolf-with-beef", owned(&[("hurt", json!(4.0))]), "minecraft:beef", once()),
        click("use-owned-wolf-with-beef", owned(&[]), "minecraft:beef", once()),
        click("use-owned-wolf-with-blue-dye", owned(&[]), "minecraft:blue_dye", once()),
        click("use-owned-wolf-with-red-dye", owned(&[]), "minecraft:red_dye", once()),
        click("use-owned-wolf-with-armor", owned(&[]), "minecraft:wolf_armor", once()),
        click("use-owned-armored-wolf-with-armor", owned(&[("armored", json!(true))]), "minecraft:wolf_armor", once()),
        click(
            "use-owned-baby-wolf-with-armor",
            pet("wolf", &["baby"], &[("tame", json!(true))]),
            "minecraft:wolf_armor",
            once(),
        ),
        click("use-owned-wolf-with-stone", owned(&[]), "minecraft:stone", once()),
        click("use-owned-sitting-wolf-with-stone", owned(&[("sitting", json!(true))]), "minecraft:stone", once()),
        click("use-owned-wolf-sneaking-with-stone", owned(&[]), "minecraft:stone", sneaking_click()),
        click("use-other-wolf-with-stone", other(), "minecraft:stone", once()),
        click("use-other-wolf-with-blue-dye", other(), "minecraft:blue_dye", once()),
        click("use-other-wolf-with-armor", other(), "minecraft:wolf_armor", once()),
        click(
            "use-other-hurt-wolf-with-beef",
            {
                let mut wolf = other();
                wolf["hurt"] = json!(4.0);
                wolf
            },
            "minecraft:beef",
            once(),
        ),
    ]
}

fn cat_rows() -> Vec<Value> {
    let owned = |extra: &[(&str, Value)]| {
        let mut flags = vec![("tame", json!(true))];
        flags.extend_from_slice(extra);
        pet("cat", &[], &flags)
    };
    let other = || pet("cat", &[], &[("tame", json!("other"))]);
    vec![
        click("use-wild-cat-with-cod", pet("cat", &[], &[]), "minecraft:cod", once()),
        click("use-wild-cat-with-stone", pet("cat", &[], &[]), "minecraft:stone", once()),
        click("use-owned-hurt-cat-with-cod", owned(&[("hurt", json!(4.0))]), "minecraft:cod", once()),
        click("use-owned-cat-with-cod", owned(&[]), "minecraft:cod", once()),
        click("use-owned-cat-with-blue-dye", owned(&[]), "minecraft:blue_dye", once()),
        click("use-owned-cat-with-red-dye", owned(&[]), "minecraft:red_dye", once()),
        click("use-owned-cat-with-stone", owned(&[]), "minecraft:stone", once()),
        click("use-owned-cat-sneaking-with-stone", owned(&[]), "minecraft:stone", sneaking_click()),
        click("use-other-cat-with-stone", other(), "minecraft:stone", once()),
        click("use-other-cat-with-blue-dye", other(), "minecraft:blue_dye", once()),
    ]
}
