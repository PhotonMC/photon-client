//! The use key on the remaining mob classes with a `mobInteract` of their own.

use serde_json::{Value, json};

use super::herding::flagged_animal;
use super::{aims, facing};

pub(super) fn rows() -> Vec<Value> {
    let once = || [aims(1).use_clicks(1).inventory().go(), aims(3).inventory().go()].concat();
    let click = |label: &str, kind: &str, flags: &[&str], hurt: Option<f64>, item: &'static str| {
        let mut entity = flagged_animal(kind, flags);
        if let Some(health) = hurt {
            entity["hurt"] = json!(health);
        }
        let aimed = facing(label, vec![entity], once()).x_rot(30.0);
        if item.is_empty() { aimed.row() } else { aimed.main_hand(item).count(2).row() }
    };
    let none = |label: &str, kind: &str, item: &'static str| click(label, kind, &[], None, item);
    let baby = |label: &str, kind: &str, item: &'static str| click(label, kind, &["baby"], None, item);
    vec![
        none("use-ocelot-with-cod", "ocelot", "minecraft:cod"),
        none("use-ocelot-with-stone", "ocelot", "minecraft:stone"),
        none("use-dolphin-with-cod", "dolphin", "minecraft:cod"),
        none("use-dolphin-with-stone", "dolphin", "minecraft:stone"),
        baby("use-baby-dolphin-with-cod", "dolphin", "minecraft:cod"),
        click("use-hurt-iron-golem-with-ingot", "iron_golem", &[], Some(50.0), "minecraft:iron_ingot"),
        click("use-healthy-iron-golem-with-ingot", "iron_golem", &[], Some(100.0), "minecraft:iron_ingot"),
        none("use-iron-golem-with-ingot", "iron_golem", "minecraft:iron_ingot"),
        none("use-iron-golem-with-stone", "iron_golem", "minecraft:stone"),
        none("use-armadillo-with-spider-eye", "armadillo", "minecraft:spider_eye"),
        none("use-armadillo-with-brush", "armadillo", "minecraft:brush"),
        none("use-armadillo-with-stone", "armadillo", "minecraft:stone"),
        baby("use-baby-armadillo-with-spider-eye", "armadillo", "minecraft:spider_eye"),
        none("use-hoglin-with-fungus", "hoglin", "minecraft:crimson_fungus"),
        none("use-hoglin-with-stone", "hoglin", "minecraft:stone"),
        baby("use-baby-hoglin-with-fungus", "hoglin", "minecraft:crimson_fungus"),
        none("use-bee-with-dandelion", "bee", "minecraft:dandelion"),
        none("use-bee-with-stone", "bee", "minecraft:stone"),
        none("use-panda-with-bamboo", "panda", "minecraft:bamboo"),
        none("use-panda-with-stone", "panda", "minecraft:stone"),
        baby("use-baby-panda-with-bamboo", "panda", "minecraft:bamboo"),
        none("use-sniffer-with-seeds", "sniffer", "minecraft:torchflower_seeds"),
        none("use-sniffer-with-stone", "sniffer", "minecraft:stone"),
        baby("use-baby-sniffer-with-seeds", "sniffer", "minecraft:torchflower_seeds"),
        none("use-polar-bear-with-cod", "polar_bear", "minecraft:cod"),
        none("use-happy-ghast-with-snowball", "happy_ghast", "minecraft:snowball"),
        none("use-allay-with-stone", "allay", "minecraft:stone"),
        none("use-allay-with-diamond", "allay", "minecraft:diamond"),
        none("use-bogged-with-shears", "bogged", "minecraft:shears"),
        none("use-copper-golem-with-honeycomb", "copper_golem", "minecraft:honeycomb"),
        none("use-copper-golem-with-copper", "copper_golem", "minecraft:copper_ingot"),
        none("use-copper-golem-with-stone", "copper_golem", "minecraft:stone"),
        none("use-piglin-with-gold", "piglin", "minecraft:gold_ingot"),
        none("use-piglin-with-stone", "piglin", "minecraft:stone"),
        baby("use-baby-piglin-with-gold", "piglin", "minecraft:gold_ingot"),
        baby("use-baby-armadillo-with-brush", "armadillo", "minecraft:brush"),
        none("use-allay-empty-handed", "allay", ""),
        none("use-panda-with-golden-dandelion", "panda", "minecraft:golden_dandelion"),
        baby("use-baby-panda-with-golden-dandelion", "panda", "minecraft:golden_dandelion"),
    ]
}
