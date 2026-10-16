//! Cases for `mth/angle-wrap-difference`, `mth/angle-approach`, `mth/atan2`, `mth/angle-helpers` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::mth::tables::*;
use crate::math::values::*;

fn angle_wrap_difference_wrap_degrees_type() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_wrap_degrees_90 = vec![
        ("zero", "00000000".to_string()),
        ("negative-zero", "80000000".to_string()),
        ("positive-boundary-below", "4233ffff".to_string()),
        ("positive-boundary", "42340000".to_string()),
        ("positive-boundary-above", "42340001".to_string()),
        ("negative-boundary-above", "c233ffff".to_string()),
        ("negative-boundary", "c2340000".to_string()),
        ("negative-boundary-below", "c2340001".to_string()),
        ("positive-quarter-turn", f32_bits(90.0)),
        ("negative-quarter-turn", f32_bits(-90.0)),
        ("positive-half-turn", f32_bits(180.0)),
        ("negative-half-turn", f32_bits(-180.0)),
        ("wrapped-positive", f32_bits(360.25)),
        ("wrapped-negative", f32_bits(-360.25)),
        ("positive-subnormal", "00000001".to_string()),
        ("negative-subnormal", "80000001".to_string()),
        ("positive-infinity", "7f800000".to_string()),
        ("nan-payload", "7fc12345".to_string()),
    ];
    for (label, value) in &mth_wrap_degrees_90 {
        out.push(case(
            format!("mth/angle-wrap-difference/wrap-degrees-90/{label}"),
            "math",
            "mth_angle_wrap_difference",
            json!({"type": "f32", "fn": "wrap_degrees_90", "value": value.clone()}),
        ));
    }
    out
}

fn degrees_difference() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_degrees_difference = vec![
        ("same", f32_bits(10.0), f32_bits(10.0)),
        ("signed-zero", "80000000".to_string(), "00000000".to_string()),
        ("simple-positive", f32_bits(10.0), f32_bits(40.0)),
        ("simple-negative", f32_bits(40.0), f32_bits(10.0)),
        ("upper-bound", f32_bits(0.0), f32_bits(180.0)),
        ("lower-bound", f32_bits(0.0), f32_bits(-180.0)),
        ("cross-positive", f32_bits(170.0), f32_bits(-170.0)),
        ("cross-negative", f32_bits(-170.0), f32_bits(170.0)),
        ("whole-turn", f32_bits(10.0), f32_bits(370.0)),
        ("fractional", f32_bits(-12.5), f32_bits(289.25)),
        ("large-finite", f32_bits(10000000000.0), f32_bits(-10000000000.0)),
        ("positive-infinity", f32_bits(0.0), "7f800000".to_string()),
        ("negative-infinity", f32_bits(0.0), "ff800000".to_string()),
        ("nan-from", "7fc12345".to_string(), f32_bits(1.0)),
        ("nan-to", f32_bits(1.0), "7fc12345".to_string()),
        ("overflow", "ff7fffff".to_string(), "7f7fffff".to_string()),
    ];
    for (label, from_value, to_value) in &mth_degrees_difference {
        out.push(case(
            format!("mth/angle-wrap-difference/degrees-difference/{label}"),
            "math",
            "mth_angle_wrap_difference",
            json!({"type": "f32", "fn": "degrees_difference", "from": from_value.clone(), "to": to_value.clone()}),
        ));
    }
    out
}

fn degrees_difference_abs() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_degrees_difference_abs = mth_degrees_difference_abs_rows();
    for (label, from_value, to_value) in &mth_degrees_difference_abs {
        out.push(case(
            format!("mth/angle-wrap-difference/degrees-difference-abs/{label}"),
            "math",
            "mth_angle_wrap_difference",
            json!({"type": "f32", "fn": "degrees_difference_abs", "from": from_value.clone(), "to": to_value.clone()}),
        ));
    }
    let mth_rotate_if_necessary = mth_rotate_if_necessary_rows();
    for (label, base, target, maximum) in &mth_rotate_if_necessary {
        let base = base.f32();
        let target = target.f32();
        let maximum = maximum.f32();
        out.push(case(
            format!("mth/angle-approach/rotate-if-necessary/{label}"),
            "math",
            "mth_angle_approach",
            json!({
                "type": "f32",
                "fn": "rotate_if_necessary",
                "base": base.clone(),
                "target": target.clone(),
                "max": maximum.clone(),
            }),
        ));
    }
    out
}

fn mth_degrees_difference_abs_rows() -> Vec<(&'static str, std::string::String, std::string::String)> {
    vec![
        ("zero", f32_bits(0.0), f32_bits(0.0)),
        ("negative-zero", "80000000".to_string(), "00000000".to_string()),
        ("simple-positive", f32_bits(10.0), f32_bits(40.0)),
        ("simple-negative", f32_bits(40.0), f32_bits(10.0)),
        ("upper-bound", f32_bits(0.0), f32_bits(180.0)),
        ("lower-bound", f32_bits(0.0), f32_bits(-180.0)),
        ("cross-positive", f32_bits(170.0), f32_bits(-170.0)),
        ("cross-negative", f32_bits(-170.0), f32_bits(170.0)),
        ("whole-turn", f32_bits(10.0), f32_bits(370.0)),
        ("fractional", f32_bits(-12.5), f32_bits(289.25)),
        ("negative-result-zero", "80000000".to_string(), "80000000".to_string()),
        ("positive-infinity", f32_bits(0.0), "7f800000".to_string()),
        ("nan-from", "7fc12345".to_string(), f32_bits(1.0)),
        ("nan-to", f32_bits(1.0), "7fc12345".to_string()),
        ("overflow", "ff7fffff".to_string(), "7f7fffff".to_string()),
        ("subnormal", "00000001".to_string(), "80000001".to_string()),
    ]
}

fn mth_rotate_if_necessary_rows() -> Vec<(&'static str, Bits, Bits, Bits)> {
    vec![
        ("same", num(10.0), num(10.0), num(5.0)),
        ("positive-under", num(10.0), num(12.0), num(5.0)),
        ("positive-boundary", num(10.0), num(15.0), num(5.0)),
        ("positive-over", num(10.0), num(20.0), num(5.0)),
        ("negative-under", num(20.0), num(18.0), num(5.0)),
        ("negative-boundary", num(20.0), num(15.0), num(5.0)),
        ("negative-over", num(20.0), num(10.0), num(5.0)),
        ("cross-positive", num(170.0), num(-170.0), num(30.0)),
        ("cross-negative", num(-170.0), num(170.0), num(30.0)),
        ("upper-bound", num(0.0), num(180.0), num(45.0)),
        ("lower-bound", num(0.0), num(-180.0), num(45.0)),
        ("zero-limit", num(10.0), num(80.0), num(0.0)),
        ("large-limit", num(10.0), num(80.0), num(100.0)),
        ("negative-limit", num(10.0), num(20.0), num(-5.0)),
        ("signed-zero", raw("80000000"), raw("00000000"), num(1.0)),
        ("positive-infinity-target", num(0.0), raw("7f800000"), num(30.0)),
        ("nan-base", raw("7fc12345"), num(0.0), num(30.0)),
        ("overflow", raw("7f7fffff"), raw("ff7fffff"), raw("7f7fffff")),
    ]
}

fn approach() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_approach = vec![
        ("same", num(10.0), num(10.0), num(2.0)),
        ("below", num(0.0), num(10.0), num(2.0)),
        ("exact", num(0.0), num(10.0), num(10.0)),
        ("over", num(0.0), num(10.0), num(20.0)),
        ("above", num(10.0), num(0.0), num(2.0)),
        ("negative-increment", num(0.0), num(10.0), num(-2.0)),
        ("zero-increment", num(0.0), num(10.0), num(0.0)),
        ("signed-zero", raw("80000000"), raw("00000000"), num(1.0)),
        ("positive-subnormal", num(0.0), raw("00000001"), num(1.0)),
        ("negative-subnormal", raw("80000001"), num(0.0), num(1.0)),
        ("max-finite-current", raw("7f7fffff"), num(0.0), num(1.0)),
        ("infinity-increment", num(0.0), num(10.0), raw("7f800000")),
        ("infinity-current", raw("7f800000"), num(10.0), num(1.0)),
        ("infinity-target", num(0.0), raw("7f800000"), num(1.0)),
        ("nan-current", raw("7fc12345"), num(0.0), num(1.0)),
        ("nan-target", num(0.0), raw("7fc12345"), num(1.0)),
        ("nan-increment", num(0.0), num(10.0), raw("7fc12345")),
        ("rounding", raw("3f800001"), raw("3f800101"), raw("33800000")),
    ];
    for (label, current, target, increment) in &mth_approach {
        let current = current.f32();
        let target = target.f32();
        let increment = increment.f32();
        out.push(case(
            format!("mth/angle-approach/approach/{label}"),
            "math",
            "mth_angle_approach",
            json!({
                "type": "f32",
                "fn": "approach",
                "current": current.clone(),
                "target": target.clone(),
                "increment": increment.clone(),
            }),
        ));
    }
    out
}

fn approach_degrees() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_approach_degrees = vec![
        ("same", num(10.0), num(10.0), num(5.0)),
        ("simple-positive", num(10.0), num(40.0), num(5.0)),
        ("simple-negative", num(40.0), num(10.0), num(5.0)),
        ("cross-positive", num(170.0), num(-170.0), num(5.0)),
        ("cross-negative", num(-170.0), num(170.0), num(5.0)),
        ("upper-bound", num(0.0), num(180.0), num(10.0)),
        ("lower-bound", num(0.0), num(-180.0), num(10.0)),
        ("whole-turn", num(10.0), num(370.0), num(5.0)),
        ("fractional", num(-12.5), num(289.25), num(7.0)),
        ("negative-increment", num(0.0), num(90.0), num(-10.0)),
        ("zero-increment", num(0.0), num(90.0), num(0.0)),
        ("signed-zero", raw("80000000"), raw("00000000"), num(1.0)),
        ("positive-subnormal", num(0.0), raw("00000001"), raw("00000001")),
        ("large-finite", num(10000000000.0), num(-10000000000.0), num(100000.0)),
        ("infinity-increment", num(0.0), num(90.0), raw("7f800000")),
        ("infinity-target", num(0.0), raw("7f800000"), num(1.0)),
        ("nan-current", raw("7fc12345"), num(0.0), num(1.0)),
        ("nan-target", num(0.0), raw("7fc12345"), num(1.0)),
    ];
    for (label, current, target, increment) in &mth_approach_degrees {
        let current = current.f32();
        let target = target.f32();
        let increment = increment.f32();
        out.push(case(
            format!("mth/angle-approach/approach-degrees/{label}"),
            "math",
            "mth_angle_approach",
            json!({
                "type": "f32",
                "fn": "approach_degrees",
                "current": current.clone(),
                "target": target.clone(),
                "increment": increment.clone(),
            }),
        ));
    }
    for (label, y, x) in mth_atan2().iter().cloned() {
        let y = y.f64();
        let x = f64_bits(x);
        out.push(case(format!("mth/atan2/{label}"), "math", "mth_atan2", json!({"y": y.clone(), "x": x})));
    }
    out
}

fn angle_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_angle_helpers = vec![
        ("pack/zero", "f32", "pack_degrees", f32_bits(0.0)),
        ("pack/negative-quarter", "f32", "pack_degrees", f32_bits(-90.0)),
        ("pack/negative-half", "f32", "pack_degrees", f32_bits(-180.0)),
        ("pack/quarter", "f32", "pack_degrees", f32_bits(90.0)),
        ("pack/half-byte-wrap", "f32", "pack_degrees", f32_bits(180.0)),
        ("pack/full-byte-wrap", "f32", "pack_degrees", f32_bits(360.0)),
        ("pack/near-full-turn", "f32", "pack_degrees", f32_bits(359.9)),
        ("pack/infinity", "f32", "pack_degrees", f32_bits(f64::INFINITY)),
        ("pack/nan", "f32", "pack_degrees", f32_bits(f64::NAN)),
        ("unpack/min", "i8", "unpack_degrees", "-128".to_string()),
        ("unpack/negative", "i8", "unpack_degrees", "-1".to_string()),
        ("unpack/zero", "i8", "unpack_degrees", "0".to_string()),
        ("unpack/positive", "i8", "unpack_degrees", "1".to_string()),
        ("unpack/max", "i8", "unpack_degrees", "127".to_string()),
        ("wrap-long/min", "i64", "wrap_degrees", "-9223372036854775808".to_string()),
        ("wrap-long/negative", "i64", "wrap_degrees", "-721".to_string()),
        ("wrap-long/exact-negative-turn", "i64", "wrap_degrees", "-360".to_string()),
        ("wrap-long/lower-bound", "i64", "wrap_degrees", "-180".to_string()),
        ("wrap-long/upper-bound", "i64", "wrap_degrees", "180".to_string()),
        ("wrap-long/exact-turn", "i64", "wrap_degrees", "360".to_string()),
        ("wrap-long/max", "i64", "wrap_degrees", "9223372036854775807".to_string()),
    ];
    for (label, width, function, input_value) in &mth_angle_helpers {
        out.push(case(
            format!("mth/angle-helpers/{label}"),
            "math",
            "mth_angle_helpers",
            json!({"type": width, "fn": function, "input": input_value.clone()}),
        ));
    }
    out
}

fn rotation_around_axis() -> Vec<Value> {
    let mut out = Vec::new();
    for (label, axis, rotation) in mth_rotation_around_axis().iter().copied() {
        out.push(case(
            format!("mth/rotation-around-axis/{label}"),
            "math",
            "mth_rotation_around_axis",
            json!({
                "axis_x": f32_bits(axis.0),
                "axis_y": f32_bits(axis.1),
                "axis_z": f32_bits(axis.2),
                "rotation_x": f32_bits(rotation.0),
                "rotation_y": f32_bits(rotation.1),
                "rotation_z": f32_bits(rotation.2),
                "rotation_w": f32_bits(rotation.3),
            }),
        ));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [
        angle_wrap_difference_wrap_degrees_type(),
        degrees_difference(),
        degrees_difference_abs(),
        approach(),
        approach_degrees(),
        angle_helpers(),
        rotation_around_axis(),
    ]
    .concat()
}
