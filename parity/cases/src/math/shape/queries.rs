//! Cases for `shape/clip`, `shape/closest`, `shape/face`, `shape/occlusion`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

fn shape_clip_query(source: (f64, f64, f64), target: (f64, f64, f64), position: (i64, i64, i64)) -> Value {
    json!({
        "from": point(source.0, source.1, source.2),
        "to": point(target.0, target.1, target.2),
        "pos": block_position(position.0, position.1, position.2),
    })
}

fn empty() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/clip/empty",
        "math",
        "shape",
        json!({
            "kind": "clip",
            "shape": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "queries": [shape_clip_query((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 0, 0))],
        }),
    ));
    out.push(case(
        "shape/clip/short-segment",
        "math",
        "shape",
        json!({
            "kind": "clip",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "queries": [shape_clip_query((0.5, 0.5, 0.5), (0.5001, 0.5, 0.5), (0, 0, 0))],
        }),
    ));
    out.push(case(
        "shape/clip/inside-directions",
        "math",
        "shape",
        json!({
            "kind": "clip",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "queries": [
                shape_clip_query((0.5, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 0, 0)),
                shape_clip_query((0.5, 0.5, 0.5), (-1.0, 0.5, 0.5), (0, 0, 0)),
                shape_clip_query((0.5, 0.5, 0.5), (0.5, 2.0, 0.5), (0, 0, 0)),
                shape_clip_query((0.5, 0.5, 0.5), (0.5, -1.0, 0.5), (0, 0, 0)),
                shape_clip_query((0.5, 0.5, 0.5), (0.5, 0.5, 2.0), (0, 0, 0)),
                shape_clip_query((0.5, 0.5, 0.5), (0.5, 0.5, -1.0), (0, 0, 0)),
                shape_clip_query((0.5, 0.5, 0.5), (2.0, 2.0, 2.0), (0, 0, 0)),
            ],
        }),
    ));
    out
}

fn outside_offset() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/clip/outside-offset",
        "math",
        "shape",
        json!({
            "kind": "clip",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "queries": [shape_clip_query((5.0, -1.5, 5.5), (2.0, -1.5, 5.5), (3, -2, 5))],
        }),
    ));
    out.push(case(
        "shape/clip/partial-box",
        "math",
        "shape",
        json!({
            "kind": "clip",
            "shape": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
            "queries": [shape_clip_query((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 0, 0))],
        }),
    ));
    out.push(case(
        "shape/clip/joined-fallback",
        "math",
        "shape",
        json!({
            "kind": "clip_join",
            "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "second": box_bits(2.0, 0.0, 0.0, 3.0, 1.0, 1.0),
            "queries": [shape_clip_query((-1.0, 0.5, 0.5), (4.0, 0.5, 0.5), (0, 0, 0))],
        }),
    ));
    out.push(case(
        "shape/closest/empty",
        "math",
        "shape",
        json!({"kind": "closest", "shape": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0), "point": point(0.5, 0.5, 0.5)}),
    ));
    out.push(case(
        "shape/closest/inside",
        "math",
        "shape",
        json!({"kind": "closest", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "point": point(0.5, 0.5, 0.5)}),
    ));
    out
}

fn axis_clamp() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/closest/axis-clamp",
        "math",
        "shape",
        json!({
            "kind": "closest",
            "shape": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
            "point": point(-1.0, 0.5, 0.5),
        }),
    ));
    out.push(case(
        "shape/closest/diagonal-clamp",
        "math",
        "shape",
        json!({"kind": "closest", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "point": point(2.0, -1.0, 3.0)}),
    ));
    out.push(case(
        "shape/closest/join-tie",
        "math",
        "shape",
        json!({
            "kind": "closest_join",
            "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "second": box_bits(2.0, 0.0, 0.0, 3.0, 1.0, 1.0),
            "point": point(1.5, 0.5, 0.5),
        }),
    ));
    out.push(case(
        "shape/closest/translated-box",
        "math",
        "shape",
        json!({
            "kind": "closest",
            "shape": box_bits(-2.0, -3.0, 4.0, -1.0, -1.0, 6.0),
            "point": point(-4.0, -5.0, 8.0),
        }),
    ));
    out.push(case(
        "shape/face/empty",
        "math",
        "shape",
        json!({"kind": "face", "shape": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0), "direction": "east"}),
    ));
    out
}

fn block() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/face/block",
        "math",
        "shape",
        json!({"kind": "face", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "north"}),
    ));
    out.push(case(
        "shape/face/axis-full",
        "math",
        "shape",
        json!({"kind": "face", "shape": box_bits(0.0, 0.25, 0.0, 1.0, 0.75, 1.0), "direction": "east"}),
    ));
    out.push(case(
        "shape/face/west-slice",
        "math",
        "shape",
        json!({"kind": "face", "shape": box_bits(0.0, 0.25, 0.0, 0.75, 0.75, 1.0), "direction": "west"}),
    ));
    out.push(case(
        "shape/face/east-slice",
        "math",
        "shape",
        json!({"kind": "face", "shape": box_bits(0.25, 0.25, 0.0, 1.0, 0.75, 1.0), "direction": "east"}),
    ));
    out.push(case(
        "shape/face/missing-boundary",
        "math",
        "shape",
        json!({"kind": "face", "shape": box_bits(0.25, 0.25, 0.0, 0.75, 0.75, 1.0), "direction": "east"}),
    ));
    out.push(case(
        "shape/face/up-slice",
        "math",
        "shape",
        json!({"kind": "face", "shape": box_bits(0.25, 0.25, 0.25, 0.75, 1.0, 0.75), "direction": "up"}),
    ));
    out
}

fn joined() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/face/joined",
        "math",
        "shape",
        json!({
            "kind": "face_join",
            "first": box_bits(0.25, 0.0, 0.0, 1.0, 0.5, 1.0),
            "second": box_bits(0.25, 0.5, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out.push(case(
        "shape/occlusion/block-block-east",
        "math",
        "shape",
        json!({
            "kind": "block_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out.push(case(
        "shape/occlusion/gap",
        "math",
        "shape",
        json!({
            "kind": "block_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out.push(case(
        "shape/occlusion/partial-hole",
        "math",
        "shape",
        json!({
            "kind": "block_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out
}

fn partial_covered() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/occlusion/partial-covered",
        "math",
        "shape",
        json!({
            "kind": "block_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out.push(case(
        "shape/occlusion/negative-direction",
        "math",
        "shape",
        json!({
            "kind": "block_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "direction": "west",
        }),
    ));
    out.push(case(
        "shape/occlusion/empty-occluder",
        "math",
        "shape",
        json!({
            "kind": "block_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "direction": "up",
        }),
    ));
    out.push(case(
        "shape/occlusion/vertical-contact",
        "math",
        "shape",
        json!({
            "kind": "block_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 0.5),
            "occluder": box_bits(0.0, 0.0, 0.5, 1.0, 1.0, 1.0),
            "direction": "south",
        }),
    ));
    out
}

fn joined_shape() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/occlusion/joined-shape",
        "math",
        "shape",
        json!({
            "kind": "block_occludes_join",
            "first": box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0),
            "second": box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out.push(case(
        "shape/occlusion/face-empty",
        "math",
        "shape",
        json!({
            "kind": "face_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 5e-08, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 5e-08, 1.0),
        }),
    ));
    out.push(case(
        "shape/occlusion/face-block-first",
        "math",
        "shape",
        json!({
            "kind": "face_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
        }),
    ));
    out.push(case(
        "shape/occlusion/face-block-second",
        "math",
        "shape",
        json!({
            "kind": "face_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        }),
    ));
    out
}

fn face_complementary() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/occlusion/face-complementary",
        "math",
        "shape",
        json!({
            "kind": "face_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/occlusion/face-gap",
        "math",
        "shape",
        json!({
            "kind": "face_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.4, 1.0),
            "occluder": box_bits(0.0, 0.6, 0.0, 1.0, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/occlusion/face-depth-complementary",
        "math",
        "shape",
        json!({
            "kind": "face_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 0.5),
            "occluder": box_bits(0.0, 0.0, 0.5, 1.0, 1.0, 1.0),
        }),
    ));
    out.push(case(
        "shape/occlusion/merged-complementary",
        "math",
        "shape",
        json!({
            "kind": "merged_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out
}

fn merged_block_first() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/occlusion/merged-block-first",
        "math",
        "shape",
        json!({
            "kind": "merged_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "direction": "east",
        }),
    ));
    out.push(case(
        "shape/occlusion/merged-first-gap",
        "math",
        "shape",
        json!({
            "kind": "merged_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
            "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out.push(case(
        "shape/occlusion/merged-second-gap",
        "math",
        "shape",
        json!({
            "kind": "merged_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "occluder": box_bits(0.25, 0.5, 0.0, 1.0, 1.0, 1.0),
            "direction": "east",
        }),
    ));
    out.push(case(
        "shape/occlusion/merged-west",
        "math",
        "shape",
        json!({
            "kind": "merged_occludes",
            "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0),
            "direction": "west",
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        empty(),
        outside_offset(),
        axis_clamp(),
        block(),
        joined(),
        partial_covered(),
        joined_shape(),
        face_complementary(),
        merged_block_first(),
    ]
    .concat()
}
