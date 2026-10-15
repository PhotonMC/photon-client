//! Cases for `vec3/rotate-clockwise-90`, `vec3/get`, `vec3/with`, `vec3/relative` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

fn basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/rotate-clockwise-90/basic",
        "math",
        "vec3_rotate_clockwise_90",
        json!({"first": vector_bits(1.25, -2.5, 3.75)}),
    ));
    out.push(case(
        "vec3/rotate-clockwise-90/negative-components",
        "math",
        "vec3_rotate_clockwise_90",
        json!({"first": vector_bits(-4.5, 6.25, -8.75)}),
    ));
    out.push(case(
        "vec3/rotate-clockwise-90/signed-zero",
        "math",
        "vec3_rotate_clockwise_90",
        json!({"first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"}}),
    ));
    out.push(case(
        "vec3/rotate-clockwise-90/subnormal-and-large",
        "math",
        "vec3_rotate_clockwise_90",
        json!({"first": {"x": "0000000000000001", "y": "8000000000000001", "z": "7fefffffffffffff"}}),
    ));
    out.push(case(
        "vec3/rotate-clockwise-90/special-values",
        "math",
        "vec3_rotate_clockwise_90",
        json!({"first": {"x": f64_bits(f64::INFINITY), "y": f64_bits(f64::NAN), "z": f64_bits(f64::NEG_INFINITY)}}),
    ));
    out.push(case("vec3/get/x", "math", "vec3_get", json!({"first": vector_bits(1.25, -2.5, 3.75), "axis": "x"})));
    out.push(case("vec3/get/y", "math", "vec3_get", json!({"first": vector_bits(-4.5, 6.25, -8.75), "axis": "y"})));
    out.push(case(
        "vec3/get/z-signed-zero",
        "math",
        "vec3_get",
        json!({"first": {"x": "0000000000000000", "y": "8000000000000000", "z": "8000000000000000"}, "axis": "z"}),
    ));
    out.push(case(
        "vec3/get/subnormal-and-large",
        "math",
        "vec3_get",
        json!({"first": {"x": "0000000000000001", "y": "8000000000000001", "z": "7fefffffffffffff"}, "axis": "z"}),
    ));
    out
}

fn special_values() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/get/special-values",
        "math",
        "vec3_get",
        json!({
            "first": {"x": f64_bits(f64::INFINITY), "y": "7ff8000000000001", "z": f64_bits(f64::NEG_INFINITY)},
            "axis": "y",
        }),
    ));
    out.push(case(
        "vec3/with/x",
        "math",
        "vec3_with",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "axis": "x", "value": f64_bits(9.5)}),
    ));
    out.push(case(
        "vec3/with/y-signed-zero",
        "math",
        "vec3_with",
        json!({
            "first": {"x": "8000000000000000", "y": "0000000000000001", "z": "7fefffffffffffff"},
            "axis": "y",
            "value": "8000000000000000",
        }),
    ));
    out.push(case(
        "vec3/with/z-subnormal",
        "math",
        "vec3_with",
        json!({"first": vector_bits(-4.5, 6.25, -8.75), "axis": "z", "value": "0000000000000001"}),
    ));
    out.push(case(
        "vec3/with/x-special-value",
        "math",
        "vec3_with",
        json!({
            "first": {"x": f64_bits(f64::INFINITY), "y": "7ff8000000000001", "z": f64_bits(f64::NEG_INFINITY)},
            "axis": "x",
            "value": "7ff8000000000001",
        }),
    ));
    out
}

fn y_max_finite() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/with/y-max-finite",
        "math",
        "vec3_with",
        json!({
            "first": {"x": "0000000000000001", "y": "8000000000000001", "z": "8000000000000000"},
            "axis": "y",
            "value": "7fefffffffffffff",
        }),
    ));
    out.push(case(
        "vec3/relative/east-basic",
        "math",
        "vec3_relative",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "direction": "east", "distance": f64_bits(9.5)}),
    ));
    out.push(case(
        "vec3/relative/west-signed-zero",
        "math",
        "vec3_relative",
        json!({
            "first": {"x": "8000000000000000", "y": "0000000000000001", "z": "8000000000000000"},
            "direction": "west",
            "distance": "0000000000000000",
        }),
    ));
    out.push(case(
        "vec3/relative/up-subnormal",
        "math",
        "vec3_relative",
        json!({
            "first": {"x": "8000000000000000", "y": "8000000000000001", "z": "7fefffffffffffff"},
            "direction": "up",
            "distance": "0000000000000001",
        }),
    ));
    out
}

fn down_overflow() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/relative/down-overflow",
        "math",
        "vec3_relative",
        json!({
            "first": {"x": "0000000000000001", "y": "fff0000000000000", "z": "8000000000000000"},
            "direction": "down",
            "distance": "7fefffffffffffff",
        }),
    ));
    out.push(case(
        "vec3/relative/north-cancellation",
        "math",
        "vec3_relative",
        json!({
            "first": {"x": "7fefffffffffffff", "y": "8000000000000001", "z": "7fefffffffffffff"},
            "direction": "north",
            "distance": "7fefffffffffffff",
        }),
    ));
    out.push(case(
        "vec3/relative/south-special",
        "math",
        "vec3_relative",
        json!({
            "first": {"x": "3ff4000000000000", "y": "c004000000000000", "z": "7ff0000000000000"},
            "direction": "south",
            "distance": "7ff8000000000001",
        }),
    ));
    out.push(case(
        "vec3/align/none-preserves",
        "math",
        "vec3_align",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "axes": []}),
    ));
    out.push(case(
        "vec3/align/all-floors",
        "math",
        "vec3_align",
        json!({"first": vector_bits(1.75, -2.25, 3.0), "axes": ["x", "y", "z"]}),
    ));
    out
}

fn xz_negative() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/align/xz-negative",
        "math",
        "vec3_align",
        json!({"first": vector_bits(-0.25, 1.75, -3.75), "axes": ["x", "z"]}),
    ));
    out.push(case(
        "vec3/align/y-signed-zero",
        "math",
        "vec3_align",
        json!({"first": vector_bits(-0.0, -0.0, 0.0), "axes": ["y"]}),
    ));
    out.push(case(
        "vec3/align/selected-special",
        "math",
        "vec3_align",
        json!({
            "first": {"x": "7ff0000000000000", "y": "7ff8000000000001", "z": "fff0000000000000"},
            "axes": ["x", "y", "z"],
        }),
    ));
    out.push(case(
        "vec3/align/reordered-duplicate",
        "math",
        "vec3_align",
        json!({
            "first": {"x": "7fefffffffffffff", "y": "0000000000000001", "z": "bff0000000000000"},
            "axes": ["z", "x", "z"],
        }),
    ));
    out.push(case(
        "vec3/projected-on/basic",
        "math",
        "vec3_projected_on",
        json!({"first": vector_bits(3.0, 4.0, 0.0), "onto": vector_bits(4.0, 0.0, 0.0)}),
    ));
    out.push(case(
        "vec3/projected-on/orthogonal",
        "math",
        "vec3_projected_on",
        json!({"first": vector_bits(0.0, 1.0, 0.0), "onto": vector_bits(1.0, 0.0, 0.0)}),
    ));
    out
}

fn zero_onto_preserves() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/projected-on/zero-onto-preserves",
        "math",
        "vec3_projected_on",
        json!({
            "first": vector_bits(1.25, -2.5, 3.75),
            "onto": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"},
        }),
    ));
    out.push(case(
        "vec3/projected-on/negative-fraction",
        "math",
        "vec3_projected_on",
        json!({"first": vector_bits(1.5, -2.25, 3.75), "onto": vector_bits(-2.0, 4.0, -1.0)}),
    ));
    out.push(case(
        "vec3/projected-on/special-values",
        "math",
        "vec3_projected_on",
        json!({
            "first": {"x": "7ff0000000000000", "y": "3ff0000000000000", "z": "fff0000000000000"},
            "onto": vector_bits(1.0, 0.0, 1.0),
        }),
    ));
    out.push(case(
        "vec3/projected-on/subnormal-onto",
        "math",
        "vec3_projected_on",
        json!({
            "first": {"x": "7fefffffffffffff", "y": "8000000000000001", "z": "0000000000000001"},
            "onto": {"x": "0000000000000001", "y": "0000000000000000", "z": "0000000000000000"},
        }),
    ));
    out.push(case(
        "vec3/to-vector3f/basic",
        "math",
        "vec3_to_vector3f",
        json!({"first": vector_bits(1.25, -2.5, 3.75)}),
    ));
    out.push(case(
        "vec3/to-vector3f/rounding",
        "math",
        "vec3_to_vector3f",
        json!({"first": vector_bits(1.00000006, -123456.789, 0.1)}),
    ));
    out
}

fn signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/to-vector3f/signed-zero",
        "math",
        "vec3_to_vector3f",
        json!({"first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000001"}}),
    ));
    out.push(case(
        "vec3/to-vector3f/subnormal",
        "math",
        "vec3_to_vector3f",
        json!({"first": {"x": "0000000000000001", "y": "8000000000000001", "z": f64_bits(1.401298464324817e-45)}}),
    ));
    out.push(case(
        "vec3/to-vector3f/overflow",
        "math",
        "vec3_to_vector3f",
        json!({"first": vector_bits(1e+300, -1e+300, 3.4028235e+38)}),
    ));
    out.push(case(
        "vec3/to-vector3f/special-values",
        "math",
        "vec3_to_vector3f",
        json!({"first": {"x": "7ff0000000000000", "y": "7ff8000000000001", "z": "fff0000000000000"}}),
    ));
    out.push(case(
        "vec3/coordinates/lower-corner",
        "math",
        "vec3_coordinates",
        json!({"kind": "lower_corner", "x": "2147483647", "y": "-2147483648", "z": "-17"}),
    ));
    out.push(case(
        "vec3/coordinates/lower-corner-offset",
        "math",
        "vec3_coordinates",
        json!({
            "kind": "lower_corner_offset",
            "x": "-4",
            "y": "5",
            "z": "-6",
            "offset_x": f64_bits(0.25),
            "offset_y": f64_bits(-1.5),
            "offset_z": f64_bits(2.75),
        }),
    ));
    out
}

fn center() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/coordinates/center",
        "math",
        "vec3_coordinates",
        json!({"kind": "center", "x": "-2", "y": "-64", "z": "7"}),
    ));
    out.push(case(
        "vec3/coordinates/bottom-center",
        "math",
        "vec3_coordinates",
        json!({"kind": "bottom_center", "x": "0", "y": "0", "z": "0"}),
    ));
    out.push(case(
        "vec3/coordinates/up-from-bottom-center",
        "math",
        "vec3_coordinates",
        json!({"kind": "up_from_bottom_center", "x": "4", "y": "-5", "z": "6", "y_offset": f64_bits(-0.75)}),
    ));
    out.push(case(
        "vec3/coordinates/offset-special-values",
        "math",
        "vec3_coordinates",
        json!({
            "kind": "lower_corner_offset",
            "x": "0",
            "y": "0",
            "z": "0",
            "offset_x": "8000000000000000",
            "offset_y": "0000000000000001",
            "offset_z": "7ff8000000000001",
        }),
    ));
    out.push(case(
        "vec3/rotate/x-basic",
        "math",
        "vec3_rotate",
        json!({"axis": "x", "first": vector_bits(1.25, -2.5, 3.75), "radians": f32_bits(0.5)}),
    ));
    out.push(case(
        "vec3/rotate/y-basic",
        "math",
        "vec3_rotate",
        json!({"axis": "y", "first": vector_bits(-4.5, 6.25, -8.75), "radians": f32_bits(-0.75)}),
    ));
    out
}

fn z_basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/rotate/z-basic",
        "math",
        "vec3_rotate",
        json!({"axis": "z", "first": vector_bits(2.0, -3.0, 4.0), "radians": f32_bits(1.25)}),
    ));
    out.push(case(
        "vec3/rotate/x-negative-angle",
        "math",
        "vec3_rotate",
        json!({"axis": "x", "first": vector_bits(-4.5, 6.25, -8.75), "radians": f32_bits(-3.0)}),
    ));
    out.push(case(
        "vec3/rotate/y-negative-angle",
        "math",
        "vec3_rotate",
        json!({"axis": "y", "first": vector_bits(1.25, -2.5, 3.75), "radians": f32_bits(2.75)}),
    ));
    out.push(case(
        "vec3/rotate/z-negative-angle",
        "math",
        "vec3_rotate",
        json!({"axis": "z", "first": vector_bits(-1.5, 2.25, -0.75), "radians": f32_bits(-2.25)}),
    ));
    out.push(case(
        "vec3/rotate/x-quantized-small-angle",
        "math",
        "vec3_rotate",
        json!({"axis": "x", "first": vector_bits(0.125, -0.25, 0.5), "radians": f32_bits(0.0001)}),
    ));
    out.push(case(
        "vec3/rotate/y-quantized-small-angle",
        "math",
        "vec3_rotate",
        json!({"axis": "y", "first": vector_bits(-0.125, 0.25, -0.5), "radians": f32_bits(-0.0001)}),
    ));
    out.push(case(
        "vec3/rotate/z-quarter-turn",
        "math",
        "vec3_rotate",
        json!({"axis": "z", "first": vector_bits(0.25, -0.5, 1.0), "radians": f32_bits(1.5707963267948966)}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        basic(),
        special_values(),
        y_max_finite(),
        down_overflow(),
        xz_negative(),
        zero_onto_preserves(),
        signed_zero(),
        center(),
        z_basic(),
    ]
    .concat()
}
