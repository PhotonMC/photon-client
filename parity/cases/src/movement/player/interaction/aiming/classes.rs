//! The use key on the remaining mob classes with a `mobInteract` of their own.

use serde_json::{Value, json};

use super::herding::{flagged_animal, sneaking_click};
use super::{aims, facing};

pub(super) fn rows() -> Vec<Value> {
    [plain_rows(), tiny_rows(), health_rows(), worn_rows(), placed_rows()].concat()
}

/// One use click, then a stretch of ticks with no input.
fn once() -> Vec<Value> {
    [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat()
}

/// A row that clicks the entity with `item` (nothing when it is empty) from the aim `x_rot`.
fn clicked(label: &str, entity: Value, (item, x_rot): (&'static str, f64), clicks: Vec<Value>) -> Value {
    let aimed = facing(label, vec![entity], clicks).x_rot(x_rot);
    if item.is_empty() { aimed.row() } else { aimed.main_hand(item).count(2).row() }
}

/// Adults (`""`) and babies (`"baby"`) with the item in hand: label, flag, kind, item.
const PLAIN: &[(&str, &str, &str, &str)] = &[
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
            let flags: &[&str] = if flag.is_empty() { &[] } else { &["baby"] };
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
    .collect()
}

/// Rows that read the equipment the mob wears: label, kind, flag, worn item, held item, sneaking.
const WORN: &[(&str, &str, &str, &str, &str, bool)] = &[
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

fn worn_rows() -> Vec<Value> {
    WORN.iter()
        .map(|&(label, kind, key, worn, held, sneaking)| {
            let mut entity = flagged_animal(kind, &[]);
            entity[key] = json!(worn);
            let clicks = if sneaking { sneaking_click() } else { once() };
            clicked(label, entity, (held, 30.0), clicks)
        })
        .collect()
}

/// Rows that put the entity where the test needs it: label, kind, flags, position, item, aim.
type Placed = (&'static str, &'static str, &'static [&'static str], [f64; 3], &'static str, f64);

const PLACED: &[Placed] = &[
    // Within reach of the eyes, but more than three blocks from the feet.
    ("use-far-ocelot-with-cod", "ocelot", &[], [0.5, 65.5, 3.7], "minecraft:cod", 0.0),
    ("use-near-ocelot-with-cod", "ocelot", &[], [0.5, 65.5, 3.0], "minecraft:cod", 0.0),
    ("use-locked-tadpole-with-slime", "tadpole", &["age_locked"], [0.5, 64.0, 2.5], "minecraft:slime_ball", 36.0),
    ("use-tadpole-with-golden-dandelion", "tadpole", &[], [0.5, 64.0, 2.5], "minecraft:golden_dandelion", 36.0),
    (
        "use-locked-tadpole-with-golden-dandelion",
        "tadpole",
        &["age_locked"],
        [0.5, 64.0, 2.5],
        "minecraft:golden_dandelion",
        36.0,
    ),
    ("use-bee-with-closed-eyeblossom", "bee", &[], [0.5, 64.0, 2.5], "minecraft:closed_eyeblossom", 30.0),
];

fn placed_rows() -> Vec<Value> {
    PLACED
        .iter()
        .map(|&(label, kind, flags, [x, y, z], item, x_rot)| {
            let mut entity = super::mob(31, kind, x, y, z);
            for flag in flags {
                entity[*flag] = json!(true);
            }
            clicked(label, entity, (item, x_rot), once())
        })
        .collect()
}
