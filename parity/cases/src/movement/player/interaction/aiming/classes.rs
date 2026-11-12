//! The use key on the remaining mob classes with a `mobInteract` of their own.

use serde_json::{Value, json};

use super::herding::{flagged_animal, sneaking_click};
use super::{aims, facing};
use crate::movement::world::put;

pub(super) fn rows() -> Vec<Value> {
    [plain_rows(), tiny_rows(), health_rows(), worn_rows(), placed_rows(), submerged_boat_rows(), cushion_attack_rows()]
        .concat()
}

/// One use click, then a stretch of ticks with no input.
fn once() -> Vec<Value> {
    [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat()
}

/// Two use clicks with ticks between them, long enough for the first swing to end.
fn twice() -> Vec<Value> {
    [once(), aims(1).use_clicks(1).inventory().go(), aims(8).inventory().go()].concat()
}

/// A row that clicks the entity with `item` (nothing when it is empty) from the aim `x_rot`.
fn clicked(label: &str, entity: Value, aim: (&'static str, f64), clicks: Vec<Value>) -> Value {
    clicked_in(label, vec![entity], aim, (clicks, false))
}

/// The same, in a thunderstorm when asked.
fn clicked_in(
    label: &str,
    entities: Vec<Value>,
    (item, x_rot): (&'static str, f64),
    (clicks, storm): (Vec<Value>, bool),
) -> Value {
    let aimed = facing(label, entities, clicks).x_rot(x_rot);
    let aimed = if storm { aimed.raining(1.0).thundering() } else { aimed };
    if item.is_empty() { aimed.row() } else { aimed.main_hand(item).count(2).row() }
}

/// Adults (`""`) and babies (`"baby"`) with the item in hand: label, flag, kind, item.
const PLAIN: &[(&str, &str, &str, &str)] = &[
    ("use-baby-polar-bear-with-golden-dandelion", "baby", "polar_bear", "minecraft:golden_dandelion"),
    ("use-polar-bear-with-golden-dandelion", "", "polar_bear", "minecraft:golden_dandelion"),
    ("use-baby-squid-with-golden-dandelion", "baby", "squid", "minecraft:golden_dandelion"),
    ("use-baby-glow-squid-with-golden-dandelion", "baby", "glow_squid", "minecraft:golden_dandelion"),
    ("use-baby-zombie-with-golden-dandelion", "baby", "zombie", "minecraft:golden_dandelion"),
    ("use-baby-slime-with-golden-dandelion", "baby", "slime", "minecraft:golden_dandelion"),
    ("use-baby-magma-cube-with-golden-dandelion", "baby", "magma_cube", "minecraft:golden_dandelion"),
    ("use-sulfur-cube-with-stone", "", "sulfur_cube", "minecraft:stone"),
    ("use-sulfur-cube-with-shears", "", "sulfur_cube", "minecraft:shears"),
    ("use-sulfur-cube-with-bucket", "", "sulfur_cube", "minecraft:bucket"),
    ("use-sulfur-cube-with-water-bucket", "", "sulfur_cube", "minecraft:water_bucket"),
    ("use-baby-sulfur-cube-with-slime", "baby", "sulfur_cube", "minecraft:slime_ball"),
    ("use-baby-sulfur-cube-with-stone", "baby", "sulfur_cube", "minecraft:stone"),
    ("use-baby-sulfur-cube-with-bucket", "baby", "sulfur_cube", "minecraft:bucket"),
    ("use-ocelot-with-cod", "", "ocelot", "minecraft:cod"),
    ("use-ocelot-with-stone", "", "ocelot", "minecraft:stone"),
    ("use-dolphin-with-cod", "", "dolphin", "minecraft:cod"),
    ("use-dolphin-with-stone", "", "dolphin", "minecraft:stone"),
    ("use-baby-dolphin-with-cod", "baby", "dolphin", "minecraft:cod"),
    ("use-iron-golem-with-ingot", "", "iron_golem", "minecraft:iron_ingot"),
    ("use-iron-golem-with-stone", "", "iron_golem", "minecraft:stone"),
    ("use-armadillo-with-spider-eye", "", "armadillo", "minecraft:spider_eye"),
    ("use-armadillo-with-brush", "", "armadillo", "minecraft:brush"),
    ("use-armadillo-with-stone", "", "armadillo", "minecraft:stone"),
    ("use-baby-armadillo-with-spider-eye", "baby", "armadillo", "minecraft:spider_eye"),
    ("use-hoglin-with-fungus", "", "hoglin", "minecraft:crimson_fungus"),
    ("use-hoglin-with-stone", "", "hoglin", "minecraft:stone"),
    ("use-baby-hoglin-with-fungus", "baby", "hoglin", "minecraft:crimson_fungus"),
    ("use-bee-with-dandelion", "", "bee", "minecraft:dandelion"),
    ("use-bee-with-wither-rose", "", "bee", "minecraft:wither_rose"),
    ("use-bee-with-open-eyeblossom", "", "bee", "minecraft:open_eyeblossom"),
    ("use-bee-with-poppy", "", "bee", "minecraft:poppy"),
    ("use-bee-with-stone", "", "bee", "minecraft:stone"),
    ("use-panda-with-bamboo", "", "panda", "minecraft:bamboo"),
    ("use-panda-with-stone", "", "panda", "minecraft:stone"),
    ("use-baby-panda-with-bamboo", "baby", "panda", "minecraft:bamboo"),
    ("use-sniffer-with-seeds", "", "sniffer", "minecraft:torchflower_seeds"),
    ("use-sniffer-with-stone", "", "sniffer", "minecraft:stone"),
    ("use-baby-sniffer-with-seeds", "baby", "sniffer", "minecraft:torchflower_seeds"),
    ("use-polar-bear-with-cod", "", "polar_bear", "minecraft:cod"),
    ("use-happy-ghast-with-snowball", "", "happy_ghast", "minecraft:snowball"),
    ("use-allay-with-stone", "", "allay", "minecraft:stone"),
    ("use-allay-with-diamond", "", "allay", "minecraft:diamond"),
    ("use-bogged-with-shears", "", "bogged", "minecraft:shears"),
    ("use-sheared-bogged-with-shears", "sheared", "bogged", "minecraft:shears"),
    ("use-sheared-bogged-with-stone", "sheared", "bogged", "minecraft:stone"),
    ("use-copper-golem-with-honeycomb", "", "copper_golem", "minecraft:honeycomb"),
    ("use-copper-golem-with-copper", "", "copper_golem", "minecraft:copper_ingot"),
    ("use-copper-golem-with-stone", "", "copper_golem", "minecraft:stone"),
    ("use-copper-golem-empty-handed", "", "copper_golem", ""),
    ("use-piglin-with-gold", "", "piglin", "minecraft:gold_ingot"),
    ("use-piglin-with-stone", "", "piglin", "minecraft:stone"),
    ("use-baby-piglin-with-gold", "baby", "piglin", "minecraft:gold_ingot"),
    ("use-baby-armadillo-with-brush", "baby", "armadillo", "minecraft:brush"),
    ("use-allay-empty-handed", "", "allay", ""),
    ("use-panda-with-golden-dandelion", "", "panda", "minecraft:golden_dandelion"),
    ("use-baby-panda-with-golden-dandelion", "baby", "panda", "minecraft:golden_dandelion"),
];

fn plain_rows() -> Vec<Value> {
    PLAIN
        .iter()
        .map(|&(label, flag, kind, item)| {
            let flags: &[&str] = if flag.is_empty() { &[] } else { &[flag] };
            clicked(label, flagged_animal(kind, flags), (item, 30.0), once())
        })
        .collect()
}

/// The tadpole and the baby bee are small: the aim is steeper.
fn tiny_rows() -> Vec<Value> {
    let rows = [
        ("use-tadpole-with-slime-ball", "tadpole", false, "minecraft:slime_ball"),
        ("use-tadpole-with-stone", "tadpole", false, "minecraft:stone"),
        ("use-tadpole-with-water-bucket", "tadpole", false, "minecraft:water_bucket"),
        ("use-baby-bee-with-dandelion", "bee", true, "minecraft:dandelion"),
    ];
    rows.iter()
        .map(|&(label, kind, baby, item)| {
            let flags: &[&str] = if baby { &["baby"] } else { &[] };
            clicked(label, flagged_animal(kind, flags), (item, 36.0), once())
        })
        .collect()
}

/// The iron golem with a health set: a hurt one takes the ingot, a healthy one does not.
fn health_rows() -> Vec<Value> {
    [
        ("use-hurt-iron-golem-with-ingot", 50.0, "minecraft:iron_ingot"),
        ("use-healthy-iron-golem-with-ingot", 100.0, "minecraft:iron_ingot"),
    ]
    .iter()
    .map(|&(label, health, item)| {
        let mut entity = flagged_animal("iron_golem", &[]);
        entity["hurt"] = json!(health);
        clicked(label, entity, (item, 30.0), once())
    })
    .chain([again_row()])
    .collect()
}

/// A golem a little short of full health: the first ingot heals it to the maximum, so a second
/// click passes and keeps the ingot.
fn again_row() -> Value {
    let mut entity = flagged_animal("iron_golem", &[]);
    entity["hurt"] = json!(90.0);
    clicked("use-iron-golem-with-ingot-twice", entity, ("minecraft:iron_ingot", 30.0), twice())
}

/// Rows that read the equipment the mob wears: label, kind, flag, worn item, held item, sneaking.
const WORN: &[(&str, &str, &str, &str, &str, bool)] = &[
    (
        "use-tnt-sulfur-cube-with-flint-and-steel",
        "sulfur_cube",
        "armored",
        "minecraft:tnt",
        "minecraft:flint_and_steel",
        false,
    ),
    ("use-tnt-sulfur-cube-with-fire-charge", "sulfur_cube", "armored", "minecraft:tnt", "minecraft:fire_charge", false),
    ("use-tnt-sulfur-cube-with-stone", "sulfur_cube", "armored", "minecraft:tnt", "minecraft:stone", false),
    (
        "use-stone-sulfur-cube-with-flint-and-steel",
        "sulfur_cube",
        "armored",
        "minecraft:stone",
        "minecraft:flint_and_steel",
        false,
    ),
    ("use-swallowing-sulfur-cube-with-stone", "sulfur_cube", "armored", "minecraft:stone", "minecraft:stone", false),
    ("use-swallowing-sulfur-cube-with-dirt", "sulfur_cube", "armored", "minecraft:stone", "minecraft:dirt", false),
    ("use-swallowing-sulfur-cube-with-shears", "sulfur_cube", "armored", "minecraft:stone", "minecraft:shears", false),
    (
        "use-harnessed-happy-ghast-with-stone",
        "happy_ghast",
        "armored",
        "minecraft:red_harness",
        "minecraft:stone",
        false,
    ),
    ("use-harnessed-happy-ghast-sneaking", "happy_ghast", "armored", "minecraft:red_harness", "minecraft:stone", true),
    (
        "use-wolf-armored-happy-ghast-with-stone",
        "happy_ghast",
        "armored",
        "minecraft:wolf_armor",
        "minecraft:stone",
        false,
    ),
    ("use-copper-golem-carrying-empty-handed", "copper_golem", "mainhand", "minecraft:stone", "", false),
    ("use-copper-golem-carrying-with-stone", "copper_golem", "mainhand", "minecraft:stone", "minecraft:stone", false),
    ("use-copper-golem-poppy-with-shears", "copper_golem", "saddled", "minecraft:poppy", "minecraft:shears", false),
    ("use-copper-golem-stone-with-shears", "copper_golem", "saddled", "minecraft:stone", "minecraft:shears", false),
    ("use-copper-golem-poppy-with-stone", "copper_golem", "saddled", "minecraft:poppy", "minecraft:stone", false),
    ("use-piglin-admiring-with-gold", "piglin", "offhand", "minecraft:gold_ingot", "minecraft:gold_ingot", false),
    ("use-piglin-holding-stone-with-gold", "piglin", "offhand", "minecraft:stone", "minecraft:gold_ingot", false),
];

/// A cube with a block of TNT in it and a fuse already burning passes the click.
fn primed_cube_row() -> Value {
    let mut entity = flagged_animal("sulfur_cube", &[]);
    entity["armored"] = json!("minecraft:tnt");
    entity["primed"] = json!(true);
    clicked("use-primed-tnt-sulfur-cube-with-stone", entity, ("minecraft:stone", 30.0), once())
}

fn worn_rows() -> Vec<Value> {
    let primed = primed_cube_row();
    WORN.iter()
        .map(|&(label, kind, key, worn, held, sneaking)| {
            let mut entity = flagged_animal(kind, &[]);
            entity[key] = json!(worn);
            let clicks = if sneaking { sneaking_click() } else { once() };
            clicked(label, entity, (held, 30.0), clicks)
        })
        .chain(std::iter::once(primed))
        .collect()
}

/// A row that puts the entity where the test needs it and holds an item.
struct Placed {
    label: &'static str,
    kind: &'static str,
    flags: &'static [&'static str],
    at: [f64; 3],
    item: &'static str,
    aim: f64,
    sneaking: bool,
    clicks_twice: bool,
}

const fn placed(label: &'static str, kind: &'static str, item: &'static str) -> Placed {
    Placed { label, kind, flags: &[], at: [0.5, 64.0, 2.5], item, aim: 30.0, sneaking: false, clicks_twice: false }
}

fn placed_table() -> Vec<Placed> {
    let tadpole = |label, item, flags| Placed { flags, aim: 36.0, ..placed(label, "tadpole", item) };
    vec![
        // Within reach of the eyes, but more than three blocks from the feet.
        Placed { at: [0.5, 65.5, 3.7], aim: 0.0, ..placed("use-far-ocelot-with-cod", "ocelot", "minecraft:cod") },
        Placed { at: [0.5, 65.5, 3.0], aim: 0.0, ..placed("use-near-ocelot-with-cod", "ocelot", "minecraft:cod") },
        // A scared armadillo fails the click, brush or not; the held food does not feed it.
        Placed {
            flags: &["scared"],
            ..placed("use-scared-armadillo-with-spider-eye", "armadillo", "minecraft:spider_eye")
        },
        Placed { flags: &["scared"], ..placed("use-scared-armadillo-with-brush", "armadillo", "minecraft:brush") },
        Placed { flags: &["scared"], ..placed("use-scared-armadillo-with-apple", "armadillo", "minecraft:apple") },
        // A dancing allay is duplicated by an amethyst shard and takes any other item as usual.
        Placed {
            flags: &["dancing"],
            ..placed("use-dancing-allay-with-amethyst-shard", "allay", "minecraft:amethyst_shard")
        },
        Placed { flags: &["dancing"], ..placed("use-dancing-allay-with-diamond", "allay", "minecraft:diamond") },
        Placed { flags: &["dancing"], ..placed("use-allay-with-amethyst-shard", "allay", "minecraft:amethyst_shard") },
        // A panda on its back is righted by the click, whatever it is given.
        Placed { flags: &["on_back"], ..placed("use-panda-on-back-with-bamboo", "panda", "minecraft:bamboo") },
        Placed { flags: &["on_back"], ..placed("use-panda-on-back-with-stone", "panda", "minecraft:stone") },
        // The click rights the panda on the client too: a second click no longer succeeds.
        Placed {
            flags: &["on_back"],
            clicks_twice: true,
            ..placed("use-panda-on-back-twice-with-stone", "panda", "minecraft:stone")
        },
        // A trusting ocelot is not fed: the click falls through to the animal's own feeding.
        Placed {
            at: [0.5, 65.5, 3.0],
            aim: 0.0,
            flags: &["trusting"],
            ..placed("use-near-trusting-ocelot-with-cod", "ocelot", "minecraft:cod")
        },
        tadpole("use-locked-tadpole-with-slime", "minecraft:slime_ball", &["age_locked"]),
        tadpole("use-tadpole-with-golden-dandelion", "minecraft:golden_dandelion", &[]),
        tadpole("use-locked-tadpole-with-golden-dandelion", "minecraft:golden_dandelion", &["age_locked"]),
        placed("use-bee-with-closed-eyeblossom", "bee", "minecraft:closed_eyeblossom"),
        placed("use-boat-with-stone", "oak_boat", "minecraft:stone"),
        Placed { sneaking: true, ..placed("use-boat-sneaking", "oak_boat", "minecraft:stone") },
        placed("use-chest-boat-with-stone", "oak_chest_boat", "minecraft:stone"),
        Placed { sneaking: true, ..placed("use-chest-boat-sneaking", "oak_chest_boat", "minecraft:stone") },
        placed("use-interaction-with-stone", "interaction", "minecraft:stone"),
        // A worried panda in thunder is scared and passes the click, even on its back; without the
        // gene or the thunder it is righted by the click.
        Placed {
            flags: &["on_back", "worried", "stormy"],
            ..placed("use-scared-panda-on-back-with-stone", "panda", "minecraft:stone")
        },
        Placed {
            flags: &["on_back", "worried"],
            ..placed("use-worried-panda-on-back-with-stone", "panda", "minecraft:stone")
        },
        Placed {
            flags: &["on_back", "stormy"],
            ..placed("use-stormy-panda-on-back-with-stone", "panda", "minecraft:stone")
        },
        Placed {
            flags: &["responds"],
            ..placed("use-responding-interaction-with-stone", "interaction", "minecraft:stone")
        },
        placed("use-leash-knot-with-stone", "leash_knot", "minecraft:stone"),
        placed("use-leash-knot-with-shears", "leash_knot", "minecraft:shears"),
    ]
}

/// Rows of an entity whose synced state the click depends on.
fn state_table() -> Vec<Placed> {
    vec![Placed {
        flags: &["dancing", "cooling"],
        ..placed("use-cooling-dancing-allay-with-amethyst-shard", "allay", "minecraft:amethyst_shard")
    }]
}

/// The classes only the newest version has. The cushion's click is a consume on the client (a
/// success without a swing), and a pass when the player sneaks.
fn newer_table() -> Vec<Placed> {
    if crate::components::is_26_2() {
        return Vec::new();
    }
    vec![
        placed("use-cushion-with-stone", "cushion", "minecraft:stone"),
        placed("use-cushion-with-cushion", "cushion", "minecraft:white_cushion"),
        placed("use-cushion-empty-handed", "cushion", ""),
        Placed { sneaking: true, ..placed("use-cushion-sneaking", "cushion", "minecraft:stone") },
        // Someone already sits on it: the click passes.
        Placed { flags: &["occupied"], ..placed("use-occupied-cushion-with-stone", "cushion", "minecraft:stone") },
        Placed { flags: &["occupied"], ..placed("use-occupied-cushion-empty-handed", "cushion", "") },
    ]
}

/// A cushion attacked (26.3): a player who may build skips the attack on it as a hanging
/// entity does; one who may not (adventure) goes through the attack as the client runs it.
fn cushion_attack_rows() -> Vec<Value> {
    if crate::components::is_26_2() {
        return Vec::new();
    }
    let cushion = || super::mob(31, "cushion", 0.5, 64.0, 2.5);
    let hit = || [aims(2).clicks(1).go(), aims(3).go()].concat();
    ["survival", "creative", "adventure"]
        .into_iter()
        .map(|mode| facing(&format!("attack-cushion-{mode}"), vec![cushion()], hit()).game_mode(mode).row())
        .collect()
}

fn placed_rows() -> Vec<Value> {
    placed_table()
        .into_iter()
        .chain(state_table())
        .chain(newer_table())
        .map(|row| {
            let [x, y, z] = row.at;
            let mut entity = super::mob(31, row.kind, x, y, z);
            for flag in row.flags.iter().filter(|flag| !matches!(**flag, "stormy" | "occupied")) {
                entity[*flag] = json!(true);
            }
            let mut others = Vec::new();
            if row.flags.contains(&"occupied") {
                // A pig on the cushion, out of the way of the click.
                entity["rider"] = json!(32);
                others.push(super::mob(32, "pig", 4.5, 64.0, 6.5));
            }
            let clicks = match (row.sneaking, row.clicks_twice) {
                (true, _) => sneaking_click(),
                (_, true) => twice(),
                _ => once(),
            };
            let storm = row.flags.contains(&"stormy");
            clicked_in(row.label, [vec![entity], others].concat(), (row.item, row.aim), (clicks, storm))
        })
        .collect()
}

/// A boat under the water for sixty ticks cannot be boarded (`outOfControlTicks`); a click soon after
/// it sank still boards it.
fn submerged_boat_rows() -> Vec<Value> {
    let pool = || -> Vec<Value> {
        (-1..2).flat_map(|x| (1..5).flat_map(move |z| (64..67).map(move |y| put(x, y, z, "minecraft:water")))).collect()
    };
    // (label, boat, ticks under water, sneaking at the click)
    let table = [
        ("use-submerged-boat-after-a-while", "oak_boat", 70, false),
        ("use-submerged-boat-soon", "oak_boat", 20, false),
        ("use-submerged-chest-boat-after-a-while", "oak_chest_boat", 70, false),
        ("use-submerged-chest-boat-sneaking", "oak_chest_boat", 70, true),
    ];
    table
        .into_iter()
        .map(|(label, kind, wait, sneaking)| {
            let click = if sneaking { sneaking_click() } else { once() };
            let clicks = [aims(wait).inventory().go(), click].concat();
            let boat = super::mob(31, kind, 0.5, 64.0, 2.5);
            let mut row = facing(label, vec![boat], clicks)
                .blocks(pool())
                .x_rot(30.0)
                .main_hand("minecraft:stone")
                .count(2)
                .row();
            row["tick_boats"] = json!(true);
            row
        })
        .collect()
}
