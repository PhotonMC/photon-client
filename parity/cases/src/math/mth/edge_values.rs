//! Cases for `coord`, `math/clamp`.
//! around the special float values and the neighbours of each boundary.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

const COORDINATES: [i32; 19] = [
    0,
    1,
    15,
    16,
    17,
    -1,
    -15,
    -16,
    -17,
    31,
    32,
    -32,
    -33,
    2147483647,
    -2147483648,
    33554431,
    -33554432,
    2097151,
    -2097152,
];

fn coordinate_conversions() -> Vec<Value> {
    let mut out = Vec::new();
    for fn_ in ["block_to_section", "section_to_block", "section_relative"] {
        for value in COORDINATES {
            out.push(case(
                format!("coord/{fn_}/{value}"),
                "math",
                "coord",
                json!({"fn": fn_, "value": value.to_string()}),
            ));
        }
    }
    out
}

/// Zeros, infinities, NaN and the smallest subnormals of `width`, as bit patterns.
fn special_values(width: &str) -> BTreeSet<String> {
    let mut specials: BTreeSet<String> =
        [0.0, -0.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN].iter().map(|value| float_bits(*value, width)).collect();
    let subnormals: [&str; 2] =
        if width == "f32" { ["00000001", "80000001"] } else { ["0000000000000001", "8000000000000001"] };
    specials.extend(subnormals.iter().map(|bits| (*bits).to_owned()));
    specials
}

fn floor_and_wrap(width: &str) -> Vec<Value> {
    let mut out = Vec::new();
    let centers: [(&str, Vec<f64>); 2] = [
        ("floor", vec![-2.0, -1.0, 1.0, 2.0, -2147483648.0, 2147483648.0]),
        ("wrap_degrees", vec![-720.0, -360.0, -180.0, 180.0, 360.0, 720.0, 100000.0]),
    ];
    for (operation, centers) in centers {
        let mut values = special_values(width);
        for value in centers {
            values.extend(neighbors(value, width));
        }
        for bits in values {
            let id = format!("math/{operation}/{width}/{bits}");
            out.push(case(id, "math", operation, json!({"type": width, "value": bits})));
        }
    }
    out
}

fn clamp(width: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for (low, high) in [(-1.0, 1.0), (0.0, 1.0)] {
        let mut values = special_values(width);
        for value in [1.0, -1.0] {
            values.extend(neighbors(value, width));
        }
        for bits in values {
            out.push(case(
                format!("math/clamp/{width}/{low:?}/{bits}"),
                "math",
                "clamp",
                json!({"type": width, "value": bits, "min": float_bits(low, width), "max": float_bits(high, width)}),
            ));
        }
    }
    out
}

pub fn cases() -> Vec<Value> {
    let mut out = coordinate_conversions();
    for width in ["f32", "f64"] {
        out.extend(floor_and_wrap(width));
        out.extend(clamp(width));
    }
    out
}
