//! Cases for `blockpos/between-closed`, `blockpos/neighbor-column`, `blockpos/square-out-south-east`, `blockpos/random-ranges` and more.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

fn between_closed() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_between_closed = [
        ("single", between_closed_single()),
        ("order", between_closed_order()),
        ("reversed", between_closed_reversed()),
        ("negative", between_closed_negative()),
        ("integer-boundaries", between_closed_integer_boundaries()),
    ];
    for (label, values) in &blockpos_between_closed {
        out.push(case(format!("blockpos/between-closed/{label}"), "math", "blockpos_between_closed", values.clone()));
    }
    let blockpos_neighbor_column = [
        ("ascending", json!({"start_x": "10", "start_y": "5", "start_z": "-2", "end_y": "7"})),
        ("descending", json!({"start_x": "-3", "start_y": "4", "start_z": "9", "end_y": "2"})),
        ("singleton", json!({"start_x": "0", "start_y": "-2", "start_z": "0", "end_y": "-2"})),
        ("integer-boundaries", between_closed_integer_boundaries_start_x()),
    ];
    for (label, values) in &blockpos_neighbor_column {
        out.push(case(format!("blockpos/neighbor-column/{label}"), "math", "blockpos_neighbor_column", values.clone()));
    }
    let blockpos_square_out_south_east = [
        ("basic", json!({"x": "1", "y": "2", "z": "3"})),
        ("negative", json!({"x": "-4", "y": "5", "z": "-6"})),
        ("zero", json!({"x": "0", "y": "0", "z": "0"})),
        ("integer-boundaries", json!({"x": "2147483647", "y": "-2147483648", "z": "2147483647"})),
    ];
    for (label, values) in &blockpos_square_out_south_east {
        out.push(case(
            format!("blockpos/square-out-south-east/{label}"),
            "math",
            "blockpos_square_out_south_east",
            values.clone(),
        ));
    }
    out
}

fn between_closed_integer_boundaries() -> Value {
    json!({
        "from": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
        "to": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
        "min_x": "2147483647",
        "min_y": "-2147483648",
        "min_z": "31",
        "max_x": "2147483647",
        "max_y": "-2147483648",
        "max_z": "31",
    })
}

fn between_closed_negative() -> Value {
    json!({
        "from": {"x": "-2", "y": "-1", "z": "-3"},
        "to": {"x": "-1", "y": "0", "z": "-2"},
        "min_x": "-3",
        "min_y": "-2",
        "min_z": "-1",
        "max_x": "-2",
        "max_y": "-1",
        "max_z": "0",
    })
}

fn between_closed_reversed() -> Value {
    json!({
        "from": {"x": "2", "y": "3", "z": "4"},
        "to": {"x": "1", "y": "2", "z": "3"},
        "min_x": "3",
        "min_y": "4",
        "min_z": "5",
        "max_x": "4",
        "max_y": "5",
        "max_z": "6",
    })
}

fn between_closed_order() -> Value {
    json!({
        "from": {"x": "1", "y": "2", "z": "3"},
        "to": {"x": "2", "y": "3", "z": "4"},
        "min_x": "-2",
        "min_y": "-1",
        "min_z": "-3",
        "max_x": "-1",
        "max_y": "0",
        "max_z": "-2",
    })
}

fn between_closed_single() -> Value {
    json!({
        "from": {"x": "2", "y": "-3", "z": "4"},
        "to": {"x": "2", "y": "-3", "z": "4"},
        "min_x": "-1",
        "min_y": "2",
        "min_z": "-3",
        "max_x": "0",
        "max_y": "3",
        "max_z": "-2",
    })
}

fn between_closed_integer_boundaries_start_x() -> Value {
    json!({
        "start_x": "2147483647",
        "start_y": "-2147483648",
        "start_z": "-2147483648",
        "end_y": "-2147483648",
    })
}

fn random_ranges() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_random_ranges = [
        ("basic", random_ranges_basic()),
        ("zero-limit", random_ranges_zero_limit()),
        ("unit-ranges", random_ranges_unit_ranges()),
        ("integer-boundaries", random_ranges_integer_boundaries()),
    ];
    for (label, values) in &blockpos_random_ranges {
        out.push(case(format!("blockpos/random-ranges/{label}"), "math", "blockpos_random_ranges", values.clone()));
    }
    let blockpos_between_closed_aabb = [
        ("fractional", box_bits(0.25, 0.25, 0.25, 1.75, 1.75, 1.75)),
        ("negative-fractional", box_bits(-1.75, -0.2, -3.0, -0.25, 0.8, -1.1)),
        ("reversed", box_bits(2.5, 1.5, 3.5, 0.5, -0.5, 1.5)),
        ("integer-boundaries", box_bits(0.0, -1.0, 2.0, 1.0, 0.0, 3.0)),
        ("signed-zero", box_bits(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0)),
    ];
    for (label, values) in &blockpos_between_closed_aabb {
        out.push(case(
            format!("blockpos/between-closed-aabb/{label}"),
            "math",
            "blockpos_between_closed_aabb",
            json!({"aabb": values.clone()}),
        ));
    }
    let blockpos_within_manhattan = [
        ("unit", json!({"origin": {"x": "0", "y": "0", "z": "0"}, "reach_x": "1", "reach_y": "1", "reach_z": "1"})),
        (
            "asymmetric",
            json!({"origin": {"x": "10", "y": "-5", "z": "7"}, "reach_x": "2", "reach_y": "1", "reach_z": "1"}),
        ),
        ("flat", json!({"origin": {"x": "-3", "y": "4", "z": "-2"}, "reach_x": "1", "reach_y": "0", "reach_z": "0"})),
        ("integer-boundaries", random_ranges_integer_boundaries_origin()),
    ];
    for (label, values) in &blockpos_within_manhattan {
        out.push(case(
            format!("blockpos/within-manhattan/{label}"),
            "math",
            "blockpos_within_manhattan",
            values.clone(),
        ));
    }
    out
}

fn random_ranges_integer_boundaries() -> Value {
    json!({
        "seed": "9223372036854775807",
        "limit": "3",
        "min_x": "2147483645",
        "min_y": "-2147483647",
        "min_z": "2147483645",
        "max_x": "-2147483647",
        "max_y": "-2147483645",
        "max_z": "-2147483647",
        "center": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
        "size": "2",
    })
}

fn random_ranges_unit_ranges() -> Value {
    json!({
        "seed": "-1",
        "limit": "3",
        "min_x": "7",
        "min_y": "-8",
        "min_z": "9",
        "max_x": "7",
        "max_y": "-8",
        "max_z": "9",
        "center": {"x": "-4", "y": "5", "z": "-6"},
        "size": "0",
    })
}

fn random_ranges_zero_limit() -> Value {
    json!({
        "seed": "12345",
        "limit": "0",
        "min_x": "-1",
        "min_y": "-1",
        "min_z": "-1",
        "max_x": "1",
        "max_y": "1",
        "max_z": "1",
        "center": {"x": "0", "y": "0", "z": "0"},
        "size": "1",
    })
}

fn random_ranges_basic() -> Value {
    json!({
        "seed": "42",
        "limit": "5",
        "min_x": "-2",
        "min_y": "0",
        "min_z": "3",
        "max_x": "2",
        "max_y": "4",
        "max_z": "7",
        "center": {"x": "10", "y": "20", "z": "-5"},
        "size": "2",
    })
}

fn random_ranges_integer_boundaries_origin() -> Value {
    json!({
        "origin": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
        "reach_x": "1",
        "reach_y": "1",
        "reach_z": "0",
    })
}

pub fn cases() -> Vec<Value> {
    [between_closed(), random_ranges()].concat()
}
