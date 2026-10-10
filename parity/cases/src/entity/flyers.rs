//! Blazes, ghasts, phantoms, bats, vexes, allays, parrots and breezes.

use serde_json::{Value, json};

use super::pose::{Fields, entity, fields, prefixed};
use crate::bits::f32_bits;
use crate::line::float_text;

/// The blaze, ghast and phantom posed by the reference's own `setupAnim`,
/// and what a phantom's renderer does to the pose: its size's scale and
/// lift, and its pitch's tip.
pub fn airborne() -> Vec<Value> {
    let mut rows = Vec::new();
    for age in [0.0, 1.5, 37.25, 211.75] {
        let age_text = float_text(age);
        rows.push(json!({
            "label": format!("blaze-{age_text}"), "layer": "minecraft:blaze#main", "age_in_ticks": f32_bits(age),
            "y_rot": f32_bits(0.0), "x_rot": f32_bits(0.0),
        }));
        rows.push(json!({"label": format!("ghast-{age_text}"), "layer": "minecraft:ghast#main", "age_in_ticks": f32_bits(age)}));
        rows.push(json!({"label": format!("phantom-{age_text}"), "layer": "minecraft:phantom#main", "flap_time": f32_bits(age + 9.0)}));
    }
    rows.push(json!({
        "label": "blaze-looks", "layer": "minecraft:blaze#main", "age_in_ticks": f32_bits(12.5),
        "y_rot": f32_bits(-47.0), "x_rot": f32_bits(23.5),
    }));
    rows.push(
        json!({"label": "phantom-flap-offset", "layer": "minecraft:phantom#main", "flap_time": f32_bits(1234.5)}),
    );
    let turn = |label: &str, size: i64, x_rot: f64, body_rot: f64| {
        json!({
            "label": format!("phantom-{label}"), "kind": "phantom", "baby": false, "size": size,
            "x_rot": f32_bits(x_rot), "body_rot": f32_bits(body_rot), "age_in_ticks": f32_bits(10.5),
        })
    };
    let mut turns = vec![
        turn("small", 0, 0.0, 0.0),
        turn("grown", 3, 25.0, 37.0),
        turn("full-size", 64, -40.0, -120.0),
        turn("tipped", 10, 80.5, 12.0),
    ];
    let mut dying = turn("dying", 4, -15.0, 60.0);
    dying["death_time"] = json!(f32_bits(7.5));
    turns.push(dying);
    vec![entity("airborne-model", "airborne_model", rows), entity("phantom-rotations", "setup_rotations", turns)]
}

/// The bat, vex and allay posed by the reference's own `setupAnim`: a bat
/// that hangs turns its head and runs one animation, one that flies runs
/// the other; a vex leaning, arms bobbing, wings beating and charging; an
/// allay flapping, tipping, holding, hovering, dancing and spinning.
pub fn flier() -> Vec<Value> {
    let bats: [(&str, Fields); 7] = [
        ("nothing", fields!()),
        ("flies", fields!(fly = 5)),
        ("flies-later", fields!(age_in_ticks = 90.0, fly = 5)),
        ("hangs", fields!(resting = true, rest = 5, y_rot = 40.0)),
        ("hangs-later", fields!(age_in_ticks = 64.0, resting = true, rest = 5)),
        ("flies-with-a-turned-head", fields!(y_rot = -55.0, fly = 5)),
        ("rests-without-hanging", fields!(y_rot = 25.0, rest = 5)),
    ];
    let bat = fields!(age_in_ticks = 37.5, y_rot = 0.0, resting = false);
    let vexes: [(&str, Fields); 8] = [
        ("hovers", fields!()),
        ("later", fields!(age_in_ticks = 91.25)),
        ("looks", fields!(y_rot = 30.0, x_rot = -15.0)),
        ("charges", fields!(charging = true)),
        ("charges-with-a-right-item", fields!(charging = true, right_item = true)),
        ("charges-with-a-left-item", fields!(charging = true, left_item = true, age_in_ticks = 12.0)),
        ("charges-with-both", fields!(charging = true, right_item = true, left_item = true, age_in_ticks = 60.0)),
        ("holds-an-item-without-charging", fields!(right_item = true)),
    ];
    let vex =
        fields!(age_in_ticks = 37.5, y_rot = 0.0, x_rot = 0.0, charging = false, right_item = false, left_item = false);
    let mut rows = prefixed("bat", "minecraft:bat#main", bat, &bats);
    rows.extend(prefixed("vex", "minecraft:vex#main", vex, &vexes));
    rows.extend(allays());
    vec![entity("flier-model", "airborne_model", rows)]
}

fn allays() -> Vec<Value> {
    let defaults = fields!(
        age_in_ticks = 37.5,
        y_rot = 0.0,
        x_rot = 0.0,
        walk_position = 0.0,
        walk_speed = 0.0,
        dancing = false,
        spinning = false,
        spinning_progress = 0.0,
        holding_progress = 0.0
    );
    let poses: [(&str, Fields); 13] = [
        ("hovers", fields!()),
        ("hovers-later", fields!(age_in_ticks = 91.25)),
        ("looks", fields!(y_rot = -30.0, x_rot = 20.0)),
        ("flies", fields!(walk_position = 3.0, walk_speed = 0.5)),
        ("flies-slowly", fields!(walk_position = 1.5, walk_speed = 0.12, age_in_ticks = 12.0)),
        ("holds", fields!(holding_progress = 1.0)),
        ("part-holds", fields!(holding_progress = 0.4, age_in_ticks = 60.0)),
        ("flies-holding", fields!(walk_position = 3.0, walk_speed = 0.5, holding_progress = 1.0)),
        ("dances", fields!(dancing = true, age_in_ticks = 20.0)),
        ("dances-later", fields!(dancing = true, age_in_ticks = 75.5)),
        ("spins", fields!(dancing = true, spinning = true, spinning_progress = 0.6)),
        ("dances-flying", fields!(dancing = true, walk_position = 2.0, walk_speed = 0.4, age_in_ticks = 44.0)),
        ("dances-holding", fields!(dancing = true, holding_progress = 1.0, spinning = true, spinning_progress = 1.0)),
    ];
    prefixed("allay", "minecraft:allay#main", defaults, &poses)
}

/// The parrot posed by the reference's own `ParrotModel.setupAnim`: each
/// pose, with the wings at rest and beating, and the texture of each
/// variant (ids past the ends clamp).
pub fn parrot() -> Vec<Value> {
    let defaults =
        fields!(age_in_ticks = 37.5, y_rot = 0.0, x_rot = 0.0, walk_position = 0.0, walk_speed = 0.0, flap_angle = 0.0);
    let mut rows = Vec::new();
    for pose in ["FLYING", "STANDING", "SITTING", "PARTY", "ON_SHOULDER"] {
        let name = pose.to_lowercase().replace('_', "-");
        let poses: [(String, Fields); 4] = [
            (name.clone(), fields!()),
            (format!("{name}-beating"), fields!(flap_angle = 0.8)),
            (format!("{name}-looking"), fields!(y_rot = 35.0, x_rot = -20.0, flap_angle = 0.3)),
            (
                format!("{name}-walking"),
                fields!(walk_position = 4.5, walk_speed = 0.6, flap_angle = 0.5, age_in_ticks = 91.25),
            ),
        ];
        for (label, set) in &poses {
            let mut row = super::pose::row(&format!("parrot-{label}"), defaults, set);
            row["layer"] = json!("minecraft:parrot#main");
            row["pose"] = json!(pose);
            rows.push(row);
        }
    }
    let textures = [-3, 0, 1, 2, 3, 4, 5, 9]
        .iter()
        .map(|variant| json!({"label": format!("variant-{variant}"), "id": variant}))
        .collect();
    vec![entity("parrot-model", "airborne_model", rows), entity("parrot-textures", "parrot_textures", textures)]
}

/// The breeze's three models posed by the reference's own
/// `BreezeModel.setupAnim`: each of the six animations alone (the looping
/// idle, the others running out), early and late, and some together.
pub fn breeze() -> Vec<Value> {
    let defaults = fields!(age_in_ticks = 37.5);
    let poses: [(&str, Fields); 11] = [
        ("nothing", fields!()),
        ("idle", fields!(idle = 3)),
        ("idle-later", fields!(age_in_ticks = 111.0, idle = 3)),
        ("shoot", fields!(age_in_ticks = 45.0, shoot = 30)),
        ("shoot-over", fields!(age_in_ticks = 120.0, shoot = 30)),
        ("slide", fields!(age_in_ticks = 50.0, slide = 40)),
        ("slide-back", fields!(age_in_ticks = 62.0, slide_back = 55)),
        ("inhale", fields!(age_in_ticks = 41.0, inhale = 33)),
        ("jump", fields!(age_in_ticks = 60.0, long_jump = 50)),
        ("idle-shooting", fields!(age_in_ticks = 45.0, idle = 3, shoot = 30)),
        ("sliding-then-back", fields!(age_in_ticks = 70.0, slide = 40, slide_back = 65, idle = 3)),
    ];
    let rows = ["main", "wind", "eyes"]
        .iter()
        .flat_map(|layer| prefixed(&format!("breeze-{layer}"), &format!("minecraft:breeze#{layer}"), defaults, &poses))
        .collect();
    vec![entity("breeze-model", "airborne_model", rows)]
}

/// The happy ghast posed by the reference's own `HappyGhastModel.setupAnim`
/// (tentacles swaying out of step; the body squeezed while it wears
/// something), grown and baby, and the harness's goggles on the brow or,
/// ridden, over the eyes.
pub fn happy_ghast() -> Vec<Value> {
    let ghast = |label: String, layer: &str, age: f64, body_item: bool, ridden: bool| {
        json!({
            "label": label, "layer": format!("minecraft:{layer}#main"), "age_in_ticks": f32_bits(age),
            "body_item": body_item, "ridden": ridden,
        })
    };
    let mut rows = Vec::new();
    for layer in ["happy_ghast", "happy_ghast_baby"] {
        for age in [0.0, 1.5, 37.25, 211.75] {
            rows.push(ghast(format!("{layer}-{}", float_text(age)), layer, age, false, false));
        }
        rows.push(ghast(format!("{layer}-harnessed"), layer, 37.25, true, false));
    }
    for layer in ["happy_ghast_harness", "happy_ghast_baby_harness"] {
        for ridden in [false, true] {
            let state = if ridden { "ridden" } else { "idle" };
            rows.push(ghast(format!("{layer}-{state}"), layer, 0.0, true, ridden));
        }
    }
    vec![entity("happy-ghast-model", "airborne_model", rows)]
}

/// Where a vex's and an allay's `ItemInHandLayer` puts what each hand holds
/// (from an identity pose).
pub fn flier_item() -> Vec<Value> {
    let defaults = fields!(
        holds_right = true,
        holds_left = false,
        age_in_ticks = 37.5,
        charging = false,
        main_arm = "right",
        walk_position = 0.0,
        walk_speed = 0.0,
        holding_progress = 0.0
    );
    let rows: [(&str, &str, Fields); 12] = [
        ("vex", "right", fields!()),
        ("vex", "left", fields!(holds_right = false, holds_left = true)),
        ("vex", "both", fields!(holds_left = true, age_in_ticks = 91.25)),
        ("vex", "charging-right", fields!(charging = true)),
        ("vex", "charging-left", fields!(charging = true, holds_right = false, holds_left = true)),
        ("vex", "charging-both", fields!(charging = true, holds_left = true, age_in_ticks = 12.0)),
        ("allay", "right", fields!(holding_progress = 1.0)),
        ("allay", "left", fields!(holds_right = false, holds_left = true, holding_progress = 1.0)),
        ("allay", "both", fields!(holds_left = true, holding_progress = 1.0, age_in_ticks = 60.0)),
        ("allay", "flying", fields!(holding_progress = 1.0, walk_position = 3.0, walk_speed = 0.5)),
        (
            "allay",
            "part-holding",
            fields!(holding_progress = 0.4, walk_position = 1.5, walk_speed = 0.12, age_in_ticks = 12.0),
        ),
        ("allay", "hovering-empty-handed", fields!(holds_right = false)),
    ];
    let rows = rows
        .iter()
        .map(|(kind, label, set)| {
            let mut row = super::pose::row(&format!("{kind}-{label}"), defaults, set);
            row["kind"] = json!(kind);
            row
        })
        .collect();
    vec![entity("flier-item-pose", "flier_item_pose", rows)]
}
