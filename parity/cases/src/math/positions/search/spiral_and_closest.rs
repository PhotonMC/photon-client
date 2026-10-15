//! Cases for `blockpos/spiral-around`, `blockpos/find-closest-match`, `blockpos/between-closed-streams`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

fn spiral_around() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_spiral_around = [
        (
            "radius-one-horizontal",
            json!({
                "center": {"x": "0", "y": "0", "z": "0"},
                "radius": "1",
                "first_direction": "north",
                "second_direction": "east",
            }),
        ),
        (
            "radius-two-horizontal",
            json!({
                "center": {"x": "5", "y": "-2", "z": "7"},
                "radius": "2",
                "first_direction": "west",
                "second_direction": "south",
            }),
        ),
        (
            "vertical",
            json!({
                "center": {"x": "2147483647", "y": "-2147483648", "z": "0"},
                "radius": "1",
                "first_direction": "up",
                "second_direction": "north",
            }),
        ),
        (
            "zero-radius",
            json!({
                "center": {"x": "-3", "y": "4", "z": "-5"},
                "radius": "0",
                "first_direction": "south",
                "second_direction": "east",
            }),
        ),
        (
            "negative-radius",
            json!({
                "center": {"x": "1", "y": "2", "z": "3"},
                "radius": "-1",
                "first_direction": "north",
                "second_direction": "east",
            }),
        ),
    ];
    for (label, values) in blockpos_spiral_around.iter() {
        out.push(case(format!("blockpos/spiral-around/{}", label), "math", "blockpos_spiral_around", values.clone()));
    }
    out
}

fn find_closest_match() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_find_closest_match = [
        (
            "origin",
            json!({
                "start": {"x": "0", "y": "0", "z": "0"},
                "horizontal_radius": "1",
                "vertical_radius": "1",
                "target": {"x": "0", "y": "0", "z": "0"},
            }),
        ),
        (
            "reachable",
            json!({
                "start": {"x": "10", "y": "-5", "z": "7"},
                "horizontal_radius": "2",
                "vertical_radius": "1",
                "target": {"x": "9", "y": "-5", "z": "8"},
            }),
        ),
        (
            "unreachable",
            json!({
                "start": {"x": "-3", "y": "4", "z": "-2"},
                "horizontal_radius": "1",
                "vertical_radius": "1",
                "target": {"x": "4", "y": "4", "z": "-2"},
            }),
        ),
        (
            "zero-radii",
            json!({
                "start": {"x": "6", "y": "-7", "z": "8"},
                "horizontal_radius": "0",
                "vertical_radius": "0",
                "target": {"x": "6", "y": "-7", "z": "9"},
            }),
        ),
        (
            "integer-boundaries",
            json!({
                "start": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
                "horizontal_radius": "1",
                "vertical_radius": "0",
                "target": {"x": "-2147483648", "y": "-2147483648", "z": "2147483647"},
            }),
        ),
    ];
    for (label, values) in blockpos_find_closest_match.iter() {
        out.push(case(
            format!("blockpos/find-closest-match/{}", label),
            "math",
            "blockpos_find_closest_match",
            values.clone(),
        ));
    }
    out
}

fn between_closed_streams() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_between_closed_streams = [
        (
            "normal",
            json!({
                "from": {"x": "2", "y": "-1", "z": "3"},
                "to": {"x": "1", "y": "0", "z": "4"},
                "bounding_box": {"min_x": "-2", "min_y": "1", "min_z": "3", "max_x": "-1", "max_y": "2", "max_z": "4"},
                "aabb": box_bits(0.25, -1.25, 2.0, 1.75, 0.75, 3.5),
                "min_x": "4",
                "min_y": "-2",
                "min_z": "5",
                "max_x": "5",
                "max_y": "-1",
                "max_z": "6",
            }),
        ),
        ("reversed", between_closed_streams_reversed()),
        ("integer-boundaries", between_closed_streams_integer_boundaries()),
    ];
    for (label, values) in blockpos_between_closed_streams.iter() {
        out.push(case(
            format!("blockpos/between-closed-streams/{}", label),
            "math",
            "blockpos_between_closed_streams",
            values.clone(),
        ));
    }
    out
}

fn between_closed_streams_integer_boundaries() -> Value {
    json!({
        "from": {"x": "2147483647", "y": "-2147483648", "z": "0"},
        "to": {"x": "2147483647", "y": "-2147483648", "z": "0"},
        "bounding_box": {
            "min_x": "2147483647",
            "min_y": "-2147483648",
            "min_z": "0",
            "max_x": "2147483647",
            "max_y": "-2147483648",
            "max_z": "0",
        },
        "aabb": box_bits(2147483647.0, -2147483648.0, 0.0, 2147483647.0, -2147483648.0, 0.0),
        "min_x": "2147483647",
        "min_y": "-2147483648",
        "min_z": "0",
        "max_x": "2147483647",
        "max_y": "-2147483648",
        "max_z": "0",
    })
}

fn between_closed_streams_reversed() -> Value {
    json!({
        "from": {"x": "-1", "y": "3", "z": "2"},
        "to": {"x": "1", "y": "1", "z": "0"},
        "bounding_box": {
            "min_x": "2",
            "min_y": "0",
            "min_z": "-1",
            "max_x": "1",
            "max_y": "-1",
            "max_z": "-2",
        },
        "aabb": box_bits(2.5, 1.5, 0.5, 0.5, -0.5, -1.5),
        "min_x": "-3",
        "min_y": "-2",
        "min_z": "-1",
        "max_x": "-2",
        "max_y": "-1",
        "max_z": "0",
    })
}

pub fn cases() -> Vec<Value> {
    [spiral_around(), find_closest_match(), between_closed_streams()].concat()
}
