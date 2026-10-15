//! Cases for `aabb/move`, `aabb/move-overload`, `aabb/of-size`, `aabb/set-endpoint`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn large_delta() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/move/large-delta",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "move",
                        "x": f64_bits(9007199254740992.0),
                        "y": f64_bits(-9007199254740992.0),
                        "z": f64_bits(0.5),
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/move/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "move", "x": f64_bits(0.25), "y": f64_bits(-0.5), "z": f64_bits(1.0)},
                    {"op": "move", "x": f64_bits(-0.125), "y": f64_bits(0.25), "z": f64_bits(-0.5)},
                ],
            }),
        ]),
    ));
    out
}

fn basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/move-overload/basic",
        "math",
        "aabb",
        merged(vec![
            box_bits(-1.25, 2.5, -3.75, 4.5, 5.75, 6.25),
            json!({
                "steps": [
                    {"op": "move_block_pos", "position": block_position(3, -2, 5)},
                    {"op": "move_vec3", "position": vector_bits(0.125, -0.25, 1.5)},
                    {"op": "move_vector3f", "position": vector_bits_f32(0.2, -0.3, 0.4)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/move-overload/block-pos-extremes",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "move_block_pos", "position": block_position(2147483647, -2147483648, -2147483647)},
                    {"op": "move_block_pos", "position": block_position(-2147483648, 2147483647, 2147483647)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/move-overload/vec3-special",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "move_vec3", "position": vector_bits(f64::INFINITY, f64::NEG_INFINITY, f64::NAN)},
                    {"op": "move_vec3", "position": vector_bits(f64::NAN, 0.5, f64::INFINITY)},
                ],
            }),
        ]),
    ));
    out
}

fn vector3f_precision() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/move-overload/vector3f-precision",
        "math",
        "aabb",
        merged(vec![
            box_bits(-100.0, -200.0, -300.0, 400.0, 500.0, 600.0),
            json!({
                "steps": [
                    {"op": "move_vector3f", "position": vector_bits_f32(0.2, -0.3, 123456.78)},
                    {
                        "op": "move_vector3f",
                        "position": vector_bits_f32(-0.3333333333333333, 0.14285714285714285, -0.0),
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/move-overload/signed-zero",
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
                {"op": "move_block_pos", "position": block_position(0, 0, 0)},
                {"op": "move_vec3", "position": vector_bits(-0.0, 0.0, -0.0)},
                {"op": "move_vector3f", "position": vector_bits_f32(-0.0, 0.0, -0.0)},
            ],
        }),
    ));
    out
}

fn chain() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/move-overload/chain",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "move_vec3", "position": vector_bits(0.25, -0.5, 1.0)},
                    {"op": "move_block_pos", "position": block_position(-2, 3, -4)},
                    {"op": "move_vector3f", "position": vector_bits_f32(0.75, 0.125, -0.25)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/of-size/unit",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "of_size",
                        "center": point(0.5, 0.5, 0.5),
                        "size_x": f64_bits(1.0),
                        "size_y": f64_bits(1.0),
                        "size_z": f64_bits(1.0),
                    },
                ],
            }),
        ]),
    ));
    out
}

fn asymmetric() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/of-size/asymmetric",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "of_size",
                        "center": point(-3.25, 4.5, 0.125),
                        "size_x": f64_bits(2.5),
                        "size_y": f64_bits(7.25),
                        "size_z": f64_bits(0.75),
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/of-size/negative-sizes",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "of_size",
                        "center": point(1.0, -2.0, 3.0),
                        "size_x": f64_bits(-4.0),
                        "size_y": f64_bits(-6.0),
                        "size_z": f64_bits(-8.0),
                    },
                ],
            }),
        ]),
    ));
    out
}

fn zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/of-size/zero",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "of_size",
                        "center": point(1.5, -2.25, 3.75),
                        "size_x": f64_bits(0.0),
                        "size_y": f64_bits(0.0),
                        "size_z": f64_bits(0.0),
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/of-size/signed-zero",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "of_size",
                        "center": point(-0.0, 0.0, -0.0),
                        "size_x": f64_bits(-0.0),
                        "size_y": f64_bits(0.0),
                        "size_z": f64_bits(-0.0),
                    },
                ],
            }),
        ]),
    ));
    out
}

fn special() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/of-size/special",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {
                        "op": "of_size",
                        "center": point(f64::INFINITY, f64::NEG_INFINITY, f64::NAN),
                        "size_x": f64_bits(2.0),
                        "size_y": f64_bits(f64::INFINITY),
                        "size_z": f64_bits(f64::NAN),
                    },
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/set-endpoint/basic",
        "math",
        "aabb",
        merged(vec![
            box_bits(-2.0, -3.0, -4.0, 5.0, 6.0, 7.0),
            json!({
                "steps": [
                    {"op": "set_min_x", "value": f64_bits(-1.5)},
                    {"op": "set_min_y", "value": f64_bits(-2.5)},
                    {"op": "set_min_z", "value": f64_bits(-3.5)},
                    {"op": "set_max_x", "value": f64_bits(4.5)},
                    {"op": "set_max_y", "value": f64_bits(5.5)},
                    {"op": "set_max_z", "value": f64_bits(6.5)},
                ],
            }),
        ]),
    ));
    out
}

fn min_chain() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/set-endpoint/min-chain",
        "math",
        "aabb",
        merged(vec![
            box_bits(0.0, 10.0, 20.0, 30.0, 40.0, 50.0),
            json!({
                "steps": [
                    {"op": "set_min_x", "value": f64_bits(1.0)},
                    {"op": "set_min_y", "value": f64_bits(11.0)},
                    {"op": "set_min_z", "value": f64_bits(21.0)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/set-endpoint/max-chain",
        "math",
        "aabb",
        merged(vec![
            box_bits(-30.0, -40.0, -50.0, 0.0, 10.0, 20.0),
            json!({
                "steps": [
                    {"op": "set_max_x", "value": f64_bits(-1.0)},
                    {"op": "set_max_y", "value": f64_bits(9.0)},
                    {"op": "set_max_z", "value": f64_bits(19.0)},
                ],
            }),
        ]),
    ));
    out
}

fn reversed_normalizes() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/set-endpoint/reversed-normalizes",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "set_min_x", "value": f64_bits(4.0)},
                    {"op": "set_min_y", "value": f64_bits(5.0)},
                    {"op": "set_min_z", "value": f64_bits(6.0)},
                    {"op": "set_max_x", "value": f64_bits(-4.0)},
                    {"op": "set_max_y", "value": f64_bits(-5.0)},
                    {"op": "set_max_z", "value": f64_bits(-6.0)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/set-endpoint/signed-zero",
        "math",
        "aabb",
        merged(vec![
            box_bits(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0),
            json!({
                "steps": [
                    {"op": "set_min_x", "value": f64_bits(-0.0)},
                    {"op": "set_max_x", "value": f64_bits(0.0)},
                    {"op": "set_min_y", "value": f64_bits(0.0)},
                    {"op": "set_max_y", "value": f64_bits(-0.0)},
                    {"op": "set_min_z", "value": f64_bits(-0.0)},
                    {"op": "set_max_z", "value": f64_bits(0.0)},
                ],
            }),
        ]),
    ));
    out
}

fn set_endpoint_special() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/set-endpoint/special",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "set_min_x", "value": f64_bits(f64::NAN)},
                    {"op": "set_max_y", "value": f64_bits(f64::INFINITY)},
                    {"op": "set_min_z", "value": f64_bits(f64::NEG_INFINITY)},
                    {"op": "set_max_x", "value": f64_bits(f64::NAN)},
                    {"op": "set_min_y", "value": f64_bits(f64::NEG_INFINITY)},
                    {"op": "set_max_z", "value": f64_bits(f64::INFINITY)},
                ],
            }),
        ]),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        large_delta(),
        basic(),
        vector3f_precision(),
        chain(),
        asymmetric(),
        zero(),
        special(),
        min_chain(),
        reversed_normalizes(),
        set_endpoint_special(),
    ]
    .concat()
}
