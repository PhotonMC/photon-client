//! Cases for `blockpos/mutable-helpers`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;

fn mutable_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_mutable_helpers = [
        ("all-overloads", mutable_helpers_all_overloads()),
        (
            "overflow-and-clamp",
            json!({
                "initial": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
                "double_x": f64_bits(2147483647.9),
                "double_y": f64_bits(-2147483648.1),
                "double_z": f64_bits(-0.01),
                "steps": [
                    {"op": "move_components", "x": "1", "y": "-1", "z": "1"},
                    {
                        "op": "set_with_offset_components",
                        "position": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
                        "x": "1",
                        "y": "-1",
                        "z": "1",
                    },
                    {"op": "clamp", "axis": "x", "minimum": "-2", "maximum": "-1"},
                    {"op": "clamp", "axis": "y", "minimum": "1", "maximum": "2"},
                    {"op": "clamp", "axis": "z", "minimum": "-4", "maximum": "-3"},
                ],
            }),
        ),
    ];
    for (label, values) in &blockpos_mutable_helpers {
        out.push(case(format!("blockpos/mutable-helpers/{label}"), "math", "blockpos_mutable_helpers", values.clone()));
    }
    out
}

fn mutable_helpers_all_overloads() -> Value {
    json!({
        "initial": {"x": "1", "y": "-2", "z": "3"},
        "double_x": f64_bits(-0.1),
        "double_y": f64_bits(2.0),
        "double_z": f64_bits(-3.9),
        "steps": [
            {"op": "set", "x": "4", "y": "-5", "z": "6"},
            {"op": "set_double", "x": f64_bits(1.9), "y": f64_bits(-2.0), "z": f64_bits(-3.1)},
            {"op": "set_block_pos", "value": {"x": "-7", "y": "8", "z": "-9"}},
            {"op": "set_packed", "value": "ffffffffffffffff"},
            {
                "op": "set_with_offset_direction",
                "position": {"x": "10", "y": "20", "z": "30"},
                "direction": "west",
            },
            {
                "op": "set_with_offset_components",
                "position": {"x": "-4", "y": "5", "z": "-6"},
                "x": "1",
                "y": "-2",
                "z": "3",
            },
            {
                "op": "set_with_offset_block_pos",
                "position": {"x": "7", "y": "-8", "z": "9"},
                "offset": {"x": "-2", "y": "3", "z": "-4"},
            },
            {"op": "move_direction", "direction": "east"},
            {"op": "move_direction_steps", "direction": "down", "steps": "-2"},
            {"op": "move_components", "x": "1", "y": "-2", "z": "3"},
            {"op": "move_block_pos", "value": {"x": "-1", "y": "2", "z": "-3"}},
            {"op": "clamp", "axis": "x", "minimum": "0", "maximum": "4"},
            {"op": "clamp", "axis": "y", "minimum": "-4", "maximum": "-2"},
            {"op": "clamp", "axis": "z", "minimum": "6", "maximum": "9"},
            {"op": "set_x", "value": "2147483647"},
            {"op": "set_y", "value": "-2147483648"},
            {"op": "set_z", "value": "0"},
            {"op": "offset_return", "x": "1", "y": "-1", "z": "2"},
            {"op": "multiply_return", "scale": "-2"},
            {"op": "relative_return", "direction": "north", "steps": "3"},
            {"op": "relative_axis_return", "axis": "z", "steps": "-4"},
            {"op": "rotate_return", "rotation": "clockwise_90"},
        ],
    })
}

pub fn cases() -> Vec<Value> {
    [mutable_helpers()].concat()
}
