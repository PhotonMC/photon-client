//! Cases for `blockpos/chain`, `blockpos/packed-helpers`, `blockpos/relative-helpers`, `blockpos/vector-helpers`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

fn chain() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_steps = [
        (
            "basic",
            vec![
                json!({"action": "offset", "x": "0", "y": "0", "z": "0"}),
                json!({"action": "offset", "x": "1", "y": "-2", "z": "3"}),
                json!({"action": "min", "x": "-4", "y": "5", "z": "2"}),
                json!({"action": "max", "x": "8", "y": "-9", "z": "10"}),
            ],
        ),
        (
            "negative-boundaries",
            vec![
                json!({"action": "offset", "x": "16", "y": "16", "z": "16"}),
                json!({"action": "min", "x": "-32", "y": "-15", "z": "-1"}),
                json!({"action": "max", "x": "-16", "y": "17", "z": "31"}),
            ],
        ),
        (
            "overflow",
            vec![
                json!({"action": "offset", "x": "1", "y": "-1", "z": "1"}),
                json!({"action": "offset", "x": "-2", "y": "2", "z": "-2"}),
            ],
        ),
        (
            "reversed-min-max",
            vec![
                json!({"action": "max", "x": "-20", "y": "30", "z": "-40"}),
                json!({"action": "min", "x": "20", "y": "-30", "z": "40"}),
            ],
        ),
    ];
    let blockpos_origins = vec![
        ("basic", (0, 0, 0)),
        ("negative-boundaries", (-17, -16, -1)),
        ("overflow", (2147483647, -2147483648, 2147483647)),
        ("reversed-min-max", (9, -4, 20)),
    ];
    for (label, steps) in blockpos_steps.iter().cloned() {
        let (x, y, z) = lookup(&blockpos_origins, label);
        out.push(case(
            format!("blockpos/chain/{label}"),
            "math",
            "blockpos",
            json!({"x": (x).to_string(), "y": (y).to_string(), "z": (z).to_string(), "steps": steps.clone()}),
        ));
    }
    out
}

fn packed_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_packed_helpers = [
        ("origin", packed_helpers_origin()),
        ("negative-fields", packed_helpers_negative_fields()),
        ("packed-boundaries", packed_helpers_packed_boundaries()),
        ("masked-overflow", packed_helpers_masked_overflow()),
        ("integer-boundaries", packed_helpers_integer_boundaries()),
    ];
    for (label, values) in &blockpos_packed_helpers {
        out.push(case(format!("blockpos/packed-helpers/{label}"), "math", "blockpos_packed_helpers", values.clone()));
    }
    let blockpos_relative_helpers = [
        ("basic", json!({"x": "10", "y": "20", "z": "30", "direction": "north", "axis": "x", "steps": "3"})),
        ("zero-steps", json!({"x": "-10", "y": "-20", "z": "-30", "direction": "up", "axis": "y", "steps": "0"})),
        ("negative-steps", json!({"x": "17", "y": "-16", "z": "1", "direction": "west", "axis": "z", "steps": "-2"})),
        ("overflow", packed_helpers_overflow()),
    ];
    for (label, values) in &blockpos_relative_helpers {
        out.push(case(
            format!("blockpos/relative-helpers/{label}"),
            "math",
            "blockpos_relative_helpers",
            values.clone(),
        ));
    }
    out
}

fn packed_helpers_integer_boundaries() -> Value {
    json!({
        "x": "2147483647",
        "y": "-2147483648",
        "z": "2147483647",
        "neighbor_node": "deadbeefcafebabe",
        "direction": "west",
        "step_x": "-2147483648",
        "step_y": "2147483647",
        "step_z": "0",
    })
}

fn packed_helpers_masked_overflow() -> Value {
    json!({
        "x": "33554432",
        "y": "2048",
        "z": "-33554433",
        "neighbor_node": "7fffffe0000007ff",
        "direction": "down",
        "step_x": "2147483647",
        "step_y": "-2147483648",
        "step_z": "-1",
    })
}

fn packed_helpers_packed_boundaries() -> Value {
    json!({
        "x": "33554431",
        "y": "2047",
        "z": "-33554432",
        "neighbor_node": "8000001ffffff800",
        "direction": "east",
        "step_x": "1",
        "step_y": "1",
        "step_z": "1",
    })
}

fn packed_helpers_negative_fields() -> Value {
    json!({
        "x": "-1",
        "y": "-2",
        "z": "-3",
        "neighbor_node": "ffffffffffffffff",
        "direction": "north",
        "step_x": "-4",
        "step_y": "5",
        "step_z": "-6",
    })
}

fn packed_helpers_origin() -> Value {
    json!({
        "x": "0",
        "y": "0",
        "z": "0",
        "neighbor_node": "0000000000000000",
        "direction": "up",
        "step_x": "1",
        "step_y": "-2",
        "step_z": "3",
    })
}

fn packed_helpers_overflow() -> Value {
    json!({
        "x": "2147483647",
        "y": "-2147483648",
        "z": "2147483647",
        "direction": "east",
        "axis": "y",
        "steps": "2147483647",
    })
}

fn vector_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_vector_helpers = [
        ("basic", json!({"x": "10", "y": "-20", "z": "30", "dx": "2", "dy": "3", "dz": "-4", "scale": "3"})),
        (
            "zero-delta-scale-one",
            json!({"x": "-10", "y": "20", "z": "-30", "dx": "0", "dy": "0", "dz": "0", "scale": "1"}),
        ),
        ("zero-scale", json!({"x": "17", "y": "-16", "z": "1", "dx": "-5", "dy": "6", "dz": "-7", "scale": "0"})),
        ("negative", json!({"x": "-17", "y": "16", "z": "-1", "dx": "5", "dy": "-6", "dz": "7", "scale": "-3"})),
        (
            "overflow",
            json!({
                "x": "2147483647",
                "y": "-2147483648",
                "z": "2147483647",
                "dx": "-2147483648",
                "dy": "2147483647",
                "dz": "-2147483648",
                "scale": "-2147483648",
            }),
        ),
    ];
    for (label, values) in &blockpos_vector_helpers {
        out.push(case(format!("blockpos/vector-helpers/{label}"), "math", "blockpos_vector_helpers", values.clone()));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [chain(), packed_helpers(), vector_helpers()].concat()
}
