//! Cases for `vec3/dot`, `vec3/distance-to-sqr`, `vec3/distance-to`, `vec3/horizontal-distances` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

fn basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/dot/basic",
        "math",
        "vec3_dot",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "second": vector_bits(4.0, -1.0, 0.5)}),
    ));
    out.push(case(
        "vec3/dot/order-and-cancellation",
        "math",
        "vec3_dot",
        json!({
            "first": vector_bits(1.0, 1.1102230246251565e-16, -1.0),
            "second": vector_bits(9007199254740992.0, 1.0, 9007199254740992.0),
        }),
    ));
    out.push(case(
        "vec3/dot/signed-zero",
        "math",
        "vec3_dot",
        json!({"first": vector_bits(-0.0, -0.0, -0.0), "second": vector_bits(1.0, 1.0, 1.0)}),
    ));
    out.push(case(
        "vec3/dot/overflow",
        "math",
        "vec3_dot",
        json!({
            "first": {"x": "7fefffffffffffff", "y": f64_bits(1.0), "z": f64_bits(-1.0)},
            "second": {"x": f64_bits(2.0), "y": f64_bits(0.0), "z": f64_bits(0.0)},
        }),
    ));
    out.push(case(
        "vec3/dot/special-values",
        "math",
        "vec3_dot",
        json!({
            "first": {"x": f64_bits(f64::INFINITY), "y": f64_bits(2.0), "z": f64_bits(-3.0)},
            "second": {"x": f64_bits(0.0), "y": f64_bits(-4.0), "z": f64_bits(5.0)},
        }),
    ));
    out.push(case(
        "vec3/distance-to-sqr/vector-basic",
        "math",
        "vec3_distance_to_sqr",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "second": vector_bits(4.5, -1.0, -0.25)}),
    ));
    out
}

fn components_basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/distance-to-sqr/components-basic",
        "math",
        "vec3_distance_to_sqr_components",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "x": f64_bits(4.5), "y": f64_bits(-1.0), "z": f64_bits(-0.25)}),
    ));
    out.push(case(
        "vec3/distance-to-sqr/order-and-rounding",
        "math",
        "vec3_distance_to_sqr",
        json!({"first": vector_bits(0.0, 0.0, 0.0), "second": vector_bits(134217728.0, 1.0, 1.1)}),
    ));
    out.push(case(
        "vec3/distance-to-sqr/signed-zero",
        "math",
        "vec3_distance_to_sqr_components",
        json!({"first": vector_bits(-0.0, -0.0, -0.0), "x": f64_bits(0.0), "y": f64_bits(0.0), "z": f64_bits(0.0)}),
    ));
    out.push(case(
        "vec3/distance-to-sqr/overflow-and-special-values",
        "math",
        "vec3_distance_to_sqr",
        json!({
            "first": vector_bits(0.0, 0.0, 0.0),
            "second": {"x": "7fefffffffffffff", "y": f64_bits(f64::INFINITY), "z": "7ff8000000000002"},
        }),
    ));
    out.push(case(
        "vec3/distance-to/basic",
        "math",
        "vec3_distance_to",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "second": vector_bits(4.5, -1.0, -0.25)}),
    ));
    out.push(case(
        "vec3/distance-to/order-and-rounding",
        "math",
        "vec3_distance_to",
        json!({"first": vector_bits(0.0, 0.0, 0.0), "second": vector_bits(134217728.0, 1.0, 1.1)}),
    ));
    out.push(case(
        "vec3/distance-to/signed-zero",
        "math",
        "vec3_distance_to",
        json!({"first": vector_bits(-0.0, -0.0, -0.0), "second": vector_bits(0.0, 0.0, 0.0)}),
    ));
    out
}

fn subnormal() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/distance-to/subnormal",
        "math",
        "vec3_distance_to",
        json!({
            "first": {"x": "0000000000000001", "y": "8000000000000001", "z": "0000000000000000"},
            "second": vector_bits(0.0, 0.0, 0.0),
        }),
    ));
    out.push(case(
        "vec3/distance-to/overflow-and-special-values",
        "math",
        "vec3_distance_to",
        json!({
            "first": vector_bits(0.0, 0.0, 0.0),
            "second": {"x": "7fefffffffffffff", "y": f64_bits(f64::INFINITY), "z": "7ff8000000000002"},
        }),
    ));
    out.push(case(
        "vec3/horizontal-distances/basic",
        "math",
        "vec3_horizontal_distances",
        json!({"first": vector_bits(3.0, 4.0, -12.0)}),
    ));
    out.push(case(
        "vec3/horizontal-distances/order-and-rounding",
        "math",
        "vec3_horizontal_distances",
        json!({"first": vector_bits(134217728.0, -123.0, 1.1)}),
    ));
    out.push(case(
        "vec3/horizontal-distances/signed-zero",
        "math",
        "vec3_horizontal_distances",
        json!({"first": vector_bits(-0.0, 5.0, 0.0)}),
    ));
    out.push(case(
        "vec3/horizontal-distances/subnormal",
        "math",
        "vec3_horizontal_distances",
        json!({"first": {"x": "0000000000000001", "y": "7fefffffffffffff", "z": "8000000000000001"}}),
    ));
    out
}

fn infinities() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/horizontal-distances/infinities",
        "math",
        "vec3_horizontal_distances",
        json!({"first": {"x": f64_bits(f64::INFINITY), "y": "7ff8000000000001", "z": f64_bits(f64::NEG_INFINITY)}}),
    ));
    out.push(case(
        "vec3/closer-than/basic",
        "math",
        "vec3_closer_than",
        json!({
            "first": vector_bits(1.25, -2.5, 3.75),
            "second": vector_bits(4.5, -1.0, -0.25),
            "distance": f64_bits(6.0),
            "distance_xz": f64_bits(5.25),
            "distance_y": f64_bits(2.0),
        }),
    ));
    out.push(case(
        "vec3/closer-than/strict-boundary",
        "math",
        "vec3_closer_than",
        json!({
            "first": vector_bits(0.0, 0.0, 0.0),
            "second": vector_bits(3.0, 4.0, 0.0),
            "distance": f64_bits(5.0),
            "distance_xz": f64_bits(3.0),
            "distance_y": f64_bits(4.0),
        }),
    ));
    out.push(case(
        "vec3/closer-than/vertical-gate",
        "math",
        "vec3_closer_than",
        json!({
            "first": vector_bits(0.0, 0.0, 0.0),
            "second": vector_bits(0.0, 4.0, 0.0),
            "distance": f64_bits(5.0),
            "distance_xz": f64_bits(1.0),
            "distance_y": f64_bits(4.0),
        }),
    ));
    out
}

fn negative_zero_thresholds() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/closer-than/negative-zero-thresholds",
        "math",
        "vec3_closer_than",
        json!({
            "first": vector_bits(-0.0, -0.0, -0.0),
            "second": vector_bits(0.0, 0.0, 0.0),
            "distance": f64_bits(-0.0),
            "distance_xz": f64_bits(-0.0),
            "distance_y": f64_bits(-0.0),
        }),
    ));
    out.push(case(
        "vec3/closer-than/special-values",
        "math",
        "vec3_closer_than",
        json!({
            "first": {"x": f64_bits(f64::NAN), "y": f64_bits(1.0), "z": f64_bits(f64::INFINITY)},
            "second": vector_bits(0.0, 0.0, 0.0),
            "distance": f64_bits(f64::INFINITY),
            "distance_xz": f64_bits(f64::INFINITY),
            "distance_y": f64_bits(f64::INFINITY),
        }),
    ));
    out.push(case(
        "vec3/offset-random/basic",
        "math",
        "vec3_offset_random",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "seed": "42", "offset": f32_bits(1.0)}),
    ));
    out.push(case(
        "vec3/offset-random/negative-offset",
        "math",
        "vec3_offset_random",
        json!({"first": vector_bits(-4.5, 6.25, 8.75), "seed": "-1", "offset": f32_bits(-2.5)}),
    ));
    out.push(case(
        "vec3/offset-random/signed-zero",
        "math",
        "vec3_offset_random",
        json!({"first": vector_bits(-0.0, 0.0, -0.0), "seed": "0", "offset": f32_bits(-0.0)}),
    ));
    out
}

fn subnormal_offset() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/offset-random/subnormal-offset",
        "math",
        "vec3_offset_random",
        json!({
            "first": {"x": "0000000000000001", "y": "8000000000000001", "z": "0000000000000000"},
            "seed": "12345",
            "offset": "00000001",
        }),
    ));
    out.push(case(
        "vec3/offset-random/special-values",
        "math",
        "vec3_offset_random",
        json!({
            "first": {"x": f64_bits(f64::INFINITY), "y": f64_bits(f64::NAN), "z": f64_bits(f64::NEG_INFINITY)},
            "seed": "-9223372036854775808",
            "offset": "7f7fffff",
        }),
    ));
    out.push(case(
        "vec3/lerp/basic",
        "math",
        "vec3_lerp",
        json!({
            "first": vector_bits(1.25, -2.5, 3.75),
            "second": vector_bits(-4.5, 6.25, 8.75),
            "alpha": f64_bits(0.25),
        }),
    ));
    out.push(case(
        "vec3/lerp/negative-extrapolation",
        "math",
        "vec3_lerp",
        json!({
            "first": vector_bits(-4.5, 6.25, 8.75),
            "second": vector_bits(1.25, -2.5, 3.75),
            "alpha": f64_bits(-0.5),
        }),
    ));
    out
}

fn over_one() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/lerp/over-one",
        "math",
        "vec3_lerp",
        json!({
            "first": vector_bits(1e+200, -1e+200, 1e-200),
            "second": vector_bits(-1e+200, 1e+200, -1e-200),
            "alpha": f64_bits(1.5),
        }),
    ));
    out.push(case(
        "vec3/lerp/signed-zero-and-subnormal",
        "math",
        "vec3_lerp",
        json!({
            "first": {"x": "8000000000000000", "y": "0000000000000001", "z": "8000000000000001"},
            "second": {"x": "0000000000000000", "y": "8000000000000000", "z": "0000000000000000"},
            "alpha": f64_bits(-0.0),
        }),
    ));
    out.push(case(
        "vec3/lerp/special-values",
        "math",
        "vec3_lerp",
        json!({
            "first": {"x": f64_bits(f64::INFINITY), "y": f64_bits(f64::NAN), "z": f64_bits(f64::NEG_INFINITY)},
            "second": {"x": f64_bits(f64::NEG_INFINITY), "y": f64_bits(1.0), "z": f64_bits(f64::INFINITY)},
            "alpha": f64_bits(0.5),
        }),
    ));
    out.push(case(
        "vec3/is-finite/all-finite",
        "math",
        "vec3_is_finite",
        json!({"first": {"x": "7fefffffffffffff", "y": "8000000000000001", "z": "0000000000000000"}}),
    ));
    out.push(case(
        "vec3/is-finite/positive-infinity",
        "math",
        "vec3_is_finite",
        json!({"first": {"x": f64_bits(f64::INFINITY), "y": f64_bits(-1.0), "z": f64_bits(2.0)}}),
    ));
    out
}

fn negative_infinity() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/is-finite/negative-infinity",
        "math",
        "vec3_is_finite",
        json!({"first": {"x": f64_bits(1.0), "y": f64_bits(f64::NEG_INFINITY), "z": f64_bits(2.0)}}),
    ));
    out.push(case(
        "vec3/is-finite/nan",
        "math",
        "vec3_is_finite",
        json!({"first": {"x": f64_bits(1.0), "y": f64_bits(-2.0), "z": "7ff8000000000001"}}),
    ));
    out.push(case(
        "vec3/is-finite/mixed-special-values",
        "math",
        "vec3_is_finite",
        json!({"first": {"x": "fff0000000000000", "y": "7ff8000000000000", "z": "0000000000000001"}}),
    ));
    out.push(case(
        "vec3/equality/basic",
        "math",
        "vec3_equality",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "second": vector_bits(1.25, -2.5, 3.75)}),
    ));
    out.push(case(
        "vec3/equality/different-component",
        "math",
        "vec3_equality",
        json!({"first": vector_bits(1.25, -2.5, 3.75), "second": vector_bits(1.25, -2.5, 4.0)}),
    ));
    out.push(case(
        "vec3/equality/signed-zero",
        "math",
        "vec3_equality",
        json!({
            "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "0000000000000000"},
            "second": vector_bits(0.0, 0.0, 0.0),
        }),
    ));
    out
}

fn nan_payloads() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/equality/nan-payloads",
        "math",
        "vec3_equality",
        json!({
            "first": {"x": "7ff8000000000001", "y": "8000000000000000", "z": "7ff0000000000000"},
            "second": {"x": "7ff0000000000001", "y": "8000000000000000", "z": "7ff0000000000000"},
        }),
    ));
    out.push(case(
        "vec3/equality/infinities",
        "math",
        "vec3_equality",
        json!({
            "first": {"x": "7ff0000000000000", "y": "fff0000000000000", "z": "0000000000000000"},
            "second": {"x": "7ff0000000000000", "y": "fff0000000000000", "z": "0000000000000000"},
        }),
    ));
    out.push(case(
        "vec3/equality/special-mismatch",
        "math",
        "vec3_equality",
        json!({
            "first": {"x": "7ff0000000000000", "y": "7ff8000000000001", "z": "8000000000000000"},
            "second": {"x": "fff0000000000000", "y": "7ff0000000000001", "z": "0000000000000000"},
        }),
    ));
    out.push(case("vec3/to-string/basic", "math", "vec3_to_string", json!({"first": vector_bits(0.125, -2.5, 3.75)})));
    out.push(case(
        "vec3/to-string/scientific",
        "math",
        "vec3_to_string",
        json!({"first": vector_bits(0.0001, 10000000.0, -0.0)}),
    ));
    out.push(case(
        "vec3/to-string/thresholds",
        "math",
        "vec3_to_string",
        json!({"first": vector_bits(0.001, 9999999.0, 100000000.0)}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        basic(),
        components_basic(),
        subnormal(),
        infinities(),
        negative_zero_thresholds(),
        subnormal_offset(),
        over_one(),
        negative_infinity(),
        nan_payloads(),
    ]
    .concat()
}
