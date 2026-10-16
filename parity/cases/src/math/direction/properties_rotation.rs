//! Cases for `direction/properties`, `direction/rotate-y`, `direction/rotate-`, `direction/axial-rotation` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn properties() -> Vec<Value> {
    let mut out = Vec::new();
    for name in ["down", "up", "north", "south", "west", "east"] {
        out.push(case(format!("direction/properties/{name}"), "math", "direction", json!({"name": name, "steps": []})));
    }
    for name in ["down", "up", "north", "south", "west", "east"] {
        out.push(case(
            format!("direction/rotate-y/{name}"),
            "math",
            "direction",
            json!({"name": name, "steps": [{"axis": "y", "clockwise": true}, {"axis": "y", "clockwise": true}]}),
        ));
    }
    for (name, axis) in [("north", "x"), ("north", "z"), ("down", "x"), ("up", "z"), ("west", "x"), ("east", "z")] {
        out.push(case(
            format!("direction/rotate-{axis}/{name}"),
            "math",
            "direction",
            json!({"name": name, "steps": [{"axis": axis, "clockwise": true}]}),
        ));
    }
    for direction in ["down", "up", "north", "south", "west", "east"] {
        for axis in ["x", "y", "z"] {
            out.push(case(
                format!("direction/axial-rotation/{direction}/{axis}"),
                "math",
                "direction_axial_rotation",
                json!({"name": direction, "axis": axis}),
            ));
        }
    }
    for direction in ["down", "up", "north", "south", "west", "east"] {
        out.push(case(
            format!("direction/y-rotation/{direction}"),
            "math",
            "direction_y_rotation",
            json!({"name": direction}),
        ));
    }
    out
}

fn data() -> Vec<Value> {
    let mut out = Vec::new();
    let direction_data_values = [
        ("3d", vec![0, 1, 2, 3, 4, 5, -1, -2, -3, -4, -5, -6, -7, 6, 11, -11, -2147483648, 2147483647]),
        ("2d", vec![0, 1, 2, 3, -1, -2, -3, -4, -5, 4, 7, -7, -2147483648, 2147483647]),
    ];
    for (width, values) in &direction_data_values {
        for value in values {
            out.push(case(
                format!("direction/data/{width}/{value}"),
                "math",
                "direction_data",
                json!({"type": width, "value": (value).to_string()}),
            ));
        }
    }
    let direction_y_rot_values = vec![
        ("zero", f64_bits(0.0)),
        ("negative-zero", f64_bits(-0.0)),
        ("south-quarter", f64_bits(44.99999999999999)),
        ("west-boundary", f64_bits(45.0)),
        ("west-quarter", f64_bits(89.99999999999999)),
        ("north-boundary", f64_bits(135.0)),
        ("east-boundary", f64_bits(225.0)),
        ("south-wrap", f64_bits(315.0)),
        ("south-full-turn", f64_bits(360.0)),
        ("negative-east-boundary", f64_bits(-45.0)),
        ("negative-east-before", f64_bits(-45.00000000000001)),
        ("negative-south-after", f64_bits(-44.99999999999999)),
        ("negative-north-boundary", f64_bits(-135.0)),
        ("negative-full-turn", f64_bits(-360.0)),
        ("subnormal", "0000000000000001".to_string()),
        ("negative-subnormal", "8000000000000001".to_string()),
        ("positive-int-bound", f64_bits(193273528320.0)),
        ("negative-int-bound", f64_bits(-193273528320.0)),
        ("positive-infinity", "7ff0000000000000".to_string()),
        ("negative-infinity", "fff0000000000000".to_string()),
        ("nan", "7ff8000000000001".to_string()),
    ];
    for (name, value) in &direction_y_rot_values {
        out.push(case(format!("direction/y-rot/{name}"), "math", "direction_y_rot", json!({"value": value})));
    }
    out
}

fn axis() -> Vec<Value> {
    let mut out = Vec::new();
    for axis in ["x", "y", "z"] {
        for direction in ["positive", "negative"] {
            out.push(case(
                format!("direction/axis/{axis}/{direction}"),
                "math",
                "direction_axis",
                json!({"axis": axis, "direction": direction}),
            ));
            out.push(case(
                format!("direction/get/{axis}/{direction}"),
                "math",
                "direction_get",
                json!({"axis": axis, "direction": direction}),
            ));
        }
    }
    for name in ["down", "up", "north", "south", "west", "east"] {
        out.push(case(
            format!("direction/unit-vectors/{name}"),
            "math",
            "direction_unit_vectors",
            json!({"name": name}),
        ));
        out.push(case(format!("direction/step/{name}"), "math", "direction_step", json!({"name": name})));
    }
    for axis in ["x", "y", "z"] {
        out.push(case(
            format!("direction/axis-orientation/{axis}"),
            "math",
            "direction_axis_orientation",
            json!({"axis": axis}),
        ));
    }
    for axis in ["x", "y", "z"] {
        out.push(case(
            format!("direction/axis-directions/{axis}"),
            "math",
            "direction_axis_directions",
            json!({"axis": axis}),
        ));
    }
    out
}

fn axis_direction_values() -> Vec<Value> {
    let mut out = Vec::new();
    for direction in ["positive", "negative"] {
        out.push(case(
            format!("direction/axis-direction-values/{direction}"),
            "math",
            "direction_axis_direction_values",
            json!({"direction": direction}),
        ));
    }
    out
}

fn axis_choose() -> Vec<Value> {
    let mut out = Vec::new();
    let direction_axis_choose_cases = [("distinct", axis_choose_distinct()), ("special", axis_choose_special())];
    for (label, values) in &direction_axis_choose_cases {
        for axis in ["x", "y", "z"] {
            out.push(case(
                format!("direction/axis-choose/{label}/{axis}"),
                "math",
                "direction_axis_choose",
                merged(vec![json!({"axis": axis}), values.clone()]),
            ));
        }
    }
    for axis in ["x", "y", "z"] {
        for direction in ["down", "up", "north", "south", "west", "east", "none"] {
            out.push(case(
                format!("direction/axis-test/{axis}/{direction}"),
                "math",
                "direction_axis_test",
                json!({"axis": axis, "direction": direction}),
            ));
        }
    }
    for axis in ["x", "y", "z"] {
        out.push(case(format!("direction/axis-plane/{axis}"), "math", "direction_axis_plane", json!({"axis": axis})));
    }
    for plane in ["horizontal", "vertical"] {
        for direction in ["down", "up", "north", "south", "west", "east", "none"] {
            out.push(case(
                format!("direction/plane-test/{plane}/{direction}"),
                "math",
                "direction_plane_test",
                json!({"plane": plane, "direction": direction}),
            ));
        }
        out.push(case(
            format!("direction/plane-order/{plane}"),
            "math",
            "direction_plane_order",
            json!({"plane": plane}),
        ));
    }
    out
}

fn axis_choose_special() -> Value {
    json!({
        "int_x": "-2147483648",
        "int_y": "2147483647",
        "int_z": "0",
        "double_x": "8000000000000000",
        "double_y": "7ff0000000000000",
        "double_z": "7ff8000000000001",
        "bool_x": false,
        "bool_y": true,
        "bool_z": false,
    })
}

fn axis_choose_distinct() -> Value {
    json!({
        "int_x": "-17",
        "int_y": "11",
        "int_z": "29",
        "double_x": f64_bits(-1.25),
        "double_y": f64_bits(2.5),
        "double_z": f64_bits(3.75),
        "bool_x": true,
        "bool_y": false,
        "bool_z": true,
    })
}

fn random() -> Vec<Value> {
    let mut out = Vec::new();
    for seed in ["0", "1", "-1", "42", "12345", "9223372036854775807", "-9223372036854775808"] {
        for plane in ["horizontal", "vertical"] {
            out.push(case(
                format!("direction/random/{seed}/{plane}"),
                "math",
                "direction_random",
                json!({"seed": seed, "plane": plane}),
            ));
            out.push(case(
                format!("direction/plane-shuffle/{seed}/{plane}"),
                "math",
                "direction_plane_shuffle",
                json!({"seed": seed, "plane": plane}),
            ));
        }
        out.push(case(format!("direction/all-shuffle/{seed}"), "math", "direction_all_shuffle", json!({"seed": seed})));
    }
    out.push(case("direction/stream", "math", "direction_stream", json!({})));
    for name in ["down", "up", "north", "south", "west", "east"] {
        out.push(case(format!("direction/to-y-rot/{name}"), "math", "direction_to_y_rot", json!({"name": name})));
    }
    for name in ["down", "up", "north", "south", "west", "east"] {
        out.push(case(format!("direction/get-y-rot/{name}"), "math", "direction_get_y_rot", json!({"name": name})));
    }
    for name in ["down", "up", "north", "south", "west", "east"] {
        out.push(case(format!("direction/rotation/{name}"), "math", "direction_rotation", json!({"name": name})));
    }
    out
}

fn name_lookup() -> Vec<Value> {
    let mut out = Vec::new();
    let direction_name_lookup_cases = [
        ("direction", vec!["down", "up", "north", "south", "west", "east", "NORTH", "North", "north ", ""]),
        ("axis", vec!["x", "y", "z", "X", "Y", "z ", ""]),
    ];
    for (kind, names) in &direction_name_lookup_cases {
        for name in names.iter().copied() {
            let label = if name.is_empty() { "empty" } else { name };
            out.push(case(
                format!("direction/name-lookup/{kind}/{label}"),
                "math",
                "direction_name_lookup",
                json!({"kind": kind, "name": name}),
            ));
        }
    }
    let direction_enum_text_cases =
        [("axis_direction", vec!["positive", "negative"]), ("plane", vec!["horizontal", "vertical"])];
    for (kind, names) in &direction_enum_text_cases {
        for name in names {
            out.push(case(
                format!("direction/enum-text/{kind}/{name}"),
                "math",
                "direction_enum_text",
                json!({"kind": kind, "name": name}),
            ));
        }
    }
    let direction_enum_name_cases =
        [("direction", vec!["down", "up", "north", "south", "west", "east"]), ("axis", vec!["x", "y", "z"])];
    for (kind, names) in &direction_enum_name_cases {
        for name in names {
            out.push(case(
                format!("direction/enum-name/{kind}/{name}"),
                "math",
                "direction_enum_name",
                json!({"kind": kind, "name": name}),
            ));
        }
    }
    out
}

fn approximate_nearest() -> Vec<Value> {
    let mut out = Vec::new();
    let direction_approximate_nearest_cases = vec![
        ("south", vector_bits(0.0, 0.0, 1.0)),
        ("north", vector_bits(0.0, 0.0, -1.0)),
        ("east", vector_bits(1.0, 0.0, 0.0)),
        ("west", vector_bits(-1.0, 0.0, 0.0)),
        ("up", vector_bits(0.0, 1.0, 0.0)),
        ("down", vector_bits(0.0, -1.0, 0.0)),
        ("zero", vector_bits(0.0, 0.0, 0.0)),
        ("all-negative", vector_bits(-1.0, -1.0, -1.0)),
        ("tie-horizontal", vector_bits(1.0, 0.0, 1.0)),
        ("tie-vertical-horizontal", vector_bits(1.0, 1.0, 0.0)),
        ("f64-to-f32", vector_bits(1.0000000596046448, 0.0, 0.0)),
        ("positive-infinity-x", json!({"x": "7ff0000000000000", "y": "0000000000000000", "z": "0000000000000000"})),
        ("negative-infinity-z", json!({"x": "0000000000000000", "y": "0000000000000000", "z": "fff0000000000000"})),
        ("nan-x", json!({"x": "7ff8000000000001", "y": "0000000000000000", "z": "0000000000000000"})),
        ("subnormal", json!({"x": "0000000000000001", "y": "0000000000000000", "z": "0000000000000000"})),
        ("max-finite", json!({"x": "7fefffffffffffff", "y": "0000000000000000", "z": "0000000000000000"})),
    ];
    for (name, vector) in &direction_approximate_nearest_cases {
        out.push(case(
            format!("direction/approximate-nearest/{name}"),
            "math",
            "direction_approximate_nearest",
            vector.clone(),
        ));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [
        properties(),
        data(),
        axis(),
        axis_direction_values(),
        axis_choose(),
        random(),
        name_lookup(),
        approximate_nearest(),
    ]
    .concat()
}
