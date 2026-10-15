//! Cases for `shape/basic`, `shape/move-overload`, `shape/optimize`, `shape/edges` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn block() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/basic/block",
        "math",
        "shape",
        json!({"kind": "single", "box": box_bits(0, 0, 0, 1, 1, 1), "steps": []}),
    ));
    out.push(case(
        "shape/basic/move",
        "math",
        "shape",
        json!({
            "kind": "single",
            "box": box_bits(-0.25, 0.125, 0.0, 0.75, 0.875, 1.25),
            "steps": [{"op": "move", "dx": f64_bits(0.5), "dy": f64_bits(-0.125), "dz": f64_bits(2)}],
        }),
    ));
    out.push(case(
        "shape/move-overload/vec3",
        "math",
        "shape",
        json!({
            "kind": "single",
            "box": box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875),
            "steps": [{"op": "move_vec3", "delta": {"x": f64_bits(-0.5), "y": f64_bits(1.25), "z": f64_bits(-1.75)}}],
        }),
    ));
    out.push(case(
        "shape/move-overload/block-pos",
        "math",
        "shape",
        json!({
            "kind": "single",
            "box": box_bits(-3.5, -2.25, 4.75, -1.0, 0.5, 6.25),
            "steps": [{"op": "move_block_pos", "delta": {"x": "-2", "y": "3", "z": "4"}}],
        }),
    ));
    out
}

fn empty() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/move-overload/empty",
        "math",
        "shape",
        json!({
            "kind": "single",
            "box": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "steps": [
                {
                    "op": "move_vec3",
                    "delta": {"x": f64_bits(f64::INFINITY), "y": f64_bits(f64::NEG_INFINITY), "z": f64_bits(f64::NAN)},
                },
                {"op": "move_block_pos", "delta": {"x": "2147483647", "y": "-2147483648", "z": "0"}},
            ],
        }),
    ));
    out.push(case(
        "shape/optimize/empty",
        "math",
        "shape",
        json!({"kind": "optimize", "shapes": [box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)]}),
    ));
    out.push(case(
        "shape/optimize/touching-row",
        "math",
        "shape",
        json!({
            "kind": "optimize",
            "shapes": [
                box_bits(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
                box_bits(0.25, 0.0, 0.0, 0.5, 1.0, 1.0),
                box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
            ],
        }),
    ));
    out
}

fn disjoint() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/optimize/disjoint",
        "math",
        "shape",
        json!({
            "kind": "optimize",
            "shapes": [
                box_bits(0.0, 0.0, 0.0, 0.25, 0.5, 0.75),
                box_bits(0.5, 0.25, 0.25, 0.75, 1.0, 0.5),
                box_bits(0.75, 0.0, 0.75, 1.0, 0.25, 1.0),
            ],
        }),
    ));
    out.push(case(
        "shape/edges/empty",
        "math",
        "shape",
        json!({"kind": "edges", "shapes": [box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)]}),
    ));
    out.push(case(
        "shape/edges/box",
        "math",
        "shape",
        json!({"kind": "edges", "shapes": [box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875)]}),
    ));
    out.push(case(
        "shape/edges/join",
        "math",
        "shape",
        json!({
            "kind": "edges",
            "shapes": [
                box_bits(0.0, 0.0, 0.0, 0.5, 0.5, 0.5),
                box_bits(0.5, 0.25, 0.25, 1.0, 0.75, 0.75),
                box_bits(0.25, 0.5, 0.5, 0.75, 1.0, 1.0),
            ],
        }),
    ));
    out
}

fn touching() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/join-coords/touching",
        "math",
        "shape",
        json!({
            "kind": "join_coords",
            "op": "or",
            "first": box_bits(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
            "second": box_bits(0.25, 0.0, 0.0, 0.5, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/join-coords/hole",
        "math",
        "shape",
        json!({
            "kind": "join_coords",
            "op": "only_first",
            "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "second": box_bits(0.25, 0.0, 0.0, 0.75, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/join-coords/empty-first",
        "math",
        "shape",
        json!({
            "kind": "join_coords",
            "op": "or",
            "first": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "second": box_bits(2.0, -1.0, 3.0, 2.5, 0.5, 4.0),
        }),
    ));
    out.push(case(
        "shape/boxes/empty",
        "math",
        "shape",
        json!({"kind": "boxes", "shapes": [box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)]}),
    ));
    out.push(case(
        "shape/boxes/box",
        "math",
        "shape",
        json!({"kind": "boxes", "shapes": [box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875)]}),
    ));
    out
}

fn join() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/boxes/join",
        "math",
        "shape",
        json!({
            "kind": "boxes",
            "shapes": [
                box_bits(0.0, 0.0, 0.0, 0.25, 0.5, 0.75),
                box_bits(0.5, 0.25, 0.25, 0.75, 1.0, 0.5),
                box_bits(0.75, 0.0, 0.75, 1.0, 0.25, 1.0),
            ],
        }),
    ));
    out.push(case(
        "shape/create-aabb/reversed",
        "math",
        "shape",
        json!({"kind": "create_aabb", "aabb": box_bits(0.75, 0.5, 0.875, 0.125, 0.25, 0.375)}),
    ));
    out.push(case(
        "shape/create-aabb/empty",
        "math",
        "shape",
        json!({"kind": "create_aabb", "aabb": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)}),
    ));
    out.push(case(
        "shape/create-aabb/array",
        "math",
        "shape",
        json!({"kind": "create_aabb", "aabb": box_bits(-1.25, 0.125, 2.0, 1.5, 0.875, 3.25)}),
    ));
    out.push(case(
        "shape/create-coords/cube",
        "math",
        "shape",
        json!({"kind": "create_coords", "box": box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875)}),
    ));
    out.push(case(
        "shape/create-coords/reversed",
        "math",
        "shape",
        json!({"kind": "create_coords", "box": box_bits(0.75, 0.5, 0.875, 0.125, 0.25, 0.375)}),
    ));
    out
}

fn array() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/create-coords/array",
        "math",
        "shape",
        json!({"kind": "create_coords", "box": box_bits(-1.25, 0.125, 2.0, 1.5, 0.875, 3.25)}),
    ));
    out.push(case("shape/infinity/snapshot", "math", "shape", json!({"kind": "infinity", "view": "snapshot"})));
    out.push(case("shape/infinity/boxes", "math", "shape", json!({"kind": "infinity", "view": "boxes"})));
    out.push(case(
        "shape/infinity/not-same-self",
        "math",
        "shape",
        json!({"kind": "infinity", "view": "not_same_self"}),
    ));
    out.push(case("shape/constants/epsilon", "math", "shape", json!({"kind": "constants", "name": "epsilon"})));
    out.push(case("shape/constants/big-epsilon", "math", "shape", json!({"kind": "constants", "name": "big_epsilon"})));
    out.push(case("shape/constants/both", "math", "shape", json!({"kind": "constants", "name": "both"})));
    out.push(case(
        "shape/empty/sliver",
        "math",
        "shape",
        json!({"kind": "single", "box": box_bits(0, 0, 0, 5e-08, 1, 1), "steps": []}),
    ));
    out.push(case(
        "shape/or/disjoint",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "or",
            "first": box_bits(0, 0, 0, 0.25, 1, 1),
            "second": box_bits(0.75, 0, 0, 1, 1, 1),
        }),
    ));
    out.push(case(
        "shape/or/touching",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "or",
            "first": box_bits(0, 0, 0, 0.5, 1, 1),
            "second": box_bits(0.5, 0, 0, 1, 1, 1),
        }),
    ));
    out
}

fn overlap() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/and/overlap",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "and",
            "first": box_bits(0, 0, 0, 0.75, 1, 1),
            "second": box_bits(0.25, 0, 0, 1, 1, 1),
        }),
    ));
    out.push(case(
        "shape/and/disjoint",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "and",
            "first": box_bits(0, 0, 0, 0.25, 1, 1),
            "second": box_bits(0.75, 0, 0, 1, 1, 1),
        }),
    ));
    out.push(case(
        "shape/only-first/hole",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "only_first",
            "first": box_bits(0, 0, 0, 0.75, 1, 1),
            "second": box_bits(0.25, 0, 0, 0.5, 1, 1),
        }),
    ));
    out.push(case(
        "shape/only-second/disjoint",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "only_second",
            "first": box_bits(0, 0, 0, 0.25, 1, 1),
            "second": box_bits(0.75, 0, 0, 1, 1, 1),
        }),
    ));
    out
}

fn false_overlap() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/false/overlap",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "false",
            "first": box_bits(0, 0, 0, 0.75, 1, 1),
            "second": box_bits(0.25, 0, 0, 1, 1, 1),
        }),
    ));
    out.push(case(
        "shape/not-same/overlap",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "not_same",
            "first": box_bits(0, 0, 0, 0.75, 1, 1),
            "second": box_bits(0.25, 0, 0, 1, 1, 1),
        }),
    ));
    out.push(case(
        "shape/second/displaced",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "second",
            "first": box_bits(0, 0, 0, 0.25, 1, 1),
            "second": box_bits(0.75, 0, 0, 1, 1, 1),
        }),
    ));
    out.push(case(
        "shape/first/displaced",
        "math",
        "shape",
        json!({
            "kind": "join",
            "op": "first",
            "first": box_bits(0.75, 0, 0, 1, 1, 1),
            "second": box_bits(0, 0, 0, 0.25, 1, 1),
        }),
    ));
    out
}

fn touching_row() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/or-many/touching-row",
        "math",
        "shape",
        json!({
            "kind": "join_many",
            "shapes": [
                box_bits(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
                box_bits(0.25, 0.0, 0.0, 0.5, 1.0, 1.0),
                box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
            ],
        }),
    ));
    out.push(case(
        "shape/or-many/disjoint",
        "math",
        "shape",
        json!({
            "kind": "join_many",
            "shapes": [
                box_bits(0.0, 0.0, 0.0, 0.25, 0.25, 0.25),
                box_bits(0.5, 0.5, 0.5, 0.75, 0.75, 0.75),
                box_bits(0.75, 0.0, 0.75, 1.0, 0.25, 1.0),
            ],
        }),
    ));
    out.push(case(
        "shape/or-many/empty-first",
        "math",
        "shape",
        json!({
            "kind": "join_many",
            "shapes": [
                box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
                box_bits(0.25, 0.0, 0.0, 0.5, 1.0, 1.0),
                box_bits(0.5, 0.0, 0.0, 0.75, 1.0, 1.0),
            ],
        }),
    ));
    out.push(case(
        "shape/single-encompassing/empty",
        "math",
        "shape",
        json!({"kind": "single_encompassing", "shapes": [box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0)]}),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [block(), empty(), disjoint(), touching(), join(), array(), overlap(), false_overlap(), touching_row()].concat()
}
