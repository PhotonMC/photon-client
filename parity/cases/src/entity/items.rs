//! Dropped items, quadrupeds, keyframe animations, glints, equippable
//! components, armor stands and experience orbs.

use serde_json::{Value, json};

use super::pose::{Fields, LOOK_WALK, entity, fields, layer_row, table};
use crate::bits::f32_bits;
use crate::line::float_text;

type Bounds = ([f64; 3], [f64; 3]);

const BLOCK: Bounds = ([-0.25, 0.0, -0.25], [0.25, 0.5, 0.25]);
const SPRITE: Bounds = ([-0.25, -0.25, -0.015625], [0.25, 0.25, 0.015625]);

fn item_row(label: String, (age, bob): (f64, f64), count: i64, seed: i64, (low, high): Bounds) -> Value {
    let mut row = json!({"label": label, "age": f32_bits(age), "bob": f32_bits(bob), "count": count, "seed": seed});
    for (index, axis) in ["x", "y", "z"].into_iter().enumerate() {
        row[format!("min_{axis}")] = json!(f32_bits(low[index]));
        row[format!("max_{axis}")] = json!(f32_bits(high[index]));
    }
    row
}

/// A dropped item's poses (`ItemEntityRenderer.submit`): the bob and spin
/// through the age, each copy of the stack (one to five) with the offsets
/// its seed gives, for a thick model (a block) and a flat one (a sprite),
/// and models that sit off the origin or have no thickness at all.
pub fn item_pose() -> Vec<Value> {
    let mut rows = Vec::new();
    for count in 1..=5 {
        rows.push(item_row(format!("block-{count}"), (12.5, 1.25), count, 1 + count, BLOCK));
        rows.push(item_row(format!("sprite-{count}"), (12.5, 1.25), count, 262 + count, SPRITE));
    }
    for age in [0.0, 0.5, 7.0, 31.25, 100.75, 1200.5, 20000.0] {
        rows.push(item_row(format!("age-{}", float_text(age)), (age, 0.7), 1, 5, BLOCK));
    }
    for bob in [0.0, 1.0, 3.1415927, 6.2831855] {
        rows.push(item_row(format!("bob-{}", float_text(bob)), (40.0, bob), 2, 5, SPRITE));
    }
    for seed in [0, 1, 187, 2025, -5, 2147483647] {
        rows.push(item_row(format!("seed-{seed}"), (3.0, 2.0), 5, seed, BLOCK));
        rows.push(item_row(format!("flat-seed-{seed}"), (3.0, 2.0), 5, seed, SPRITE));
    }
    let odd: [(&str, i64, Bounds); 5] = [
        ("thick-sprite", 3, ([-0.25, -0.25, -0.05], [0.25, 0.25, 0.05])),
        ("off-centre", 4, ([0.0, 0.125, 0.0], [0.5, 0.375, 0.2])),
        ("flat-plane", 3, ([-0.25, -0.25, 0.0], [0.25, 0.25, 0.0])),
        ("depth-at-threshold", 3, ([-0.25, -0.25, 0.0], [0.25, 0.25, 0.0625])),
        ("depth-over-threshold", 3, ([-0.25, -0.25, 0.0], [0.25, 0.25, 0.0626])),
    ];
    for (label, count, bounds) in odd {
        rows.push(item_row(label.to_owned(), (9.0, 0.3), count, 17, bounds));
    }
    vec![entity("item-entity-pose", "item_entity_pose", rows)]
}

/// The pig and the cow, grown and baby, from the layer table, posed by the
/// reference's `QuadrupedModel.setupAnim`: standing, looking about, walking
/// at several points of the stride and speeds.
pub fn quadruped() -> Vec<Value> {
    let poses: [(&str, Fields); 7] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("looking-up", fields!(head_y_rot = 70.0, x_rot = -60.0)),
        ("walk-early", fields!(walk_position = 0.9, walk_speed = 0.4)),
        ("walk-mid", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("walk-fast", fields!(walk_position = 21.6, walk_speed = 1.0)),
        ("walk-and-look", fields!(head_y_rot = 12.0, x_rot = -8.0, walk_position = 3.4, walk_speed = 0.6)),
    ];
    let layers = [
        "minecraft:pig#main",
        "minecraft:pig_baby#main",
        "minecraft:cow#main",
        "minecraft:cow_baby#main",
        "minecraft:cold_pig#main",
        "minecraft:mooshroom#main",
        "minecraft:mooshroom_baby#main",
    ];
    let rows = layers
        .iter()
        .flat_map(|layer| poses.iter().map(|(label, set)| layer_row(layer, label, &LOOK_WALK, set)))
        .collect();
    vec![entity("quadruped-model", "quadruped_model", rows)]
}

/// The reference's keyframe animations: every definition, a rabbit model
/// posed by the reference's own `KeyframeAnimation.apply` at points of the
/// hop and of the idle head tilt, and `AnimationState`'s timing.
pub fn animation() -> Vec<Value> {
    let pose = |layer: &str, definition: &str, millis: i64, scale: f64| {
        json!({
            "label": format!("{layer}-{definition}-{millis}-{}", float_text(scale)), "layer": layer,
            "definition": definition, "millis": millis, "scale": f32_bits(scale),
        })
    };
    let mut rows = Vec::new();
    let rabbits = [
        ("minecraft:rabbit#main", "Rabbit.HOP", "Rabbit.IDLE_HEAD_TILT"),
        ("minecraft:rabbit_baby#main", "BabyRabbit.HOP", "BabyRabbit.IDLE_HEAD_TILT"),
    ];
    for (layer, hop, tilt) in rabbits {
        for millis in [0, 62, 125, 208, 250, 291, 375, 500, 583, 749, 750, 1100] {
            rows.push(pose(layer, hop, millis, 1.0));
        }
        rows.extend([pose(layer, hop, 300, 0.5), pose(layer, hop, 300, 0.0)]);
        for millis in [0, 400, 1000, 2000, 3500, 3999, 4000, 6000] {
            rows.push(pose(layer, tilt, millis, 1.0));
        }
    }
    vec![
        table("animation-definitions", "animation_definitions"),
        entity("keyframe-animation", "keyframe_animation", rows),
        entity("animation-states", "animation_states", animation_states()),
    ]
}

fn call(name: &str, age: f64, more: Value) -> Value {
    let mut call = json!({"call": name, "age": f32_bits(age)});
    crate::case::merge(&mut call, more);
    call
}

fn animation_states() -> Vec<Value> {
    let tick = |tick: i64| json!({ "tick": tick });
    let forward = |ticks: i64, scale: f64| json!({"ticks": ticks, "scale": f32_bits(scale)});
    let when = |condition: bool, tick: i64| json!({"condition": condition, "tick": tick});
    vec![
        json!({"label": "start-and-run", "steps": [
            call("start", 10.0, tick(10)), call("start", 10.5, tick(10)), call("start", 25.25, tick(10)),
        ]}),
        json!({"label": "start-if-stopped", "steps": [
            call("start_if_stopped", 3.0, tick(3)), call("start_if_stopped", 9.5, tick(8)),
            call("stop", 12.0, json!({})), call("start_if_stopped", 15.0, tick(14)),
        ]}),
        json!({"label": "animate-when", "steps": [
            call("animate_when", 5.0, when(true, 5)), call("animate_when", 9.0, when(true, 9)),
            call("animate_when", 11.0, when(false, 11)), call("animate_when", 20.0, when(true, 20)),
        ]}),
        json!({"label": "fast-forward", "steps": [
            call("fast_forward", 30.0, forward(5, 1.5)), call("start", 30.0, tick(30)),
            call("fast_forward", 30.0, forward(5, 1.5)), call("fast_forward", 30.5, forward(7, 0.4)),
        ]}),
    ]
}

/// What decides an enchantment glint and how the reference draws it:
/// `hasFoil` of real stacks with component patches as the wire carries
/// them, and the matrix that moves the armor glint's texture.
pub fn glint() -> Vec<Value> {
    let foil = |label: &str, item: &str, patch: &str| json!({"label": label, "item": item, "patch": patch});
    let rows = vec![
        json!({"label": "foil-defaults", "all": true}),
        foil("foil-plain-armor", "minecraft:diamond_chestplate", "0000"),
        foil("foil-enchanted-armor", "minecraft:diamond_chestplate", "01000d0200030101"),
        foil("foil-enchanted-but-overridden-off", "minecraft:diamond_chestplate", "02000d0100011500"),
        foil("foil-overridden-on", "minecraft:leather_boots", "01001501"),
        foil("foil-removed-override", "minecraft:enchanted_golden_apple", "000115"),
        foil("foil-no-enchantments", "minecraft:iron_helmet", "01000d00"),
        foil("foil-enchanted-book-overridden-off", "minecraft:enchanted_book", "01001500"),
    ];
    let matrices = [
        (0, 0.5),
        (1234, 0.5),
        (27500, 0.5),
        (55000, 0.5),
        (1_000_000, 0.5),
        (987654321, 0.5),
        (5000, 0.0),
        (5000, 1.0),
        (123456, 0.25),
    ]
    .iter()
    .map(|(millis, speed)| {
        json!({"label": format!("glint-matrix-{millis}-{}", float_text(*speed)), "millis": millis, "speed": speed})
    })
    .collect();
    vec![entity("item-foil", "item_foil", rows), entity("glint-layers", "glint_layers", matrices)]
}

fn name(text: &str) -> String {
    format!("{:02x}{}", text.len(), crate::bits::hex(text.as_bytes()))
}

/// An `equippable` component patch: slot, equip sound, asset, overlay,
/// allowed entities, five flags and the shearing sound.
fn equippable(slot: u8, asset: Option<&str>, sound: &str) -> String {
    let asset = asset.map_or("00".to_owned(), |asset| format!("01{}", name(asset)));
    format!("010020{slot:02x}{sound}{asset}0000010100000001")
}

/// The `equippable` component of real stacks: the item's own, a patch that
/// gives it another asset, a slot and an asset to an item that is not worn,
/// no asset at all, a sound the patch names whole, and a patch that removes
/// it.
pub fn equippable_cases() -> Vec<Value> {
    let row = |label: &str, item: &str, patch: String| json!({"label": label, "item": item, "patch": patch});
    let pig = format!("00{}00", name("minecraft:entity.pig.ambient"));
    let ranged = format!("00{}013f800000", name("minecraft:x"));
    let rows = vec![
        row("equippable-default-chestplate", "minecraft:diamond_chestplate", "0000".to_owned()),
        row("equippable-default-stick", "minecraft:stick", "0000".to_owned()),
        row("equippable-other-asset", "minecraft:leather_helmet", equippable(4, Some("minecraft:netherite"), "01")),
        row("equippable-on-a-stick", "minecraft:stick", equippable(3, Some("minecraft:gold"), "01")),
        row("equippable-no-asset", "minecraft:iron_boots", equippable(1, None, "01")),
        row("equippable-whole-sound", "minecraft:stick", equippable(2, Some("minecraft:chainmail"), &pig)),
        row("equippable-whole-sound-range", "minecraft:stick", equippable(7, Some("minecraft:saddle"), &ranged)),
        row("equippable-removed", "minecraft:diamond_chestplate", "000120".to_owned()),
    ];
    vec![entity("item-equippable", "item_equippable", rows)]
}

type Poses = [(&'static str, [f64; 3]); 6];

const STAND_DEFAULT: Poses = [
    ("head_pose", [0.0, 0.0, 0.0]),
    ("body_pose", [0.0, 0.0, 0.0]),
    ("left_arm_pose", [-10.0, 0.0, -10.0]),
    ("right_arm_pose", [-15.0, 0.0, 10.0]),
    ("left_leg_pose", [-1.0, 0.0, -1.0]),
    ("right_leg_pose", [1.0, 0.0, 1.0]),
];
const STAND_CUSTOM: Poses = [
    ("head_pose", [12.5, -35.0, 7.25]),
    ("body_pose", [-8.0, 20.0, 3.0]),
    ("left_arm_pose", [-80.0, 15.0, -45.0]),
    ("right_arm_pose", [30.0, -60.0, 100.0]),
    ("left_leg_pose", [25.0, 0.0, -12.0]),
    ("right_leg_pose", [-35.0, 10.0, 12.0]),
];

fn stand(layer: &str, label: &str, poses: &Poses, (arms, plate, y_rot): (bool, bool, f64)) -> Value {
    let mut row = json!({
        "label": format!("{layer}-{label}"), "layer": layer, "y_rot": f32_bits(y_rot),
        "show_arms": arms, "show_base_plate": plate,
    });
    for (key, angles) in poses {
        row[*key] = json!(angles.iter().map(|angle| f32_bits(*angle)).collect::<Vec<_>>());
    }
    row
}

/// An armor stand's model and the models of the armor it wears (grown and
/// small): the default pose, a custom one, arms shown or not, the base
/// plate and a turn.
pub fn armor_stand() -> Vec<Value> {
    let mut rows = Vec::new();
    for size in ["armor_stand", "armor_stand_small"] {
        for part in ["main", "helmet", "chestplate", "leggings", "boots"] {
            let layer = format!("minecraft:{size}#{part}");
            rows.extend([
                stand(&layer, "default", &STAND_DEFAULT, (false, true, 0.0)),
                stand(&layer, "with-arms", &STAND_DEFAULT, (true, true, 0.0)),
                stand(&layer, "custom", &STAND_CUSTOM, (true, true, 47.5)),
                stand(&layer, "no-plate", &STAND_CUSTOM, (false, false, -120.0)),
            ]);
        }
    }
    vec![entity("armor-stand-model", "armor_stand_model", rows)]
}

/// Every equipment asset of the game's resources, through the reference's
/// own codec.
pub fn equipment_layer() -> Vec<Value> {
    vec![table("equipment-layers", "equipment_layers")]
}

fn orb(label: String, set: Fields) -> Value {
    let defaults = fields!(icon = 3, age = 0.0, yaw = 0.0, pitch = 0.0, px = 0.0, py = 0.0, pz = 3.0, light = 15728880);
    super::pose::row(&label, defaults, set)
}

/// An experience orb's quad as `ExperienceOrbRenderer.submit` writes it:
/// each icon's texels, the colour swaying with its age, and the quad turned
/// to the camera for several ways of looking, at several places.
pub fn xp_orb() -> Vec<Value> {
    let mut rows: Vec<Value> =
        (0..11).map(|icon| orb(format!("icon-{icon}"), fields!(icon = icon, age = 12.5))).collect();
    for age in [0.0, 1.5, 7.3, 18.85, 40.25, 99.9, 300.0, 5000.5] {
        rows.push(orb(format!("age-{}", float_text(age)), fields!(age = age)));
    }
    for (yaw, pitch) in [(0.0, 0.0), (90.0, 0.0), (-135.5, 20.0), (37.5, -60.0), (180.0, 89.0), (400.0, 12.0)] {
        let label = format!("looking-{}-{}", float_text(yaw), float_text(pitch));
        rows.push(orb(label, fields!(yaw = yaw, pitch = pitch, age = 4.0)));
    }
    rows.extend([
        orb("left-below".to_owned(), fields!(px = -2.5, py = -1.25, pz = 1.75, yaw = 20.0, pitch = 10.0)),
        orb("far".to_owned(), fields!(px = 30.5, py = 4.0, pz = -22.25, yaw = -70.0, pitch = -5.0, age = 77.7)),
        orb("dark".to_owned(), fields!(light = 0)),
        orb("sky-lit".to_owned(), fields!(light = 15 << 20)),
        orb("torch-lit".to_owned(), fields!(light = 15 << 4)),
    ]);
    vec![entity("xp-orb", "experience_orb", rows)]
}
