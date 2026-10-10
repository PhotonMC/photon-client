//! Variant rotations: the model state, UV lock and cull face rotation.

use serde_json::{Value, json};

use super::geometry::FACINGS;
use crate::bits::f32_bits;
use crate::line::float_text;

const QUADRANTS: [i32; 4] = [0, 90, 180, 270];

fn state(label: String, (x, y, z): (i32, i32, i32), uvlock: bool) -> Value {
    json!({"label": label, "x": x, "y": y, "z": z, "uvlock": uvlock})
}

pub fn model_state() -> Vec<Value> {
    let mut rows = Vec::new();
    // UV lock off: the face transformations stay identity.
    let wrapped = [0, 90, 180, 270, 360, -90, -180, 450];
    for x in wrapped {
        for y in wrapped {
            rows.push(state(format!("x{x}-y{y}"), (x, y, 0), false));
        }
    }
    // With UV lock on, the face transformations come from a table the
    // reference builds once from quaternions. Every quadrant combination is
    // driven, because the table is keyed by the composed group and several
    // combinations land on the same element.
    for x in QUADRANTS {
        for y in QUADRANTS {
            for z in QUADRANTS {
                rows.push(state(format!("uvlock-x{x}-y{y}-z{z}"), (x, y, z), true));
            }
        }
    }
    // The third axis on its own, and combined with the other two, because
    // the group composition is not commutative.
    for z in [90, 180, 270] {
        rows.push(state(format!("z{z}"), (0, 0, z), false));
    }
    for x in QUADRANTS {
        for y in QUADRANTS {
            for z in [90, 180, 270] {
                rows.push(state(format!("x{x}-y{y}-z{z}"), (x, y, z), false));
            }
        }
    }
    rows
}

fn locked(label: String, (x, y, z): (i32, i32, i32), uvlock: bool, facing: &str, u: f64) -> Value {
    json!({
        "label": label, "x": x, "y": y, "z": z, "uvlock": uvlock, "facing": facing,
        "u": f32_bits(u), "v": f32_bits(0.375),
    })
}

/// What a UV-locked face does to a texture coordinate. Every rotation and
/// every face is driven, because the tables differ per pair, and the sample
/// coordinates include the corners, the centre and points off the face.
pub fn uv_lock() -> Vec<Value> {
    let samples = [0.0, 1.0, 0.5, 0.25, 0.125, -0.5, 1.5];
    let mut rows = Vec::new();
    for x in QUADRANTS {
        for y in QUADRANTS {
            for facing in FACINGS {
                for u in samples {
                    let label = format!("x{x}-y{y}-{facing}-u{}", float_text(u));
                    rows.push(locked(label, (x, y, 0), true, facing, u));
                }
            }
        }
    }
    // The third axis, and the unlocked case where the coordinate passes
    // through untouched.
    for z in [90, 180, 270] {
        for facing in FACINGS {
            rows.push(locked(
                format!("z{z}-{facing}"),
                (0, 0, z),
                true,
                facing,
                0.25,
            ));
        }
    }
    for facing in FACINGS {
        rows.push(locked(
            format!("unlocked-{facing}"),
            (90, 180, 0),
            false,
            facing,
            0.25,
        ));
    }
    rows
}

/// Where a face's cullface direction lands once the variant is turned.
pub fn cull_rotation() -> Vec<Value> {
    let mut rows = Vec::new();
    for x in QUADRANTS {
        for y in QUADRANTS {
            for z in QUADRANTS {
                for facing in FACINGS {
                    rows.push(json!({
                        "label": format!("x{x}-y{y}-z{z}-{facing}"), "x": x, "y": y, "z": z, "facing": facing,
                    }));
                }
            }
        }
    }
    rows
}
