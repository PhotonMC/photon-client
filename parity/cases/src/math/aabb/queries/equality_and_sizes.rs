//! Cases for `aabb/equality`, `aabb/to-string`, `aabb/sizes`, `aabb/has-nan` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn infinities() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/equality/infinities",
        "math",
        "aabb",
        merged(vec![
            box_bits(
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::INFINITY,
                f64::INFINITY,
            ),
            json!({
                "steps": [
                    {
                        "op": "equals",
                        "box": box_bits(
                            f64::NEG_INFINITY,
                            f64::NEG_INFINITY,
                            f64::NEG_INFINITY,
                            f64::INFINITY,
                            f64::INFINITY,
                            f64::INFINITY,
                        ),
                    },
                    {"op": "equals", "box": box_bits(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0)},
                    {"op": "hash_code"},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/to-string/basic",
        "math",
        "aabb",
        merged(vec![box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875), json!({"steps": [{"op": "to_string"}]})]),
    ));
    out
}

fn scientific() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/to-string/scientific",
        "math",
        "aabb",
        merged(vec![
            box_bits(0.0001, 10000000.0, -0.0, 0.0002, 10000001.0, 2.0),
            json!({"steps": [{"op": "to_string"}]}),
        ]),
    ));
    out.push(case(
        "aabb/to-string/signed-zero",
        "math",
        "aabb",
        merged(vec![box_bits(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0), json!({"steps": [{"op": "to_string"}]})]),
    ));
    out.push(case(
        "aabb/to-string/infinity",
        "math",
        "aabb",
        merged(vec![
            box_bits(
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::INFINITY,
                f64::INFINITY,
            ),
            json!({"steps": [{"op": "to_string"}]}),
        ]),
    ));
    out.push(case(
        "aabb/to-string/nan",
        "math",
        "aabb",
        merged(vec![box_bits(f64::NAN, 0.0, 0.0, 1.0, 1.0, 1.0), json!({"steps": [{"op": "to_string"}]})]),
    ));
    out.push(case("aabb/sizes/unit", "math", "aabb", merged(vec![unit(), json!({"steps": [{"op": "sizes"}]})])));
    out
}

fn rectangular() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/sizes/rectangular",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-2.0),
            "min_y": f64_bits(0.5),
            "min_z": f64_bits(10.0),
            "max_x": f64_bits(1.0),
            "max_y": f64_bits(4.5),
            "max_z": f64_bits(12.5),
            "steps": [{"op": "sizes"}],
        }),
    ));
    out.push(case(
        "aabb/sizes/reversed-normalizes",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(3.0),
            "min_y": f64_bits(2.0),
            "min_z": f64_bits(1.0),
            "max_x": f64_bits(-1.0),
            "max_y": f64_bits(-2.0),
            "max_z": f64_bits(-3.0),
            "steps": [{"op": "sizes"}],
        }),
    ));
    out.push(case(
        "aabb/sizes/zero",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(1.5),
            "min_y": f64_bits(-2.0),
            "min_z": f64_bits(4.0),
            "max_x": f64_bits(1.5),
            "max_y": f64_bits(-2.0),
            "max_z": f64_bits(4.0),
            "steps": [{"op": "sizes"}],
        }),
    ));
    out
}

fn signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/sizes/signed-zero",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-0.0),
            "min_y": f64_bits(-0.0),
            "min_z": f64_bits(-0.0),
            "max_x": f64_bits(0.0),
            "max_y": f64_bits(0.0),
            "max_z": f64_bits(0.0),
            "steps": [{"op": "sizes"}],
        }),
    ));
    out.push(case(
        "aabb/sizes/large-sum-rounding",
        "math",
        "aabb",
        merged(vec![unit(), json!({"max_z": f64_bits(9007199254740992.0), "steps": [{"op": "sizes"}]})]),
    ));
    out.push(case("aabb/has-nan/finite", "math", "aabb", merged(vec![unit(), json!({"steps": [{"op": "has_nan"}]})])));
    out.push(case(
        "aabb/has-nan/x-min",
        "math",
        "aabb",
        merged(vec![unit(), json!({"min_x": f64_bits(f64::NAN), "steps": [{"op": "has_nan"}]})]),
    ));
    out.push(case(
        "aabb/has-nan/y-max",
        "math",
        "aabb",
        merged(vec![unit(), json!({"max_y": f64_bits(f64::NAN), "steps": [{"op": "has_nan"}]})]),
    ));
    out.push(case(
        "aabb/has-nan/z-both",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"min_z": f64_bits(f64::NAN), "max_z": f64_bits(f64::NAN), "steps": [{"op": "has_nan"}]}),
        ]),
    ));
    out
}

fn positive_infinity() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/has-nan/positive-infinity",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"min_x": f64_bits(f64::INFINITY), "max_x": f64_bits(f64::INFINITY), "steps": [{"op": "has_nan"}]}),
        ]),
    ));
    out.push(case(
        "aabb/has-nan/negative-infinity",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "min_y": f64_bits(f64::NEG_INFINITY),
                "max_y": f64_bits(f64::NEG_INFINITY),
                "steps": [{"op": "has_nan"}],
            }),
        ]),
    ));
    out.push(case(
        "aabb/positions/unit",
        "math",
        "aabb",
        merged(vec![unit(), json!({"steps": [{"op": "positions"}]})]),
    ));
    out.push(case(
        "aabb/positions/reversed-normalizes",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(3.0),
            "min_y": f64_bits(2.0),
            "min_z": f64_bits(1.0),
            "max_x": f64_bits(-1.0),
            "max_y": f64_bits(-2.0),
            "max_z": f64_bits(-3.0),
            "steps": [{"op": "positions"}],
        }),
    ));
    out
}

fn asymmetric() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/positions/asymmetric",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-7.25),
            "min_y": f64_bits(0.125),
            "min_z": f64_bits(10.5),
            "max_x": f64_bits(2.75),
            "max_y": f64_bits(8.875),
            "max_z": f64_bits(11.25),
            "steps": [{"op": "positions"}],
        }),
    ));
    out.push(case(
        "aabb/positions/signed-zero",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-0.0),
            "min_y": f64_bits(-0.0),
            "min_z": f64_bits(-0.0),
            "max_x": f64_bits(0.0),
            "max_y": f64_bits(0.0),
            "max_z": f64_bits(0.0),
            "steps": [{"op": "positions"}],
        }),
    ));
    out.push(case(
        "aabb/positions/infinite",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(f64::NEG_INFINITY),
            "min_y": f64_bits(f64::NEG_INFINITY),
            "min_z": f64_bits(f64::NEG_INFINITY),
            "max_x": f64_bits(f64::INFINITY),
            "max_y": f64_bits(f64::INFINITY),
            "max_z": f64_bits(f64::INFINITY),
            "steps": [{"op": "positions"}],
        }),
    ));
    out
}

fn after_move() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/positions/after-move",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "positions"},
                    {"op": "move", "x": f64_bits(1.25), "y": f64_bits(-0.5), "z": f64_bits(2.0)},
                    {"op": "positions"},
                ],
            }),
        ]),
    ));
    let axis_box = box_bits(-4.0, -3.0, -2.0, 4.0, 3.0, 2.0);
    out.push(case(
        "aabb/axis-limit/min-all",
        "math",
        "aabb",
        merged(vec![
            axis_box.clone(),
            json!({"steps": [{"op": "min", "axis": "x"}, {"op": "min", "axis": "y"}, {"op": "min", "axis": "z"}]}),
        ]),
    ));
    out.push(case(
        "aabb/axis-limit/max-all",
        "math",
        "aabb",
        merged(vec![
            axis_box.clone(),
            json!({"steps": [{"op": "max", "axis": "x"}, {"op": "max", "axis": "y"}, {"op": "max", "axis": "z"}]}),
        ]),
    ));
    out
}

fn mixed() -> Vec<Value> {
    let mut out = Vec::new();
    let axis_box = box_bits(-4.0, -3.0, -2.0, 4.0, 3.0, 2.0);
    out.push(case(
        "aabb/axis-limit/mixed",
        "math",
        "aabb",
        merged(vec![
            axis_box.clone(),
            json!({
                "steps": [
                    {"op": "min", "axis": "z"},
                    {"op": "max", "axis": "x"},
                    {"op": "min", "axis": "y"},
                    {"op": "max", "axis": "z"},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/axis-limit/reversed-normalizes",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(4.0),
            "min_y": f64_bits(3.0),
            "min_z": f64_bits(2.0),
            "max_x": f64_bits(-4.0),
            "max_y": f64_bits(-3.0),
            "max_z": f64_bits(-2.0),
            "steps": [{"op": "min", "axis": "x"}, {"op": "max", "axis": "y"}],
        }),
    ));
    out
}

fn axis_limit_signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/axis-limit/signed-zero",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-0.0),
            "min_y": f64_bits(-0.0),
            "min_z": f64_bits(-0.0),
            "max_x": f64_bits(0.0),
            "max_y": f64_bits(0.0),
            "max_z": f64_bits(0.0),
            "steps": [
                {"op": "min", "axis": "x"},
                {"op": "max", "axis": "y"},
                {"op": "min", "axis": "z"},
                {"op": "max", "axis": "x"},
            ],
        }),
    ));
    out.push(case(
        "aabb/axis-limit/nan",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "min_x": f64_bits(f64::NAN),
                "steps": [{"op": "min", "axis": "x"}, {"op": "max", "axis": "x"}, {"op": "min", "axis": "y"}],
            }),
        ]),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        infinities(),
        scientific(),
        rectangular(),
        signed_zero(),
        positive_infinity(),
        asymmetric(),
        after_move(),
        mixed(),
        axis_limit_signed_zero(),
    ]
    .concat()
}
