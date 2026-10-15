//! Cases for `aabb/intersect`, `aabb/minmax`, `aabb/move`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn contained() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/intersect/contained",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "intersect",
                        "box": {
                            "min_x": f64_bits(0.25),
                            "min_y": f64_bits(0.25),
                            "min_z": f64_bits(0.25),
                            "max_x": f64_bits(0.75),
                            "max_y": f64_bits(0.75),
                            "max_z": f64_bits(0.75),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn partial() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/intersect/partial",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "intersect",
                        "box": {
                            "min_x": f64_bits(-0.5),
                            "min_y": f64_bits(0.25),
                            "min_z": f64_bits(-1.0),
                            "max_x": f64_bits(0.5),
                            "max_y": f64_bits(1.5),
                            "max_z": f64_bits(0.5),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn disjoint_normalizes() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/intersect/disjoint-normalizes",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "intersect",
                        "box": {
                            "min_x": f64_bits(2.0),
                            "min_y": f64_bits(2.0),
                            "min_z": f64_bits(2.0),
                            "max_x": f64_bits(3.0),
                            "max_y": f64_bits(3.0),
                            "max_z": f64_bits(3.0),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn touching_face() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/intersect/touching-face",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "intersect",
                        "box": {
                            "min_x": f64_bits(1.0),
                            "min_y": f64_bits(0.25),
                            "min_z": f64_bits(0.25),
                            "max_x": f64_bits(2.0),
                            "max_y": f64_bits(0.75),
                            "max_z": f64_bits(0.75),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/intersect/signed-zero",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-0.0),
            "min_y": f64_bits(-0.0),
            "min_z": f64_bits(-0.0),
            "max_x": f64_bits(1.0),
            "max_y": f64_bits(1.0),
            "max_z": f64_bits(1.0),
            "steps": [
                {
                    "op": "intersect",
                    "box": {
                        "min_x": f64_bits(0.0),
                        "min_y": f64_bits(0.0),
                        "min_z": f64_bits(0.0),
                        "max_x": f64_bits(0.0),
                        "max_y": f64_bits(0.0),
                        "max_z": f64_bits(0.0),
                    },
                },
            ],
        }),
    ));
    out
}

fn chain() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/intersect/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "intersect",
                        "box": {
                            "min_x": f64_bits(-0.5),
                            "min_y": f64_bits(0.25),
                            "min_z": f64_bits(0.25),
                            "max_x": f64_bits(0.75),
                            "max_y": f64_bits(1.5),
                            "max_z": f64_bits(0.75),
                        },
                    },
                    {
                        "op": "intersect",
                        "box": {
                            "min_x": f64_bits(0.5),
                            "min_y": f64_bits(-1.0),
                            "min_z": f64_bits(0.5),
                            "max_x": f64_bits(2.0),
                            "max_y": f64_bits(0.5),
                            "max_z": f64_bits(2.0),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn minmax_contained() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/minmax/contained",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "minmax",
                        "box": {
                            "min_x": f64_bits(0.25),
                            "min_y": f64_bits(0.25),
                            "min_z": f64_bits(0.25),
                            "max_x": f64_bits(0.75),
                            "max_y": f64_bits(0.75),
                            "max_z": f64_bits(0.75),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn minmax_partial() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/minmax/partial",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "minmax",
                        "box": {
                            "min_x": f64_bits(-0.5),
                            "min_y": f64_bits(0.25),
                            "min_z": f64_bits(-1.0),
                            "max_x": f64_bits(0.5),
                            "max_y": f64_bits(1.5),
                            "max_z": f64_bits(0.5),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn disjoint() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/minmax/disjoint",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "minmax",
                        "box": {
                            "min_x": f64_bits(2.0),
                            "min_y": f64_bits(2.0),
                            "min_z": f64_bits(2.0),
                            "max_x": f64_bits(3.0),
                            "max_y": f64_bits(3.0),
                            "max_z": f64_bits(3.0),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn minmax_touching_face() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/minmax/touching-face",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "minmax",
                        "box": {
                            "min_x": f64_bits(1.0),
                            "min_y": f64_bits(0.25),
                            "min_z": f64_bits(0.25),
                            "max_x": f64_bits(2.0),
                            "max_y": f64_bits(0.75),
                            "max_z": f64_bits(0.75),
                        },
                    },
                ],
            }),
        ]),
    ));
    out
}

fn minmax_signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/minmax/signed-zero",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-0.0),
            "min_y": f64_bits(-0.0),
            "min_z": f64_bits(-0.0),
            "max_x": f64_bits(1.0),
            "max_y": f64_bits(1.0),
            "max_z": f64_bits(1.0),
            "steps": [
                {
                    "op": "minmax",
                    "box": {
                        "min_x": f64_bits(0.0),
                        "min_y": f64_bits(0.0),
                        "min_z": f64_bits(0.0),
                        "max_x": f64_bits(0.0),
                        "max_y": f64_bits(0.0),
                        "max_z": f64_bits(0.0),
                    },
                },
            ],
        }),
    ));
    out
}

fn minmax_chain() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/minmax/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "minmax",
                        "box": {
                            "min_x": f64_bits(-0.5),
                            "min_y": f64_bits(0.25),
                            "min_z": f64_bits(0.25),
                            "max_x": f64_bits(0.75),
                            "max_y": f64_bits(1.5),
                            "max_z": f64_bits(0.75),
                        },
                    },
                    {
                        "op": "minmax",
                        "box": {
                            "min_x": f64_bits(-1.0),
                            "min_y": f64_bits(-1.0),
                            "min_z": f64_bits(-1.0),
                            "max_x": f64_bits(0.5),
                            "max_y": f64_bits(0.5),
                            "max_z": f64_bits(2.0),
                        },
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/move/positive",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "move", "x": f64_bits(1.0), "y": f64_bits(2.0), "z": f64_bits(3.0)}]}),
        ]),
    ));
    out
}

fn negative() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/move/negative",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "move", "x": f64_bits(-1.0), "y": f64_bits(-2.0), "z": f64_bits(-3.0)}]}),
        ]),
    ));
    out.push(case(
        "aabb/move/mixed",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "move", "x": f64_bits(-0.25), "y": f64_bits(0.5), "z": f64_bits(1.25)}]}),
        ]),
    ));
    out.push(case(
        "aabb/move/signed-zero",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-0.0),
            "min_y": f64_bits(-0.0),
            "min_z": f64_bits(-0.0),
            "max_x": f64_bits(1.0),
            "max_y": f64_bits(1.0),
            "max_z": f64_bits(1.0),
            "steps": [{"op": "move", "x": f64_bits(-0.0), "y": f64_bits(0.0), "z": f64_bits(-0.0)}],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        contained(),
        partial(),
        disjoint_normalizes(),
        touching_face(),
        signed_zero(),
        chain(),
        minmax_contained(),
        minmax_partial(),
        disjoint(),
        minmax_touching_face(),
        minmax_signed_zero(),
        minmax_chain(),
        negative(),
    ]
    .concat()
}
