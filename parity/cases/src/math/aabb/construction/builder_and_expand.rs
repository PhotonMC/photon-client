//! Cases for `aabb-builder/empty`, `aabb-builder/single`, `aabb-builder/multiple`, `aabb-builder/float-precision` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn empty() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case("aabb-builder/empty", "math", "aabb_builder", json!({"points": []})));
    out.push(case(
        "aabb-builder/single",
        "math",
        "aabb_builder",
        json!({"points": [vector_bits_f32(1.25, -2.5, 3.75)]}),
    ));
    out.push(case(
        "aabb-builder/multiple",
        "math",
        "aabb_builder",
        json!({
            "points": [
                vector_bits_f32(3.5, -1.25, 8.0),
                vector_bits_f32(-4.0, 2.5, -6.75),
                vector_bits_f32(1.0, -3.0, 4.25),
            ],
        }),
    ));
    out.push(case(
        "aabb-builder/float-precision",
        "math",
        "aabb_builder",
        json!({
            "points": [
                vector_bits_f32(0.2, -0.3, 123456.78),
                vector_bits_f32(-0.3333333333333333, 0.14285714285714285, -0.0),
            ],
        }),
    ));
    out.push(case(
        "aabb-builder/special-values",
        "math",
        "aabb_builder",
        json!({
            "points": [
                vector_bits_f32(-0.0, 0.0, -0.0),
                vector_bits_f32(0.0, -0.0, 0.0),
                vector_bits_f32(f64::NAN, f64::INFINITY, f64::NEG_INFINITY),
            ],
        }),
    ));
    out
}

fn block_pos() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/constructor/block-pos",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "from_block_pos", "position": block_position(-2, 3, 4)},
                    {"op": "from_block_pos", "position": block_position(2147483647, -2147483648, 0)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/constructor/vec3",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "from_vec3", "begin": vector_bits(3.0, -2.0, 1.0), "end": vector_bits(-4.0, 5.0, -6.0)},
                    {"op": "from_vec3", "begin": vector_bits(-0.0, 0.0, -0.0), "end": vector_bits(0.0, -0.0, 0.0)},
                    {
                        "op": "from_vec3",
                        "begin": vector_bits(f64::INFINITY, f64::NEG_INFINITY, f64::NAN),
                        "end": vector_bits(0.0, 1.0, 2.0),
                    },
                ],
            }),
        ]),
    ));
    out
}

fn unit_cube() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/constructor/unit-cube",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "unit_cube_from_lower_corner", "position": vector_bits(1.25, -2.5, 0.125)},
                    {"op": "unit_cube_from_lower_corner", "position": vector_bits(-0.0, 0.0, -0.0)},
                    {
                        "op": "unit_cube_from_lower_corner",
                        "position": vector_bits(f64::INFINITY, f64::NEG_INFINITY, f64::NAN),
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/constructor/full-blocks",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "encapsulating_full_blocks",
                        "first": block_position(-3, 4, 8),
                        "second": block_position(2, -1, 5),
                    },
                    {
                        "op": "encapsulating_full_blocks",
                        "first": block_position(10, 20, 30),
                        "second": block_position(-10, -20, -30),
                    },
                ],
            }),
        ]),
    ));
    out
}

fn full_blocks_overflow() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/constructor/full-blocks-overflow",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "encapsulating_full_blocks",
                        "first": block_position(2147483647, 2147483647, 2147483647),
                        "second": block_position(2147483647, 2147483647, 2147483647),
                    },
                    {
                        "op": "encapsulating_full_blocks",
                        "first": block_position(-2147483648, -2147483648, -2147483648),
                        "second": block_position(-2147483648, -2147483648, -2147483648),
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/constructor/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "from_block_pos", "position": block_position(2, 3, 4)},
                    {"op": "unit_cube_from_lower_corner", "position": vector_bits(-1.5, 0.25, 2.75)},
                    {"op": "from_vec3", "begin": vector_bits(5.0, 6.0, 7.0), "end": vector_bits(1.0, 2.0, 3.0)},
                    {
                        "op": "encapsulating_full_blocks",
                        "first": block_position(-2, -1, 0),
                        "second": block_position(4, 5, 6),
                    },
                ],
            }),
        ]),
    ));
    out
}

fn expand_deflate_unit_box() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/expand-deflate/unit",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "expand", "x": f64_bits(1.0), "y": f64_bits(-0.5), "z": f64_bits(0.0)},
                    {"op": "deflate", "amount": f64_bits(0.5)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/expand-overload/vec3",
        "math",
        "aabb",
        merged(vec![unit(), json!({"steps": [{"op": "expand_vec3", "delta": vector_bits(1.0, -0.5, 0.0)}]})]),
    ));
    out.push(case(
        "aabb/expand-overload/signed-zero",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(-0.0),
            "min_y": f64_bits(-0.0),
            "min_z": f64_bits(-0.0),
            "max_x": f64_bits(1.0),
            "max_y": f64_bits(1.0),
            "max_z": f64_bits(1.0),
            "steps": [{"op": "expand_vec3", "delta": vector_bits(-0.0, 0.0, -0.0)}],
        }),
    ));
    out
}

fn chain() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/expand-overload/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "expand_vec3", "delta": vector_bits(-0.25, 0.5, 1.0)},
                    {"op": "expand_vec3", "delta": vector_bits(0.125, -0.75, -0.5)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/contract/positive",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "contract", "x": f64_bits(0.25), "y": f64_bits(0.5), "z": f64_bits(1.0)}]}),
        ]),
    ));
    out.push(case(
        "aabb/contract/negative",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "contract", "x": f64_bits(-0.25), "y": f64_bits(-0.5), "z": f64_bits(-1.0)}]}),
        ]),
    ));
    out.push(case(
        "aabb/contract/mixed",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "contract", "x": f64_bits(-0.25), "y": f64_bits(0.0), "z": f64_bits(0.5)}]}),
        ]),
    ));
    out
}

fn signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/contract/signed-zero",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "contract", "x": f64_bits(-0.0), "y": f64_bits(0.0), "z": f64_bits(-0.0)}]}),
        ]),
    ));
    out.push(case(
        "aabb/contract/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "contract", "x": f64_bits(0.25), "y": f64_bits(-0.25), "z": f64_bits(0.0)},
                    {"op": "contract", "x": f64_bits(-0.125), "y": f64_bits(0.125), "z": f64_bits(-0.5)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/inflate/scalar-positive",
        "math",
        "aabb",
        merged(vec![unit(), json!({"steps": [{"op": "inflate", "amount": f64_bits(0.25)}]})]),
    ));
    out.push(case(
        "aabb/inflate/vector-mixed",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "inflate_xyz", "x": f64_bits(0.25), "y": f64_bits(-0.5), "z": f64_bits(0.75)}]}),
        ]),
    ));
    out.push(case(
        "aabb/inflate/scalar-negative",
        "math",
        "aabb",
        merged(vec![unit(), json!({"steps": [{"op": "inflate", "amount": f64_bits(-0.75)}]})]),
    ));
    out
}

fn inflate_signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/inflate/signed-zero",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "inflate_xyz", "x": f64_bits(-0.0), "y": f64_bits(0.0), "z": f64_bits(-0.0)}]}),
        ]),
    ));
    out.push(case(
        "aabb/inflate/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "inflate_xyz", "x": f64_bits(0.25), "y": f64_bits(0.5), "z": f64_bits(0.0)},
                    {"op": "inflate", "amount": f64_bits(-0.125)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/deflate/vector-mixed",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "deflate_xyz", "x": f64_bits(0.25), "y": f64_bits(-0.5), "z": f64_bits(0.75)}]}),
        ]),
    ));
    out.push(case(
        "aabb/deflate/vector-reversed",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "deflate_xyz", "x": f64_bits(2.0), "y": f64_bits(2.0), "z": f64_bits(2.0)}]}),
        ]),
    ));
    out
}

fn deflate_signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/deflate/signed-zero",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [{"op": "deflate_xyz", "x": f64_bits(-0.0), "y": f64_bits(0.0), "z": f64_bits(-0.0)}]}),
        ]),
    ));
    out.push(case(
        "aabb/deflate/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "deflate_xyz", "x": f64_bits(0.25), "y": f64_bits(0.5), "z": f64_bits(0.0)},
                    {"op": "deflate", "amount": f64_bits(-0.125)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/deflate/special-values",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "deflate_xyz",
                        "x": f64_bits(f64::NAN),
                        "y": f64_bits(f64::INFINITY),
                        "z": f64_bits(f64::NEG_INFINITY),
                    },
                ],
            }),
        ]),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        empty(),
        block_pos(),
        unit_cube(),
        full_blocks_overflow(),
        expand_deflate_unit_box(),
        chain(),
        signed_zero(),
        inflate_signed_zero(),
        deflate_signed_zero(),
    ]
    .concat()
}
