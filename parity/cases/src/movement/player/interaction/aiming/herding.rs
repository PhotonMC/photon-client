//! The use key on farm animals: milking, shears, dyes, feeding, the age lock and riding.

use serde_json::{Value, json};

use super::{aims, facing, mob};
use crate::movement::player::interaction::Interaction;

pub(super) fn rows() -> Vec<Value> {
    [
        cow_rows(),
        mooshroom_rows(),
        goat_rows(),
        brown_mooshroom_rows(),
        sheep_rows(),
        horse_rows(),
        other_horse_rows(),
        sneaking_horse_rows(),
        age_lock_rows(),
    ]
    .concat()
}

/// The use key on cows, which milk a bucket and eat wheat.
fn cow_rows() -> Vec<Value> {
    let use_once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let at = |label: &str, kind: &str| facing(label, vec![mob(31, kind, 0.5, 64.0, 2.5)], use_once());
    // An entity spawned with a flag the oracle applies (a baby, a golem without its pumpkin).
    let with_flag = |label: &str, kind: &str, flag: &str| {
        let mut entity = mob(31, kind, 0.5, 64.0, 2.5);
        entity[flag] = json!(true);
        facing(label, vec![entity], use_once())
    };
    vec![
        at("use-cow-with-bucket", "cow").main_hand("minecraft:bucket").x_rot(20.0).row(),
        at("use-cow-with-stack-of-buckets", "cow").main_hand("minecraft:bucket").count(3).x_rot(20.0).row(),
        at("use-cow-with-wheat", "cow").main_hand("minecraft:wheat").x_rot(20.0).row(),
        with_flag("use-baby-cow-with-wheat", "cow", "baby").main_hand("minecraft:wheat").x_rot(30.0).row(),
        at("use-cow-with-stone", "cow").main_hand("minecraft:stone").x_rot(20.0).row(),
        with_flag("use-baby-cow-with-bucket", "cow", "baby").main_hand("minecraft:bucket").x_rot(30.0).row(),
        at("use-mooshroom-with-bucket", "mooshroom").main_hand("minecraft:bucket").x_rot(20.0).row(),
    ]
}

/// The use key on mooshrooms: a bowl is filled with stew, shears work on an adult.
fn mooshroom_rows() -> Vec<Value> {
    let use_once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let at = |label: &str| facing(label, vec![mob(31, "mooshroom", 0.5, 64.0, 2.5)], use_once());
    let baby = |label: &str| {
        let mut entity = mob(31, "mooshroom", 0.5, 64.0, 2.5);
        entity["baby"] = json!(true);
        facing(label, vec![entity], use_once())
    };
    vec![
        at("use-mooshroom-with-bowl").main_hand("minecraft:bowl").x_rot(20.0).row(),
        at("use-mooshroom-with-stack-of-bowls").main_hand("minecraft:bowl").count(2).x_rot(20.0).row(),
        at("use-mooshroom-with-bowl-in-creative")
            .main_hand("minecraft:bowl")
            .off_hand("minecraft:mushroom_stew")
            .game_mode("creative")
            .x_rot(20.0)
            .row(),
        at("use-mooshroom-with-shears").main_hand("minecraft:shears").x_rot(20.0).row(),
        baby("use-baby-mooshroom-with-shears").main_hand("minecraft:shears").x_rot(30.0).row(),
        baby("use-baby-mooshroom-with-bowl").main_hand("minecraft:bowl").x_rot(30.0).row(),
        at("use-mooshroom-with-flower").main_hand("minecraft:dandelion").x_rot(20.0).row(),
    ]
}

/// The use key on goats: a bucket is milked (a screaming goat sounds different), wheat feeds.
fn goat_rows() -> Vec<Value> {
    let use_once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let with = |label: &str, flag: Option<&str>| {
        let mut entity = mob(31, "goat", 0.5, 64.0, 2.5);
        if let Some(flag) = flag {
            entity[flag] = json!(true);
        }
        facing(label, vec![entity], use_once())
    };
    vec![
        with("use-goat-with-bucket", None).main_hand("minecraft:bucket").x_rot(20.0).row(),
        with("use-screaming-goat-with-bucket", Some("screaming")).main_hand("minecraft:bucket").x_rot(20.0).row(),
        with("use-baby-goat-with-bucket", Some("baby")).main_hand("minecraft:bucket").x_rot(30.0).row(),
        with("use-goat-with-wheat", None).main_hand("minecraft:wheat").x_rot(20.0).row(),
        with("use-baby-goat-with-wheat", Some("baby")).main_hand("minecraft:wheat").x_rot(30.0).row(),
        with("use-goat-with-stone", None).main_hand("minecraft:stone").x_rot(20.0).row(),
    ]
}

/// The use key on brown mooshrooms, which eat a flower for the stew they give next.
fn brown_mooshroom_rows() -> Vec<Value> {
    let use_once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let brown_mooshroom = || flagged_animal("mooshroom", &["brown"]);
    let brown = |label: &str| facing(label, vec![brown_mooshroom()], use_once());
    vec![
        brown("use-brown-mooshroom-with-flowers").main_hand("minecraft:dandelion").count(2).x_rot(20.0).row(),
        brown("use-brown-mooshroom-with-wither-rose").main_hand("minecraft:wither_rose").count(2).x_rot(20.0).row(),
        brown("use-brown-mooshroom-with-pink-petals").main_hand("minecraft:pink_petals").count(2).x_rot(20.0).row(),
        brown("use-brown-mooshroom-with-flowers-in-creative")
            .main_hand("minecraft:dandelion")
            .count(2)
            .game_mode("creative")
            .x_rot(20.0)
            .row(),
        brown("use-brown-mooshroom-with-stone").main_hand("minecraft:stone").x_rot(20.0).row(),
        facing("use-brown-mooshroom-twice-with-flowers", vec![brown_mooshroom()], two_use_clicks())
            .main_hand("minecraft:dandelion")
            .count(2)
            .x_rot(20.0)
            .row(),
        // One flower in the main hand, a bowl in the other: the first click eats the flower, the
        // second finds the hand empty and goes on to the bowl, which holds the flower's effects.
        facing("use-brown-mooshroom-flower-then-bowl", vec![brown_mooshroom()], two_use_clicks())
            .main_hand("minecraft:dandelion")
            .off_hand("minecraft:bowl")
            .x_rot(20.0)
            .row(),
        facing(
            "use-baby-brown-mooshroom-with-flower",
            vec![{
                let mut entity = brown_mooshroom();
                entity["baby"] = json!(true);
                entity
            }],
            use_once(),
        )
        .main_hand("minecraft:dandelion")
        .x_rot(30.0)
        .row(),
    ]
}

/// Two right clicks two ticks apart, the inventory reported after each.
fn two_use_clicks() -> Vec<Value> {
    [
        aims(1).use_clicks(1).inventory().go(),
        aims(2).inventory().go(),
        aims(1).use_clicks(1).inventory().go(),
        aims(2).inventory().go(),
    ]
    .concat()
}

/// Shears, wheat and dyes on sheep: shears only consume the click, a dye of another colour on a
/// sheep with wool swings and sounds, the rest is feeding.
fn sheep_rows() -> Vec<Value> {
    let sheep = |flags: &[&str], dye: Option<u8>| {
        let mut entity = flagged_animal("sheep", flags);
        if let Some(dye) = dye {
            entity["dye"] = json!(dye);
        }
        entity
    };
    let once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let click = |label: &str, entity: Value, item: &'static str| {
        facing(label, vec![entity], once()).main_hand(item).count(2).x_rot(30.0).row()
    };
    vec![
        click("use-sheep-with-shears", sheep(&[], None), "minecraft:shears"),
        click("use-leashed-sheep-with-shears", sheep(&["leash_holder"], None), "minecraft:shears"),
        click("use-leashed-sheep-with-red-dye", sheep(&["leash_holder"], None), "minecraft:red_dye"),
        tied_to_a_pig("use-sheep-tied-to-a-pig-with-shears", "minecraft:shears"),
        tied_to_a_pig("use-sheep-tied-to-a-pig-with-wheat", "minecraft:wheat"),
        click("use-sheared-sheep-with-shears", sheep(&["sheared"], None), "minecraft:shears"),
        click("use-baby-sheep-with-shears", sheep(&["baby"], None), "minecraft:shears"),
        click("use-sheep-with-wheat", sheep(&[], None), "minecraft:wheat"),
        click("use-baby-sheep-with-wheat", sheep(&["baby"], None), "minecraft:wheat"),
        click("use-sheep-with-red-dye", sheep(&[], None), "minecraft:red_dye"),
        click("use-white-sheep-with-white-dye", sheep(&[], None), "minecraft:white_dye"),
        click("use-red-sheep-with-red-dye", sheep(&[], Some(14)), "minecraft:red_dye"),
        click("use-red-sheep-with-white-dye", sheep(&[], Some(14)), "minecraft:white_dye"),
        click("use-sheared-sheep-with-red-dye", sheep(&["sheared"], None), "minecraft:red_dye"),
        click("use-baby-sheep-with-red-dye", sheep(&["baby"], None), "minecraft:red_dye"),
        click("use-sheep-with-stone", sheep(&[], None), "minecraft:stone"),
    ]
}

/// A horse-family animal clicked once with the use key, the inventory reported.
fn horse_at(label: &str, kind: &str, flags: &[&str]) -> Interaction {
    let once = [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    facing(label, vec![flagged_animal(kind, flags)], once).x_rot(30.0)
}

/// Two ticks of sneaking, then the click: sneaking takes a tick to lower the eye.
pub(super) fn sneaking_click() -> Vec<Value> {
    [
        aims(2).held(&["sneak"]).inventory().go(),
        aims(1).use_clicks(1).held(&["sneak"]).inventory().go(),
        aims(3).held(&["sneak"]).inventory().go(),
    ]
    .concat()
}

/// The horse family: food, anger, the chest and riding, with the golden dandelion bypass.
fn horse_rows() -> Vec<Value> {
    let at = horse_at;
    let golden = "minecraft:golden_dandelion";
    vec![
        at("use-horse-empty-handed", "horse", &[]).row(),
        at("use-horse-with-stone", "horse", &[]).main_hand("minecraft:stone").count(2).row(),
        at("use-horse-with-wheat", "horse", &[]).main_hand("minecraft:wheat").count(2).row(),
        at("use-baby-horse-with-golden", "horse", &["baby"]).main_hand(golden).count(2).row(),
        at("use-tamed-baby-horse-with-golden", "horse", &["baby", "tamed"]).main_hand(golden).count(2).row(),
        // The dandelion in the off hand sends the baby to the animal rules, which feed it.
        at("use-baby-horse-wheat-and-golden-off-hand", "horse", &["baby"])
            .main_hand("minecraft:wheat")
            .off_hand(golden)
            .count(2)
            .row(),
    ]
}

/// Undead horses, chested horses and llamas, and the sneaking owner.
fn other_horse_rows() -> Vec<Value> {
    let at = horse_at;
    let golden = "minecraft:golden_dandelion";
    vec![
        at("use-baby-zombie-horse-with-golden", "zombie_horse", &["baby"]).main_hand(golden).count(2).row(),
        at("use-tamed-baby-zombie-horse-with-golden", "zombie_horse", &["baby", "tamed"])
            .main_hand(golden)
            .count(2)
            .row(),
        at("use-tamed-baby-zombie-horse-with-rotten-flesh", "zombie_horse", &["baby", "tamed"])
            .main_hand("minecraft:rotten_flesh")
            .count(2)
            .row(),
        at("use-skeleton-horse-with-stone", "skeleton_horse", &[]).main_hand("minecraft:stone").count(2).row(),
        at("use-tamed-skeleton-horse-with-stone", "skeleton_horse", &["tamed"])
            .main_hand("minecraft:stone")
            .count(2)
            .row(),
        at("use-tamed-baby-skeleton-horse-with-wheat", "skeleton_horse", &["baby", "tamed"])
            .main_hand("minecraft:wheat")
            .count(2)
            .row(),
        at("use-tamed-donkey-with-chest", "donkey", &["tamed"]).main_hand("minecraft:chest").count(2).row(),
        at("use-donkey-with-chest", "donkey", &[]).main_hand("minecraft:chest").count(2).row(),
        at("use-tamed-donkey-with-chest-in-creative", "donkey", &["tamed"])
            .main_hand("minecraft:chest")
            .count(2)
            .game_mode("creative")
            .row(),
        at("use-tamed-horse-with-chest", "horse", &["tamed"]).main_hand("minecraft:chest").count(2).row(),
        at("use-zombie-horse-with-red-mushroom", "zombie_horse", &[])
            .main_hand("minecraft:red_mushroom")
            .count(2)
            .row(),
        at("use-zombie-horse-with-wheat", "zombie_horse", &[]).main_hand("minecraft:wheat").count(2).row(),
        at("use-llama-with-apple", "llama", &[]).main_hand("minecraft:apple").count(2).row(),
        at("use-llama-with-hay-block", "llama", &[]).main_hand("minecraft:hay_block").count(2).row(),
        at("use-llama-with-stone", "llama", &[]).main_hand("minecraft:stone").count(2).row(),
    ]
}

/// The owner's sneak: a tamed adult opens the inventory, anything else is as without it.
fn sneaking_horse_rows() -> Vec<Value> {
    vec![
        facing(
            "use-tamed-adult-horse-sneaking-empty-handed",
            vec![flagged_animal("horse", &["tamed"])],
            sneaking_click(),
        )
        .x_rot(30.0)
        .row(),
        facing("use-untamed-horse-sneaking-with-stone", vec![flagged_animal("horse", &[])], sneaking_click())
            .x_rot(30.0)
            .main_hand("minecraft:stone")
            .count(2)
            .row(),
        facing("use-tamed-horse-sneaking", vec![flagged_animal("horse", &["tamed"])], sneaking_click())
            .x_rot(30.0)
            .main_hand("minecraft:stone")
            .count(2)
            .row(),
        facing("use-tamed-horse-sneaking-with-wheat", vec![flagged_animal("horse", &["tamed"])], sneaking_click())
            .x_rot(30.0)
            .main_hand("minecraft:wheat")
            .count(2)
            .row(),
    ]
}

/// A sheep held on a lead by a pig beside it, clicked with an item.
fn tied_to_a_pig(label: &str, item: &'static str) -> Value {
    let mut sheep = flagged_animal("sheep", &[]);
    sheep["leash_holder"] = json!(32);
    let pig = mob(32, "pig", 0.5, 64.0, 4.5);
    facing(label, vec![sheep, pig], [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat())
        .main_hand(item)
        .count(2)
        .x_rot(30.0)
        .row()
}

/// An animal at the usual spot with the named spawn flags set.
pub(super) fn flagged_animal(kind: &str, flags: &[&str]) -> Value {
    let mut entity = mob(31, kind, 0.5, 64.0, 2.5);
    for flag in flags {
        entity[*flag] = json!(true);
    }
    entity
}

/// The golden dandelion on animals: babies lock or unlock their growth, a locked baby is not fed.
fn age_lock_rows() -> Vec<Value> {
    let once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let twice = || {
        [
            aims(1).use_clicks(1).inventory().go(),
            aims(2).inventory().go(),
            aims(1).use_clicks(1).inventory().go(),
            aims(2).inventory().go(),
        ]
        .concat()
    };
    let animal = flagged_animal;
    let golden = "minecraft:golden_dandelion";
    vec![
        facing("age-lock-baby-chicken", vec![animal("chicken", &["baby"])], once())
            .main_hand(golden)
            .count(2)
            .x_rot(30.0)
            .row(),
        facing("age-lock-baby-chicken-twice", vec![animal("chicken", &["baby"])], twice())
            .main_hand(golden)
            .count(2)
            .x_rot(30.0)
            .row(),
        facing("age-lock-adult-chicken", vec![animal("chicken", &[])], once())
            .main_hand(golden)
            .count(2)
            .x_rot(30.0)
            .row(),
        facing("age-lock-locked-baby-chicken-with-seeds", vec![animal("chicken", &["baby", "age_locked"])], once())
            .main_hand("minecraft:wheat_seeds")
            .count(2)
            .x_rot(30.0)
            .row(),
        facing("age-lock-locked-baby-chicken-with-golden", vec![animal("chicken", &["baby", "age_locked"])], once())
            .main_hand(golden)
            .count(2)
            .x_rot(30.0)
            .row(),
        // The first click locks the baby on this client; the second click, with the main hand
        // empty, goes on to the seeds, which a locked baby does not eat.
        facing("age-lock-then-seeds", vec![animal("chicken", &["baby"])], twice())
            .main_hand(golden)
            .off_hand("minecraft:wheat_seeds")
            .x_rot(30.0)
            .row(),
        facing("age-lock-baby-brown-mooshroom", vec![animal("mooshroom", &["baby", "brown"])], once())
            .main_hand(golden)
            .count(2)
            .x_rot(30.0)
            .row(),
        facing("age-lock-baby-chicken-in-creative", vec![animal("chicken", &["baby"])], once())
            .main_hand(golden)
            .count(2)
            .game_mode("creative")
            .x_rot(30.0)
            .row(),
    ]
}
