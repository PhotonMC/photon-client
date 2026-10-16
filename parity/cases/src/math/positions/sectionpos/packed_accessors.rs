//! Cases for `sectionpos/packed-accessors`, `sectionpos/block-coordinates`, `sectionpos/relative-to-block`, `sectionpos/spatial` and more.

use serde_json::{Value, json};

use crate::case::case;

fn packed_accessors() -> Vec<Value> {
    let mut out = Vec::new();
    let sectionpos_packed_accessors = [
        ("zero", "0000000000000000"),
        ("positive", "0000040000300002"),
        ("negative", "ffffffffffffffff"),
        ("minimums", "8000020000080000"),
        ("maximums", "7ffffdfffff7ffff"),
        ("alternating", "aa55aa5555aa55aa"),
    ];
    for (label, packed) in &sectionpos_packed_accessors {
        out.push(case(
            format!("sectionpos/packed-accessors/{label}"),
            "math",
            "sectionpos_packed_accessors",
            json!({"packed": packed}),
        ));
    }
    let sectionpos_block_coordinates = [
        ("origin", json!({"x": "0", "y": "0", "z": "0"})),
        ("positive", json!({"x": "1", "y": "2", "z": "3"})),
        ("negative", json!({"x": "-1", "y": "-2", "z": "-3"})),
        ("boundary", json!({"x": "134217727", "y": "-134217728", "z": "134217727"})),
        ("shift-wrap", json!({"x": "134217728", "y": "-134217729", "z": "-134217728"})),
        ("coordinate-boundaries", json!({"x": "2147483647", "y": "-2147483648", "z": "-2147483647"})),
    ];
    for (label, values) in &sectionpos_block_coordinates {
        out.push(case(
            format!("sectionpos/block-coordinates/{label}"),
            "math",
            "sectionpos_block_coordinates",
            values.clone(),
        ));
    }
    out
}

fn relative_to_block() -> Vec<Value> {
    let mut out = Vec::new();
    let sectionpos_relative_to_block = [
        ("origin-zero", json!({"x": "0", "y": "0", "z": "0", "relative": "0"})),
        ("positive-corner", json!({"x": "1", "y": "2", "z": "3", "relative": "4095"})),
        ("negative-all", json!({"x": "-1", "y": "-2", "z": "-3", "relative": "-1"})),
        ("mixed-relative", json!({"x": "-2", "y": "3", "z": "-4", "relative": "1322"})),
        ("short-min", json!({"x": "1", "y": "2", "z": "3", "relative": "-32768"})),
        ("coordinate-boundaries", json!({"x": "134217727", "y": "-134217728", "z": "-134217729", "relative": "4095"})),
    ];
    for (label, values) in &sectionpos_relative_to_block {
        out.push(case(
            format!("sectionpos/relative-to-block/{label}"),
            "math",
            "sectionpos_relative_to_block",
            values.clone(),
        ));
    }
    let sectionpos_spatial = [
        ("origin", json!({"x": "0", "y": "0", "z": "0"})),
        ("positive", json!({"x": "1", "y": "2", "z": "3"})),
        ("negative", json!({"x": "-1", "y": "-2", "z": "-3"})),
        ("boundary", json!({"x": "134217727", "y": "-134217728", "z": "134217727"})),
        ("shift-wrap", json!({"x": "134217728", "y": "-134217729", "z": "-134217728"})),
        ("coordinate-boundaries", json!({"x": "2147483647", "y": "-2147483648", "z": "-2147483647"})),
    ];
    for (label, values) in &sectionpos_spatial {
        out.push(case(format!("sectionpos/spatial/{label}"), "math", "sectionpos_spatial", values.clone()));
    }
    out
}

fn packed_transformations() -> Vec<Value> {
    let mut out = Vec::new();
    let sectionpos_packed_transformations = [
        ("origin", json!({"block_node": "0000000000000000", "section_node": "0000000000000000", "x": "0", "z": "0"})),
        ("positive", json!({"block_node": "0000004000003002", "section_node": "0000040000300002", "x": "1", "z": "3"})),
        (
            "negative",
            json!({"block_node": "ffffffffffffdffe", "section_node": "ffffffffffdffffe", "x": "-1", "z": "-3"}),
        ),
        (
            "block-boundaries",
            json!({
                "block_node": "7fffffe0000007ff",
                "section_node": "7ffffe0000080000",
                "x": "2097151",
                "z": "-2097152",
            }),
        ),
        (
            "negative-boundaries",
            json!({
                "block_node": "8000001ffffff800",
                "section_node": "800001fffff7ffff",
                "x": "-2097152",
                "z": "2097151",
            }),
        ),
        (
            "arbitrary-bits",
            json!({
                "block_node": "aa55aa5555aa55aa",
                "section_node": "aa55aa5555aa55aa",
                "x": "2147483647",
                "z": "-2147483648",
            }),
        ),
    ];
    for (label, values) in &sectionpos_packed_transformations {
        out.push(case(
            format!("sectionpos/packed-transformations/{label}"),
            "math",
            "sectionpos_packed_transformations",
            values.clone(),
        ));
    }
    out
}

fn as_long() -> Vec<Value> {
    let mut out = Vec::new();
    let sectionpos_as_long = [
        (
            "origin",
            json!({
                "block_node": "0000000000000000",
                "x": "0",
                "y": "0",
                "z": "0",
                "section_x": "0",
                "section_y": "0",
                "section_z": "0",
            }),
        ),
        (
            "positive-boundaries",
            json!({
                "block_node": "000003c00001f010",
                "x": "1",
                "y": "2",
                "z": "3",
                "section_x": "1",
                "section_y": "2",
                "section_z": "3",
            }),
        ),
        ("negative-boundaries", as_long_negative_boundaries()),
        ("packed-positive-limits", as_long_packed_positive_limits()),
        ("packed-negative-limits", as_long_packed_negative_limits()),
        ("arbitrary-and-i32", as_long_arbitrary_and_i32()),
    ];
    for (label, values) in &sectionpos_as_long {
        out.push(case(format!("sectionpos/as-long/{label}"), "math", "sectionpos_as_long", values.clone()));
    }
    out
}

fn as_long_arbitrary_and_i32() -> Value {
    json!({
        "block_node": "deadbeefcafebabe",
        "x": "2147483647",
        "y": "-2147483648",
        "z": "-2147483647",
        "section_x": "2147483647",
        "section_y": "-2147483648",
        "section_z": "-2147483647",
    })
}

fn as_long_packed_negative_limits() -> Value {
    json!({
        "block_node": "8000002000000800",
        "x": "-2097152",
        "y": "-524288",
        "z": "-2097152",
        "section_x": "-2097152",
        "section_y": "-524288",
        "section_z": "-2097152",
    })
}

fn as_long_packed_positive_limits() -> Value {
    json!({
        "block_node": "7fffffdffffff7ff",
        "x": "2097151",
        "y": "524287",
        "z": "2097151",
        "section_x": "2097151",
        "section_y": "524287",
        "section_z": "2097151",
    })
}

fn as_long_negative_boundaries() -> Value {
    json!({
        "block_node": "ffffffffffff0fff",
        "x": "-1",
        "y": "-2",
        "z": "-3",
        "section_x": "-1",
        "section_y": "-2",
        "section_z": "-3",
    })
}

pub fn cases() -> Vec<Value> {
    [packed_accessors(), relative_to_block(), packed_transformations(), as_long()].concat()
}
