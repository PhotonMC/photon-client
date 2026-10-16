//! Cases for `aabb/distance`, `aabb/clip`, `aabb/clip-coords`, `aabb/clip-many` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

/// A clip scenario: label, box, start, end and the expected steps.
type Triple = (f64, f64, f64);
type ClipCoordinates = (&'static str, [f64; 6], Triple, Triple, Vec<(Value, Value)>);

fn bounding_box(
    min_x: impl Into<Value>,
    min_y: impl Into<Value>,
    min_z: impl Into<Value>,
    max_x: impl Into<Value>,
    max_y: impl Into<Value>,
    max_z: impl Into<Value>,
) -> Value {
    let min_x: Value = min_x.into();
    let min_y: Value = min_y.into();
    let min_z: Value = min_z.into();
    let max_x: Value = max_x.into();
    let max_y: Value = max_y.into();
    let max_z: Value = max_z.into();
    json!({
        "min_x": (min_x).to_string(),
        "min_y": (min_y).to_string(),
        "min_z": (min_z).to_string(),
        "max_x": (max_x).to_string(),
        "max_y": (max_y).to_string(),
        "max_z": (max_z).to_string(),
    })
}

fn clip_many_step(
    boxes: impl Into<Value>,
    source: (f64, f64, f64),
    target: (f64, f64, f64),
    position: (i64, i64, i64),
) -> Value {
    let boxes: Value = boxes.into();
    json!({
        "op": "clip_many",
        "boxes": boxes.clone(),
        "from": point(source.0, source.1, source.2),
        "to": point(target.0, target.1, target.2),
        "pos": block_position(position.0, position.1, position.2),
    })
}

fn point_inside_boundary() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/distance/point-inside-boundary",
        "math",
        "aabb",
        merged(vec![
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            json!({
                "steps": [
                    {"op": "distance_point", "point": point(0.5, 0.5, 0.5)},
                    {"op": "distance_point", "point": point(0.0, 0.0, 0.0)},
                    {"op": "distance_point", "point": point(1.0, 1.0, 1.0)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/distance/point-outside",
        "math",
        "aabb",
        merged(vec![
            box_bits(-2.0, -1.0, 0.0, 3.0, 4.0, 5.0),
            json!({
                "steps": [
                    {"op": "distance_point", "point": point(5.0, -3.0, 7.0)},
                    {"op": "distance_point", "point": point(-4.0, 2.0, 1.0)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/distance/point-special",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    {"op": "distance_point", "point": point(f64::INFINITY, 0.5, 0.5)},
                    {"op": "distance_point", "point": point(f64::NEG_INFINITY, 0.5, 0.5)},
                    {"op": "distance_point", "point": point(f64::NAN, 0.5, 0.5)},
                ],
            }),
        ]),
    ));
    out
}

fn box_overlap_touch() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/distance/box-overlap-touch",
        "math",
        "aabb",
        merged(vec![
            box_bits(0.0, 0.0, 0.0, 2.0, 2.0, 2.0),
            json!({
                "steps": [
                    {"op": "distance_box", "box": box_bits(1.0, 1.0, 1.0, 3.0, 3.0, 3.0)},
                    {"op": "distance_box", "box": box_bits(2.0, 2.0, 2.0, 4.0, 4.0, 4.0)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/distance/box-separated-diagonal",
        "math",
        "aabb",
        merged(vec![
            box_bits(-2.0, -1.0, 0.0, -1.0, 1.0, 2.0),
            json!({
                "steps": [
                    {"op": "distance_box", "box": box_bits(2.0, 3.0, 5.0, 4.0, 6.0, 7.0)},
                    {"op": "distance_box", "box": box_bits(-5.0, -4.0, -3.0, -3.0, -2.0, -1.0)},
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/distance/box-reversed-and-special",
        "math",
        "aabb",
        json!({
            "min_x": f64_bits(4.0),
            "min_y": f64_bits(3.0),
            "min_z": f64_bits(2.0),
            "max_x": f64_bits(-4.0),
            "max_y": f64_bits(-3.0),
            "max_z": f64_bits(-2.0),
            "steps": [
                {"op": "distance_box", "box": box_bits(10.0, 10.0, 10.0, 11.0, 11.0, 11.0)},
                {"op": "distance_box", "box": box_bits(f64::NAN, 0.0, 0.0, 1.0, 1.0, 1.0)},
            ],
        }),
    ));
    out
}

fn clip() -> Vec<Value> {
    let mut out = Vec::new();
    let clip_rays = [
        ("west-to-east", ((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5))),
        ("east-to-west", ((2.0, 0.5, 0.5), (-1.0, 0.5, 0.5))),
        ("inside-to-east", ((0.5, 0.5, 0.5), (2.0, 0.5, 0.5))),
        ("diagonal", ((-1.0, -1.0, -1.0), (2.0, 2.0, 2.0))),
        ("parallel-miss", ((-1.0, 2.0, 0.5), (2.0, 2.0, 0.5))),
        ("below-threshold", ((-1.0, 0.5, 0.5), (-0.99999999, 0.5, 0.5))),
    ];
    for (label, (source, target)) in clip_rays.iter().copied() {
        out.push(case(
            format!("aabb/clip/{label}"),
            "math",
            "aabb",
            merged(vec![
                unit(),
                json!({
                    "steps": [
                        {
                            "op": "clip",
                            "from": point(source.0, source.1, source.2),
                            "to": point(target.0, target.1, target.2),
                        },
                    ],
                }),
            ]),
        ));
    }
    out
}

fn clip_coords() -> Vec<Value> {
    let mut out = Vec::new();
    let clip_coordinate_cases: Vec<ClipCoordinates> = vec![
        ("west-to-east", [0.0, 0.0, 0.0, 1.0, 1.0, 1.0], (-1.0, 0.5, 0.5), (2.0, 0.5, 0.5), vec![]),
        ("east-to-west", [0.0, 0.0, 0.0, 1.0, 1.0, 1.0], (2.0, 0.5, 0.5), (-1.0, 0.5, 0.5), vec![]),
        ("diagonal", [-2.0, -1.0, 0.0, 3.0, 2.0, 4.0], (-3.0, -2.0, -1.0), (4.0, 3.0, 5.0), vec![]),
        ("parallel-miss", [0.0, 0.0, 0.0, 1.0, 1.0, 1.0], (-1.0, 2.0, 0.5), (2.0, 2.0, 0.5), vec![]),
        ("reversed-bounds", [1.0, 1.0, 1.0, 0.0, 0.0, 0.0], (-1.0, 0.5, 0.5), (2.0, 0.5, 0.5), vec![]),
        (
            "threshold-special",
            [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            (-1.0, 0.5, 0.5),
            (-1.0 + 1.0e-8, 0.5, 0.5),
            vec![
                (point(0.5, 0.5, 0.5), point(0.5, 0.5, 0.5)),
                (point(f64::NAN, 0.5, 0.5), point(2.0, 0.5, 0.5)),
                (point(f64::NEG_INFINITY, 0.5, 0.5), point(f64::INFINITY, 0.5, 0.5)),
            ],
        ),
    ];
    for (label, bounds, source, target, extra) in clip_coordinate_cases {
        let step = |from: Value, to: Value| {
            json!({
                "op": "clip_coords", "min_x": f64_bits(bounds[0]), "min_y": f64_bits(bounds[1]),
                "min_z": f64_bits(bounds[2]), "max_x": f64_bits(bounds[3]), "max_y": f64_bits(bounds[4]),
                "max_z": f64_bits(bounds[5]), "from": from, "to": to,
            })
        };
        let mut steps = vec![step(point(source.0, source.1, source.2), point(target.0, target.1, target.2))];
        steps.extend(extra.into_iter().map(|(from, to)| step(from, to)));
        out.push(case(
            format!("aabb/clip-coords/{label}"),
            "math",
            "aabb",
            merged(vec![unit(), json!({ "steps": steps })]),
        ));
    }
    out
}

fn west_hit() -> Vec<Value> {
    let mut out = Vec::new();
    let clip_many_unit = vec![box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)];
    out.push(case(
        "aabb/clip-many/west-hit",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [clip_many_step(clip_many_unit.clone(), (-1.0, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 0, 0))]}),
        ]),
    ));
    out.push(case(
        "aabb/clip-many/east-hit-offset",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [clip_many_step(clip_many_unit.clone(), (5.0, -1.5, 5.5), (2.0, -1.5, 5.5), (3, -2, 5))]}),
        ]),
    ));
    out.push(case(
        "aabb/clip-many/vertical-and-depth",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    clip_many_step(clip_many_unit.clone(), (0.5, -1.0, 0.5), (0.5, 2.0, 0.5), (0, 0, 0)),
                    clip_many_step(clip_many_unit.clone(), (0.5, 2.0, 0.5), (0.5, -1.0, 0.5), (0, 0, 0)),
                    clip_many_step(clip_many_unit.clone(), (0.5, 0.5, -1.0), (0.5, 0.5, 2.0), (0, 0, 0)),
                    clip_many_step(clip_many_unit.clone(), (0.5, 0.5, 2.0), (0.5, 0.5, -1.0), (0, 0, 0)),
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/clip-many/diagonal-priority",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [clip_many_step(clip_many_unit.clone(), (-1.0, -1.0, -1.0), (2.0, 2.0, 2.0), (0, 0, 0))]}),
        ]),
    ));
    out
}

fn nearest_far_first() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "aabb/clip-many/nearest-far-first",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    clip_many_step(
                        Value::Array(vec![
                            json!(box_bits(3.0, 0.0, 0.0, 4.0, 1.0, 1.0)),
                            json!(box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
                        ]),
                        (9.0, 0.5, 0.5),
                        (16.0, 0.5, 0.5),
                        (10, 0, 0),
                    ),
                ],
            }),
        ]),
    ));
    out.push(case(
        "aabb/clip-many/nearest-near-first",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    clip_many_step(
                        Value::Array(vec![
                            json!(box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
                            json!(box_bits(3.0, 0.0, 0.0, 4.0, 1.0, 1.0)),
                        ]),
                        (-1.0, 0.5, 0.5),
                        (6.0, 0.5, 0.5),
                        (0, 0, 0),
                    ),
                ],
            }),
        ]),
    ));
    out
}

fn empty() -> Vec<Value> {
    let mut out = Vec::new();
    let clip_many_unit = vec![box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)];
    out.push(case(
        "aabb/clip-many/empty",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [clip_many_step(Value::Array(vec![]), (-1.0, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 0, 0))]}),
        ]),
    ));
    out.push(case(
        "aabb/clip-many/translated-miss",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({"steps": [clip_many_step(clip_many_unit.clone(), (-1.0, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 3, 0))]}),
        ]),
    ));
    out.push(case(
        "aabb/clip-many/interior-and-threshold",
        "math",
        "aabb",
        merged(vec![
            unit(),
            json!({
                "steps": [
                    clip_many_step(clip_many_unit.clone(), (0.5, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 0, 0)),
                    clip_many_step(clip_many_unit.clone(), (-1.0, 0.5, 0.5), (-0.99999999, 0.5, 0.5), (0, 0, 0)),
                ],
            }),
        ]),
    ));
    out
}

fn of_bounding_box() -> Vec<Value> {
    let mut out = Vec::new();
    let bounding_box_cases = [
        ("normal", vec![bounding_box(-4, -2, 3, 5, 6, 9)]),
        ("inverted-all", vec![bounding_box(4, 5, 6, -1, -2, -3)]),
        ("inverted-axes", vec![bounding_box(4, -2, 8, -1, 6, 3)]),
        ("integer-maximum-overflow", vec![bounding_box(-2, -3, -4, 2147483647, 2147483647, 2147483647)]),
        (
            "integer-limits",
            vec![bounding_box(-2147483648, -2147483648, -2147483648, 2147483647, 2147483647, 2147483647)],
        ),
        ("chain", vec![bounding_box(10, 20, 30, 12, 24, 36), bounding_box(-8, -6, -4, -2, 0, 2)]),
    ];
    for (label, boxes) in &bounding_box_cases {
        let steps = boxes.iter().map(|box_| json!({"op": "of_bounding_box", "box": box_})).collect::<Vec<_>>();
        out.push(case(
            format!("aabb/of-bounding-box/{label}"),
            "math",
            "aabb",
            merged(vec![unit(), json!({"steps": steps.clone()})]),
        ));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [
        point_inside_boundary(),
        box_overlap_touch(),
        clip(),
        clip_coords(),
        west_hit(),
        nearest_far_first(),
        empty(),
        of_bounding_box(),
    ]
    .concat()
}
