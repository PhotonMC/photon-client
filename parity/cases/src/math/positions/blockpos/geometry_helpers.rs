//! Cases for `blockpos/geometry-helpers`, `blockpos/distance-helpers`, `blockpos/integer-helpers`, `blockpos/clamp-location` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn geometry_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_geometry_helpers = [
        (
            "basic",
            json!({
                "x": "1",
                "y": "2",
                "z": "3",
                "rotation": "clockwise_90",
                "up_x": "4",
                "up_y": "5",
                "up_z": "6",
                "at_y": "9",
            }),
        ),
        (
            "none-negative",
            json!({
                "x": "-10",
                "y": "20",
                "z": "-30",
                "rotation": "none",
                "up_x": "-2",
                "up_y": "3",
                "up_z": "-4",
                "at_y": "-7",
            }),
        ),
        ("half-turn", geometry_helpers_half_turn()),
        ("counterclockwise", geometry_helpers_counterclockwise()),
        ("overflow", geometry_helpers_overflow()),
    ];
    for (label, values) in blockpos_geometry_helpers.iter() {
        out.push(case(
            format!("blockpos/geometry-helpers/{}", label),
            "math",
            "blockpos_geometry_helpers",
            values.clone(),
        ));
    }
    out
}

fn geometry_helpers_overflow() -> Value {
    json!({
        "x": "2147483647",
        "y": "-2147483648",
        "z": "-2147483648",
        "rotation": "180",
        "up_x": "-2147483648",
        "up_y": "2147483647",
        "up_z": "-2147483648",
        "at_y": "2147483647",
    })
}

fn geometry_helpers_counterclockwise() -> Value {
    json!({
        "x": "-17",
        "y": "16",
        "z": "-1",
        "rotation": "counterclockwise_90",
        "up_x": "5",
        "up_y": "-6",
        "up_z": "7",
        "at_y": "31",
    })
}

fn geometry_helpers_half_turn() -> Value {
    json!({
        "x": "17",
        "y": "-16",
        "z": "1",
        "rotation": "180",
        "up_x": "-5",
        "up_y": "6",
        "up_z": "-7",
        "at_y": "0",
    })
}

fn distance_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_distance_helpers = [
        ("basic", distance_helpers_basic()),
        ("center", distance_helpers_center()),
        ("negative", distance_helpers_negative()),
        ("integer-boundaries", distance_helpers_integer_boundaries()),
        ("special-point", distance_helpers_special_point()),
    ];
    for (label, values) in blockpos_distance_helpers.iter() {
        out.push(case(
            format!("blockpos/distance-helpers/{}", label),
            "math",
            "blockpos_distance_helpers",
            values.clone(),
        ));
    }
    let blockpos_integer_helpers = [
        ("basic", json!({"x": "0", "y": "0", "z": "0", "other": {"x": "3", "y": "-4", "z": "5"}})),
        ("negative", json!({"x": "-10", "y": "20", "z": "-30", "other": {"x": "-7", "y": "24", "z": "-35"}})),
        ("same", json!({"x": "17", "y": "-16", "z": "1", "other": {"x": "17", "y": "-16", "z": "1"}})),
        ("min-abs-edge", distance_helpers_min_abs_edge()),
        ("integer-boundaries", distance_helpers_integer_boundaries_x()),
    ];
    for (label, values) in blockpos_integer_helpers.iter() {
        out.push(case(
            format!("blockpos/integer-helpers/{}", label),
            "math",
            "blockpos_integer_helpers",
            values.clone(),
        ));
    }
    out
}

fn distance_helpers_special_point() -> Value {
    json!({
        "x": "1",
        "y": "2",
        "z": "3",
        "other": {"x": "1", "y": "2", "z": "3"},
        "point": {"x": "7ff8000000000002", "y": "7ff0000000000000", "z": "fff0000000000000"},
        "distance": "7ff8000000000002",
    })
}

fn distance_helpers_integer_boundaries() -> Value {
    json!({
        "x": "2147483647",
        "y": "-2147483648",
        "z": "2147483647",
        "other": {"x": "-2147483648", "y": "2147483647", "z": "-2147483648"},
        "point": vector_bits(2147483647.5, -2147483648.5, 2147483647.5),
        "distance": f64_bits(4294967296.0),
    })
}

fn distance_helpers_negative() -> Value {
    json!({
        "x": "17",
        "y": "-16",
        "z": "1",
        "other": {"x": "-17", "y": "16", "z": "-1"},
        "point": vector_bits(-16.5, 15.5, -0.5),
        "distance": f64_bits(64.0),
    })
}

fn distance_helpers_center() -> Value {
    json!({
        "x": "-10",
        "y": "20",
        "z": "-30",
        "other": {"x": "-7", "y": "24", "z": "-30"},
        "point": vector_bits(-9.5, 20.5, -29.5),
        "distance": f64_bits(4.0),
    })
}

fn distance_helpers_basic() -> Value {
    json!({
        "x": "0",
        "y": "0",
        "z": "0",
        "other": {"x": "3", "y": "4", "z": "0"},
        "point": vector_bits(1.0, 2.0, 3.0),
        "distance": f64_bits(5.0),
    })
}

fn distance_helpers_integer_boundaries_x() -> Value {
    json!({
        "x": "2147483647",
        "y": "-2147483648",
        "z": "2147483647",
        "other": {"x": "-2147483648", "y": "2147483647", "z": "-2147483648"},
    })
}

fn distance_helpers_min_abs_edge() -> Value {
    json!({
        "x": "0",
        "y": "-2147483648",
        "z": "0",
        "other": {"x": "-2147483648", "y": "0", "z": "-2147483648"},
    })
}

fn clamp_location() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_clamp_location = [
        ("inside", json!({"x": "10", "y": "20", "z": "30", "location": vector_bits(10.5, 20.5, 30.5)})),
        ("outside", json!({"x": "-10", "y": "20", "z": "-30", "location": vector_bits(-11.0, 21.0, -29.0)})),
        ("boundary", json!({"x": "0", "y": "-1", "z": "1", "location": vector_bits(-0.0, -0.0, 1.0)})),
        (
            "integer-boundaries",
            json!({
                "x": "2147483647",
                "y": "-2147483648",
                "z": "2147483647",
                "location": vector_bits(2147483647.5, -2147483648.5, 2147483647.5),
            }),
        ),
        (
            "special",
            json!({
                "x": "1",
                "y": "2",
                "z": "3",
                "location": {"x": "7ff8000000000002", "y": "7ff0000000000000", "z": "fff0000000000000"},
            }),
        ),
    ];
    for (label, values) in blockpos_clamp_location.iter() {
        out.push(case(format!("blockpos/clamp-location/{}", label), "math", "blockpos_clamp_location", values.clone()));
    }
    out
}

fn coordinate_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_coordinate_helpers = [
        (
            "fractional",
            json!({
                "x": f64_bits(2.75),
                "y": f64_bits(-1.0),
                "z": f64_bits(-0.01),
                "point": vector_bits(3.1, -2.2, 4.9),
                "first": {"x": "3", "y": "-4", "z": "5"},
                "second": {"x": "-2", "y": "6", "z": "1"},
            }),
        ),
        (
            "negative-fractional",
            json!({
                "x": f64_bits(-0.01),
                "y": f64_bits(-2.0),
                "z": f64_bits(3.0),
                "point": vector_bits(-1.1, 2.0, -3.9),
                "first": {"x": "-10", "y": "20", "z": "-30"},
                "second": {"x": "-7", "y": "24", "z": "-35"},
            }),
        ),
        (
            "signed-zero",
            json!({
                "x": f64_bits(-0.0),
                "y": f64_bits(0.0),
                "z": f64_bits(-0.0),
                "point": vector_bits(-0.0, 0.0, -0.0),
                "first": {"x": "0", "y": "-1", "z": "2"},
                "second": {"x": "0", "y": "-1", "z": "2"},
            }),
        ),
        (
            "integer-boundaries",
            json!({
                "x": f64_bits(2147483647.0),
                "y": f64_bits(-2147483648.0),
                "z": f64_bits(31.0),
                "point": vector_bits(2147483646.5, -2147483647.5, 31.5),
                "first": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
                "second": {"x": "-2147483648", "y": "2147483647", "z": "-2147483648"},
            }),
        ),
    ];
    for (label, values) in blockpos_coordinate_helpers.iter() {
        out.push(case(
            format!("blockpos/coordinate-helpers/{}", label),
            "math",
            "blockpos_coordinate_helpers",
            values.clone(),
        ));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [geometry_helpers(), distance_helpers(), clamp_location(), coordinate_helpers()].concat()
}
