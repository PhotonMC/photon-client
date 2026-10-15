//! Cases for `direction/approximate-nearest-overloads`, `direction/nearest`, `direction/axis-step-order`, `direction/is-facing-angle` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::values::*;

fn approximate_nearest_overloads() -> Vec<Value> {
    let mut out = Vec::new();
    let direction_approximate_nearest_overload_cases = [
        ("cardinal", approximate_nearest_overloads_cardinal()),
        ("tie", approximate_nearest_overloads_tie()),
        ("zero", approximate_nearest_overloads_zero()),
        ("negative", approximate_nearest_overloads_negative()),
        ("f32-nan", approximate_nearest_overloads_f32_nan()),
        ("f32-infinity", approximate_nearest_overloads_f32_infinity()),
        ("precision-boundary", approximate_nearest_overloads_precision()),
        ("signed-zero", approximate_nearest_overloads_signed_zero()),
    ];
    for (name, values) in direction_approximate_nearest_overload_cases.iter() {
        out.push(case(
            format!("direction/approximate-nearest-overloads/{}", name),
            "math",
            "direction_approximate_nearest_overloads",
            values.clone(),
        ));
    }
    let direction_nearest_cases = vec![
        ("components/x-positive", "components", 3, 1, 2, "none"),
        ("components/x-negative", "components", -3, 1, 2, "none"),
        ("components/z-positive", "components", 1, 2, 4, "none"),
        ("components/z-negative", "components", 1, 2, -4, "none"),
        ("components/y-positive", "components", 1, 5, 2, "none"),
        ("components/y-negative", "components", 1, -5, 2, "none"),
        ("components/tie-x-z", "components", 4, 1, 4, "west"),
        ("components/tie-x-y", "components", 4, 4, 1, "south"),
        ("components/zero-none", "components", 0, 0, 0, "none"),
        ("components/zero-fallback", "components", 0, 0, 0, "north"),
        ("components/min-x", "components", -2147483648, 0, 0, "none"),
        ("components/min-y", "components", 0, -2147483648, 0, "east"),
        ("vec3i/x-positive", "vec3i", 3, 1, 2, "none"),
        ("vec3i/z-negative", "vec3i", 1, 2, -4, "none"),
        ("vec3i/tie", "vec3i", 2, 2, 0, "down"),
        ("vec3i/zero-none", "vec3i", 0, 0, 0, "none"),
        ("vec3i/min-z", "vec3i", 0, 0, -2147483648, "up"),
    ];
    for (name, kind, x, y, z, or_else) in direction_nearest_cases.iter() {
        out.push(case(
            format!("direction/nearest/{}", name),
            "math",
            "direction_nearest",
            json!({
                "kind": kind,
                "x": (x).to_string(),
                "y": (y).to_string(),
                "z": (z).to_string(),
                "or_else": or_else,
            }),
        ));
    }
    out
}

fn approximate_nearest_overloads_signed_zero() -> Value {
    json!({
        "f32_x": "80000000",
        "f32_y": "00000000",
        "f32_z": "00000000",
        "x": "8000000000000000",
        "y": "0000000000000000",
        "z": "0000000000000000",
    })
}

fn approximate_nearest_overloads_precision() -> Value {
    json!({
        "f32_x": "3f800000",
        "f32_y": f32_bits(0.0),
        "f32_z": f32_bits(0.0),
        "x": "3ff0000010000000",
        "y": f64_bits(0.0),
        "z": f64_bits(0.0),
    })
}

fn approximate_nearest_overloads_f32_infinity() -> Value {
    json!({
        "f32_x": "7f800000",
        "f32_y": f32_bits(0.0),
        "f32_z": f32_bits(0.0),
        "x": "7ff0000000000000",
        "y": f64_bits(0.0),
        "z": f64_bits(0.0),
    })
}

fn approximate_nearest_overloads_f32_nan() -> Value {
    json!({
        "f32_x": "7fc00001",
        "f32_y": f32_bits(0.0),
        "f32_z": f32_bits(0.0),
        "x": "7ff8000000000001",
        "y": f64_bits(0.0),
        "z": f64_bits(0.0),
    })
}

fn approximate_nearest_overloads_negative() -> Value {
    json!({
        "f32_x": f32_bits(-1.0),
        "f32_y": f32_bits(-2.0),
        "f32_z": f32_bits(-3.0),
        "x": f64_bits(-1.0),
        "y": f64_bits(-2.0),
        "z": f64_bits(-3.0),
    })
}

fn approximate_nearest_overloads_zero() -> Value {
    json!({
        "f32_x": f32_bits(0.0),
        "f32_y": f32_bits(0.0),
        "f32_z": f32_bits(0.0),
        "x": f64_bits(0.0),
        "y": f64_bits(0.0),
        "z": f64_bits(0.0),
    })
}

fn approximate_nearest_overloads_tie() -> Value {
    json!({
        "f32_x": f32_bits(1.0),
        "f32_y": f32_bits(0.0),
        "f32_z": f32_bits(1.0),
        "x": f64_bits(1.0),
        "y": f64_bits(0.0),
        "z": f64_bits(1.0),
    })
}

fn approximate_nearest_overloads_cardinal() -> Value {
    json!({
        "f32_x": f32_bits(0.0),
        "f32_y": f32_bits(0.0),
        "f32_z": f32_bits(1.0),
        "x": f64_bits(0.0),
        "y": f64_bits(0.0),
        "z": f64_bits(1.0),
    })
}

fn axis_step_order() -> Vec<Value> {
    let mut out = Vec::new();
    let direction_axis_step_values = vec![
        ("x-smaller", vector_bits(1.0, 0.0, 2.0)),
        ("z-smaller", vector_bits(2.0, 0.0, 1.0)),
        ("tie", vector_bits(2.0, 0.0, 2.0)),
        ("negative-x-smaller", vector_bits(-1.0, 0.0, 2.0)),
        ("negative-both", vector_bits(-2.0, 0.0, -3.0)),
        ("zero", vector_bits(0.0, 0.0, 0.0)),
        ("signed-zero", json!({"x": "8000000000000000", "y": "0000000000000000", "z": "0000000000000000"})),
        ("subnormal-x", json!({"x": "0000000000000001", "y": "0000000000000000", "z": "0000000000000002"})),
        ("positive-infinity-x", json!({"x": "7ff0000000000000", "y": "0000000000000000", "z": f64_bits(1.0)})),
        ("positive-infinity-z", json!({"x": f64_bits(1.0), "y": "0000000000000000", "z": "7ff0000000000000"})),
        ("both-infinity", json!({"x": "7ff0000000000000", "y": "0000000000000000", "z": "7ff0000000000000"})),
        ("nan-x", json!({"x": "7ff8000000000001", "y": "0000000000000000", "z": f64_bits(1.0)})),
        ("nan-z", json!({"x": f64_bits(1.0), "y": "0000000000000000", "z": "7ff8000000000001"})),
    ];
    for (name, vector) in direction_axis_step_values.iter() {
        out.push(case(
            format!("direction/axis-step-order/{}", name),
            "math",
            "direction_axis_step_order",
            vector.clone(),
        ));
    }
    out
}

fn is_facing_angle() -> Vec<Value> {
    let mut out = Vec::new();
    let direction_facing_angle_cases = vec![
        ("south-zero", "south", f32_bits(0.0)),
        ("south-quarter", "south", f32_bits(90.0)),
        ("south-half-turn", "south", f32_bits(180.0)),
        ("south-three-quarter", "south", f32_bits(270.0)),
        ("north-zero", "north", f32_bits(0.0)),
        ("north-half-turn", "north", f32_bits(180.0)),
        ("east-zero", "east", f32_bits(0.0)),
        ("east-negative-quarter", "east", f32_bits(-90.0)),
        ("west-zero", "west", f32_bits(0.0)),
        ("west-quarter", "west", f32_bits(90.0)),
        ("up-zero", "up", f32_bits(0.0)),
        ("down-negative-quarter", "down", f32_bits(-90.0)),
        ("south-before-quarter", "south", f32_bits(89.0)),
        ("south-after-quarter", "south", f32_bits(91.0)),
        ("west-diagonal", "west", f32_bits(45.0)),
        ("east-diagonal", "east", f32_bits(-45.0)),
        ("negative-angle", "south", f32_bits(-45.0)),
        ("full-turn", "south", f32_bits(360.0)),
        ("signed-zero", "south", "80000000".to_string()),
        ("positive-subnormal", "south", "00000001".to_string()),
        ("negative-subnormal", "south", "80000001".to_string()),
        ("positive-infinity", "south", "7f800000".to_string()),
        ("negative-infinity", "south", "ff800000".to_string()),
        ("nan", "south", "7fc00001".to_string()),
        ("max-finite", "east", "7f7fffff".to_string()),
        ("min-negative-finite", "west", "ff7fffff".to_string()),
    ];
    for (name, direction, angle) in direction_facing_angle_cases.iter() {
        out.push(case(
            format!("direction/is-facing-angle/{}", name),
            "math",
            "direction_is_facing_angle",
            json!({"name": direction, "angle": angle.clone()}),
        ));
    }
    out
}

fn helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let axis_cycle_helpers = [
        (
            "basic",
            json!({
                "x": "10",
                "y": "-20",
                "z": "30",
                "double_x": f64_bits(1.25),
                "double_y": f64_bits(-2.5),
                "double_z": f64_bits(3.75),
            }),
        ),
        (
            "boundaries",
            json!({
                "x": "2147483647",
                "y": "-2147483648",
                "z": "-1",
                "double_x": "8000000000000000",
                "double_y": "7ff8000000000002",
                "double_z": "7ff0000000000000",
            }),
        ),
    ];
    for (label, values) in axis_cycle_helpers.iter() {
        out.push(case(format!("axis-cycle/helpers/{}", label), "math", "axis_cycle_helpers", values.clone()));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [approximate_nearest_overloads(), axis_step_order(), is_facing_angle(), helpers()].concat()
}
