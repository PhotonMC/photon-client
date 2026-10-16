//! Cases for `mth/ray-intersects-aabb`, `mth/fast-inv-sqrt`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn ray_intersects_aabb() -> Vec<Value> {
    let mut out = Vec::new();
    let ray_intersects_aabb = ray_intersects_aabb_rows();
    for (label, ray_start, ray_dir, aabb) in &ray_intersects_aabb {
        out.push(case(
            format!("mth/ray-intersects-aabb/{label}"),
            "math",
            "mth_ray_intersects_aabb",
            json!({"ray_start": ray_start.clone(), "ray_dir": ray_dir.clone(), "aabb": aabb.clone()}),
        ));
    }
    let mth_fast_inv_sqrt = mth_fast_inv_sqrt_rows();
    for (label, input_value) in &mth_fast_inv_sqrt {
        out.push(case(
            format!("mth/fast-inv-sqrt/{label}"),
            "math",
            "mth_fast_inv_sqrt",
            json!({"input": input_value}),
        ));
    }
    out
}

fn ray_intersects_aabb_rows() -> Vec<(&'static str, serde_json::Value, serde_json::Value, serde_json::Value)> {
    [
        ray_intersects_aabb_inside_positive_x(),
        ray_intersects_aabb_outside_above_away(),
        ray_intersects_aabb_tangent_top_edge(),
        ray_intersects_aabb_large_coordinates(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn ray_intersects_aabb_inside_positive_x()
-> Vec<(&'static str, serde_json::Value, serde_json::Value, serde_json::Value)> {
    vec![
        (
            "inside-positive-x",
            vector_bits(0.5, 0.5, 0.5),
            vector_bits(1.0, 0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "inside-negative-x",
            vector_bits(0.5, 0.5, 0.5),
            vector_bits(-1.0, -0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "outside-left-toward",
            vector_bits(-1.0, 0.5, 0.5),
            vector_bits(1.0, 0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "outside-left-away",
            vector_bits(-1.0, 0.5, 0.5),
            vector_bits(-1.0, 0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "outside-right-toward",
            vector_bits(2.0, 0.5, 0.5),
            vector_bits(-1.0, 0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "outside-right-away",
            vector_bits(2.0, 0.5, 0.5),
            vector_bits(1.0, 0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "outside-below-toward",
            vector_bits(0.5, -1.0, 0.5),
            vector_bits(0.1, 1.0, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
    ]
}

fn ray_intersects_aabb_outside_above_away()
-> Vec<(&'static str, serde_json::Value, serde_json::Value, serde_json::Value)> {
    vec![
        (
            "outside-above-away",
            vector_bits(0.5, 2.0, 0.5),
            vector_bits(0.1, 1.0, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "outside-front-toward",
            vector_bits(0.5, 0.5, -1.0),
            vector_bits(0.0, 0.1, 1.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "outside-back-away",
            vector_bits(0.5, 0.5, 2.0),
            vector_bits(0.0, 0.1, 1.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "parallel-x-miss",
            vector_bits(2.0, 0.5, 0.5),
            vector_bits(0.0, 1.0, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "parallel-negative-zero-x",
            vector_bits(2.0, 0.5, 0.5),
            vector_bits(-0.0, 1.0, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "diagonal-hit",
            vector_bits(-1.0, -1.0, -1.0),
            vector_bits(1.0, 1.0, 1.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "diagonal-skew-miss",
            vector_bits(-1.0, -1.0, 2.0),
            vector_bits(1.0, 1.0, -1.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
    ]
}

fn ray_intersects_aabb_tangent_top_edge() -> Vec<(&'static str, serde_json::Value, serde_json::Value, serde_json::Value)>
{
    vec![
        (
            "tangent-top-edge",
            vector_bits(-1.0, 1.0, 0.5),
            vector_bits(1.0, 0.0, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "tangent-corner",
            vector_bits(-1.0, 1.0, 1.0),
            vector_bits(1.0, 0.0, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "boundary-outward",
            vector_bits(0.5, 0.5, 0.0),
            vector_bits(0.0, 0.1, -1.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "boundary-inward",
            vector_bits(0.5, 0.5, 0.0),
            vector_bits(0.0, 0.1, 1.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "near-axis-hit",
            vector_bits(-1.0, 0.5, 0.5),
            vector_bits(1.0, 1e-06, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "zero-direction-inside",
            vector_bits(0.5, 0.5, 0.5),
            vector_bits(0.0, 0.0, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "zero-direction-outside",
            vector_bits(2.0, 0.5, 0.5),
            vector_bits(0.0, 0.0, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
    ]
}

fn ray_intersects_aabb_large_coordinates()
-> Vec<(&'static str, serde_json::Value, serde_json::Value, serde_json::Value)> {
    vec![
        (
            "large-coordinates",
            vector_bits(1000000000.0, 0.5, 0.5),
            vector_bits(-1.0, 0.1, 0.0),
            box_bits(999999999.0, 0.0, 0.0, 1000000001.0, 1.0, 1.0),
        ),
        (
            "reversed-box",
            vector_bits(-1.0, 0.5, 0.5),
            vector_bits(1.0, 0.1, 0.0),
            box_bits(1.0, 1.0, 1.0, 0.0, 0.0, 0.0),
        ),
        ("point-box", vector_bits(-1.0, 0.0, 0.0), vector_bits(1.0, 0.1, 0.1), box_bits(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
        (
            "negative-box",
            vector_bits(-3.0, -2.5, -2.0),
            vector_bits(1.0, 0.2, 0.1),
            box_bits(-2.0, -2.0, -1.0, -1.0, -1.0, 0.0),
        ),
        (
            "nan-start",
            vector_bits(f64::NAN, 0.5, 0.5),
            vector_bits(1.0, 0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "nan-direction",
            vector_bits(-1.0, 0.5, 0.5),
            vector_bits(f64::NAN, 0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "infinite-direction",
            vector_bits(-1.0, 0.5, 0.5),
            vector_bits(f64::INFINITY, 0.1, 0.0),
            box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        ),
        (
            "infinite-box",
            vector_bits(-1.0, 0.5, 0.5),
            vector_bits(1.0, 0.1, 0.0),
            box_bits(f64::NEG_INFINITY, 0.0, 0.0, f64::INFINITY, 1.0, 1.0),
        ),
    ]
}

fn mth_fast_inv_sqrt_rows() -> Vec<(&'static str, std::string::String)> {
    vec![
        ("zero", f64_bits(0.0)),
        ("negative-zero", "8000000000000000".to_string()),
        ("one", f64_bits(1.0)),
        ("one-below", "3fefffffffffffff".to_string()),
        ("one-above", "3ff0000000000001".to_string()),
        ("two", f64_bits(2.0)),
        ("quarter", f64_bits(0.25)),
        ("four", f64_bits(4.0)),
        ("pi", f64_bits(3.141592653589793)),
        ("large", f64_bits(1e+20)),
        ("max", "7fefffffffffffff".to_string()),
        ("min-normal", "0010000000000000".to_string()),
        ("positive-subnormal", "0000000000000001".to_string()),
        ("negative-one", f64_bits(-1.0)),
        ("negative-two", f64_bits(-2.0)),
        ("negative-max", "ffefffffffffffff".to_string()),
        ("negative-min-normal", "8010000000000000".to_string()),
        ("negative-subnormal", "8000000000000001".to_string()),
        ("infinity", "7ff0000000000000".to_string()),
        ("negative-infinity", "fff0000000000000".to_string()),
        ("nan", "7ff8000000000000".to_string()),
        ("nan-payload", "7ff0000000000001".to_string()),
        ("negative-nan-payload", "fff0000000000001".to_string()),
        ("tiny-neighbor", "0000000000000002".to_string()),
    ]
}

pub fn cases() -> Vec<Value> {
    [ray_intersects_aabb()].concat()
}
