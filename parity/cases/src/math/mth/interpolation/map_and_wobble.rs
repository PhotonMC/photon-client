//! Cases for `mth/map-f32`, `mth/wobble-f64`, `mth/interpolation`.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

const MIN_SUBNORMAL_F32: f64 = 1.401298464324817e-45;
const MAXIMUM_FINITE_F32: f64 = 3.4028234663852886e+38;
const MAXIMUM_FINITE_F64: f64 = 1.7976931348623157e+308;
const MIN_SUBNORMAL_F64: f64 = 5e-324;

fn map_f32() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_map_f32 = mth_map_f32_rows();
    for (label, value, from_min, from_max, to_min, to_max) in mth_map_f32.iter().copied() {
        out.push(case(
            format!("mth/map-f32/{label}"),
            "math",
            "mth_map_f32",
            json!({
                "value": f32_bits(value),
                "from_min": f32_bits(from_min),
                "from_max": f32_bits(from_max),
                "to_min": f32_bits(to_min),
                "to_max": f32_bits(to_max),
            }),
        ));
    }
    let wobble_coords = wobble_coords_rows();
    for (label, coord) in &wobble_coords {
        out.push(case(format!("mth/wobble-f64/{label}"), "math", "mth_wobble_f64", json!({"coord": coord.clone()})));
    }
    out
}

fn mth_map_f32_rows() -> Vec<(&'static str, f64, f64, f64, f64, f64)> {
    vec![
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", MIN_SUBNORMAL_F32, 0.0, MIN_SUBNORMAL_F32, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, MIN_SUBNORMAL_F32),
        ("precision-order", 0.1, 10000000000.0, 10001000000.0, -10000000000.0, 10000000000.0),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, f64::NEG_INFINITY, f64::INFINITY, 0.0, 1.0),
        ("positive-infinity-value", f64::INFINITY, 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", f64::NEG_INFINITY, 0.0, 1.0, 0.0, 1.0),
        ("nan-value", f64::NAN, 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, f64::NAN, 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -MAXIMUM_FINITE_F32, MAXIMUM_FINITE_F32),
    ]
}

fn wobble_coords_rows() -> Vec<(&'static str, std::string::String)> {
    vec![
        ("zero", f64_bits(0.0)),
        ("negative-zero", "8000000000000000".to_string()),
        ("positive-integer", f64_bits(1.0)),
        ("negative-integer", f64_bits(-1.0)),
        ("positive-fraction", f64_bits(0.25)),
        ("negative-fraction", f64_bits(-0.25)),
        ("positive-boundary", f64_bits(0.0003333333333333333)),
        ("negative-boundary", f64_bits(-0.0003333333333333333)),
        ("below-positive-boundary", f64_bits(0.00033333333333333327)),
        ("above-positive-boundary", f64_bits(0.0003333333333333334)),
        ("large-positive", f64_bits(1000000.25)),
        ("large-negative", f64_bits(-1000000.25)),
        ("positive-floor-overflow", f64_bits(715827.883)),
        ("negative-floor-overflow", f64_bits(-715827.883)),
        ("maximum-finite", f64_bits(MAXIMUM_FINITE_F64)),
        ("positive-subnormal", "0000000000000001".to_string()),
        ("negative-subnormal", "8000000000000001".to_string()),
        ("positive-infinity", "7ff0000000000000".to_string()),
        ("negative-infinity", "fff0000000000000".to_string()),
        ("negative-nan-payload", "fff8000000000001".to_string()),
    ]
}

fn lerp2() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_lerp2 = mth_lerp2_rows();
    for (label, alpha1, alpha2, x00, x10, x01, x11) in mth_lerp2.iter().cloned() {
        out.push(case(
            format!("mth/interpolation/lerp2/{label}"),
            "math",
            "mth_interpolation",
            json!({
                "fn": "lerp2",
                "alpha1": interpolation_bits(alpha1, "f64"),
                "alpha2": interpolation_bits(alpha2, "f64"),
                "x00": interpolation_bits(x00.clone(), "f64"),
                "x10": interpolation_bits(x10.clone(), "f64"),
                "x01": interpolation_bits(x01.clone(), "f64"),
                "x11": interpolation_bits(x11.clone(), "f64"),
            }),
        ));
    }
    out
}

fn mth_lerp2_rows() -> Vec<(&'static str, f64, f64, Bits, Bits, Bits, Bits)> {
    vec![
        ("basic", 0.25, 0.5, num(0.0), num(10.0), num(20.0), num(30.0)),
        ("zero-alpha", 0.0, 0.0, num(1.0), num(2.0), num(3.0), num(4.0)),
        ("one-alpha", 1.0, 1.0, num(1.0), num(2.0), num(3.0), num(4.0)),
        ("extrapolation", 1.5, -0.5, num(0.0), num(10.0), num(20.0), num(30.0)),
        ("negative-alpha", -0.25, 0.75, num(-1.0), num(3.0), num(-5.0), num(7.0)),
        ("reversed-grid", 0.75, 0.25, num(10.0), num(0.0), num(-10.0), num(-20.0)),
        ("signed-zero-alpha", -0.0, 0.0, num(-0.0), num(1.0), num(-0.0), num(1.0)),
        (
            "signed-zero-grid",
            0.5,
            0.5,
            raw("8000000000000000"),
            raw("0000000000000000"),
            raw("8000000000000000"),
            raw("0000000000000000"),
        ),
        (
            "subnormal-grid",
            0.5,
            0.5,
            num(MIN_SUBNORMAL_F64),
            num(2.0 * MIN_SUBNORMAL_F64),
            num(-MIN_SUBNORMAL_F64),
            num(-2.0 * MIN_SUBNORMAL_F64),
        ),
        ("tiny-alpha", 2.225073858507202e-308, 0.5, num(1.0), num(2.0), num(3.0), num(4.0)),
        ("large-finite", 0.25, 0.75, num(1e+300), num(-1e+300), num(1e+300), num(-1e+300)),
        (
            "overflow-grid",
            0.5,
            0.5,
            num(MAXIMUM_FINITE_F64),
            num(-MAXIMUM_FINITE_F64),
            num(MAXIMUM_FINITE_F64),
            num(-MAXIMUM_FINITE_F64),
        ),
        ("positive-infinity-grid", 0.25, 0.75, num(f64::INFINITY), num(1.0), num(2.0), num(f64::INFINITY)),
        ("negative-infinity-grid", 0.25, 0.75, num(f64::NEG_INFINITY), num(-1.0), num(-2.0), num(f64::NEG_INFINITY)),
        ("nan-alpha", f64::NAN, 0.5, num(0.0), num(1.0), num(2.0), num(3.0)),
        ("nan-first-corner", 0.5, 0.5, num(f64::NAN), num(1.0), num(2.0), num(3.0)),
        ("nan-last-corner", 0.5, 0.5, num(0.0), num(1.0), num(2.0), num(f64::NAN)),
        ("precision-order", 0.1, 0.9, num(1e+20), num(1.00000000000001e+20), num(-1e+20), num(-9.9999999999999e+19)),
        (
            "mixed-exponents",
            0.3333333333333333,
            0.6666666666666666,
            num(3.054936363499605e-151),
            num(3.273390607896142e+150),
            num(-3.054936363499605e-151),
            num(-3.273390607896142e+150),
        ),
        ("underflow", MIN_SUBNORMAL_F64, 0.5, num(1.0), num(2.0), num(-1.0), num(-2.0)),
    ]
}

pub fn cases() -> Vec<Value> {
    [map_f32(), lerp2()].concat()
}
