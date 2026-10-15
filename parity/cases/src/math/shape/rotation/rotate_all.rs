//! Cases for `shape/rotate-all`, `shape/rotate-attach-face`, `shape/rotate-attach-face-initial`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

fn cube_grid() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate-all/cube-grid",
        "math",
        "shape",
        json!({"kind": "rotate_all", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}),
    ));
    out.push(case(
        "shape/rotate-all/empty",
        "math",
        "shape",
        json!({"kind": "rotate_all", "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)}),
    ));
    out.push(case(
        "shape/rotate-attach-face/asymmetric",
        "math",
        "shape",
        json!({"kind": "rotate_attach_face", "box": rotate_box.clone()}),
    ));
    out.push(case(
        "shape/rotate-attach-face/cube-grid",
        "math",
        "shape",
        json!({"kind": "rotate_attach_face", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}),
    ));
    out.push(case(
        "shape/rotate-attach-face/empty",
        "math",
        "shape",
        json!({"kind": "rotate_attach_face", "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)}),
    ));
    out.push(case(
        "shape/rotate-attach-face-initial/identity",
        "math",
        "shape",
        json!({"kind": "rotate_attach_face_initial", "box": rotate_box.clone(), "initial": "IDENTITY"}),
    ));
    out.push(case(
        "shape/rotate-attach-face-initial/x-90",
        "math",
        "shape",
        json!({"kind": "rotate_attach_face_initial", "box": rotate_box.clone(), "initial": "ROT_90_X_NEG"}),
    ));
    out
}

fn rotate_attach_face_initial_y_kind() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/rotate-attach-face-initial/y-180",
        "math",
        "shape",
        json!({
            "kind": "rotate_attach_face_initial",
            "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "initial": "ROT_180_FACE_XZ",
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [cube_grid(), rotate_attach_face_initial_y_kind()].concat()
}
