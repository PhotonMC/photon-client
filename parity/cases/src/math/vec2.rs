//! Cases for `vec2/normalize`, `vec2/chain`, `vec2/rotate`, `vec2/dot` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

fn normalize() -> Vec<Value> {
    let mut out = Vec::new();
    let mut vectors: Vec<(String, (String, String))> = Vec::new();
    for x in [-1, 0, 1] {
        for y in [-1, 0, 1] {
            vectors.push((format!("direction-{x}-{y}"), (f32_bits(x), f32_bits(y))));
        }
    }
    let named = [
        ("negative-zero", "80000000".to_owned(), "80000000".to_owned()),
        ("subnormal", "00000001".to_owned(), "00000001".to_owned()),
        ("max-finite", "7f7fffff".to_owned(), "7f7fffff".to_owned()),
        ("unequal", f32_bits(3), f32_bits(-7)),
    ];
    for (label, x, y) in named {
        vectors.push((label.to_owned(), (x, y)));
    }
    for (index, bits) in neighbors(0.0001, "f32").into_iter().enumerate() {
        vectors.push((format!("normalization-boundary-{index}"), (bits, "00000000".to_owned())));
    }
    for (label, (x, y)) in vectors {
        out.push(case(
            format!("vec2/normalize/{}", label.clone()),
            "math",
            "vec2",
            json!({"x": x.clone(), "y": y.clone(), "steps": [{"action": "normalize"}]}),
        ));
    }
    for factor in [-2.0, -0.0, 0.0, 0.37, 0.98, 3.0] {
        out.push(case(
            format!("vec2/chain/{}", f32_bits(factor)),
            "math",
            "vec2",
            json!({
                "x": f32_bits(1),
                "y": f32_bits(-1),
                "steps": [
                    {"action": "normalize"},
                    {"action": "scale", "factor": f32_bits(factor)},
                    {"action": "normalize"},
                ],
            }),
        ));
    }
    out.push(case(
        "vec2/rotate/basic",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(1.25, -2.5), "angle": f64_bits(0.5)}),
    ));
    out.push(case(
        "vec2/rotate/negative-components",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(-4.5, 6.25), "angle": f64_bits(-0.75)}),
    ));
    out
}

fn zero_angle() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec2/rotate/zero-angle",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(2.0, -3.0), "angle": f64_bits(0.0)}),
    ));
    out.push(case(
        "vec2/rotate/quarter-turn",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(1.0, 0.0), "angle": f64_bits(1.5707963267948966)}),
    ));
    out.push(case(
        "vec2/rotate/half-turn",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(0.25, -0.5), "angle": f64_bits(3.141592653589793)}),
    ));
    out.push(case(
        "vec2/rotate/negative-quarter-turn",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(0.0, 1.0), "angle": f64_bits(-1.5707963267948966)}),
    ));
    out.push(case(
        "vec2/rotate/small-quantized-angle",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(-0.125, 0.25), "angle": f64_bits(0.0001)}),
    ));
    out.push(case(
        "vec2/rotate/full-turn",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(1.0, -1.0), "angle": f64_bits(6.283185307179586)}),
    ));
    out.push(case(
        "vec2/rotate/combined",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(3.5, -7.25), "angle": f64_bits(1.234567890123)}),
    ));
    out
}

fn signed_zero_angle() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec2/rotate/signed-zero-angle",
        "math",
        "vec2_rotate",
        json!({"first": {"x": "80000000", "y": "00000000"}, "angle": "8000000000000000"}),
    ));
    out.push(case(
        "vec2/rotate/signed-zero-vector",
        "math",
        "vec2_rotate",
        json!({"first": {"x": "00000000", "y": "80000000"}, "angle": "0000000000000000"}),
    ));
    out.push(case(
        "vec2/rotate/subnormal-vector",
        "math",
        "vec2_rotate",
        json!({"first": {"x": "00000001", "y": "80000001"}, "angle": f64_bits(0.7853981633974483)}),
    ));
    out.push(case(
        "vec2/rotate/max-finite-vector",
        "math",
        "vec2_rotate",
        json!({"first": {"x": "7f7fffff", "y": "ff7fffff"}, "angle": f64_bits(0.25)}),
    ));
    out.push(case(
        "vec2/rotate/special-vector",
        "math",
        "vec2_rotate",
        json!({"first": {"x": "7fc00001", "y": "ff800000"}, "angle": f64_bits(0.5)}),
    ));
    out.push(case(
        "vec2/rotate/positive-infinity-angle",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(1.0, -2.0), "angle": f64_bits(f64::INFINITY)}),
    ));
    out.push(case(
        "vec2/rotate/negative-infinity-angle",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(-4.0, 6.0), "angle": f64_bits(f64::NEG_INFINITY)}),
    ));
    out
}

fn nan_angle() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec2/rotate/nan-angle",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(3.5, -7.25), "angle": "7ff8000000000001"}),
    ));
    out.push(case(
        "vec2/rotate/large-finite-angle",
        "math",
        "vec2_rotate",
        json!({"first": vector2_bits(0.5, -1.0), "angle": "7fefffffffffffff"}),
    ));
    out.push(case(
        "vec2/dot/basic",
        "math",
        "vec2_dot",
        json!({"first": vector2_bits(1.25, -2.5), "second": vector2_bits(4.0, -1.0)}),
    ));
    out.push(case(
        "vec2/dot/order-and-rounding",
        "math",
        "vec2_dot",
        json!({"first": vector2_bits(1.0, 1.1920928955078125e-07), "second": vector2_bits(1.0, 1.0)}),
    ));
    out.push(case(
        "vec2/dot/signed-zero",
        "math",
        "vec2_dot",
        json!({"first": {"x": "80000000", "y": "80000000"}, "second": vector2_bits(1.0, 1.0)}),
    ));
    out.push(case(
        "vec2/dot/overflow",
        "math",
        "vec2_dot",
        json!({"first": {"x": "7f7fffff", "y": f32_bits(1.0)}, "second": vector2_bits(2.0, 0.0)}),
    ));
    out.push(case(
        "vec2/dot/special-values",
        "math",
        "vec2_dot",
        json!({"first": {"x": "7f800000", "y": f32_bits(2.0)}, "second": {"x": "00000000", "y": f32_bits(-4.0)}}),
    ));
    out
}

fn vector_basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec2/add/vector-basic",
        "math",
        "vec2_add_vec2",
        json!({"first": vector2_bits(1.25, -2.5), "second": vector2_bits(-0.25, 4.0)}),
    ));
    out.push(case(
        "vec2/add/vector-signed-zero",
        "math",
        "vec2_add_vec2",
        json!({"first": {"x": "80000000", "y": "80000000"}, "second": {"x": "80000000", "y": "80000000"}}),
    ));
    out.push(case(
        "vec2/add/vector-overflow",
        "math",
        "vec2_add_vec2",
        json!({"first": {"x": "7f7fffff", "y": "ff7fffff"}, "second": {"x": "7f7fffff", "y": "ff7fffff"}}),
    ));
    out.push(case(
        "vec2/add/scalar-basic",
        "math",
        "vec2_add_scalar",
        json!({"first": vector2_bits(-1.5, 2.25), "value": f32_bits(0.5)}),
    ));
    out.push(case(
        "vec2/add/scalar-special-values",
        "math",
        "vec2_add_scalar",
        json!({"first": {"x": "7f800000", "y": f32_bits(2.0)}, "value": "ff800000"}),
    ));
    out.push(case(
        "vec2/distance-to-sqr/vector-basic",
        "math",
        "vec2_distance_to_sqr",
        json!({"first": vector2_bits(1.25, -2.5), "second": vector2_bits(4.5, -1.0)}),
    ));
    out.push(case(
        "vec2/distance-to-sqr/order-and-rounding",
        "math",
        "vec2_distance_to_sqr",
        json!({"first": vector2_bits(0.0, 0.0), "second": vector2_bits(4096.0, 1.1)}),
    ));
    out
}

fn signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec2/distance-to-sqr/signed-zero",
        "math",
        "vec2_distance_to_sqr",
        json!({"first": vector2_bits(-0.0, -0.0), "second": vector2_bits(0.0, 0.0)}),
    ));
    out.push(case(
        "vec2/distance-to-sqr/overflow",
        "math",
        "vec2_distance_to_sqr",
        json!({"first": vector2_bits(0.0, 0.0), "second": {"x": "7f7fffff", "y": "7f7fffff"}}),
    ));
    out.push(case(
        "vec2/distance-to-sqr/special-values",
        "math",
        "vec2_distance_to_sqr",
        json!({"first": vector2_bits(0.0, 0.0), "second": {"x": "7f800000", "y": "7fc00002"}}),
    ));
    out.push(case("vec2/negated/basic", "math", "vec2_negated", json!({"first": vector2_bits(1.25, -2.5)})));
    out.push(case(
        "vec2/negated/signed-zero",
        "math",
        "vec2_negated",
        json!({"first": {"x": "80000000", "y": "00000000"}}),
    ));
    out.push(case(
        "vec2/negated/subnormal",
        "math",
        "vec2_negated",
        json!({"first": {"x": "00000001", "y": "80000001"}}),
    ));
    out.push(case(
        "vec2/negated/infinity-and-boundary",
        "math",
        "vec2_negated",
        json!({"first": {"x": "7f7fffff", "y": "ff800000"}}),
    ));
    out.push(case(
        "vec2/negated/nan-payload",
        "math",
        "vec2_negated",
        json!({"first": {"x": "7fc00001", "y": "ffc00002"}}),
    ));
    out
}

fn basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec2/equality/basic",
        "math",
        "vec2_equality",
        json!({"first": vector2_bits(1.25, -2.5), "second": vector2_bits(1.25, -2.5)}),
    ));
    out.push(case(
        "vec2/equality/different-component",
        "math",
        "vec2_equality",
        json!({"first": vector2_bits(1.25, -2.5), "second": vector2_bits(1.25, -2.25)}),
    ));
    out.push(case(
        "vec2/equality/signed-zero",
        "math",
        "vec2_equality",
        json!({"first": {"x": "80000000", "y": "00000000"}, "second": {"x": "00000000", "y": "00000000"}}),
    ));
    out.push(case(
        "vec2/equality/nan-payloads",
        "math",
        "vec2_equality",
        json!({"first": {"x": "7fc00001", "y": "ffc00002"}, "second": {"x": "7fc00002", "y": "ffc00001"}}),
    ));
    out.push(case(
        "vec2/equality/infinities",
        "math",
        "vec2_equality",
        json!({"first": {"x": "7f800000", "y": "ff800000"}, "second": {"x": "7f800000", "y": "ff800000"}}),
    ));
    out.push(case(
        "vec2/equality/special-mismatch",
        "math",
        "vec2_equality",
        json!({"first": {"x": "7f800000", "y": "7fc00001"}, "second": {"x": "ff800000", "y": "7fc00002"}}),
    ));
    for name in ["zero", "one", "unit_x", "neg_unit_x", "unit_y", "neg_unit_y", "max", "min"] {
        out.push(case(format!("vec2/constants/{name}"), "math", "vec2_constant", json!({"name": name})));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [normalize(), zero_angle(), signed_zero_angle(), nan_angle(), vector_basic(), signed_zero(), basic()].concat()
}
