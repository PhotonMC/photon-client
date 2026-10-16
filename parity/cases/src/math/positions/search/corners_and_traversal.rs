//! Cases for `blockpos/between-corners-direction`, `blockpos/breadth-first-traversal`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

fn between_corners_direction() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_between_corners_direction = [
        (
            "positive-z",
            json!({
                "aabb": box_bits(1.2, -1.8, 2.0, 3.9, 1.1, 4.2),
                "first": {"x": "1", "y": "-1", "z": "2"},
                "second": {"x": "3", "y": "1", "z": "4"},
                "first_x": "1",
                "first_y": "-1",
                "first_z": "2",
                "second_x": "3",
                "second_y": "1",
                "second_z": "4",
                "direction": vector_bits(0.5, 0.25, 2.0),
            }),
        ),
        ("negative-x", between_corners_direction_negative_x()),
        ("negative-z", between_corners_direction_negative_z()),
        ("signed-zero", between_corners_direction_signed_zero()),
        ("nan-direction", between_corners_direction_nan_direction()),
        ("integer-boundary-overflow", between_corners_direction_integer_boundary()),
    ];
    for (label, values) in &blockpos_between_corners_direction {
        out.push(case(
            format!("blockpos/between-corners-direction/{label}"),
            "math",
            "blockpos_between_corners_direction",
            values.clone(),
        ));
    }
    out
}

fn between_corners_direction_integer_boundary() -> Value {
    json!({
        "aabb": box_bits(2147483647.0, 0.0, 0.0, -2147483648.0, 0.0, 0.0),
        "first": {"x": "2147483647", "y": "0", "z": "0"},
        "second": {"x": "-2147483648", "y": "0", "z": "0"},
        "first_x": "2147483647",
        "first_y": "0",
        "first_z": "0",
        "second_x": "-2147483648",
        "second_y": "0",
        "second_z": "0",
        "direction": vector_bits(-1.0, 0.0, 0.0),
    })
}

fn between_corners_direction_nan_direction() -> Value {
    json!({
        "aabb": box_bits(0.0, -2.0, 3.0, 2.0, 0.0, 4.0),
        "first": {"x": "0", "y": "-2", "z": "3"},
        "second": {"x": "2", "y": "0", "z": "4"},
        "first_x": "0",
        "first_y": "-2",
        "first_z": "3",
        "second_x": "2",
        "second_y": "0",
        "second_z": "4",
        "direction": vector_bits(f64::NAN, f64::NAN, 1.0),
    })
}

fn between_corners_direction_signed_zero() -> Value {
    json!({
        "aabb": box_bits(-0.0, -0.0, -0.0, 1.0, 1.0, 1.0),
        "first": {"x": "-1", "y": "2", "z": "-3"},
        "second": {"x": "0", "y": "3", "z": "-2"},
        "first_x": "-1",
        "first_y": "2",
        "first_z": "-3",
        "second_x": "0",
        "second_y": "3",
        "second_z": "-2",
        "direction": vector_bits(-0.0, -0.0, 0.0),
    })
}

fn between_corners_direction_negative_z() -> Value {
    json!({
        "aabb": box_bits(-0.2, 0.0, -0.2, 1.8, 1.9, 1.8),
        "first": {"x": "0", "y": "0", "z": "0"},
        "second": {"x": "1", "y": "1", "z": "1"},
        "first_x": "0",
        "first_y": "0",
        "first_z": "0",
        "second_x": "1",
        "second_y": "1",
        "second_z": "1",
        "direction": vector_bits(0.5, -1.0, -2.0),
    })
}

fn between_corners_direction_negative_x() -> Value {
    json!({
        "aabb": box_bits(4.8, -1.2, -2.1, -0.2, 2.8, 1.7),
        "first": {"x": "4", "y": "2", "z": "-3"},
        "second": {"x": "-1", "y": "-1", "z": "0"},
        "first_x": "4",
        "first_y": "2",
        "first_z": "-3",
        "second_x": "-1",
        "second_y": "-1",
        "second_z": "0",
        "direction": vector_bits(-2.0, -0.5, 0.25),
    })
}

fn breadth_first_traversal() -> Vec<Value> {
    let mut out = Vec::new();
    let blockpos_breadth_first_traversal = [
        ("breadth-first", breadth_first_traversal_breadth_first()),
        ("stop", breadth_first_traversal_stop()),
        (
            "skip-root",
            json!({
                "start": {"x": "4", "y": "-2", "z": "7"},
                "max_depth": "3",
                "max_count": "1",
                "nodes": [
                    traversal_node((4, -2, 7), "skip", vec![(5, -2, 7)]),
                    traversal_node((5, -2, 7), "accept", vec![]),
                ],
            }),
        ),
        ("max-count", breadth_first_traversal_max_count()),
        ("depth-limit", breadth_first_traversal_depth_limit()),
        ("packed-visited", breadth_first_traversal_packed_visited()),
    ];
    for (label, values) in &blockpos_breadth_first_traversal {
        out.push(case(
            format!("blockpos/breadth-first-traversal/{label}"),
            "math",
            "blockpos_breadth_first_traversal",
            values.clone(),
        ));
    }
    out
}

fn breadth_first_traversal_breadth_first() -> Value {
    json!({
        "start": {"x": "0", "y": "0", "z": "0"},
        "max_depth": "2",
        "max_count": "99",
        "nodes": [
            traversal_node((0, 0, 0), "accept", vec![(1, 0, 0), (0, 0, 1)]),
            traversal_node((1, 0, 0), "accept", vec![(2, 0, 0), (0, 0, 0)]),
            traversal_node((0, 0, 1), "accept", vec![(2, 0, 0), (0, 0, 2)]),
            traversal_node((2, 0, 0), "skip", vec![(3, 0, 0)]),
            traversal_node((0, 0, 2), "accept", vec![(0, 0, 3)]),
        ],
    })
}

fn breadth_first_traversal_max_count() -> Value {
    json!({
        "start": {"x": "0", "y": "0", "z": "0"},
        "max_depth": "4",
        "max_count": "2",
        "nodes": [
            traversal_node((0, 0, 0), "accept", vec![(1, 0, 0), (2, 0, 0)]),
            traversal_node((1, 0, 0), "accept", vec![(3, 0, 0)]),
            traversal_node((2, 0, 0), "accept", vec![]),
        ],
    })
}

fn breadth_first_traversal_stop() -> Value {
    json!({
        "start": {"x": "0", "y": "0", "z": "0"},
        "max_depth": "3",
        "max_count": "99",
        "nodes": [
            traversal_node((0, 0, 0), "accept", vec![(1, 0, 0), (0, 0, 1)]),
            traversal_node((1, 0, 0), "stop", vec![(2, 0, 0)]),
            traversal_node((0, 0, 1), "accept", vec![(0, 0, 2)]),
        ],
    })
}

fn breadth_first_traversal_packed_visited() -> Value {
    json!({
        "start": {"x": "0", "y": "0", "z": "0"},
        "max_depth": "1",
        "max_count": "99",
        "nodes": [
            traversal_node((0, 0, 0), "accept", vec![(33554432, 0, 0)]),
            traversal_node((33554432, 0, 0), "stop", vec![]),
        ],
    })
}

fn breadth_first_traversal_depth_limit() -> Value {
    json!({
        "start": {"x": "-1", "y": "3", "z": "5"},
        "max_depth": "1",
        "max_count": "99",
        "nodes": [
            traversal_node((-1, 3, 5), "accept", vec![(0, 3, 5)]),
            traversal_node((0, 3, 5), "accept", vec![(1, 3, 5)]),
        ],
    })
}

pub fn cases() -> Vec<Value> {
    [between_corners_direction(), breadth_first_traversal()].concat()
}
