//! The guardian's model, eye and beam.

use serde_json::{Value, json};

use super::aquatic::{named_row, vector};
use super::pose::{Fields, entity, fields};
use crate::bits::f32_bits;

/// Where the eye is, where it looks and what it looks at.
fn looking(at: [f64; 3], direction: [f64; 3]) -> Value {
    json!({
        "eye_position": vector([0.0, 1.0, 0.0]), "look_direction": vector(direction),
        "look_at_position": vector(at),
    })
}

const AHEAD: [f64; 3] = [0.0, 0.0, 1.0];

/// The guardian posed by the reference's own `GuardianModel.setupAnim`: the
/// head's look, the spikes drawn out as they breathe (and withdrawn), the
/// tail swaying with its animation, and the eye following what it looks at
/// (above or below it, to either side, straight on, or at its own place),
/// for a guardian and an elder (whose scaled layer the spikes' places
/// ignore).
pub fn guardian() -> Vec<Value> {
    let defaults = fields!(y_rot = 0.0, x_rot = 0.0, age_in_ticks = 37.5, spikes = 1.0, tail = 0.0);
    let poses: [(&str, Fields); 7] = [
        ("rests", fields!()),
        ("later", fields!(age_in_ticks = 91.25)),
        ("withdraws-its-spikes", fields!(spikes = 0.0)),
        ("half-withdrawn", fields!(age_in_ticks = 12.0, spikes = 0.4)),
        ("looks", fields!(y_rot = 30.0, x_rot = -15.0)),
        ("tail-sways", fields!(tail = 1.3)),
        ("tail-sways-back", fields!(tail = 7.7)),
    ];
    let eyes = [
        ("eye-above", looking([3.0, 4.0, 5.0], AHEAD)),
        ("eye-below", looking([-2.0, -1.5, 4.0], AHEAD)),
        ("eye-level", looking([1.0, 1.0, 6.0], AHEAD)),
        ("eye-to-the-left", looking([6.0, 2.0, 1.0], [1.0, 0.0, 0.5])),
        ("eye-straight-on", looking([0.0, 2.0, 5.0], AHEAD)),
        ("eye-at-its-own-place", looking([0.0, 3.0, 0.0], AHEAD)),
    ];
    let mut rows = Vec::new();
    for layer in ["minecraft:guardian#main", "minecraft:elder_guardian#main"] {
        rows.extend(poses.iter().map(|(label, set)| named_row(layer, label, defaults, set)));
        for (label, eye) in &eyes {
            let mut row = named_row(layer, label, defaults, fields!());
            row["eye"] = eye.clone();
            rows.push(row);
        }
    }
    vec![entity("guardian-model", "guardian_model", rows)]
}

/// A beam from `at` along `vector` at `time`, with the attack charged to
/// `scale`.
fn beam(label: &str, along: [f64; 3], (time, scale): (f64, f64), at: [f64; 3]) -> Value {
    json!({
        "label": label, "x": f32_bits(at[0]), "y": f32_bits(at[1]), "z": f32_bits(at[2]),
        "vector": vector(along), "time": f32_bits(time), "scale": f32_bits(scale),
        "tex_v_off": f32_bits(time * 0.5 % 1.0),
    })
}

/// A guardian's beam as the reference's own `renderBeam` builds it from the
/// pose its eye leaves: along each axis and slantwise, short and long (the
/// texture repeating along it), across the attack charging (the colour, the
/// spin and the cap's texture swapping on even and odd ticks) and from an
/// offset eye.
pub fn guardian_beam() -> Vec<Value> {
    let usual = (12.5, 0.3);
    let origin = [0.0; 3];
    let rows = vec![
        beam("along-z", [0.0, 0.0, 6.0], usual, origin),
        beam("along-x", [5.0, 0.0, 0.0], usual, origin),
        beam("straight-up", [0.0, 4.0, 0.0], usual, origin),
        beam("straight-down", [0.0, -4.0, 0.0], usual, origin),
        beam("slantwise", [3.0, -1.5, 4.5], usual, origin),
        beam("behind", [-2.0, 0.5, -7.0], usual, origin),
        beam("short", [0.2, 0.1, 0.3], usual, origin),
        beam("on-the-eye", [0.0, 0.0, 0.0], usual, origin),
        beam("just-started", [0.0, 0.0, 6.0], (0.5, 0.006), origin),
        beam("odd-tick", [1.0, 2.0, 3.0], (7.25, 0.09), origin),
        beam("charged", [1.0, 2.0, 3.0], (80.0, 1.0), origin),
        beam("elder-charged", [1.0, 2.0, 3.0], (60.0, 1.0), origin),
        beam("from-the-eye", [2.0, -1.0, 3.0], usual, [10.5, 64.9, -3.25]),
        beam("far", [20.0, 6.0, -30.0], (33.75, 0.17), origin),
    ];
    vec![entity("guardian-beam", "guardian_beam", rows)]
}
