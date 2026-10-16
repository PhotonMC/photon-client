//! Cases for `mth/clamped-lerp`, `mth/quantize`, `mth/inverse-lerp`, `mth/lerp` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::mth::tables::*;
use crate::math::values::*;

const MIN_SUBNORMAL_F64: f64 = 5e-324;
const MAXIMUM_FINITE_F64: f64 = 1.7976931348623157e+308;
const MIN_SUBNORMAL_F32: f64 = 1.401298464324817e-45;
const MAXIMUM_FINITE_F32: f64 = 3.4028234663852886e+38;

fn clamped_lerp() -> Vec<Value> {
    let mut out = Vec::new();
    for width in ["f32", "f64"] {
        let subnormal = if width == "f32" { 1.401298464324817e-45 } else { 5e-324 };
        let maximum_finite = if width == "f32" { 3.4028234663852886e+38 } else { 1.7976931348623157e+308 };
        let mth_clamped_lerp = vec![
            ("negative-factor", -0.5, -2.5, 4.5),
            ("negative-infinity-factor", f64::NEG_INFINITY, -2.5, 4.5),
            ("negative-zero-factor", -0.0, -0.0, 1.0),
            ("zero-factor", 0.0, -3.25, 7.5),
            ("within-factor", 0.25, -4.0, 12.0),
            ("half-factor", 0.5, -4.0, 12.0),
            ("one-factor", 1.0, -4.0, 12.0),
            ("above-factor", 1.5, -4.0, 12.0),
            ("positive-infinity-factor", f64::INFINITY, -4.0, 12.0),
            ("nan-factor", f64::NAN, 1.0, 2.0),
            ("reversed-range", 0.25, 10.0, -10.0),
            ("precision-order", 0.1, 1e+20, 1.00000000000001e+20),
            ("subnormal-range", 0.5, 0.0, subnormal),
            ("maximum-finite-range", 0.5, -maximum_finite, maximum_finite),
            ("infinite-range", 0.5, f64::NEG_INFINITY, f64::INFINITY),
            ("nan-min-out-of-range", -0.5, f64::NAN, 2.0),
        ];
        for (label, factor, minimum, maximum) in mth_clamped_lerp.iter().copied() {
            out.push(case(
                format!("mth/clamped-lerp/{width}/{label}"),
                "math",
                "mth_clamped_lerp",
                json!({
                    "type": width,
                    "factor": float_bits(factor, width),
                    "min": float_bits(minimum, width),
                    "max": float_bits(maximum, width),
                }),
            ));
        }
    }
    for (label, value, resolution) in mth_quantize().iter().copied() {
        out.push(case(
            format!("mth/quantize/{label}"),
            "math",
            "mth_quantize",
            json!({"value": f64_bits(value), "resolution": (resolution).to_string()}),
        ));
    }
    out
}

fn subnormal(width: &str) -> f64 {
    if width == "f32" { 1.401298464324817e-45 } else { 5e-324 }
}

fn maximum_finite(width: &str) -> f64 {
    if width == "f32" { 3.4028234663852886e+38 } else { 1.7976931348623157e+308 }
}

fn inverse_lerp_rows(width: &str) -> Vec<(&'static str, f64, f64, f64)> {
    let (subnormal, maximum_finite) = (subnormal(width), maximum_finite(width));
    vec![
        ("basic", 1.25, 0.0, 5.0),
        ("below", -1.0, 0.0, 5.0),
        ("above", 7.0, 0.0, 5.0),
        ("negative-range", -2.0, -5.0, 5.0),
        ("reverse-bounds", 2.0, 5.0, -5.0),
        ("exact-min", -2.0, -2.0, 6.0),
        ("exact-max", 6.0, -2.0, 6.0),
        ("signed-zero", -0.0, 0.0, 1.0),
        ("zero-range", 1.0, 1.0, 1.0),
        ("subnormal", 2.0 * subnormal, subnormal, 3.0 * subnormal),
        ("maximum-finite-overflow", maximum_finite, -maximum_finite, maximum_finite),
        ("infinite-value", f64::INFINITY, 0.0, 1.0),
        ("nan-value", f64::NAN, 0.0, 1.0),
        ("infinite-bound", 1.0, 0.0, f64::INFINITY),
    ]
}

fn lerp_rows(width: &str) -> Vec<(&'static str, f64, f64, f64)> {
    let (subnormal, maximum_finite) = (subnormal(width), maximum_finite(width));
    vec![
        ("basic", 0.25, 0.0, 8.0),
        ("negative-extrapolation", -0.5, 2.0, 10.0),
        ("over-one", 1.5, 2.0, 10.0),
        ("negative-range", 0.25, -4.0, 8.0),
        ("reverse-endpoints", 0.25, 8.0, -4.0),
        ("same-endpoints", 0.75, 3.0, 3.0),
        ("alpha-signed-zero", -0.0, 0.0, 1.0),
        ("negative-zero-start", 0.0, -0.0, 1.0),
        ("subnormal", 2.0 * subnormal, subnormal, 3.0 * subnormal),
        ("maximum-finite-difference", 0.5, -maximum_finite, maximum_finite),
        ("infinite-alpha", f64::INFINITY, 0.0, 1.0),
        ("nan-alpha", f64::NAN, 0.0, 1.0),
        ("infinite-endpoint", 0.5, 0.0, f64::INFINITY),
        ("nan-endpoint", 0.5, f64::NAN, 1.0),
    ]
}

fn inverse_lerp_cases(width: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for (label, value, minimum, maximum) in inverse_lerp_rows(width) {
        out.push(case(
            format!("mth/inverse-lerp/{width}/{label}"),
            "math",
            "mth_inverse_lerp",
            json!({
                "type": width,
                "value": float_bits(value, width),
                "min": float_bits(minimum, width),
                "max": float_bits(maximum, width),
            }),
        ));
    }
    out
}

fn lerp_cases(width: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for (label, alpha, p0, p1) in lerp_rows(width) {
        out.push(case(
            format!("mth/lerp/{width}/{label}"),
            "math",
            "mth_lerp",
            json!({
                "type": width,
                "alpha": float_bits(alpha, width),
                "p0": float_bits(p0, width),
                "p1": float_bits(p1, width),
            }),
        ));
    }
    out
}

fn section_inputs() -> Vec<Value> {
    let mut out = Vec::new();
    for width in ["f32", "f64"] {
        out.extend(inverse_lerp_cases(width));
        out.extend(lerp_cases(width));
    }
    out
}

fn mth() -> Vec<Value> {
    let mut out = Vec::new();
    for (operation, path_name) in [("mth_lerp_int", "lerp-int"), ("mth_lerp_discrete", "lerp-discrete")] {
        for (label, alpha, p0, p1) in mth_integer_lerp().iter().copied() {
            out.push(case(
                format!("mth/{path_name}/{label}"),
                "math",
                operation,
                json!({"alpha": f32_bits(alpha), "p0": (p0).to_string(), "p1": (p1).to_string()}),
            ));
        }
    }
    for (operation, path_name) in
        [("mth_smoothstep", "smoothstep"), ("mth_smoothstep_derivative", "smoothstep-derivative")]
    {
        for (label, value) in mth_smoothstep().iter().copied() {
            out.push(case(format!("mth/{path_name}/{label}"), "math", operation, json!({"x": f64_bits(value)})));
        }
    }
    out
}

fn rot_lerp() -> Vec<Value> {
    let mut out = Vec::new();
    for width in ["f32", "f64"] {
        let subnormal = if width == "f32" { 1.401298464324817e-45 } else { 5e-324 };
        let maximum_finite = if width == "f32" { 3.4028234663852886e+38 } else { 1.7976931348623157e+308 };
        let mth_rot_lerp = vec![
            ("basic", 0.25, 10.0, 50.0),
            ("zero-alpha", -0.0, 45.0, 90.0),
            ("one-alpha", 1.0, 45.0, 90.0),
            ("negative-alpha", -0.5, 10.0, 50.0),
            ("over-one-alpha", 1.5, 10.0, 50.0),
            ("positive-crossing", 0.5, 170.0, -170.0),
            ("negative-crossing", 0.5, -170.0, 170.0),
            ("positive-half-turn", 1.0, 0.0, 180.0),
            ("negative-half-turn", 1.0, 0.0, -180.0),
            ("full-turn", 0.5, 12.0, 372.0),
            ("signed-zero", 0.5, -0.0, 0.0),
            ("subnormal-difference", 1.0, 0.0, subnormal),
            ("maximum-finite", 0.5, -maximum_finite, maximum_finite),
            ("infinite-to", 0.5, 0.0, f64::INFINITY),
            ("nan-to", 0.5, 0.0, f64::NAN),
        ];
        for (label, alpha, from_value, to_value) in mth_rot_lerp.iter().copied() {
            out.push(case(
                format!("mth/rot-lerp/{width}/{label}"),
                "math",
                "mth_rot_lerp",
                json!({
                    "type": width,
                    "alpha": float_bits(alpha, width),
                    "from": float_bits(from_value, width),
                    "to": float_bits(to_value, width),
                }),
            ));
        }
    }
    for (label, alpha, from_value, to_value) in mth_rot_lerp_rad().iter().copied() {
        out.push(case(
            format!("mth/rot-lerp-rad/{label}"),
            "math",
            "mth_rot_lerp_rad",
            json!({"alpha": f32_bits(alpha), "from": f32_bits(from_value), "to": f32_bits(to_value)}),
        ));
    }
    out
}

fn triangle_wave() -> Vec<Value> {
    let mut out = Vec::new();
    for (label, index, period) in mth_triangle_wave().iter().copied() {
        out.push(case(
            format!("mth/triangle-wave/{label}"),
            "math",
            "mth_triangle_wave",
            json!({"index": f32_bits(index), "period": f32_bits(period)}),
        ));
    }
    out
}

fn clamped_map_f64() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_clamped_map_f64 = mth_clamped_map_f64_rows();
    for (label, value, from_min, from_max, to_min, to_max) in mth_clamped_map_f64.iter().copied() {
        out.push(case(
            format!("mth/clamped-map-f64/{label}"),
            "math",
            "mth_clamped_map_f64",
            json!({
                "value": f64_bits(value),
                "from_min": f64_bits(from_min),
                "from_max": f64_bits(from_max),
                "to_min": f64_bits(to_min),
                "to_max": f64_bits(to_max),
            }),
        ));
    }
    out
}

fn mth_clamped_map_f64_rows() -> Vec<(&'static str, f64, f64, f64, f64, f64)> {
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
        ("source-subnormal", MIN_SUBNORMAL_F64, 0.0, MIN_SUBNORMAL_F64, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, MIN_SUBNORMAL_F64),
        ("precision-order", 0.1, 1e+20, 1.00000000000001e+20, -1e+20, 1e+20),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, f64::NEG_INFINITY, f64::INFINITY, 0.0, 1.0),
        ("positive-infinity-value", f64::INFINITY, 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", f64::NEG_INFINITY, 0.0, 1.0, 0.0, 1.0),
        ("nan-value", f64::NAN, 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, f64::NAN, 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -MAXIMUM_FINITE_F64, MAXIMUM_FINITE_F64),
    ]
}

fn clamped_map_f32() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_clamped_map_f32 = mth_clamped_map_f32_rows();
    for (label, value, from_min, from_max, to_min, to_max) in mth_clamped_map_f32.iter().copied() {
        out.push(case(
            format!("mth/clamped-map-f32/{label}"),
            "math",
            "mth_clamped_map_f32",
            json!({
                "value": f32_bits(value),
                "from_min": f32_bits(from_min),
                "from_max": f32_bits(from_max),
                "to_min": f32_bits(to_min),
                "to_max": f32_bits(to_max),
            }),
        ));
    }
    out
}

fn mth_clamped_map_f32_rows() -> Vec<(&'static str, f64, f64, f64, f64, f64)> {
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

fn map_f64() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_map_f64 = mth_map_f64_rows();
    for (label, value, from_min, from_max, to_min, to_max) in mth_map_f64.iter().copied() {
        out.push(case(
            format!("mth/map-f64/{label}"),
            "math",
            "mth_map_f64",
            json!({
                "value": f64_bits(value),
                "from_min": f64_bits(from_min),
                "from_max": f64_bits(from_max),
                "to_min": f64_bits(to_min),
                "to_max": f64_bits(to_max),
            }),
        ));
    }
    out
}

fn mth_map_f64_rows() -> Vec<(&'static str, f64, f64, f64, f64, f64)> {
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
        ("source-subnormal", MIN_SUBNORMAL_F64, 0.0, MIN_SUBNORMAL_F64, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, MIN_SUBNORMAL_F64),
        ("precision-order", 0.1, 1e+20, 1.00000000000001e+20, -1e+20, 1e+20),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, f64::NEG_INFINITY, f64::INFINITY, 0.0, 1.0),
        ("positive-infinity-value", f64::INFINITY, 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", f64::NEG_INFINITY, 0.0, 1.0, 0.0, 1.0),
        ("nan-value", f64::NAN, 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, f64::NAN, 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -MAXIMUM_FINITE_F64, MAXIMUM_FINITE_F64),
    ]
}

pub fn cases() -> Vec<Value> {
    [
        clamped_lerp(),
        section_inputs(),
        mth(),
        rot_lerp(),
        triangle_wave(),
        clamped_map_f64(),
        clamped_map_f32(),
        map_f64(),
    ]
    .concat()
}
