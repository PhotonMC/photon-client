//! Cases for `aabb/contains`, `aabb/intersects`, `aabb/predicate-overload`, `aabb/collided-along-vector` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn contains_unit_box() -> Vec<Value> {
    let mut out = Vec::new();
    let inside = json!({"op": "contains", "x": f64_bits(0.5), "y": f64_bits(0.5), "z": f64_bits(0.5)});
    let outside = json!({"op": "contains", "x": f64_bits(2.0), "y": f64_bits(0.0), "z": f64_bits(0.0)});
    out.push(case(
        "aabb/contains/unit",
        "math",
        "aabb",
        merged(vec![unit(), json!({"steps": [inside.clone(), outside.clone()]})]),
    ));
    out.push(case(
        "aabb/intersects/overlap",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "intersects", "box": unit()},
                    {
                        "op": "intersects",
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

fn basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/predicate-overload/basic",
        "math",
        "aabb",
        merged(vec![
            box_bits(-2.0, -2.0, -2.0, 2.0, 2.0, 2.0),
            json!({
                "steps": [
                    {"op": "contains_vec3", "position": vector_bits(0.5, -0.5, 1.5)},
                    merged(vec![json!({"op": "intersects_coords"}), box_bits(-1.0, -1.0, -1.0, 0.0, 0.0, 0.0)]),
                    {"op": "intersects_vec3", "min": vector_bits(1.0, 1.0, 1.0), "max": vector_bits(-1.0, -1.0, -1.0)},
                    {"op": "intersects_block_pos", "position": block_position(-1, -1, -1)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/predicate-overload/contains-boundaries",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "contains_vec3", "position": vector_bits(0.0, 0.0, 0.0)},
                    {"op": "contains_vec3", "position": vector_bits(1.0, 0.5, 0.5)},
                    {"op": "contains_vec3", "position": vector_bits(f64::NAN, 0.5, 0.5)},
                    {"op": "contains_vec3", "position": vector_bits(f64::INFINITY, 0.5, 0.5)},
                ],
            }),
        ]),
    ));
    out
}

fn raw_coordinates() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/predicate-overload/raw-coordinates",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    merged(vec![json!({"op": "intersects_coords"}), box_bits(-0.5, -0.5, -0.5, 0.5, 0.5, 0.5)]),
                    merged(vec![json!({"op": "intersects_coords"}), box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)]),
                    merged(vec![json!({"op": "intersects_coords"}), box_bits(0.5, 0.5, 0.5, -0.5, -0.5, -0.5)]),
                    merged(vec![json!({"op": "intersects_coords"}), box_bits(f64::NAN, 0.0, 0.0, 1.0, 1.0, 1.0)]),
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/predicate-overload/vec3-normalizes",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "intersects_vec3", "min": vector_bits(1.0, 1.0, 1.0), "max": vector_bits(-1.0, -1.0, -1.0)},
                    {"op": "intersects_vec3", "min": vector_bits(0.0, -0.0, 0.0), "max": vector_bits(-0.0, 0.0, -0.0)},
                    {
                        "op": "intersects_vec3",
                        "min": vector_bits(f64::NAN, 0.0, 0.0),
                        "max": vector_bits(1.0, 1.0, 1.0),
                    },
                ],
            }),
        ]),
    ));
    out
}

fn block_pos_boundaries() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/predicate-overload/block-pos-boundaries",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-2147483648.0),
            "min_y": f64_bits(0.0),
            "min_z": f64_bits(0.0),
            "max_x": f64_bits(-2147483647.0),
            "max_y": f64_bits(1.0),
            "max_z": f64_bits(1.0),
            "steps": [
                {"op": "intersects_block_pos", "position": block_position(-2147483648, 0, 0)},
                {"op": "intersects_block_pos", "position": block_position(2147483647, 0, 0)},
            ],
        }),
    ));
    out.push(case(
        "aabb/predicate-overload/special-chain",
        "math",
        "aabb",
        merged(vec![
            box_bits(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0),
            json!({
                "steps": [
                    merged(vec![
                        json!({"op": "intersects_coords"}),
                        box_bits(f64::INFINITY, -1.0, -1.0, f64::INFINITY, 1.0, 1.0),
                    ]),
                    {"op": "intersects_vec3", "min": vector_bits(-2.0, -2.0, -2.0), "max": vector_bits(2.0, 2.0, 2.0)},
                    {"op": "contains_vec3", "position": vector_bits(-1.0, 0.0, 0.0)},
                    {"op": "intersects_block_pos", "position": block_position(0, 0, 0)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/collided-along-vector/empty",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "collided_along_vector", "vector": vector_bits(1.0, 0.0, 0.0), "boxes": []}]}),
        ]),
    ));
    out
}

fn from_contained() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/collided-along-vector/from-contained",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(0.0, 0.0, 0.0),
                        "boxes": [box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75)],
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/collided-along-vector/to-contained",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(2.0, 0.0, 0.0),
                        "boxes": [box_bits(2.5, 0.0, 0.0, 3.5, 1.0, 1.0)],
                    },
                ],
            }),
        ]),
    ));
    out
}

fn crossing() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/collided-along-vector/crossing",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(3.0, 0.0, 0.0),
                        "boxes": [box_bits(1.5, 0.0, 0.0, 2.0, 1.0, 1.0)],
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/collided-along-vector/miss",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(3.0, 0.0, 0.0),
                        "boxes": [box_bits(1.5, 2.0, 0.0, 2.0, 3.0, 1.0)],
                    },
                ],
            }),
        ]),
    ));
    out
}

fn many() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/collided-along-vector/many",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(3.0, 0.0, 0.0),
                        "boxes": [box_bits(1.5, 2.0, 0.0, 2.0, 3.0, 1.0), box_bits(1.5, 0.0, 0.0, 2.0, 1.0, 1.0)],
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/collided-along-vector/epsilon-special",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(0.0, 0.0, 0.0),
                        "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)],
                    },
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(2e-07, 0.0, 0.0),
                        "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)],
                    },
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(f64::NAN, 0.0, 0.0),
                        "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)],
                    },
                ],
            }),
        ]),
    ));
    out
}

fn tiny_moving_box() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/collided-along-vector/tiny-moving-box",
        "math",
        "aabb",
        merged(vec![
            box_bits(0.0, 0.0, 0.0, 1e-08, 1e-08, 1e-08),
            json!({
                "steps": [
                    {
                        "op": "collided_along_vector",
                        "vector": vector_bits(1.0, 0.0, 0.0),
                        "boxes": [box_bits(0.5, 0.0, 0.0, 0.6, 1.0, 1.0)],
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/equality/identical",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "equals", "box": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)}, {"op": "hash_code"}]}),
        ]),
    ));
    out.push(case(
        "aabb/equality/different-endpoint",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "equals", "box": box_bits(0.0, 0.0, 0.0, 2.0, 1.0, 1.0)}, {"op": "hash_code"}]}),
        ]),
    ));
    out
}

fn signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/equality/signed-zero",
        "math",
        "aabb",
        merged(vec![
            raw_box(f64_bits(-0.0), f64_bits(0.0), f64_bits(0.0), f64_bits(1.0), f64_bits(1.0), f64_bits(1.0)),
            json!({
                "steps": [
                    {
                        "op": "equals",
                        "box": raw_box(
                            f64_bits(0.0),
                            f64_bits(0.0),
                            f64_bits(0.0),
                            f64_bits(1.0),
                            f64_bits(1.0),
                            f64_bits(1.0),
                        ),
                    },
                    {"op": "hash_code"},
                ],
            }),
        ]),
    ));
    out
}

fn nan_payload() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/equality/nan-payload",
        "math",
        "aabb",
        merged(vec![
            raw_box(
                "7ff8000000000000",
                f64_bits(0.0),
                f64_bits(0.0),
                f64_bits(f64::INFINITY),
                f64_bits(1.0),
                f64_bits(1.0),
            ),
            json!({
                "steps": [
                    {
                        "op": "equals",
                        "box": raw_box(
                            "7ff8000000000001",
                            f64_bits(0.0),
                            f64_bits(0.0),
                            f64_bits(f64::INFINITY),
                            f64_bits(1.0),
                            f64_bits(1.0),
                        ),
                    },
                    {"op": "hash_code"},
                ],
            }),
        ]),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        contains_unit_box(),
        basic(),
        raw_coordinates(),
        block_pos_boundaries(),
        from_contained(),
        crossing(),
        many(),
        tiny_moving_box(),
        signed_zero(),
        nan_payload(),
    ]
    .concat()
}
