//! Wolves, cats, foxes, pandas, bears, hoglins, ravagers and striders, and
//! the rotations their renderers add.

use serde_json::{Value, json};

use super::pose::{Field, Fields, LOOK_WALK, age_scale, entity, fields, layer_row, row, table};
use crate::line::float_text;

fn with(base: &[(&'static str, Field)], extra: Fields) -> Vec<(&'static str, Field)> {
    [base, extra].concat()
}

/// Rows of every pose on every layer, each with the layer's age scale when
/// `scaled`.
fn posed(layers: &[&str], defaults: Fields, poses: &[(&str, Fields)], scaled: bool) -> Vec<Value> {
    let mut rows = Vec::new();
    for layer in layers {
        for (label, set) in poses {
            let mut row = layer_row(layer, label, defaults, set);
            if scaled {
                row["age_scale"] = json!(crate::bits::f32_bits(age_scale(layer)));
            }
            rows.push(row);
        }
    }
    rows
}

/// The wolf, grown and baby, posed by the reference's own
/// `WolfModel.setupAnim`: the walk, the tail (angry, by health), sitting,
/// the head roll and the body rolled by a shake.
pub fn wolf() -> Vec<Value> {
    let defaults = with(
        &LOOK_WALK,
        fields!(angry = false, sitting = false, tail_angle = 0.6283185, head_roll_angle = 0.0, shake_anim = 0.0),
    );
    let poses: [(&str, Fields); 11] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("angry", fields!(walk_position = 7.3, walk_speed = 0.83, angry = true, tail_angle = 1.5393804)),
        ("tame-hurt", fields!(tail_angle = 1.2)),
        ("sitting", fields!(sitting = true, walk_position = 7.3, walk_speed = 0.83)),
        ("sitting-looking", fields!(sitting = true, head_y_rot = 14.0, x_rot = -20.0, tail_angle = 1.7)),
        ("interested", fields!(head_roll_angle = 0.3, head_y_rot = 10.0)),
        ("shaking", fields!(shake_anim = 0.9, head_roll_angle = 0.1)),
        ("shaking-late", fields!(shake_anim = 1.65)),
        ("shaking-sitting", fields!(shake_anim = 0.5, sitting = true)),
    ];
    let layers = ["minecraft:wolf#main", "minecraft:wolf_baby#main"];
    vec![entity("wolf-model", "wolf_model", posed(&layers, &defaults, &poses, true))]
}

/// The cat and the ocelot, grown and baby, posed by the reference's own
/// `setupAnim`: the walk, crouching, sprinting, sitting, lying down (the
/// tail on its own progress) and relaxing.
pub fn feline() -> Vec<Value> {
    let defaults = with(
        &LOOK_WALK,
        fields!(
            crouching = false,
            sprinting = false,
            sitting = false,
            lie_down_amount = 0.0,
            lie_down_amount_tail = 0.0,
            relax_state_one_amount = 0.0
        ),
    );
    let walk = (7.3, 0.83);
    let poses: [(&str, Fields); 12] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = walk.0, walk_speed = walk.1)),
        ("crouching", fields!(crouching = true, walk_position = walk.0, walk_speed = walk.1)),
        ("sprinting", fields!(sprinting = true, walk_position = walk.0, walk_speed = walk.1)),
        ("sitting", fields!(sitting = true, walk_position = walk.0, walk_speed = walk.1)),
        ("sitting-looking", fields!(sitting = true, head_y_rot = 14.0, x_rot = -20.0)),
        ("crouching-sitting", fields!(crouching = true, sitting = true)),
        ("lying", fields!(lie_down_amount = 0.6, lie_down_amount_tail = 0.32, walk_position = 3.1, walk_speed = 0.4)),
        ("lying-down", fields!(lie_down_amount = 1.0, lie_down_amount_tail = 1.0, head_y_rot = 9.0, x_rot = -12.0)),
        ("relaxed", fields!(relax_state_one_amount = 0.7, x_rot = 20.0)),
        ("lying-relaxed", fields!(lie_down_amount = 0.45, lie_down_amount_tail = 0.16, relax_state_one_amount = 0.3)),
    ];
    let layers =
        ["minecraft:cat#main", "minecraft:cat_baby#main", "minecraft:ocelot#main", "minecraft:ocelot_baby#main"];
    vec![entity("feline-model", "feline_model", posed(&layers, &defaults, &poses, true))]
}

/// The polar bear, grown and baby, posed by the reference's own
/// `PolarBearModel.setupAnim`: the quadruped's walk and look, and how far
/// it has stood up.
pub fn polar_bear() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(stand_scale = 0.0));
    let poses: [(&str, Fields); 5] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("rising", fields!(stand_scale = 0.5, walk_position = 2.1, walk_speed = 0.3)),
        ("risen", fields!(stand_scale = 1.0, head_y_rot = 12.0, x_rot = -20.0)),
    ];
    let layers = ["minecraft:polar_bear#main", "minecraft:polar_bear_baby#main"];
    vec![entity("polar-bear-model", "polar_bear_model", posed(&layers, &defaults, &poses, true))]
}

/// The hoglin and the zoglin, grown and baby, posed by the reference's own
/// `setupAnim`: the ears and legs by the walk, the head by the look and the
/// attack swing.
pub fn hoglin() -> Vec<Value> {
    let defaults = fields!(head_y_rot = 0.0, walk_position = 0.0, walk_speed = 0.0, attack_ticks = 0);
    let poses: [(&str, Fields); 7] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("swing-start", fields!(attack_ticks = 10, head_y_rot = 8.0)),
        ("swing-mid", fields!(attack_ticks = 7, walk_position = 2.2, walk_speed = 0.4)),
        ("swing-peak", fields!(attack_ticks = 5)),
        ("swing-end", fields!(attack_ticks = 1)),
    ];
    let layers =
        ["minecraft:hoglin#main", "minecraft:hoglin_baby#main", "minecraft:zoglin#main", "minecraft:zoglin_baby#main"];
    vec![entity("hoglin-model", "hoglin_model", posed(&layers, defaults, &poses, false))]
}

/// The fox, grown and baby, posed by the reference's own
/// `FoxModel.setupAnim` (the walk, the crouching, sleeping, sitting and
/// pouncing poses, the faceplant), and the textures it wears.
pub fn fox() -> Vec<Value> {
    let defaults = with(
        &LOOK_WALK,
        fields!(
            head_roll_angle = 0.0,
            crouch_amount = 0.0,
            crouching = false,
            sleeping = false,
            sitting = false,
            faceplanted = false,
            pouncing = false,
            age_in_ticks = 37.5
        ),
    );
    let poses: [(&str, Fields); 13] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("walking-slow", fields!(walk_position = 2.1, walk_speed = 0.2, head_roll_angle = 0.2)),
        ("interested", fields!(head_roll_angle = 0.34, head_y_rot = 10.0)),
        ("crouching", fields!(crouching = true, crouch_amount = 3.4, walk_position = 1.0, walk_speed = 0.3)),
        ("crouched", fields!(crouching = true, crouch_amount = 5.0, head_y_rot = 20.0)),
        ("sleeping", fields!(sleeping = true, head_roll_angle = 0.2, head_y_rot = 15.0)),
        ("sitting", fields!(sitting = true, head_y_rot = 14.0, x_rot = -20.0)),
        ("sleeping-and-sitting", fields!(sleeping = true, sitting = true)),
        ("pouncing", fields!(pouncing = true, crouch_amount = 4.0, x_rot = -30.0)),
        ("pouncing-crouched", fields!(pouncing = true, crouching = true, crouch_amount = 5.0)),
        ("faceplanted", fields!(faceplanted = true, x_rot = 60.0, head_y_rot = 5.0)),
    ];
    let layers = ["minecraft:fox#main", "minecraft:fox_baby#main"];
    vec![
        entity("fox-model", "fox_model", posed(&layers, &defaults, &poses, true)),
        table("fox-textures", "fox_textures"),
    ]
}

/// The panda, grown and baby, posed by the reference's own
/// `PandaModel.setupAnim` (the unhappy shake, the sneeze, sitting with
/// eating or fright, lying on the back, the roll), what its renderer turns
/// the pose by (and a cat's and a fox's), and the textures it wears.
pub fn panda() -> Vec<Value> {
    let defaults = with(
        &LOOK_WALK,
        fields!(
            age_in_ticks = 37.5,
            unhappy = false,
            sneezing = false,
            sneeze_time = 0,
            eating = false,
            scared = false,
            sit_amount = 0.0,
            lie_on_back_amount = 0.0,
            roll_amount = 0.0
        ),
    );
    let layers = ["minecraft:panda#main", "minecraft:panda_baby#main"];
    vec![
        entity("panda-model", "panda_model", panda_rows(&layers, &defaults)),
        table("panda-textures", "panda_textures"),
        entity("setup-rotations", "setup_rotations", turns()),
    ]
}

/// The panda's poses; `sneeze` is the sneeze timer of the sneezing rows.
fn panda_rows(layers: &[&str], defaults: Fields) -> Vec<Value> {
    let poses: [(&str, Fields); 14] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("unhappy", fields!(unhappy = true, walk_position = 2.0, walk_speed = 0.3)),
        ("sitting-part", fields!(sit_amount = 0.45, head_y_rot = 12.0, x_rot = -9.0)),
        ("sitting", fields!(sit_amount = 1.0, walk_position = 3.3, walk_speed = 0.5)),
        ("eating", fields!(sit_amount = 1.0, eating = true)),
        ("scared", fields!(sit_amount = 1.0, scared = true)),
        ("eating-and-scared", fields!(sit_amount = 0.8, eating = true, scared = true)),
        ("lying-on-back", fields!(lie_on_back_amount = 0.7, walk_position = 1.1, walk_speed = 0.4)),
        ("lying-on-back-fully", fields!(lie_on_back_amount = 1.0, head_y_rot = 8.0)),
        ("rolling", fields!(roll_amount = 0.6, walk_position = 1.9, walk_speed = 0.6)),
        ("rolling-sitting", fields!(roll_amount = 1.0, sit_amount = 0.5)),
        ("unhappy-and-lying", fields!(unhappy = true, lie_on_back_amount = 0.5, sit_amount = 0.3)),
    ];
    let sneezes = [
        ("sneeze-start", 1),
        ("sneeze-wind-up", 14),
        ("sneeze-hold", 15),
        ("sneeze-release", 17),
        ("sneeze-release-late", 19),
        ("sneeze-done", 20),
    ];
    let mut rows = Vec::new();
    for layer in layers {
        let row = |(label, set): &(&str, Fields)| layer_row(layer, label, defaults, set);
        rows.extend(poses[..4].iter().map(row));
        for (label, time) in sneezes {
            rows.push(layer_row(layer, label, defaults, fields!(sneezing = true, sneeze_time = time)));
        }
        rows.extend(poses[4..].iter().map(row));
    }
    rows
}

fn turn_defaults() -> Vec<(&'static str, Field)> {
    fields!(
        body_rot = 37.0,
        x_rot = 0.0,
        age_in_ticks = 37.5,
        roll_time = 0.0,
        sit_amount = 0.0,
        lie_on_back_amount = 0.0,
        scared = false,
        sitting = false,
        eating = false,
        lie_down_amount = 0.0,
        on_sleeping_player = false,
        pouncing = false,
        faceplanted = false
    )
    .to_vec()
}

fn turn(kind: &str, label: &str, baby: bool, set: Fields) -> Value {
    let label = format!("{kind}-{}{label}", if baby { "baby-" } else { "" });
    let mut row = row(&label, &turn_defaults(), set);
    row["kind"] = json!(kind);
    row["baby"] = json!(baby);
    row
}

fn turns() -> Vec<Value> {
    let mut rows = Vec::new();
    let rolls = [0.0, 0.4, 3.5, 7.0, 7.6, 8.0, 11.25, 15.9, 16.0, 20.5, 23.99, 24.0, 28.7, 31.5, 32.0, 40.0];
    for baby in [false, true] {
        for roll_time in rolls {
            let label = format!("roll-{}", float_text(roll_time));
            rows.push(turn("panda", &label, baby, fields!(roll_time = roll_time, x_rot = -12.0)));
        }
        let poses: [(&str, &str, Fields); 14] = [
            ("panda", "sit-part", fields!(sit_amount = 0.4, x_rot = 18.0, sitting = true)),
            ("panda", "sitting", fields!(sit_amount = 1.0, sitting = true, eating = true)),
            ("panda", "eating-bob", fields!(sit_amount = 1.0, sitting = true, eating = true, age_in_ticks = 91.7)),
            ("panda", "scared", fields!(sit_amount = 1.0, scared = true, sitting = true, age_in_ticks = 12.3)),
            ("panda", "lying-on-back", fields!(lie_on_back_amount = 0.65, x_rot = -20.0)),
            ("panda", "lying-fully", fields!(lie_on_back_amount = 1.0)),
            (
                "panda",
                "rolling-while-sitting",
                fields!(roll_time = 5.5, sit_amount = 0.7, lie_on_back_amount = 0.2, x_rot = 9.0),
            ),
            ("cat", "standing", fields!()),
            ("cat", "lying-part", fields!(lie_down_amount = 0.45)),
            ("cat", "lying", fields!(lie_down_amount = 1.0)),
            ("cat", "lying-on-a-sleeper", fields!(lie_down_amount = 0.8, on_sleeping_player = true)),
            ("fox", "standing", fields!(x_rot = 33.0)),
            ("fox", "pouncing", fields!(x_rot = 33.0, pouncing = true)),
            ("fox", "faceplanted", fields!(x_rot = -47.5, faceplanted = true)),
        ];
        rows.extend(poses.iter().map(|(kind, label, set)| turn(kind, label, baby, set)));
    }
    rows
}
