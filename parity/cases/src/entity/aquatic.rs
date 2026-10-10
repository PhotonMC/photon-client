//! Squid, fish, dolphins, turtles, axolotls, frogs, nautiluses and
//! guardians.

use serde_json::{Value, json};

use super::pose::{Field, Fields, entity, fields, row};
use crate::bits::{f32_bits, f64_bits};
use crate::line::float_text;

/// The squid, grown and baby, in both the squid's and the glow squid's
/// layers, posed by the reference's own `SquidModel.setupAnim` (every
/// tentacle leans by the beat's angle); how its renderer turns the pose (the
/// body's two turns, lower for a baby, no fall once dead); and the light a
/// glow squid gives itself while it is dark.
pub fn squid() -> Vec<Value> {
    let mut rows = Vec::new();
    for layer in ["squid", "squid_baby", "glow_squid", "glow_squid_baby"] {
        for angle in [0.0, 0.2, 0.785398, -0.4] {
            rows.push(json!({
                "label": format!("{layer}-{}", float_text(angle)), "layer": format!("minecraft:{layer}#main"),
                "tentacle_angle": f32_bits(angle),
            }));
        }
    }
    let mut turns = Vec::new();
    for baby in [false, true] {
        let poses = [
            ("still", 0.0, 0.0, 0.0),
            ("tipped", 37.0, -63.5, 0.0),
            ("spinning", -120.0, -89.9, 417.25),
            ("head-down", 12.0, -90.0, 1203.5),
        ];
        for (label, body_rot, x, z) in poses {
            turns.push(json!({
                "label": format!("squid-{}{label}", if baby { "baby-" } else { "" }), "kind": "squid",
                "baby": baby, "body_rot": f32_bits(body_rot), "x_rot": f32_bits(0.0),
                "age_in_ticks": f32_bits(10.5), "x_body_rot": f32_bits(x), "z_body_rot": f32_bits(z),
            }));
        }
    }
    let mut light = Vec::new();
    for dark in [0, 1, 3, 5, 9, 10, 11, 100] {
        for block_light in [0, 7, 15] {
            light.push(json!({
                "label": format!("dark-{dark}-light-{block_light}"), "dark_ticks": dark, "block_light": block_light,
            }));
        }
    }
    vec![
        entity("squid-model", "squid_model", rows),
        entity("squid-rotations", "setup_rotations", turns),
        entity("glow-squid-light", "glow_squid_light", light),
    ]
}

const FISH_LAYERS: &[&str] = &[
    "cod#main",
    "salmon#main",
    "salmon_small#main",
    "salmon_large#main",
    "tropical_fish_small#main",
    "tropical_fish_small#pattern",
    "tropical_fish_large#main",
    "tropical_fish_large#pattern",
    "pufferfish_small#main",
    "pufferfish_medium#main",
    "pufferfish_big#main",
];

/// The fish, posed by the reference's own `setupAnim` of each model (the
/// tail's, the back half's or the fins' swing, wider out of water); how
/// their renderers turn the pose; the textures they wear and a tropical
/// fish's by its packed variant.
pub fn fish() -> Vec<Value> {
    let mut rows = Vec::new();
    for layer in FISH_LAYERS {
        for age in [0.0, 3.5, 17.25, 101.0] {
            for in_water in [true, false] {
                let place = if in_water { "water" } else { "land" };
                rows.push(json!({
                    "label": format!("{layer}-{}-{place}", float_text(age)), "layer": format!("minecraft:{layer}"),
                    "age_in_ticks": f32_bits(age), "in_water": in_water,
                }));
            }
        }
    }
    vec![
        entity("fish-model", "fish_model", rows),
        entity("fish-rotations", "setup_rotations", fish_turns()),
        entity("fish-textures", "fish_textures", fish_textures()),
    ]
}

fn fish_turns() -> Vec<Value> {
    let poses = [
        ("swimming", 37.0, 10.5, true, 0.0),
        ("swimming-later", -120.0, 61.25, true, 0.0),
        ("on-land", 37.0, 10.5, false, 0.0),
        ("on-land-later", 200.0, 33.75, false, 0.0),
        ("dying", 12.0, 21.5, true, 9.0),
        ("dying-on-land", 90.0, 21.5, false, 25.0),
    ];
    let mut turns = Vec::new();
    for kind in ["cod", "salmon", "tropical_fish", "pufferfish"] {
        for (label, body_rot, age, in_water, death) in poses {
            let mut turn = json!({
                "label": format!("{kind}-{label}"), "kind": kind, "baby": false, "body_rot": f32_bits(body_rot),
                "x_rot": f32_bits(0.0), "age_in_ticks": f32_bits(age), "in_water": in_water,
                "feet_x": f32_bits(1.25), "feet_y": f32_bits(64.5), "feet_z": f32_bits(-3.75),
                "scale": f32_bits(1.0),
            });
            if death != 0.0 {
                turn["death_time"] = json!(f32_bits(death));
            }
            turns.push(turn);
        }
    }
    turns
}

fn fish_textures() -> Vec<Value> {
    let mut packed: Vec<i64> = [0, 1].iter().flat_map(|base| (0..6).map(move |index| base | index << 8)).collect();
    packed.extend([0x0600, 0x0002, 0x0101, 0xffff, 0x10100, 0x0e03_0500, 0x11c8_0000, -1]);
    let mut rows: Vec<Value> = ["cod", "salmon", "pufferfish"].iter().map(|name| json!({"label": name})).collect();
    rows.extend(
        packed.iter().map(|value| json!({"label": format!("packed-{:x}", value & 0xffff_ffff), "packed_id": value})),
    );
    rows
}

/// The dolphin, grown and baby, posed by the reference's own
/// `DolphinModel.setupAnim`, and where its layer carries an item; the
/// turtle, grown and baby, posed by `TurtleModel.setupAnim`.
pub fn sea_animal() -> Vec<Value> {
    let mut dolphins = Vec::new();
    let poses = [
        ("still", 0.0, 0.0, 12.5, false),
        ("turned", 33.5, -17.25, 12.5, false),
        ("swimming", 33.5, -17.25, 12.5, true),
        ("swimming-later", -80.0, 41.0, 77.75, true),
    ];
    for layer in ["dolphin", "dolphin_baby"] {
        for (label, y_rot, x_rot, age, moving) in poses {
            dolphins.push(json!({
                "label": format!("{layer}-{label}"), "layer": format!("minecraft:{layer}#main"),
                "y_rot": f32_bits(y_rot), "x_rot": f32_bits(x_rot), "age_in_ticks": f32_bits(age),
                "is_moving": moving,
            }));
        }
    }
    let pitches: Vec<Value> = [0.0, -0.0, 10.0, -10.0, 60.0, -60.0, 90.0, -90.0, 33.3]
        .iter()
        .map(|x_rot| json!({"label": format!("pitch-{}", float_text(*x_rot)), "x_rot": f32_bits(*x_rot)}))
        .collect();
    vec![
        entity("dolphin-model", "sea_animal_model", dolphins),
        entity("dolphin-item-pose", "dolphin_item_pose", pitches),
        entity("turtle-model", "sea_animal_model", turtles()),
    ]
}

fn turtles() -> Vec<Value> {
    let poses = [
        ("standing", 0.0, 0.0, true, false, false),
        ("walking", 3.7, 0.6, true, false, false),
        ("laying", 3.7, 0.6, true, true, false),
        ("swimming", 3.7, 0.6, false, false, false),
        ("swimming-faster", 11.2, 1.0, false, false, false),
        ("with-an-egg", 3.7, 0.6, true, false, true),
        ("with-an-egg-swimming", 5.1, 0.3, false, false, true),
    ];
    let mut rows = Vec::new();
    for layer in ["turtle", "turtle_baby"] {
        for (label, position, speed, on_land, laying, egg) in poses {
            rows.push(json!({
                "label": format!("{layer}-{label}"), "layer": format!("minecraft:{layer}#main"),
                "head_y_rot": f32_bits(25.0), "x_rot": f32_bits(-12.5), "walk_position": f32_bits(position),
                "walk_speed": f32_bits(speed), "on_land": on_land, "laying_egg": laying, "has_egg": egg,
            }));
        }
    }
    rows
}

fn axolotl_defaults() -> Vec<(&'static str, Field)> {
    fields!(
        y_rot = 0.0,
        x_rot = 0.0,
        walk_position = 0.0,
        walk_speed = 0.0,
        age_in_ticks = 37.5,
        playing_dead = 0.0,
        moving = 0.0,
        in_water = 0.0,
        on_ground = 0.0
    )
    .to_vec()
}

fn axolotl_row(prefix: &str, layer: &str, label: &str, set: Fields) -> Value {
    let mut row = row(&format!("{prefix}-{label}"), &axolotl_defaults(), set);
    row["layer"] = json!(layer);
    row
}

/// The axolotl posed by the reference's own `setupAnim`: an adult from the
/// eased factors of swimming, hovering, crawling, lying still and playing
/// dead (alone and blended); a baby from the keyframe animations whose
/// states have started.
pub fn axolotl() -> Vec<Value> {
    let adults: [(&str, Fields); 12] = [
        ("still", fields!()),
        ("swimming", fields!(in_water = 1.0, moving = 1.0)),
        ("swimming-turned", fields!(y_rot = 33.0, x_rot = -25.0, in_water = 1.0, moving = 1.0)),
        ("hovering", fields!(in_water = 1.0)),
        ("crawling", fields!(on_ground = 1.0, moving = 1.0)),
        ("lying-still", fields!(on_ground = 1.0)),
        ("playing-dead", fields!(playing_dead = 1.0)),
        ("swimming-half", fields!(age_in_ticks = 91.25, in_water = 0.5, moving = 0.5)),
        ("hovering-into-lying", fields!(age_in_ticks = 12.0, in_water = 0.3, on_ground = 0.7)),
        ("crawling-into-swimming", fields!(age_in_ticks = 55.5, in_water = 0.4, on_ground = 0.6, moving = 0.8)),
        ("dead-while-swimming", fields!(age_in_ticks = 20.0, in_water = 0.6, moving = 1.0, playing_dead = 0.4)),
        ("a-sliver-of-a-factor", fields!(in_water = 0.000_005, moving = 0.9)),
    ];
    let babies: [(&str, Fields); 10] = [
        ("nothing", fields!()),
        ("swim", fields!(swim = 3)),
        ("swim-later", fields!(age_in_ticks = 90.0, swim = 3)),
        ("walk", fields!(walk_position = 4.5, walk_speed = 0.4, walk = 5)),
        ("walk-fast", fields!(age_in_ticks = 71.0, walk_position = 9.0, walk_speed = 1.0, walk = 5)),
        ("idle-on-ground", fields!(idle_on_ground = 2)),
        ("idle-underwater", fields!(idle_under_water = 2)),
        ("idle-underwater-on-ground", fields!(idle_under_water_on_ground = 2)),
        ("play-dead", fields!(play_dead = 1)),
        ("play-dead-later", fields!(age_in_ticks = 44.0, play_dead = 1)),
    ];
    let mut rows: Vec<Value> =
        adults.iter().map(|(label, set)| axolotl_row("adult", "minecraft:axolotl#main", label, set)).collect();
    rows.extend(babies.iter().map(|(label, set)| axolotl_row("baby", "minecraft:axolotl_baby#main", label, set)));
    vec![entity("axolotl-model", "axolotl_model", rows)]
}

/// The frog and the tadpole posed by the reference's own `setupAnim`: a
/// frog walking (or swimming) by its stride, the speed capped at one and the
/// animations whose states have started; a tadpole's tail swing, wider out
/// of water.
pub fn frog() -> Vec<Value> {
    let defaults = fields!(walk_position = 0.0, walk_speed = 0.0, age_in_ticks = 37.5, swimming = false);
    let poses: [(&str, Fields); 16] = [
        ("still", fields!()),
        ("walking", fields!(walk_position = 4.5, walk_speed = 0.3)),
        ("walking-fast", fields!(walk_position = 9.0, walk_speed = 1.0)),
        ("swimming", fields!(walk_position = 3.0, walk_speed = 0.3, swimming = true)),
        ("swimming-fast", fields!(walk_position = 7.5, walk_speed = 0.9, swimming = true)),
        ("jump", fields!(age_in_ticks = 12.5, jump = 10)),
        ("jump-over", fields!(age_in_ticks = 40.0, jump = 10)),
        ("croak", fields!(age_in_ticks = 40.0, croak = 10)),
        ("croak-over", fields!(age_in_ticks = 100.0, croak = 10)),
        ("tongue", fields!(age_in_ticks = 14.5, tongue = 10)),
        ("tongue-over", fields!(age_in_ticks = 40.0, tongue = 10)),
        ("floating", fields!(age_in_ticks = 33.0, swim_idle = 10)),
        ("floating-looped", fields!(age_in_ticks = 100.0, swim_idle = 10)),
        ("jump-walking", fields!(age_in_ticks = 12.5, walk_position = 4.5, walk_speed = 0.3, jump = 10)),
        (
            "croak-swimming",
            fields!(age_in_ticks = 40.0, walk_position = 3.0, walk_speed = 0.3, swimming = true, croak = 10),
        ),
        (
            "floating-swimming",
            fields!(age_in_ticks = 33.0, walk_position = 3.0, walk_speed = 0.0, swimming = true, swim_idle = 10),
        ),
    ];
    let mut rows: Vec<Value> = poses
        .iter()
        .map(|(label, set)| {
            let mut row = row(&format!("frog-{label}"), defaults, set);
            row["layer"] = json!("minecraft:frog#main");
            row
        })
        .collect();
    for (label, in_water) in [("in-water", true), ("out-of-water", false)] {
        rows.push(json!({
            "label": format!("tadpole-{label}"), "layer": "minecraft:tadpole#main",
            "age_in_ticks": f32_bits(37.5), "in_water": in_water,
        }));
    }
    vec![entity("frog-model", "frog_model", rows)]
}

/// `minecraft:<name>#<part>` -> `<name>`.
pub fn model_name(layer: &str) -> &str {
    let name = layer.split_once(':').map_or(layer, |(_, rest)| rest);
    name.split_once('#').map_or(name, |(name, _)| name)
}

/// Rows labelled by the layer's model name.
pub fn named_row(layer: &str, label: &str, defaults: Fields, set: Fields) -> Value {
    let mut row = row(&format!("{}-{label}", model_name(layer)), defaults, set);
    row["layer"] = json!(layer);
    row
}

/// The nautilus models posed by the reference's own `setupAnim`: the body
/// following the look within ten degrees, the swim animation by the stride
/// and the age, a zombie nautilus alike, and its corals shown unless body
/// armor is worn.
pub fn nautilus() -> Vec<Value> {
    let defaults =
        fields!(age_in_ticks = 37.5, y_rot = 0.0, x_rot = 0.0, walk_position = 0.0, walk_speed = 0.0, armored = false);
    let mut rows = Vec::new();
    for layer in ["minecraft:nautilus#main", "minecraft:nautilus_baby#main"] {
        let poses: [(&str, Fields); 6] = [
            ("still", fields!()),
            ("later", fields!(age_in_ticks = 91.25)),
            ("looks-a-little", fields!(y_rot = 6.5, x_rot = -4.0)),
            ("looks-past-the-limit", fields!(y_rot = -48.0, x_rot = 35.0)),
            ("swims", fields!(walk_position = 4.5, walk_speed = 0.3)),
            ("swims-fast", fields!(age_in_ticks = 12.0, walk_position = 9.0, walk_speed = 1.0)),
        ];
        rows.extend(poses.iter().map(|(label, set)| named_row(layer, label, defaults, set)));
    }
    let swims = fields!(y_rot = -7.0, x_rot = 12.0, walk_position = 3.0, walk_speed = 0.5, age_in_ticks = 91.25);
    for layer in ["minecraft:nautilus#saddle", "minecraft:nautilus_armor#main"] {
        rows.push(named_row(layer, "still", defaults, fields!()));
        rows.push(named_row(layer, "looks-and-swims", defaults, swims));
    }
    let zombie_swims = fields!(y_rot = -7.0, x_rot = 12.0, walk_position = 3.0, walk_speed = 0.5);
    for layer in ["minecraft:zombie_nautilus#main", "minecraft:zombie_nautilus_coral#main"] {
        rows.push(named_row(layer, "still", defaults, fields!()));
        rows.push(named_row(layer, "looks-and-swims", defaults, zombie_swims));
        rows.push(named_row(layer, "armored", defaults, fields!(armored = true)));
    }
    vec![entity("nautilus-model", "nautilus_model", rows)]
}

pub fn vector(values: [f64; 3]) -> Vec<String> {
    values.into_iter().map(f64_bits).collect()
}
