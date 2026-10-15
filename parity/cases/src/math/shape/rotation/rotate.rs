//! Cases for `shape/rotate`, `shape/rotate-center`, `shape/rotate-all-center`, `shape/rotate-horizontal-axis` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn center_bits(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Value {
    json!({"x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

fn identity() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate/identity",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "IDENTITY"}),
    ));
    out.push(case(
        "shape/rotate/x-90",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_90_X_NEG"}),
    ));
    out.push(case(
        "shape/rotate/y-90",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_90_Y_NEG"}),
    ));
    out.push(case(
        "shape/rotate/z-90",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_90_Z_NEG"}),
    ));
    out.push(case(
        "shape/rotate/x-180",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_180_FACE_YZ"}),
    ));
    out.push(case(
        "shape/rotate/y-180",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_180_FACE_XZ"}),
    ));
    out.push(case(
        "shape/rotate/z-180",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_180_FACE_XY"}),
    ));
    out
}

fn x_90_positive() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate/x-90-positive",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_90_X_POS"}),
    ));
    out.push(case(
        "shape/rotate/y-90-positive",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_90_Y_POS"}),
    ));
    out.push(case(
        "shape/rotate/z-90-positive",
        "math",
        "shape",
        json!({"kind": "rotate", "box": rotate_box.clone(), "rotation": "ROT_90_Z_POS"}),
    ));
    out.push(case(
        "shape/rotate/cube-grid",
        "math",
        "shape",
        json!({"kind": "rotate", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0), "rotation": "ROT_90_Y_NEG"}),
    ));
    out.push(case(
        "shape/rotate/joined-cells",
        "math",
        "shape",
        json!({
            "kind": "rotate_join",
            "first": box_bits(0.0, 0.0, 0.0, 0.5, 0.5, 1.0),
            "second": box_bits(0.5, 0.5, 0.0, 1.0, 1.0, 1.0),
            "rotation": "ROT_90_Z_NEG",
        }),
    ));
    out
}

fn y_asymmetric() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate-center/y-asymmetric",
        "math",
        "shape",
        json!({
            "kind": "rotate_center",
            "box": rotate_box.clone(),
            "rotation": "ROT_90_Y_NEG",
            "center": center_bits(0.25, 0.5, 0.75),
        }),
    ));
    out.push(case(
        "shape/rotate-center/x-cube",
        "math",
        "shape",
        json!({
            "kind": "rotate_center",
            "box": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "rotation": "ROT_90_X_NEG",
            "center": center_bits(0.25, 0.5, 0.75),
        }),
    ));
    out.push(case(
        "shape/rotate-center/z-grid",
        "math",
        "shape",
        json!({
            "kind": "rotate_center",
            "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "rotation": "ROT_90_Z_POS",
            "center": center_bits(0.75, 0.25, 0.5),
        }),
    ));
    out.push(case(
        "shape/rotate-center/identity",
        "math",
        "shape",
        json!({
            "kind": "rotate_center",
            "box": rotate_box.clone(),
            "rotation": "IDENTITY",
            "center": center_bits(2.0, -1.0, 3.0),
        }),
    ));
    out
}

fn asymmetric() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate-all-center/asymmetric",
        "math",
        "shape",
        json!({"kind": "rotate_all_center", "box": rotate_box.clone(), "center": center_bits(0.25, 0.5, 0.75)}),
    ));
    out.push(case(
        "shape/rotate-all-center/grid",
        "math",
        "shape",
        json!({
            "kind": "rotate_all_center",
            "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "center": center_bits(0.75, 0.25, 0.5),
        }),
    ));
    out.push(case(
        "shape/rotate-all-center/empty",
        "math",
        "shape",
        json!({
            "kind": "rotate_all_center",
            "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "center": center_bits(2.0, -1.0, 3.0),
        }),
    ));
    out.push(case(
        "shape/rotate-horizontal-axis/asymmetric",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal_axis", "box": rotate_box.clone()}),
    ));
    out.push(case(
        "shape/rotate-horizontal-axis/cube-grid",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal_axis", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}),
    ));
    out.push(case(
        "shape/rotate-horizontal-axis/empty",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal_axis", "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)}),
    ));
    out
}

fn rotate_horizontal_axis_center_asymmetric() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate-horizontal-axis-center/asymmetric",
        "math",
        "shape",
        json!({
            "kind": "rotate_horizontal_axis_center",
            "box": rotate_box.clone(),
            "center": center_bits(0.25, 0.5, 0.75),
        }),
    ));
    out.push(case(
        "shape/rotate-horizontal-axis-center/grid",
        "math",
        "shape",
        json!({
            "kind": "rotate_horizontal_axis_center",
            "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "center": center_bits(0.75, 0.25, 0.5),
        }),
    ));
    out.push(case(
        "shape/rotate-horizontal-axis-center/empty",
        "math",
        "shape",
        json!({
            "kind": "rotate_horizontal_axis_center",
            "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "center": center_bits(2.0, -1.0, 3.0),
        }),
    ));
    out.push(case(
        "shape/rotate-all-axis/asymmetric",
        "math",
        "shape",
        json!({"kind": "rotate_all_axis", "box": rotate_box.clone()}),
    ));
    out.push(case(
        "shape/rotate-all-axis/cube-grid",
        "math",
        "shape",
        json!({"kind": "rotate_all_axis", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}),
    ));
    out
}

fn empty() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate-all-axis/empty",
        "math",
        "shape",
        json!({"kind": "rotate_all_axis", "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)}),
    ));
    out.push(case(
        "shape/rotate-all-axis-center/asymmetric",
        "math",
        "shape",
        json!({"kind": "rotate_all_axis_center", "box": rotate_box.clone(), "center": center_bits(0.25, 0.5, 0.75)}),
    ));
    out.push(case(
        "shape/rotate-all-axis-center/grid",
        "math",
        "shape",
        json!({
            "kind": "rotate_all_axis_center",
            "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "center": center_bits(0.75, 0.25, 0.5),
        }),
    ));
    out.push(case(
        "shape/rotate-all-axis-center/empty",
        "math",
        "shape",
        json!({
            "kind": "rotate_all_axis_center",
            "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "center": center_bits(2.0, -1.0, 3.0),
        }),
    ));
    out.push(case(
        "shape/rotate-horizontal/asymmetric",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal", "box": rotate_box.clone()}),
    ));
    out.push(case(
        "shape/rotate-horizontal/cube-grid",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}),
    ));
    out
}

fn rotate_horizontal_empty() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate-horizontal/empty",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal", "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)}),
    ));
    out.push(case(
        "shape/rotate-horizontal-initial/identity",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal_initial", "box": rotate_box.clone(), "initial": "IDENTITY"}),
    ));
    out.push(case(
        "shape/rotate-horizontal-initial/x-90",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal_initial", "box": rotate_box.clone(), "initial": "ROT_90_X_NEG"}),
    ));
    out.push(case(
        "shape/rotate-horizontal-initial/y-180",
        "math",
        "shape",
        json!({
            "kind": "rotate_horizontal_initial",
            "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "initial": "ROT_180_FACE_XZ",
        }),
    ));
    out.push(case(
        "shape/rotate-horizontal-initial/y-90",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal_initial", "box": rotate_box.clone(), "initial": "ROT_90_Y_NEG"}),
    ));
    out.push(case(
        "shape/rotate-horizontal-initial/z-90",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal_initial", "box": rotate_box.clone(), "initial": "ROT_90_Z_NEG"}),
    ));
    out
}

fn rotate_horizontal_initial_xy_kind() -> Vec<Value> {
    let mut out = Vec::new();
    let rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/rotate-horizontal-initial/xy-180",
        "math",
        "shape",
        json!({"kind": "rotate_horizontal_initial", "box": rotate_box.clone(), "initial": "ROT_180_FACE_XY"}),
    ));
    out.push(case(
        "shape/rotate-all-initial/identity",
        "math",
        "shape",
        json!({
            "kind": "rotate_all_initial",
            "box": rotate_box.clone(),
            "initial": "IDENTITY",
            "center": center_bits(0.25, 0.5, 0.75),
        }),
    ));
    out.push(case(
        "shape/rotate-all-initial/x-90",
        "math",
        "shape",
        json!({
            "kind": "rotate_all_initial",
            "box": rotate_box.clone(),
            "initial": "ROT_90_X_NEG",
            "center": center_bits(0.25, 0.5, 0.75),
        }),
    ));
    out.push(case(
        "shape/rotate-all-initial/y-180",
        "math",
        "shape",
        json!({
            "kind": "rotate_all_initial",
            "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "initial": "ROT_180_FACE_XZ",
            "center": center_bits(0.75, 0.25, 0.5),
        }),
    ));
    out.push(case(
        "shape/rotate-all/asymmetric",
        "math",
        "shape",
        json!({"kind": "rotate_all", "box": rotate_box.clone()}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        identity(),
        x_90_positive(),
        y_asymmetric(),
        asymmetric(),
        rotate_horizontal_axis_center_asymmetric(),
        empty(),
        rotate_horizontal_empty(),
        rotate_horizontal_initial_xy_kind(),
    ]
    .concat()
}
