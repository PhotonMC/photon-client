//! Cases for `shape/single-encompassing`, `shape/axis-limits`, `shape/axis-limits-at`, `shape/equal` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn touching() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/single-encompassing/touching",
        "math",
        "shape",
        json!({
            "kind": "single_encompassing",
            "shapes": [box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0), box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0)],
        }),
    ));
    out.push(case(
        "shape/single-encompassing/disjoint",
        "math",
        "shape",
        json!({
            "kind": "single_encompassing",
            "shapes": [box_bits(0.0, 0.0, 0.0, 0.25, 0.5, 0.75), box_bits(0.75, 0.25, 0.5, 1.0, 1.0, 1.0)],
        }),
    ));
    out.push(case(
        "shape/axis-limits/empty",
        "math",
        "shape",
        json!({"kind": "axis_limits", "shapes": [box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)]}),
    ));
    out.push(case(
        "shape/axis-limits/disjoint",
        "math",
        "shape",
        json!({
            "kind": "axis_limits",
            "shapes": [box_bits(0.25, 0.0, 0.5, 0.5, 0.25, 0.75), box_bits(0.75, 0.5, -0.5, 1.0, 1.0, 0.0)],
        }),
    ));
    out.push(case(
        "shape/axis-limits/negative",
        "math",
        "shape",
        json!({
            "kind": "axis_limits",
            "shapes": [box_bits(-2.0, -0.75, 1.5, -1.25, -0.25, 2.0), box_bits(-0.5, 0.5, -2.0, 0.25, 1.25, -1.5)],
        }),
    ));
    out
}

fn empty() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/axis-limits-at/empty",
        "math",
        "shape",
        json!({
            "kind": "axis_limits_at",
            "shapes": [box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)],
            "queries": [
                {"op": "min", "axis": "x", "b": f64_bits(0.5), "c": f64_bits(0.5)},
                {"op": "max", "axis": "z", "b": f64_bits(0.5), "c": f64_bits(0.5)},
            ],
        }),
    ));
    out.push(case(
        "shape/axis-limits-at/disjoint",
        "math",
        "shape",
        json!({
            "kind": "axis_limits_at",
            "shapes": [box_bits(0.25, 0.0, 0.5, 0.5, 0.25, 0.75), box_bits(0.75, 0.5, -0.5, 1.0, 1.0, 0.0)],
            "queries": [
                {"op": "min", "axis": "x", "b": f64_bits(0.125), "c": f64_bits(0.625)},
                {"op": "max", "axis": "x", "b": f64_bits(0.125), "c": f64_bits(0.625)},
                {"op": "min", "axis": "y", "b": f64_bits(-0.25), "c": f64_bits(0.875)},
                {"op": "max", "axis": "y", "b": f64_bits(-0.25), "c": f64_bits(0.875)},
                {"op": "min", "axis": "z", "b": f64_bits(0.375), "c": f64_bits(0.125)},
                {"op": "max", "axis": "z", "b": f64_bits(0.375), "c": f64_bits(0.125)},
                {"op": "min", "axis": "x", "b": f64_bits(0.75), "c": f64_bits(0.25)},
            ],
        }),
    ));
    out
}

fn negative() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/axis-limits-at/negative",
        "math",
        "shape",
        json!({
            "kind": "axis_limits_at",
            "shapes": [box_bits(-2.0, -0.75, 1.5, -1.25, -0.25, 2.0), box_bits(-0.5, 0.5, -2.0, 0.25, 1.25, -1.5)],
            "queries": [
                {"op": "min", "axis": "x", "b": f64_bits(-0.5), "c": f64_bits(1.75)},
                {"op": "max", "axis": "x", "b": f64_bits(-0.5), "c": f64_bits(1.75)},
                {"op": "min", "axis": "y", "b": f64_bits(1.75), "c": f64_bits(-1.5)},
                {"op": "max", "axis": "y", "b": f64_bits(1.75), "c": f64_bits(-1.5)},
                {"op": "min", "axis": "z", "b": f64_bits(-1.5), "c": f64_bits(-0.5)},
                {"op": "max", "axis": "z", "b": f64_bits(-1.5), "c": f64_bits(-0.5)},
            ],
        }),
    ));
    out.push(case(
        "shape/equal/same-box",
        "math",
        "shape",
        json!({
            "kind": "equal",
            "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "second": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/equal/same-translated-box",
        "math",
        "shape",
        json!({
            "kind": "equal",
            "first": box_bits(-2.0, 1.0, 3.0, -1.0, 2.0, 4.0),
            "second": box_bits(-2.0, 1.0, 3.0, -1.0, 2.0, 4.0),
        }),
    ));
    out
}

fn different_size() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/equal/different-size",
        "math",
        "shape",
        json!({
            "kind": "equal",
            "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "second": box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/equal/disjoint",
        "math",
        "shape",
        json!({
            "kind": "equal",
            "first": box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0),
            "second": box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/equal/partial-overlap",
        "math",
        "shape",
        json!({
            "kind": "equal",
            "first": box_bits(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
            "second": box_bits(0.25, 0.0, 0.0, 1.0, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/equal/both-empty",
        "math",
        "shape",
        json!({
            "kind": "equal",
            "first": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "second": box_bits(-2.0, -1.0, 3.0, -1.99999995, 0.0, 4.0),
        }),
    ));
    out
}

fn cube() -> Vec<Value> {
    let mut out = Vec::new();
    let object_equals_cube = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875);
    out.push(case(
        "shape/object-equals/cube",
        "math",
        "shape",
        json!({
            "kind": "object_equals",
            "shape": object_equals_cube.clone(),
            "same": object_equals_cube.clone(),
            "different": box_bits(0.125, 0.25, 0.375, 0.75, 0.75, 0.875),
        }),
    ));
    let object_equals_array = box_bits(-2.0, -0.75, 1.5, -1.25, -0.25, 2.0);
    out.push(case(
        "shape/object-equals/array",
        "math",
        "shape",
        json!({
            "kind": "object_equals",
            "shape": object_equals_array.clone(),
            "same": object_equals_array.clone(),
            "different": box_bits(-2.0, -0.75, 1.5, -1.25, -0.25, 2.25),
        }),
    ));
    out.push(case("shape/object-equals/constants", "math", "shape", json!({"kind": "object_equals_constants"})));
    out.push(case(
        "shape/to-string/empty",
        "math",
        "shape",
        json!({"kind": "to_string", "shape": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)}),
    ));
    out.push(case(
        "shape/to-string/cube",
        "math",
        "shape",
        json!({"kind": "to_string", "shape": box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875)}),
    ));
    out.push(case(
        "shape/to-string/scientific",
        "math",
        "shape",
        json!({"kind": "to_string", "shape": box_bits(0.0001, 10000000.0, -0.0, 0.0002, 10000001.0, 2.0)}),
    ));
    out
}

fn infinity() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/to-string/infinity",
        "math",
        "shape",
        json!({
            "kind": "to_string",
            "shape": box_bits(
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::INFINITY,
                f64::INFINITY,
            ),
        }),
    ));
    out.push(case(
        "shape/to-string/nan",
        "math",
        "shape",
        json!({"kind": "to_string", "shape": box_bits(f64::NAN, 0.0, 0.0, 1.0, 1.0, 1.0)}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [touching(), empty(), negative(), different_size(), cube(), infinity()].concat()
}
