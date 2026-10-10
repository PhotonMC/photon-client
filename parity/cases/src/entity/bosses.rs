//! Cube mobs, shulkers, the ender dragon and end crystals.

use serde_json::{Value, json};

use super::pose::entity;
use crate::bits::{f32_bits, f64_bits};
use crate::case::merge;
use crate::line::float_text;

/// The turn fields every cube mob row shares.
fn cube_turn(label: String, kind: &str, baby: bool, size: i64, squish: f64) -> Value {
    json!({
        "label": label, "kind": kind, "baby": baby, "body_rot": f32_bits(37.0), "x_rot": f32_bits(0.0),
        "age_in_ticks": f32_bits(10.5), "size": size, "squish": f32_bits(squish),
    })
}

fn feet() -> Value {
    json!({"feet_x": f32_bits(3.5), "feet_y": f32_bits(-1.25), "feet_z": f32_bits(0.75), "scale": f32_bits(1.0)})
}

/// The magma cube posed by the reference's own `MagmaCubeModel.setupAnim`
/// (its slices lift apart by the squish, none while it is squashed), and
/// the pose both cube mobs' renderers leave: their size and squish scale it
/// (the slime's drawn a hair smaller first).
pub fn cube_mob() -> Vec<Value> {
    let model: Vec<Value> = [-0.5, -0.1, 0.0, 0.3, 1.0, 1.7]
        .iter()
        .map(|squish| {
            json!({
                "label": format!("magma-cube-{}", float_text(*squish)), "layer": "minecraft:magma_cube#main",
                "squish": f32_bits(*squish),
            })
        })
        .collect();
    let mut rows = Vec::new();
    for kind in ["slime", "magma_cube"] {
        for size in [1, 2, 4, 127] {
            for squish in [-0.5, 0.0, 0.35, 1.0] {
                let label = format!("{kind}-size-{size}-squish-{}", float_text(squish));
                let mut row = cube_turn(label, kind, false, size, squish);
                merge(&mut row, feet());
                rows.push(row);
            }
        }
    }
    rows.extend(sulfur_cubes());
    let mut dying = cube_turn("slime-dying".to_owned(), "slime", false, 3, 0.2);
    merge(&mut dying, json!({"body_rot": f32_bits(-80.0), "age_in_ticks": f32_bits(3.0), "death_time": f32_bits(7.5)}));
    rows.push(dying);
    vec![entity("magma-cube-model", "magma_cube_model", model), entity("cube-mob-rotations", "setup_rotations", rows)]
}

fn sulfur_cubes() -> Vec<Value> {
    let mut rows = Vec::new();
    for (baby, size) in [(false, 2), (true, 1), (false, 3)] {
        for squish in [-0.5, 0.4] {
            for fuse in [0.0, 1.5, 6.25, 9.9, 10.0, 30.0] {
                let small = if baby { "small-" } else { "" };
                let label = format!("sulfur-cube-{small}size-{size}-fuse-{}", float_text(fuse));
                let mut row = cube_turn(label, "sulfur_cube", baby, size, squish);
                merge(&mut row, json!({"fuse_remaining": f32_bits(fuse)}));
                merge(&mut row, feet());
                rows.push(row);
            }
        }
    }
    // A cube that holds a block: no squish, and the block put where
    // `SulfurCubeInnerLayer` draws it.
    for (baby, size) in [(false, 2), (true, 1)] {
        for fuse in [0.0, 6.25] {
            let age = if baby { "baby" } else { "grown" };
            let label = format!("sulfur-block-{age}-size-{size}-fuse-{}", float_text(fuse));
            let mut row = cube_turn(label, "sulfur_cube", baby, size, 0.4);
            merge(&mut row, json!({"fuse_remaining": f32_bits(fuse), "contained_block": true}));
            merge(&mut row, feet());
            rows.push(row);
        }
    }
    // The overlay the block is submitted with: hurt, and a lit fuse (which
    // wins over hurt).
    let overlays =
        [("hurt", 0.0, true), ("lit", 1.5, false), ("lit-and-hurt", 1.5, true), ("fuse-between-flashes", 6.25, true)];
    for (name, fuse, hurt) in overlays {
        let label = format!("sulfur-block-overlay-{name}");
        let mut row = cube_turn(label, "sulfur_cube", false, 2, 0.4);
        merge(&mut row, json!({"fuse_remaining": f32_bits(fuse), "contained_block": true}));
        if hurt {
            merge(&mut row, json!({"red_overlay": true}));
        }
        rows.push(row);
    }
    rows
}

/// The shulker posed by the reference's own `ShulkerModel.setupAnim` (the
/// lid's rise, its twist past a third open, its bob once the lid is more
/// than open, and the head's turn from the body), and the turns its
/// renderer leaves for each face it can be attached to.
pub fn shulker() -> Vec<Value> {
    let mut rows = Vec::new();
    for peek in [0.0, 0.15, 0.3, 0.31, 0.5, 0.75, 1.0] {
        for head in [0.0, 90.0, -137.5] {
            rows.push(json!({
                "label": format!("shulker-peek-{}-head-{}", float_text(peek), float_text(head)),
                "layer": "minecraft:shulker#main", "peek_amount": f32_bits(peek),
                "age_in_ticks": f32_bits(37.25), "x_rot": f32_bits(-12.5), "y_head_rot": f32_bits(head),
                "y_body_rot": f32_bits(0.0),
            }));
        }
    }
    let turn = |label: String, body_rot: f64, age: f64, face: i64| {
        let mut row = json!({
            "label": label, "kind": "shulker", "baby": false, "body_rot": f32_bits(body_rot),
            "x_rot": f32_bits(0.0), "age_in_ticks": f32_bits(age), "attach_face": face,
        });
        merge(&mut row, feet());
        row
    };
    let mut turns = Vec::new();
    for face in 0..6 {
        for body_rot in [0.0, 37.0] {
            turns.push(turn(format!("shulker-face-{face}-body-{}", float_text(body_rot)), body_rot, 10.5, face));
        }
    }
    let mut dying = turn("shulker-dying".to_owned(), 20.0, 3.0, 4);
    dying["death_time"] = json!(f32_bits(7.5));
    turns.push(dying);
    vec![entity("shulker-model", "shulker_model", rows), entity("shulker-rotations", "setup_rotations", turns)]
}

/// `count` flight samples climbing by `climb` and turning by `turn` a tick.
fn history(count: u32, climb: f64, turn: f64, (start_y, start_rot): (f64, f64)) -> Vec<Value> {
    (0..count)
        .map(|index| {
            let i = f64::from(index);
            let y = start_y + climb * i + 0.3 * (i * 0.4).sin();
            let rot = (start_rot + turn * i + 180.0).rem_euclid(360.0) - 180.0;
            json!({"y": f64_bits(y), "y_rot": f32_bits(rot)})
        })
        .collect()
}

const START: (f64, f64) = (70.0, -170.0);

fn dragon(
    label: &str,
    history: Vec<Value>,
    (flap, partial): (f64, f64),
    (landing, sitting, egg): (bool, bool, f64),
) -> Value {
    json!({
        "label": format!("dragon-{label}"), "layer": "minecraft:ender_dragon#main",
        "flap_time": f32_bits(flap), "partial_ticks": f32_bits(partial), "landing_or_taking_off": landing,
        "sitting": sitting, "distance_to_egg": f64_bits(egg), "history": history,
    })
}

/// The ender dragon's model posed by the reference's own
/// `EnderDragonModel.setupAnim`: from a flight history of few and of many
/// samples (the ring wraps after sixty-four), climbing or descending and
/// turning one way or the other, at some points of its wing beat and
/// between two ticks, flying, landing or taking off (the neck follows the
/// distance to the egg), and sitting.
pub fn ender_dragon() -> Vec<Value> {
    let flying = (false, false, 2500.0);
    let usual = (0.37, 0.0);
    let rows = vec![
        dragon("one-sample", history(1, 0.0, 0.0, START), usual, flying),
        dragon("straight", history(30, 0.0, 0.0, START), usual, flying),
        dragon("climbs-and-turns-left", history(40, 0.35, 6.5, START), (1.62, 0.5), flying),
        dragon("descends-and-turns-right", history(50, -0.5, -9.25, (95.0, -170.0)), (2.9, 0.25), flying),
        dragon("wraps-the-ring", history(150, 0.05, 3.1, START), (7.123, 1.0), flying),
        dragon("wraps-the-ring-between-ticks", history(130, -0.12, -2.7, START), (0.0, 0.7), flying),
        dragon("flaps-up", history(45, 0.2, 4.0, START), (0.2, 0.0), flying),
        dragon("flaps-down", history(45, 0.2, 4.0, START), (0.7, 0.0), flying),
        dragon("lands-far-from-the-egg", history(45, -0.4, 2.0, START), usual, (true, false, 3600.0)),
        dragon("lands-near-the-egg", history(45, -0.4, 2.0, START), usual, (true, false, 9.0)),
        dragon("sits", history(45, 0.0, 0.0, START), (0.9, 0.0), (false, true, 2500.0)),
        dragon("sits-and-turns", history(45, 0.0, 5.0, START), usual, (false, true, 2500.0)),
    ];
    vec![entity("ender-dragon-model", "airborne_model", rows)]
}

/// The rays of a dying ender dragon as the reference's own `submitRays`
/// builds them: few rays and many (the count grows with the square of the
/// death time), through the last fifth of it (the rays lengthen and widen,
/// the middle fades) and past the end, from an offset pose.
pub fn dragon_rays() -> Vec<Value> {
    let rays = |label: &str, death_time: f64, at: [f64; 3]| {
        json!({
            "label": label, "x": f32_bits(at[0]), "y": f32_bits(at[1]), "z": f32_bits(at[2]),
            "death_time": f32_bits(death_time),
        })
    };
    let below = [0.0, -1.0, -2.0];
    let times = [
        ("none", 0.0),
        ("barely", 0.02),
        ("few", 0.1),
        ("some", 0.4),
        ("half", 0.5),
        ("three-quarters", 0.75),
        ("at-the-edge", 0.8),
        ("overdriving", 0.9),
        ("nearly-done", 0.99),
        ("done", 1.0),
        ("over", 1.04),
    ];
    let mut rows: Vec<Value> = times.iter().map(|(label, time)| rays(label, *time, below)).collect();
    rows.push(rays("from-an-offset", 0.33, [10.5, 64.9, -3.25]));
    vec![entity("dragon-rays", "dragon_rays", rows)]
}

/// The end crystal's model posed by the reference's own
/// `EndCrystalModel.setupAnim`: at ages through the first beats and far on,
/// with its base and without.
pub fn end_crystal() -> Vec<Value> {
    let mut rows = Vec::new();
    for age in [0.0, 0.5, 3.25, 10.0, 17.7, 31.4, 100.125, 1234.5, 98765.25] {
        for bottom in [true, false] {
            let base = if bottom { "base" } else { "no-base" };
            rows.push(json!({
                "label": format!("crystal-{}-{base}", float_text(age)), "layer": "minecraft:end_crystal#main",
                "age_in_ticks": f32_bits(age), "shows_bottom": bottom,
            }));
        }
    }
    vec![entity("end-crystal-model", "airborne_model", rows)]
}

/// An end crystal's beam as the reference's own `submitCrystalBeams` builds
/// it from the pose it is submitted at: along each axis and slantwise,
/// short and long, over the ticks and from an offset pose, with some light.
pub fn crystal_beam() -> Vec<Value> {
    let beam = |label: &str, delta: [f64; 3], time: f64, at: [f64; 3], light: i64| {
        json!({
            "label": label, "x": f32_bits(at[0]), "y": f32_bits(at[1]), "z": f32_bits(at[2]),
            "dx": f32_bits(delta[0]), "dy": f32_bits(delta[1]), "dz": f32_bits(delta[2]),
            "time": f32_bits(time), "light": light,
        })
    };
    let (origin, full) = ([0.0; 3], 15728880);
    let rows = vec![
        beam("along-z", [0.0, 0.0, 6.0], 12.5, origin, full),
        beam("along-x", [5.0, 0.0, 0.0], 12.5, origin, full),
        beam("straight-up", [0.0, 9.0, 0.0], 12.5, origin, full),
        beam("straight-down", [0.0, -4.0, 0.0], 12.5, origin, full),
        beam("slantwise", [3.0, -1.5, 4.5], 12.5, origin, full),
        beam("behind", [-2.0, 0.5, -7.0], 12.5, origin, full),
        beam("short", [0.2, 0.1, 0.3], 12.5, origin, full),
        beam("at-the-start", [0.0, 0.0, 0.0], 12.5, origin, full),
        beam("long", [20.0, 30.0, -40.0], 3.0, origin, full),
        beam("just-started", [4.0, 8.0, 1.0], 0.0, origin, full),
        beam("later", [4.0, 8.0, 1.0], 1234.5, origin, full),
        beam("lit", [4.0, 8.0, 1.0], 12.5, origin, 240),
        beam("from-an-offset", [2.0, 5.0, -3.0], 12.5, [10.5, 64.9, -3.25], full),
    ];
    vec![entity("crystal-beam", "crystal_beam", rows)]
}
