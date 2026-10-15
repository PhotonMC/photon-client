//! Cases for `vec3/to-string`.

use serde_json::{Value, json};

use crate::case::case;

fn signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/to-string/signed-zero",
        "math",
        "vec3_to_string",
        json!({"first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"}}),
    ));
    out.push(case(
        "vec3/to-string/infinity",
        "math",
        "vec3_to_string",
        json!({"first": {"x": "fff0000000000000", "y": "7ff0000000000000", "z": "fff0000000000000"}}),
    ));
    out.push(case(
        "vec3/to-string/nan",
        "math",
        "vec3_to_string",
        json!({"first": {"x": "7ff8000000000001", "y": "7ff0000000000001", "z": "fff8000000000001"}}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [signed_zero()].concat()
}
