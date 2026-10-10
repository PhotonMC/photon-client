//! Element faces, element and Euler rotations, face UV rotations, quad
//! winding and the nearest direction of a vector.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::line::float_text;

pub const FACINGS: &[&str] = &["down", "up", "north", "south", "west", "east"];

fn bits(values: &[f64]) -> Vec<String> {
    values.iter().copied().map(f32_bits).collect()
}

fn plain(rescale: bool) -> &'static str {
    if rescale { "rescale" } else { "plain" }
}

/// Label, `from` and `to` of each box.
const BOXES: &[(&str, [f64; 3], [f64; 3])] = &[
    ("full", [0.0, 0.0, 0.0], [16.0, 16.0, 16.0]),
    ("slab", [0.0, 0.0, 0.0], [16.0, 8.0, 16.0]),
    ("offset", [2.0, 3.0, 4.0], [14.0, 13.0, 12.0]),
    ("thin", [7.5, 0.0, 7.5], [8.5, 16.0, 8.5]),
    ("degenerate", [4.0, 4.0, 4.0], [4.0, 12.0, 12.0]),
    // The element grammar allows reaching outside the block.
    ("oversized", [-16.0, -16.0, -16.0], [32.0, 32.0, 32.0]),
    ("fractional", [0.5, 1.25, 2.125], [15.5, 14.75, 13.875]),
    // Inverted bounds are not rejected by the corner layout itself.
    ("inverted", [12.0, 12.0, 12.0], [4.0, 4.0, 4.0]),
];

pub fn face_geometry() -> Vec<Value> {
    let mut rows = Vec::new();
    for (name, from, to) in BOXES {
        for facing in FACINGS {
            rows.push(json!({
                "label": format!("{name}-{facing}"), "from": bits(from), "to": bits(to), "facing": facing,
            }));
        }
    }
    rows
}

/// The angles real models use, plus values the grammar allows but no
/// vanilla model writes.
const ANGLES: &[f64] = &[0.0, 22.5, -22.5, 45.0, -45.0, 90.0, 180.0, 270.0, 360.0, 30.0, -7.5];

pub fn element_rotation() -> Vec<Value> {
    let mut rows = Vec::new();
    for axis in ["x", "y", "z"] {
        for angle in ANGLES {
            for rescale in [false, true] {
                rows.push(json!({
                    "label": format!("{axis}-{}-{}", float_text(*angle), plain(rescale)),
                    "from": bits(&[0.0, 0.0, 0.0]), "to": bits(&[16.0, 16.0, 16.0]), "facing": "up",
                    "origin": bits(&[8.0, 8.0, 8.0]), "axis": axis, "angle": f32_bits(*angle),
                    "rescale": rescale,
                }));
            }
        }
    }
    // A non-centred origin and a partial box, which is what stairs and
    // torches actually look like.
    for facing in ["down", "north", "east"] {
        rows.push(json!({
            "label": format!("offset-origin-{facing}"),
            "from": bits(&[4.0, 0.0, 4.0]), "to": bits(&[12.0, 10.0, 12.0]), "facing": facing,
            "origin": bits(&[0.0, 8.0, 16.0]), "axis": "y", "angle": f32_bits(22.5), "rescale": true,
        }));
    }
    rows
}

type Quad = [[f64; 3]; 4];

const WINDINGS: &[(&str, Quad)] = &[
    // A full-block face in the reference's own corner order for each side.
    ("up-ordered", [[0.0, 1.0, 0.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]]),
    ("down-ordered", [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0]]),
    ("north-ordered", [[1.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
    ("south-ordered", [[0.0, 1.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0]]),
    ("west-ordered", [[0.0, 1.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0]]),
    ("east-ordered", [[1.0, 1.0, 1.0], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]),
    // The same face wound backwards: the normal flips and so does the
    // direction the bakery settles on.
    ("up-reversed", [[1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]]),
    // Shuffled corners of a real face, which is what a rotation leaves
    // behind and what the rewind exists to repair.
    ("up-shuffled", [[1.0, 1.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 1.0]]),
    // A partial box rather than the whole block.
    ("slab-top", [[0.0, 0.5, 0.0], [0.0, 0.5, 1.0], [1.0, 0.5, 1.0], [1.0, 0.5, 0.0]]),
    // Degenerate input: all four corners equal, so the normal is not
    // finite.
    ("degenerate", [[0.5, 0.5, 0.5]; 4]),
    // Collinear corners, another way to lose the normal.
    ("collinear", [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]]),
    // A diagonal quad, whose nearest direction is not exact.
    ("diagonal", [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]]),
];

pub fn quad_winding() -> Vec<Value> {
    WINDINGS
        .iter()
        .map(|(label, quad)| {
            let corners: Vec<Vec<String>> = quad.iter().map(|corner| bits(corner)).collect();
            json!({"label": label, "corners": corners})
        })
        .collect()
}

const TRIPLES: &[[f64; 3]] = &[
    [0.0, 0.0, 0.0],
    [22.5, 0.0, 0.0],
    [0.0, 22.5, 0.0],
    [0.0, 0.0, 22.5],
    [90.0, 0.0, 0.0],
    [0.0, 90.0, 0.0],
    [0.0, 0.0, 90.0],
    [22.5, 45.0, 0.0],
    [0.0, 22.5, 45.0],
    [45.0, 0.0, 22.5],
    [22.5, 22.5, 22.5],
    [45.0, 45.0, 45.0],
    [90.0, 90.0, 90.0],
    [-22.5, 30.0, -7.5],
    [180.0, 90.0, -45.0],
    [360.0, 360.0, 360.0],
];

pub fn euler_rotation() -> Vec<Value> {
    let mut rows = Vec::new();
    for [x, y, z] in TRIPLES {
        for rescale in [false, true] {
            let label = format!("{}-{}-{}-{}", float_text(*x), float_text(*y), float_text(*z), plain(rescale));
            rows.push(json!({
                "label": label, "x": f32_bits(*x), "y": f32_bits(*y), "z": f32_bits(*z), "rescale": rescale,
            }));
        }
    }
    rows
}

const UV_SETS: &[(&str, [f64; 4])] = &[
    ("full", [0.0, 0.0, 16.0, 16.0]),
    ("partial", [2.0, 4.0, 14.0, 12.0]),
    // A flipped rectangle, which models use to mirror a texture.
    ("flipped-u", [16.0, 0.0, 0.0, 16.0]),
    ("flipped-both", [16.0, 16.0, 0.0, 0.0]),
    ("fractional", [0.5, 1.25, 15.5, 14.75]),
];

/// Accepted rotations, values that wrap onto them, and values that do not.
const DEGREES: &[i32] = &[0, 90, 180, 270, 360, 450, -90, -180, -270, -360, 720, 45, 30, 1, -45, 100];

pub fn face_uv_rotation() -> Vec<Value> {
    let mut rows = Vec::new();
    for (name, uvs) in UV_SETS {
        for degrees in DEGREES {
            rows.push(json!({"label": format!("{name}-{degrees}"), "uvs": bits(uvs), "degrees": degrees}));
        }
    }
    rows
}

/// Which face a vector points at. The search starts at north with the
/// smallest positive float as the bar, so a vector with no positive
/// component at all comes back north rather than as nothing.
const NEAREST: &[(&str, f64, f64, f64)] = &[
    ("down", 0.0, -1.0, 0.0),
    ("up", 0.0, 1.0, 0.0),
    ("north", 0.0, 0.0, -1.0),
    ("south", 0.0, 0.0, 1.0),
    ("west", -1.0, 0.0, 0.0),
    ("east", 1.0, 0.0, 0.0),
    ("zero", 0.0, 0.0, 0.0),
    ("all-negative", -1.0, -1.0, -1.0),
    ("diagonal", 1.0, 1.0, 0.0),
    ("tie-up-east", 1.0, 1.0, 1.0),
    ("tiny-positive", 1e-44, 0.0, 0.0),
    ("below-the-bar", 1e-46, 0.0, 0.0),
    ("negative-zero", -0.0, -0.0, -0.0),
    ("nan", f64::NAN, 0.0, 0.0),
    ("infinity", f64::INFINITY, 0.0, 0.0),
    ("negative-infinity", f64::NEG_INFINITY, 0.0, 0.0),
    ("mixed", 0.5, -0.9, 0.2),
];

pub fn nearest_direction() -> Vec<Value> {
    NEAREST
        .iter()
        .map(|(label, x, y, z)| json!({"label": label, "x": f32_bits(*x), "y": f32_bits(*y), "z": f32_bits(*z)}))
        .collect()
}
