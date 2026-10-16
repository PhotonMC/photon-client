//! Cases for `mth/fast-inv-cube-root`, `mth/hsv-to-color`, `mth/get-seed`, `mth/inv-sqrt` and more.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::mth::tables::*;
use crate::math::values::*;

fn fast_inv_cube_root() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_fast_inv_cube_root = vec![
        ("zero", "00000000".to_string()),
        ("negative-zero", "80000000".to_string()),
        ("one", "3f800000".to_string()),
        ("one-below", "3f7fffff".to_string()),
        ("one-above", "3f800001".to_string()),
        ("eight", "41000000".to_string()),
        ("eighth", "3e000000".to_string()),
        ("twenty-seven", "41d80000".to_string()),
        ("pi", f32_bits(3.141592653589793)),
        ("large", f32_bits(1e+20)),
        ("max", "7f7fffff".to_string()),
        ("min-normal", "00800000".to_string()),
        ("positive-subnormal", "00000001".to_string()),
        ("negative-one", "bf800000".to_string()),
        ("negative-eight", "c1000000".to_string()),
        ("negative-max", "ff7fffff".to_string()),
        ("negative-min-normal", "80800000".to_string()),
        ("negative-subnormal", "80000001".to_string()),
        ("infinity", "7f800000".to_string()),
        ("negative-infinity", "ff800000".to_string()),
        ("nan", "7fc00000".to_string()),
        ("nan-payload", "7fc12345".to_string()),
        ("negative-nan-payload", "ffc12345".to_string()),
        ("tiny-neighbor", "00000002".to_string()),
    ];
    for (label, input_value) in &mth_fast_inv_cube_root {
        out.push(case(
            format!("mth/fast-inv-cube-root/{label}"),
            "math",
            "mth_fast_inv_cube_root",
            json!({"input": input_value}),
        ));
    }
    out
}

fn hsv_to_color() -> Vec<Value> {
    let mut out = Vec::new();
    for (label, hue, saturation, value, alpha) in &mth_hsv_to_color() {
        let hue = hue.f32();
        let saturation = saturation.f32();
        let value = value.f32();
        for mode in ["rgb", "argb"] {
            out.push(case(
                format!("mth/hsv-to-color/{label}/{mode}"),
                "math",
                "mth_hsv_to_color",
                json!({
                    "mode": mode,
                    "hue": hue.clone(),
                    "saturation": saturation.clone(),
                    "value": value.clone(),
                    "alpha": (alpha).to_string(),
                }),
            ));
        }
    }
    out
}

fn get_seed() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_get_seed = mth_get_seed_rows();
    for (label, x, y, z) in &mth_get_seed {
        out.push(case(format!("mth/get-seed/{label}"), "math", "mth_get_seed", json!({"x": x, "y": y, "z": z})));
    }
    let mth_inv_sqrt_f32 = mth_inv_sqrt_f32_rows();
    for (label, input_value) in &mth_inv_sqrt_f32 {
        out.push(case(
            format!("mth/inv-sqrt/f32/{label}"),
            "math",
            "mth_inv_sqrt",
            json!({"type": "f32", "input": input_value}),
        ));
    }
    out
}

fn mth_get_seed_rows() -> Vec<(&'static str, i32, i32, i32)> {
    vec![
        ("zero", 0, 0, 0),
        ("simple-positive", 1, 2, 3),
        ("simple-negative", -1, -2, -3),
        ("mixed-signs", -123, 456, -789),
        ("x-only", 123456, 0, 0),
        ("y-only", 0, -789, 0),
        ("z-only", 0, 0, 654321),
        ("x-min", -2147483648, 0, 0),
        ("x-max", 2147483647, 0, 0),
        ("y-min", 0, -2147483648, 0),
        ("y-max", 0, 2147483647, 0),
        ("z-min", 0, 0, -2147483648),
        ("z-max", 0, 0, 2147483647),
        ("all-min", -2147483648, -2147483648, -2147483648),
        ("all-max", 2147483647, 2147483647, 2147483647),
        ("alternating-boundaries", -2147483648, 2147483647, -2147483648),
        ("reversed-boundaries", 2147483647, -2147483648, 2147483647),
        ("x-product-within-i32", 686, 0, 0),
        ("x-product-wraps-positive", 687, 0, 0),
        ("x-product-wraps-negative", -687, 0, 0),
        ("large-mixed-positive", 1000000000, -1000000000, 1000000000),
        ("large-mixed-negative", -1000000000, 1000000000, -1000000000),
        ("hash-like", 123456789, -987654321, 135792468),
        ("section-like", -2097152, 319, 2097151),
    ]
}

fn mth_inv_sqrt_f32_rows() -> Vec<(&'static str, &'static str)> {
    vec![
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", "3f800000"),
        ("two", "40000000"),
        ("quarter", "3e800000"),
        ("maximum-finite", "7f7fffff"),
        ("minimum-normal", "00800000"),
        ("positive-subnormal", "00000001"),
        ("negative-one", "bf800000"),
        ("negative-subnormal", "80000001"),
        ("infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("nan", "7fc00000"),
        ("nan-payload", "7fc12345"),
        ("negative-nan-payload", "ffc12345"),
    ]
}

fn inv_sqrt_f64() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_inv_sqrt_f64 = vec![
        ("zero", "0000000000000000"),
        ("negative-zero", "8000000000000000"),
        ("one", "3ff0000000000000"),
        ("two", "4000000000000000"),
        ("quarter", "3fd0000000000000"),
        ("maximum-finite", "7fefffffffffffff"),
        ("minimum-normal", "0010000000000000"),
        ("positive-subnormal", "0000000000000001"),
        ("negative-one", "bff0000000000000"),
        ("negative-subnormal", "8000000000000001"),
        ("infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("nan", "7ff8000000000000"),
        ("nan-payload", "7ff0000000000001"),
        ("negative-nan-payload", "fff0000000000001"),
    ];
    for (label, input_value) in &mth_inv_sqrt_f64 {
        out.push(case(
            format!("mth/inv-sqrt/f64/{label}"),
            "math",
            "mth_inv_sqrt",
            json!({"type": "f64", "input": input_value}),
        ));
    }
    let mth_next_int = [
        ("normal", 42, -10, 10),
        ("same-bounds", 42, 5, 5),
        ("inverted-bounds", 42, 10, -10),
        ("positive-boundary", 123, 2147483640, 2147483647),
        ("negative-boundary", -123, -2147483648, -2147483641),
        ("single-bit-range", 0, 0, 1),
    ];
    for (label, seed, minimum, maximum) in &mth_next_int {
        out.push(case(
            format!("mth/next-random/i32/{label}"),
            "math",
            "mth_next_random",
            json!({
                "type": "i32",
                "seed": (seed).to_string(),
                "min": (minimum).to_string(),
                "max": (maximum).to_string(),
            }),
        ));
    }
    out
}

fn next_random() -> Vec<Value> {
    let mut out = Vec::new();
    for width in ["f32", "f64"] {
        let maximum_finite = if width == "f32" { 3.4028234663852886e+38 } else { 1.7976931348623157e+308 };
        let mth_next_float = [
            ("normal", 42, -1.0, 1.0),
            ("same-bounds", 42, 0.5, 0.5),
            ("inverted-bounds", 42, 1.0, -1.0),
            ("signed-zero", 42, -0.0, 0.0),
            ("wide-finite", 123, -maximum_finite, maximum_finite),
            ("infinite-max", 123, 1.0, f64::INFINITY),
            ("nan-min", 123, f64::NAN, 1.0),
        ];
        for (label, seed, minimum, maximum) in mth_next_float.iter().copied() {
            out.push(case(
                format!("mth/next-random/{width}/{label}"),
                "math",
                "mth_next_random",
                json!({
                    "type": width,
                    "seed": (seed).to_string(),
                    "min": float_bits(minimum, width),
                    "max": float_bits(maximum, width),
                }),
            ));
        }
    }
    out
}

fn inclusive() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_random_between_inclusive = [
        ("normal", 42, -10, 10),
        ("same-bounds", 42, 5, 5),
        ("positive-boundary", 123, 2147483640, 2147483647),
        ("negative-boundary", -123, -2147483648, -2147483641),
        ("wide-finite", 7, -100, 100),
        ("single-bit-range", 0, 0, 1),
    ];
    for (label, seed, minimum, maximum) in &mth_random_between_inclusive {
        out.push(case(
            format!("mth/random-helpers/inclusive/{label}"),
            "math",
            "mth_random_helpers",
            json!({
                "type": "i32",
                "fn": "inclusive",
                "seed": (seed).to_string(),
                "min": (minimum).to_string(),
                "max": (maximum).to_string(),
            }),
        ));
    }
    let mth_random_between = [
        ("normal", 42, -1.0, 1.0),
        ("same-bounds", 42, 0.5, 0.5),
        ("reversed-bounds", 42, 1.0, -1.0),
        ("signed-zero", 42, -0.0, 0.0),
        ("infinite-max", 123, 1.0, f64::INFINITY),
        ("nan-min", 123, f64::NAN, 1.0),
    ];
    for (label, seed, minimum, maximum) in mth_random_between.iter().copied() {
        out.push(case(
            format!("mth/random-helpers/between/{label}"),
            "math",
            "mth_random_helpers",
            json!({
                "type": "f32",
                "fn": "between",
                "seed": (seed).to_string(),
                "min": f32_bits(minimum),
                "max": f32_bits(maximum),
            }),
        ));
    }
    out
}

fn normal() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_normal = [
        ("normal", 42, 0.0, 1.0),
        ("shifted", 123, 2.5, 0.75),
        ("zero-deviation", 42, 4.0, 0.0),
        ("negative-deviation", -123, 1.0, -2.0),
        ("negative-zero-mean", 42, -0.0, 0.0),
        ("infinite-mean", 7, f64::INFINITY, 1.0),
    ];
    for (label, seed, mean, deviation) in mth_normal.iter().copied() {
        out.push(case(
            format!("mth/random-helpers/normal/{label}"),
            "math",
            "mth_random_helpers",
            json!({
                "type": "f32",
                "fn": "normal",
                "seed": (seed).to_string(),
                "mean": f32_bits(mean),
                "deviation": f32_bits(deviation),
            }),
        ));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [fast_inv_cube_root(), hsv_to_color(), get_seed(), inv_sqrt_f64(), next_random(), inclusive(), normal()].concat()
}
