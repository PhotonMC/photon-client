//! Cases for `vec3/vector-to`, `vec3/cross`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn overflow() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/vector-to/overflow",
        "math",
        "vec3",
        json!({
            "x": "7fefffffffffffff",
            "y": "8000000000000000",
            "z": "ffefffffffffffff",
            "steps": [
                {
                    "action": "vector_to",
                    "target": {"x": "ffefffffffffffff", "y": "7fefffffffffffff", "z": "7fefffffffffffff"},
                },
            ],
        }),
    ));
    out.push(case(
        "vec3/vector-to/special-values",
        "math",
        "vec3",
        json!({
            "x": f64_bits(1.0),
            "y": f64_bits(-2.0),
            "z": f64_bits(0.0),
            "steps": [
                {
                    "action": "vector_to",
                    "target": {
                        "x": f64_bits(f64::INFINITY),
                        "y": f64_bits(f64::NEG_INFINITY),
                        "z": "7ff8000000000002",
                    },
                },
            ],
        }),
    ));
    out
}

fn basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/cross/basic",
        "math",
        "vec3",
        json!({
            "x": f64_bits(1.0),
            "y": f64_bits(2.0),
            "z": f64_bits(3.0),
            "steps": [{"action": "cross", "other": vector_bits(4.0, 5.0, 6.0)}],
        }),
    ));
    out.push(case(
        "vec3/cross/chain",
        "math",
        "vec3",
        json!({
            "x": f64_bits(1.0),
            "y": f64_bits(2.0),
            "z": f64_bits(3.0),
            "steps": [
                {"action": "cross", "other": vector_bits(4.0, 5.0, 6.0)},
                {"action": "cross", "other": vector_bits(-1.0, 0.5, 2.0)},
                {"action": "normalize"},
            ],
        }),
    ));
    out.push(case(
        "vec3/cross/signed-zero",
        "math",
        "vec3",
        json!({
            "x": f64_bits(0.0),
            "y": f64_bits(-0.0),
            "z": f64_bits(0.0),
            "steps": [{"action": "cross", "other": vector_bits(0.0, 0.0, 0.0)}],
        }),
    ));
    out
}

fn cross_overflow() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/cross/overflow",
        "math",
        "vec3",
        json!({
            "x": "7fefffffffffffff",
            "y": f64_bits(0.0),
            "z": f64_bits(0.0),
            "steps": [{"action": "cross", "other": vector_bits(0.0, 0.0, 2.0)}],
        }),
    ));
    out.push(case(
        "vec3/cross/special-values",
        "math",
        "vec3",
        json!({
            "x": f64_bits(f64::INFINITY),
            "y": f64_bits(2.0),
            "z": f64_bits(3.0),
            "steps": [{"action": "cross", "other": vector_bits(0.0, 0.0, 5.0)}],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [overflow(), basic(), cross_overflow()].concat()
}
