//! Cases for `sectionpos/offset`, `sectionpos/ranges`, `sectionpos/blocks-inside`, `sectionpos/around-and-at` and more.

use serde_json::{Value, json};

use crate::case::case;

fn offset() -> Vec<Value> {
    let mut out = Vec::new();
    let sectionpos_offset = [
        (
            "origin-zero",
            json!({
                "section_node": "0000000000000000",
                "direction": "north",
                "step_x": "0",
                "step_y": "0",
                "step_z": "0",
                "x": "0",
                "y": "0",
                "z": "0",
            }),
        ),
        (
            "positive",
            json!({
                "section_node": "0000040000300002",
                "direction": "east",
                "step_x": "1",
                "step_y": "2",
                "step_z": "3",
                "x": "1",
                "y": "2",
                "z": "3",
            }),
        ),
        ("negative", offset_negative()),
        ("packed-positive-limit", offset_packed_positive_limit()),
        ("i32-wrap", offset_i32_wrap()),
        ("arbitrary-zero", offset_arbitrary_zero()),
    ];
    for (label, values) in sectionpos_offset.iter() {
        out.push(case(format!("sectionpos/offset/{}", label), "math", "sectionpos_offset", values.clone()));
    }
    out
}

fn offset_arbitrary_zero() -> Value {
    json!({
        "section_node": "aa55aa5555aa55aa",
        "direction": "south",
        "step_x": "0",
        "step_y": "0",
        "step_z": "0",
        "x": "305419896",
        "y": "-305419896",
        "z": "1432778632",
    })
}

fn offset_i32_wrap() -> Value {
    json!({
        "section_node": "8000020000080000",
        "direction": "west",
        "step_x": "2147483647",
        "step_y": "-2147483648",
        "step_z": "-1",
        "x": "2147483647",
        "y": "-2147483648",
        "z": "2147483647",
    })
}

fn offset_packed_positive_limit() -> Value {
    json!({
        "section_node": "7ffffdfffff7ffff",
        "direction": "up",
        "step_x": "1",
        "step_y": "1",
        "step_z": "1",
        "x": "2097151",
        "y": "524287",
        "z": "2097151",
    })
}

fn offset_negative() -> Value {
    json!({
        "section_node": "ffffffffffffffff",
        "direction": "down",
        "step_x": "-1",
        "step_y": "-2",
        "step_z": "-3",
        "x": "-1",
        "y": "-2",
        "z": "-3",
    })
}

fn ranges() -> Vec<Value> {
    let mut out = Vec::new();
    let sectionpos_ranges = [
        ("between-single", ranges_between_single()),
        ("between-order", ranges_between_order()),
        ("between-negative", ranges_between_negative()),
        ("cube-origin", json!({"mode": "cube", "x": "0", "y": "0", "z": "0", "radius": "1"})),
        (
            "cube-boundary",
            json!({"mode": "cube", "x": "2147483647", "y": "-2147483648", "z": "-2147483647", "radius": "0"}),
        ),
        ("around-chunk", ranges_around_chunk()),
    ];
    for (label, values) in sectionpos_ranges.iter() {
        out.push(case(format!("sectionpos/ranges/{}", label), "math", "sectionpos_ranges", values.clone()));
    }
    let sectionpos_blocks_inside = [
        ("origin", json!({"x": "0", "y": "0", "z": "0"})),
        ("negative", json!({"x": "-1", "y": "-2", "z": "-3"})),
        ("coordinate-boundaries", json!({"x": "134217727", "y": "-134217728", "z": "134217727"})),
    ];
    for (label, values) in sectionpos_blocks_inside.iter() {
        out.push(case(
            format!("sectionpos/blocks-inside/{}", label),
            "math",
            "sectionpos_blocks_inside",
            values.clone(),
        ));
    }
    let sectionpos_around_and_at = [
        ("coordinates-interior", json!({"mode": "coordinates", "x": "1", "y": "2", "z": "3"})),
        ("coordinates-origin", json!({"mode": "coordinates", "x": "0", "y": "0", "z": "0"})),
        ("coordinates-x-boundary", json!({"mode": "coordinates", "x": "16", "y": "1", "z": "1"})),
        ("blockpos-overload", json!({"mode": "blockpos", "x": "15", "y": "16", "z": "31"})),
        ("packed-overload", json!({"mode": "packed", "block_node": "ffffffffffffffff"})),
        ("coordinate-overflow", json!({"mode": "coordinates", "x": "2147483647", "y": "1", "z": "1"})),
    ];
    for (label, values) in sectionpos_around_and_at.iter() {
        out.push(case(
            format!("sectionpos/around-and-at/{}", label),
            "math",
            "sectionpos_around_and_at_block_pos",
            values.clone(),
        ));
    }
    out
}

fn ranges_between_negative() -> Value {
    json!({
        "mode": "between",
        "min_x": "-2",
        "min_y": "-1",
        "min_z": "-3",
        "max_x": "-1",
        "max_y": "0",
        "max_z": "-2",
    })
}

fn ranges_between_order() -> Value {
    json!({
        "mode": "between",
        "min_x": "1",
        "min_y": "2",
        "min_z": "3",
        "max_x": "2",
        "max_y": "3",
        "max_z": "4",
    })
}

fn ranges_between_single() -> Value {
    json!({
        "mode": "between",
        "min_x": "2",
        "min_y": "-3",
        "min_z": "4",
        "max_x": "2",
        "max_y": "-3",
        "max_z": "4",
    })
}

fn ranges_around_chunk() -> Value {
    json!({
        "mode": "around-chunk",
        "chunk_x": "2",
        "chunk_z": "-3",
        "radius": "1",
        "min_section": "-1",
        "max_section": "0",
    })
}

fn double_coordinates() -> Vec<Value> {
    let mut out = Vec::new();
    let sectionpos_double_coordinates = [
        ("pos-positive-boundary", json!({"fn": "pos_to_section", "value": "4030000000000000"})),
        ("pos-negative-subnormal", json!({"fn": "pos_to_section", "value": "8000000000000001"})),
        ("pos-maximum-finite", json!({"fn": "pos_to_section", "value": "7fefffffffffffff"})),
        ("block-positive-fraction", json!({"fn": "block_to_section", "value": "403fffffffffffff"})),
        ("block-negative-boundary", json!({"fn": "block_to_section", "value": "c030000000000001"})),
        ("block-nan", json!({"fn": "block_to_section", "value": "7ff8000000000001"})),
    ];
    for (label, values) in sectionpos_double_coordinates.iter() {
        out.push(case(
            format!("sectionpos/double-coordinates/{}", label),
            "math",
            "sectionpos_double_coordinates",
            values.clone(),
        ));
    }
    let sectionpos_of_overloads = [
        ("blockpos-boundaries", json!({"mode": "blockpos", "x": "16", "y": "-17", "z": "31"})),
        ("blockpos-int-boundaries", json!({"mode": "blockpos", "x": "2147483647", "y": "-2147483648", "z": "-1"})),
        ("chunkpos-section", json!({"mode": "chunkpos", "x": "-3", "z": "7", "section_y": "-64"})),
        ("packed-sign-extension", json!({"mode": "packed", "section_node": "ffffffffffffffff"})),
        (
            "position-fractions",
            json!({"mode": "position", "x": "8000000000000001", "y": "4030800000000000", "z": "c030000000000001"}),
        ),
        (
            "position-special-values",
            json!({"mode": "position", "x": "7fefffffffffffff", "y": "7ff8000000000001", "z": "fff0000000000000"}),
        ),
    ];
    for (label, values) in sectionpos_of_overloads.iter() {
        out.push(case(format!("sectionpos/of-overloads/{}", label), "math", "sectionpos_of_overloads", values.clone()));
    }
    let relative = [
        ("origin", (0, 0, 0)),
        ("positive-boundary", (15, 15, 15)),
        ("section-next", (16, 16, 16)),
        ("negative-one", (-1, -1, -1)),
        ("negative-section", (-17, -16, -1)),
        ("integer-boundaries", (2147483647, -2147483648, 31)),
    ];
    for (label, (x, y, z)) in relative {
        out.push(case(
            format!("section-relative/{label}"),
            "math",
            "section_relative",
            json!({"x": x.to_string(), "y": y.to_string(), "z": z.to_string()}),
        ));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [offset(), ranges(), double_coordinates()].concat()
}
