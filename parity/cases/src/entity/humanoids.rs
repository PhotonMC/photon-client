//! Zombies, skeletons, piglins, spiders, creepers, villagers, illagers and
//! witches.

use serde_json::{Value, json};

use super::aquatic::model_name;
use super::pose::{Field, Fields, LOOK_WALK, entity, fields, layer_row, row, table};
use crate::line::float_text;

fn with(base: &[(&'static str, Field)], extra: Fields) -> Vec<(&'static str, Field)> {
    [base, extra].concat()
}

fn every(layers: &[&str], defaults: Fields, poses: &[(&str, Fields)]) -> Vec<Value> {
    layers.iter().flat_map(|layer| poses.iter().map(|(label, set)| layer_row(layer, label, defaults, set))).collect()
}

/// The zombie family, grown and baby, from the layer table, posed by the
/// reference's `HumanoidModel.setupAnim` and `animateZombieArms` (and the
/// drowned's swim): standing, looking about, walking, swinging, aggressive
/// or not, and swimming.
pub fn zombie() -> Vec<Value> {
    let defaults =
        with(&LOOK_WALK, fields!(age_in_ticks = 0.0, attack_time = 0.0, aggressive = false, swim_amount = 0.0));
    let swing = fields!(
        head_y_rot = 12.0,
        x_rot = -8.0,
        walk_position = 7.3,
        walk_speed = 0.83,
        attack_time = 0.3,
        aggressive = true,
        age_in_ticks = 64.0
    );
    let poses: [(&str, Fields); 9] = [
        ("standing", fields!()),
        ("aggressive", fields!(aggressive = true, age_in_ticks = 17.5)),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 3.25)),
        ("walk-early", fields!(walk_position = 0.9, walk_speed = 0.4, age_in_ticks = 40.0)),
        ("walk-fast", fields!(walk_position = 21.6, walk_speed = 1.0, age_in_ticks = 123.456)),
        ("swing-early", fields!(attack_time = 0.1, age_in_ticks = 5.0)),
        ("swing-mid", fields!(attack_time = 0.55, aggressive = true, age_in_ticks = 9.0)),
        ("swing-late", fields!(attack_time = 0.9, walk_position = 3.4, walk_speed = 0.6)),
        ("walk-aggressive-swing", swing),
    ];
    let zombies = [
        "minecraft:zombie#main",
        "minecraft:zombie_baby#main",
        "minecraft:husk#main",
        "minecraft:giant#main",
        "minecraft:drowned#main",
        "minecraft:drowned_baby#main",
    ];
    let mut rows = every(&zombies, &defaults, &poses);
    // The zombie villager's own classes pose as a zombie does; a few rows
    // show it.
    let villagers = [
        "minecraft:zombie_villager#main",
        "minecraft:zombie_villager_baby#main",
        "minecraft:zombie_villager_no_hat#main",
    ];
    rows.extend(every(&villagers, &defaults, &[("standing", fields!()), ("walk-aggressive-swing", swing)]));
    let swims: [(&str, Fields); 2] = [
        ("swimming-quarter", fields!(swim_amount = 0.25, age_in_ticks = 11.0, walk_position = 2.0, walk_speed = 0.5)),
        ("swimming-full", fields!(swim_amount = 1.0, age_in_ticks = 33.3, head_y_rot = 20.0, x_rot = 15.0)),
    ];
    let swimmers = ["minecraft:drowned#main", "minecraft:drowned_baby#main", "minecraft:zombie#main"];
    rows.extend(every(&swimmers, &defaults, &swims));
    vec![entity("zombie-model", "zombie_model", rows)]
}

/// The skeleton family from the layer table, posed by
/// `SkeletonModel.setupAnim`: the humanoid pose, and the arms an aggressive
/// skeleton with no bow swings in front of it.
pub fn skeleton() -> Vec<Value> {
    let defaults = with(
        &LOOK_WALK,
        fields!(age_in_ticks = 0.0, attack_time = 0.0, swim_amount = 0.0, aggressive = false, holding_bow = false),
    );
    let poses: [(&str, Fields); 9] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 3.25)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83, age_in_ticks = 40.0)),
        ("aggressive-with-bow", fields!(aggressive = true, holding_bow = true, age_in_ticks = 17.5, attack_time = 0.4)),
        ("aggressive-no-bow", fields!(aggressive = true, age_in_ticks = 17.5)),
        ("aggressive-swing-early", fields!(aggressive = true, attack_time = 0.15, age_in_ticks = 5.0)),
        (
            "aggressive-swing-mid",
            fields!(aggressive = true, attack_time = 0.55, age_in_ticks = 9.0, walk_position = 3.4, walk_speed = 0.6),
        ),
        (
            "aggressive-swing-late",
            fields!(aggressive = true, attack_time = 0.9, age_in_ticks = 123.456, head_y_rot = 12.0, x_rot = -8.0),
        ),
        ("swimming", fields!(swim_amount = 0.6, age_in_ticks = 11.0, walk_position = 2.0, walk_speed = 0.5)),
    ];
    let layers = [
        "minecraft:skeleton#main",
        "minecraft:stray#main",
        "minecraft:stray#outer",
        "minecraft:bogged#main",
        "minecraft:bogged#outer",
        "minecraft:parched#main",
        "minecraft:wither_skeleton#main",
    ];
    vec![entity("skeleton-model", "skeleton_model", every(&layers, &defaults, &poses))]
}

fn piglin_defaults() -> Vec<(&'static str, Field)> {
    with(
        &LOOK_WALK,
        fields!(
            arm_pose = "default",
            main_arm = "right",
            ticks_using_item = 0.0,
            max_crossbow_charge = 25.0,
            age_in_ticks = 0.0,
            attack_time = 0.0,
            swim_amount = 0.0,
            aggressive = false,
            dancing = false
        ),
    )
}

/// The arm poses of a piglin holding its main arm on `arm`.
fn piglin_arms(layer: &str, arm: &'static str, defaults: Fields) -> Vec<Value> {
    let melee = "attacking_with_melee_weapon";
    let poses: [(String, Fields); 6] = [
        (
            format!("melee-{arm}"),
            fields!(arm_pose = melee, main_arm = arm, age_in_ticks = 11.0, walk_position = 2.0, walk_speed = 0.3),
        ),
        (
            format!("melee-swing-{arm}"),
            fields!(arm_pose = melee, main_arm = arm, attack_time = 0.35, age_in_ticks = 11.0, aggressive = true),
        ),
        (
            format!("melee-swing-late-{arm}"),
            fields!(
                arm_pose = melee,
                main_arm = arm,
                attack_time = 0.85,
                age_in_ticks = 3.5,
                head_y_rot = 10.0,
                x_rot = -5.0
            ),
        ),
        (
            format!("crossbow-hold-{arm}"),
            fields!(arm_pose = "crossbow_hold", main_arm = arm, head_y_rot = -14.0, x_rot = 20.0),
        ),
        (
            format!("admiring-{arm}"),
            fields!(arm_pose = "admiring_item", main_arm = arm, age_in_ticks = 7.0, head_y_rot = 30.0, x_rot = -12.0),
        ),
        (format!("admiring-swing-{arm}"), fields!(arm_pose = "admiring_item", main_arm = arm, attack_time = 0.4)),
    ];
    let mut rows: Vec<Value> = poses.iter().map(|(label, set)| layer_row(layer, label, defaults, set)).collect();
    for used in [0.0, 6.0, 12.5, 25.0, 40.0] {
        let label = format!("crossbow-charge-{arm}-{}", float_text(used));
        let set = fields!(arm_pose = "crossbow_charge", main_arm = arm, ticks_using_item = used);
        rows.push(layer_row(layer, &label, defaults, set));
    }
    rows
}

/// The piglin family from the layer table, posed by `PiglinModel` and
/// `ZombifiedPiglinModel.setupAnim`: the ears flapping with the walk, the
/// arms of a zombified piglin held out, and a piglin dancing.
pub fn piglin() -> Vec<Value> {
    let defaults = piglin_defaults();
    let poses: [(&str, Fields); 7] = [
        ("standing", fields!()),
        ("ears-in-the-wind", fields!(age_in_ticks = 57.5)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83, age_in_ticks = 40.0)),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 3.25)),
        ("swing", fields!(attack_time = 0.5, age_in_ticks = 9.0)),
        ("aggressive", fields!(aggressive = true, attack_time = 0.25, age_in_ticks = 21.0)),
        ("swimming", fields!(swim_amount = 0.5, age_in_ticks = 4.0, walk_position = 2.0, walk_speed = 0.4)),
    ];
    let dances: [(&str, Fields); 2] = [
        ("dancing", fields!(arm_pose = "dancing", dancing = true, age_in_ticks = 15.0)),
        ("dancing-late", fields!(arm_pose = "dancing", dancing = true, age_in_ticks = 333.3, head_y_rot = 10.0)),
    ];
    let layers = [
        "minecraft:piglin#main",
        "minecraft:piglin_baby#main",
        "minecraft:piglin_brute#main",
        "minecraft:zombified_piglin#main",
        "minecraft:zombified_piglin_baby#main",
    ];
    let mut rows = Vec::new();
    for layer in layers {
        rows.extend(every(&[layer], &defaults, &poses));
        if !layer.contains("zombified") {
            rows.extend(piglin_arms(layer, "right", &defaults));
            rows.extend(piglin_arms(layer, "left", &defaults));
            rows.extend(every(&[layer], &defaults, &dances));
        }
    }
    vec![entity("piglin-model", "piglin_model", rows)]
}

/// The spider and the cave spider posed by the reference's
/// `SpiderModel.setupAnim` (eight legs, each pair a quarter of a stride
/// apart) and the creeper by `CreeperModel.setupAnim` (the legs swinging in
/// diagonal pairs), drawn.
pub fn spider() -> Vec<Value> {
    let poses: [(&str, Fields); 5] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking-early", fields!(walk_position = 0.9, walk_speed = 0.4)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("running", fields!(walk_position = 21.6, walk_speed = 1.0, head_y_rot = 12.0, x_rot = -8.0)),
    ];
    let spiders = ["minecraft:spider#main", "minecraft:cave_spider#main"];
    let creepers = ["minecraft:creeper#main", "minecraft:creeper#armor"];
    vec![
        entity("spider-model", "spider_model", every(&spiders, &LOOK_WALK, &poses)),
        entity("creeper-model", "creeper_model", every(&creepers, &LOOK_WALK, &poses)),
    ]
}

const VILLAGER_TYPES: &[&str] = &["desert", "jungle", "plains", "savanna", "snow", "swamp", "taiga"];
const PROFESSIONS: &[&str] = &[
    "none",
    "armorer",
    "butcher",
    "cartographer",
    "cleric",
    "farmer",
    "fisherman",
    "fletcher",
    "leatherworker",
    "librarian",
    "mason",
    "nitwit",
    "shepherd",
    "toolsmith",
    "weaponsmith",
];

/// The villagers: the built-in registries' order of the types and the
/// professions; the models posed by the reference's
/// `VillagerModel.setupAnim`; and the passes of the reference's own
/// `VillagerProfessionLayer` for each type and profession, grown and baby,
/// at each level.
pub fn villager() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(age_in_ticks = 0.0, unhappy = false));
    let poses: [(&str, Fields); 5] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 3.25)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83, age_in_ticks = 40.0)),
        ("unhappy", fields!(unhappy = true, age_in_ticks = 17.5)),
        (
            "unhappy-walking",
            fields!(
                unhappy = true,
                walk_position = 2.0,
                walk_speed = 0.4,
                head_y_rot = 12.0,
                x_rot = -8.0,
                age_in_ticks = 63.0
            ),
        ),
    ];
    let layers = [
        "minecraft:villager#main",
        "minecraft:villager_baby#main",
        "minecraft:villager_no_hat#main",
        "minecraft:villager_baby_no_hat#main",
        "minecraft:wandering_trader#main",
    ];
    let layer_row = |path: &str, baby: bool, (kind, profession, level): (&str, &str, i64), label: String| json!({"label": label, "path": path, "baby": baby, "type": kind, "profession": profession, "level": level});
    let mut passes = Vec::new();
    for path in ["villager", "zombie_villager"] {
        for baby in [false, true] {
            for kind in VILLAGER_TYPES {
                for profession in PROFESSIONS {
                    let label = format!("{path}-{}{kind}-{profession}-3", if baby { "baby-" } else { "" });
                    passes.push(layer_row(path, baby, (kind, profession, 3), label));
                }
            }
        }
        for level in [0, 1, 2, 4, 5, 9] {
            let label = format!("{path}-plains-farmer-level-{level}");
            passes.push(layer_row(path, false, ("plains", "farmer", level), label));
        }
    }
    vec![
        table("villager-registries", "villager_registries"),
        entity("villager-model", "villager_model", every(&layers, &defaults, &poses)),
        entity("villager-layers", "villager_layers", passes),
    ]
}

fn illager_defaults() -> Vec<(&'static str, Field)> {
    with(
        &LOOK_WALK,
        fields!(
            age_in_ticks = 0.0,
            riding = false,
            arm_pose = "NEUTRAL",
            main_arm = "right",
            attack_time = 0.0,
            has_item = false,
            max_crossbow_charge = 0,
            ticks_using_item = 0.0,
            hat = false
        ),
    )
}

fn illager(layer: &str, label: &str, set: Fields) -> Value {
    let mut row = row(&format!("{}-{label}", model_name(layer)), &illager_defaults(), set);
    row["layer"] = json!(layer);
    row
}

/// The illagers' and the witch's models posed by the reference's own
/// `IllagerModel.setupAnim` and `WitchModel.setupAnim`: every arm pose the
/// illager model branches on, riding, walking and looking; the witch's nose
/// by the entity's number and by what it holds.
pub fn illager_witch() -> Vec<Value> {
    let mut rows = Vec::new();
    for layer in
        ["minecraft:evoker#main", "minecraft:vindicator#main", "minecraft:pillager#main", "minecraft:illusioner#main"]
    {
        let hat = layer.ends_with("illusioner#main");
        rows.push(illager(layer, "crossed", fields!(arm_pose = "CROSSED", hat = hat)));
        let walking = fields!(
            walk_position = 7.3,
            walk_speed = 0.83,
            age_in_ticks = 40.0,
            head_y_rot = -35.5,
            x_rot = 48.0,
            hat = hat
        );
        rows.push(illager(layer, "neutral-walking", walking));
    }
    let evoker = "minecraft:evoker#main";
    let poses: [(&str, Fields); 8] = [
        ("riding", fields!(riding = true, head_y_rot = 12.0, x_rot = -8.0)),
        ("attacking-empty", fields!(arm_pose = "ATTACKING", attack_time = 0.35, age_in_ticks = 21.5)),
        (
            "attacking-item-right",
            fields!(arm_pose = "ATTACKING", has_item = true, attack_time = 0.35, age_in_ticks = 21.5),
        ),
        (
            "attacking-item-left",
            fields!(arm_pose = "ATTACKING", main_arm = "left", has_item = true, attack_time = 0.7, age_in_ticks = 9.25),
        ),
        ("spellcasting", fields!(arm_pose = "SPELLCASTING", age_in_ticks = 13.75)),
        ("bow", fields!(arm_pose = "BOW_AND_ARROW", head_y_rot = 20.0, x_rot = -15.0)),
        ("crossbow-hold", fields!(arm_pose = "CROSSBOW_HOLD", head_y_rot = -20.0, x_rot = 10.0)),
        ("celebrating", fields!(arm_pose = "CELEBRATING", age_in_ticks = 33.0)),
    ];
    rows.extend(poses.iter().map(|(label, set)| illager(evoker, label, set)));
    for ticks in [0.0, 12.5, 40.0] {
        let set = fields!(arm_pose = "CROSSBOW_CHARGE", max_crossbow_charge = 25, ticks_using_item = ticks);
        rows.push(illager(evoker, &format!("crossbow-charge-{}", float_text(ticks)), set));
    }
    vec![entity("illager-model", "illager_model", rows), entity("witch-model", "witch_model", witches())]
}

fn witches() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(entity_id = 0, holding_item = false, age_in_ticks = 0.0));
    let poses: [(&str, Fields); 5] = [
        ("standing", fields!()),
        ("sniffing-id-7", fields!(entity_id = 7, age_in_ticks = 31.5, head_y_rot = -35.5, x_rot = 48.0)),
        ("sniffing-id-13", fields!(entity_id = 13, age_in_ticks = 9.0)),
        ("walking", fields!(entity_id = 3, age_in_ticks = 40.0, walk_position = 7.3, walk_speed = 0.83)),
        ("holding", fields!(entity_id = 7, holding_item = true, age_in_ticks = 31.5)),
    ];
    poses.iter().map(|(label, set)| row(label, &defaults, set)).collect()
}
