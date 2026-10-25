//! The use key on wolves and cats: taming, feeding, collar dyes, wolf armor and sitting.

use serde_json::{Value, json};

use super::herding::{flagged_animal, sneaking_click};
use super::{aims, facing};

pub(super) fn rows() -> Vec<Value> {
    [wolf_rows(), cat_rows(), nautilus_rows(), parrot_rows()].concat()
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
    let aimed = facing(label, vec![entity], clicks).x_rot(30.0);
    let mut row = if item.is_empty() { aimed.row() } else { aimed.main_hand(item).count(2).row() };
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
        {
            let mut row =
                facing("use-owned-hurt-wolf-with-beef-in-creative", vec![owned(&[("hurt", json!(4.0))])], once())
                    .main_hand("minecraft:beef")
                    .count(2)
                    .game_mode("creative")
                    .x_rot(30.0)
                    .row();
            row["player_uuid"] = json!(true);
            row
        },
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

/// Nautilus: tamed with pufferfish, fed fish, ridden, and fitted with a saddle or armor.
fn nautilus_rows() -> Vec<Value> {
    let tame = |kind: &str, flags: &[&str], extra: &[(&str, Value)]| {
        let mut extras = vec![("tame", json!(true))];
        extras.extend_from_slice(extra);
        pet(kind, flags, &extras)
    };
    let wild = |kind: &str, flags: &[&str]| pet(kind, flags, &[]);
    let saddle = "minecraft:saddle";
    let armor = "minecraft:iron_nautilus_armor";
    vec![
        click("use-wild-nautilus-with-pufferfish", wild("nautilus", &[]), "minecraft:pufferfish", once()),
        click("use-wild-nautilus-with-cod", wild("nautilus", &[]), "minecraft:cod", once()),
        click("use-wild-nautilus-with-saddle", wild("nautilus", &[]), saddle, once()),
        click("use-wild-baby-nautilus-with-cod", wild("nautilus", &["baby"]), "minecraft:cod", once()),
        click("use-tame-nautilus-with-stone", tame("nautilus", &[], &[]), "minecraft:stone", once()),
        click("use-tame-nautilus-empty-handed", tame("nautilus", &[], &[]), "", once()),
        click("use-tame-nautilus-sneaking", tame("nautilus", &[], &[]), "minecraft:stone", sneaking_click()),
        click("use-tame-nautilus-with-cod", tame("nautilus", &[], &[]), "minecraft:cod", once()),
        click(
            "use-tame-hurt-nautilus-with-cod",
            tame("nautilus", &[], &[("hurt", json!(1.0))]),
            "minecraft:cod",
            once(),
        ),
        click("use-tame-nautilus-with-saddle", tame("nautilus", &[], &[]), saddle, once()),
        click("use-tame-nautilus-with-armor", tame("nautilus", &[], &[]), armor, once()),
        click("use-tame-baby-nautilus-with-saddle", tame("nautilus", &["baby"], &[]), saddle, once()),
        click("use-tame-baby-nautilus-with-cod", tame("nautilus", &["baby"], &[]), "minecraft:cod", once()),
        click("use-saddled-nautilus-with-saddle", tame("nautilus", &["saddled"], &[]), saddle, once()),
        click("use-tame-zombie-nautilus-with-saddle", tame("zombie_nautilus", &[], &[]), saddle, once()),
        click("use-wild-zombie-nautilus-with-saddle", wild("zombie_nautilus", &[]), saddle, once()),
    ]
}

/// Parrots: tamed with seeds, poisoned by cookies, and sat by an owner while grounded.
fn parrot_rows() -> Vec<Value> {
    let owned = |extra: &[(&str, Value)]| {
        let mut flags = vec![("tame", json!(true))];
        flags.extend_from_slice(extra);
        pet("parrot", &[], &flags)
    };
    let grounded = || owned(&[("grounded", json!(true))]);
    vec![
        click("use-wild-parrot-with-seeds", pet("parrot", &[], &[]), "minecraft:wheat_seeds", once()),
        click("use-wild-parrot-with-cookie", pet("parrot", &[], &[]), "minecraft:cookie", once()),
        click("use-wild-parrot-with-stone", pet("parrot", &[], &[]), "minecraft:stone", once()),
        click("use-flying-owned-parrot-with-stone", owned(&[]), "minecraft:stone", once()),
        click("use-grounded-owned-parrot-with-stone", grounded(), "minecraft:stone", once()),
        click("use-grounded-owned-parrot-with-seeds", grounded(), "minecraft:wheat_seeds", once()),
        click("use-grounded-owned-parrot-with-cookie", grounded(), "minecraft:cookie", once()),
        click(
            "use-grounded-other-parrot-with-stone",
            pet("parrot", &[], &[("tame", json!("other")), ("grounded", json!(true))]),
            "minecraft:stone",
            once(),
        ),
        click(
            "use-grounded-wild-parrot-with-stone",
            pet("parrot", &[], &[("grounded", json!(true))]),
            "minecraft:stone",
            once(),
        ),
    ]
}
