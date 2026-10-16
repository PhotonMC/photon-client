//! Cases for `chunkpos/codec`, `chunkpos/block-coordinates`, `chunkpos/region-coordinates`, `chunkpos/block-positions` and more.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

fn decode() -> Vec<Value> {
    let mut out = Vec::new();
    let chunkpos_codec_decodes = [
        ("basic", vec!["1", "-2"]),
        ("zero", vec!["0", "0"]),
        ("negative", vec!["-1", "-2147483648"]),
        ("boundaries", vec!["2147483647", "-2147483648"]),
        ("alternating", vec!["1437226410", "-1437226411"]),
        ("empty", vec![]),
        ("short", vec!["1"]),
        ("long", vec!["1", "2", "3"]),
    ];
    for (label, values) in &chunkpos_codec_decodes {
        out.push(case(
            format!("chunkpos/codec/decode/{label}"),
            "math",
            "chunkpos_codec",
            json!({"mode": "decode", "values": values.clone()}),
        ));
    }
    let chunkpos_codec_encodes = [
        ("basic", json!({"x": "1", "z": "-2"})),
        ("zero", json!({"x": "0", "z": "0"})),
        ("boundaries", json!({"x": "2147483647", "z": "-2147483648"})),
        ("alternating", json!({"x": "1437226410", "z": "-1437226411"})),
    ];
    for (label, values) in &chunkpos_codec_encodes {
        out.push(case(
            format!("chunkpos/codec/encode/{label}"),
            "math",
            "chunkpos_codec",
            merged(vec![json!({"mode": "encode"}), values.clone()]),
        ));
    }
    out
}

fn block_coordinates() -> Vec<Value> {
    let mut out = Vec::new();
    let chunkpos_block_coordinates = [
        ("origin", json!({"x": "0", "z": "0", "offset": "0"})),
        ("positive", json!({"x": "1", "z": "2", "offset": "7"})),
        ("negative", json!({"x": "-1", "z": "-2", "offset": "15"})),
        ("boundary", json!({"x": "134217727", "z": "-134217728", "offset": "15"})),
        ("shift-wrap", json!({"x": "134217728", "z": "-134217729", "offset": "-1"})),
        ("offset-wrap", json!({"x": "1", "z": "-2", "offset": "2147483647"})),
    ];
    for (label, values) in &chunkpos_block_coordinates {
        out.push(case(
            format!("chunkpos/block-coordinates/{label}"),
            "math",
            "chunkpos_block_coordinates",
            values.clone(),
        ));
    }
    let chunkpos_region_coordinates = [
        ("origin", json!({"x": "0", "z": "0", "region_x": "0", "region_z": "0"})),
        ("positive-boundary", json!({"x": "31", "z": "32", "region_x": "1", "region_z": "2"})),
        ("negative-boundary", json!({"x": "-1", "z": "-32", "region_x": "-1", "region_z": "-2"})),
        ("mixed", json!({"x": "-33", "z": "63", "region_x": "67108863", "region_z": "-67108864"})),
        ("shift-wrap", json!({"x": "134217728", "z": "-134217729", "region_x": "67108864", "region_z": "-67108865"})),
        (
            "coordinate-boundaries",
            json!({"x": "2147483647", "z": "-2147483648", "region_x": "-67108864", "region_z": "67108863"}),
        ),
    ];
    for (label, values) in &chunkpos_region_coordinates {
        out.push(case(
            format!("chunkpos/region-coordinates/{label}"),
            "math",
            "chunkpos_region_coordinates",
            values.clone(),
        ));
    }
    out
}

fn block_positions() -> Vec<Value> {
    let mut out = Vec::new();
    let chunkpos_block_positions = [
        ("origin", block_positions_origin()),
        ("positive", block_positions_positive()),
        ("negative", block_positions_negative()),
        ("boundary", block_positions_boundary()),
        ("offset-wrap", block_positions_offset_wrap()),
        ("shift-wrap", block_positions_shift_wrap()),
    ];
    for (label, values) in &chunkpos_block_positions {
        out.push(case(format!("chunkpos/block-positions/{label}"), "math", "chunkpos_block_positions", values.clone()));
    }
    let chunkpos_representation = [
        ("origin", json!({"x": "0", "z": "0", "hash_x": "0", "hash_z": "0"})),
        ("basic", json!({"x": "1", "z": "-2", "hash_x": "1", "hash_z": "-2"})),
        ("negative", json!({"x": "-1", "z": "-2147483648", "hash_x": "-2147483648", "hash_z": "2147483647"})),
        ("boundaries", json!({"x": "2147483647", "z": "-2147483648", "hash_x": "2147483647", "hash_z": "-2147483648"})),
        ("alternating", json!({"x": "1437226410", "z": "-1437226411", "hash_x": "-559038737", "hash_z": "1013904223"})),
        (
            "hash-overflow",
            json!({"x": "2147483647", "z": "2147483647", "hash_x": "-2147483648", "hash_z": "-2147483648"}),
        ),
    ];
    for (label, values) in &chunkpos_representation {
        out.push(case(format!("chunkpos/representation/{label}"), "math", "chunkpos_representation", values.clone()));
    }
    let chunkpos_from_blockpos = [
        ("origin", json!({"x": "0", "y": "0", "z": "0"})),
        ("positive-boundary", json!({"x": "15", "y": "99", "z": "31"})),
        ("negative-boundary", json!({"x": "-1", "y": "-99", "z": "-16"})),
        ("mixed", json!({"x": "-17", "y": "42", "z": "16"})),
        ("max-boundary", json!({"x": "2147483647", "y": "2147483647", "z": "-2147483648"})),
        ("min-boundary", json!({"x": "-2147483648", "y": "-2147483648", "z": "2147483647"})),
    ];
    for (label, values) in &chunkpos_from_blockpos {
        out.push(case(format!("chunkpos/from-blockpos/{label}"), "math", "chunkpos_from_blockpos", values.clone()));
    }
    out
}

fn block_positions_negative() -> Value {
    json!({
        "x": "-1",
        "z": "-2",
        "block_x": "15",
        "y": "-1",
        "block_z": "0",
        "contains": [
            {"x": "-16", "y": "0", "z": "-32"},
            {"x": "-1", "y": "1", "z": "-17"},
            {"x": "-17", "y": "0", "z": "-20"},
            {"x": "-1", "y": "0", "z": "-16"},
        ],
    })
}

fn block_positions_positive() -> Value {
    json!({
        "x": "1",
        "z": "2",
        "block_x": "7",
        "y": "-64",
        "block_z": "12",
        "contains": [
            {"x": "16", "y": "0", "z": "32"},
            {"x": "31", "y": "100", "z": "47"},
            {"x": "15", "y": "0", "z": "32"},
            {"x": "16", "y": "0", "z": "48"},
        ],
    })
}

fn block_positions_origin() -> Value {
    json!({
        "x": "0",
        "z": "0",
        "block_x": "0",
        "y": "0",
        "block_z": "0",
        "contains": [
            {"x": "0", "y": "-99", "z": "0"},
            {"x": "15", "y": "999", "z": "15"},
            {"x": "-1", "y": "0", "z": "0"},
            {"x": "16", "y": "0", "z": "0"},
        ],
    })
}

fn block_positions_shift_wrap() -> Value {
    json!({
        "x": "134217728",
        "z": "-134217729",
        "block_x": "-1",
        "y": "42",
        "block_z": "15",
        "contains": [
            {"x": "-2147483648", "y": "0", "z": "2147483632"},
            {"x": "-2147483633", "y": "0", "z": "2147483647"},
            {"x": "-2147483632", "y": "0", "z": "2147483632"},
        ],
    })
}

fn block_positions_offset_wrap() -> Value {
    json!({
        "x": "1",
        "z": "-2",
        "block_x": "2147483647",
        "y": "-2147483648",
        "block_z": "-2147483648",
        "contains": [
            {"x": "16", "y": "0", "z": "-32"},
            {"x": "31", "y": "0", "z": "-17"},
            {"x": "-2147483633", "y": "0", "z": "-2147483648"},
        ],
    })
}

fn block_positions_boundary() -> Value {
    json!({
        "x": "134217727",
        "z": "-134217728",
        "block_x": "15",
        "y": "2147483647",
        "block_z": "0",
        "contains": [
            {"x": "2147483632", "y": "-2147483648", "z": "-2147483648"},
            {"x": "2147483647", "y": "0", "z": "-2147483633"},
            {"x": "2147483647", "y": "0", "z": "-2147483632"},
        ],
    })
}

fn distances() -> Vec<Value> {
    let mut out = Vec::new();
    let chunkpos_distances = [
        ("origin", json!({"x": "0", "z": "0", "other_x": "0", "other_z": "0"})),
        ("basic", json!({"x": "1", "z": "-2", "other_x": "4", "other_z": "3"})),
        ("negative", json!({"x": "-32", "z": "-17", "other_x": "-1", "other_z": "-64"})),
        ("axis-dominant", json!({"x": "100", "z": "-200", "other_x": "-123", "other_z": "-150"})),
        (
            "difference-wrap",
            json!({"x": "2147483647", "z": "-2147483648", "other_x": "-2147483648", "other_z": "2147483647"}),
        ),
        ("signed-minimum", json!({"x": "0", "z": "0", "other_x": "-2147483648", "other_z": "-2147483648"})),
    ];
    for (label, values) in &chunkpos_distances {
        out.push(case(format!("chunkpos/distances/{label}"), "math", "chunkpos_distances", values.clone()));
    }
    let chunkpos_from_section_node = [
        ("origin", json!({"section_node": "0000000000000000"})),
        ("positive", json!({"section_node": "0000040000300002"})),
        ("negative", json!({"section_node": "ffffffffffffffff"})),
        ("minimums", json!({"section_node": "8000020000080000"})),
        ("maximums", json!({"section_node": "7ffffdfffff7ffff"})),
        ("y-discard", json!({"section_node": "0000abfffef7ffff"})),
    ];
    for (label, values) in &chunkpos_from_section_node {
        out.push(case(
            format!("chunkpos/from-section-node/{label}"),
            "math",
            "chunkpos_from_section_node",
            values.clone(),
        ));
    }
    let chunkpos_range_closed = [
        ("from-to-single", json!({"mode": "from-to", "from_x": "0", "from_z": "0", "to_x": "0", "to_z": "0"})),
        ("from-to-increasing", json!({"mode": "from-to", "from_x": "1", "from_z": "2", "to_x": "3", "to_z": "4"})),
        ("from-to-decreasing", json!({"mode": "from-to", "from_x": "3", "from_z": "4", "to_x": "1", "to_z": "2"})),
        ("from-to-fixed-x", json!({"mode": "from-to", "from_x": "-5", "from_z": "7", "to_x": "-5", "to_z": "4"})),
        ("from-to-fixed-z", json!({"mode": "from-to", "from_x": "-2", "from_z": "6", "to_x": "2", "to_z": "6"})),
        ("center-radius-zero", json!({"mode": "center-radius", "x": "9", "z": "-4", "radius": "0"})),
        ("center-radius-positive", json!({"mode": "center-radius", "x": "-2", "z": "3", "radius": "2"})),
        ("center-radius-negative", json!({"mode": "center-radius", "x": "4", "z": "-6", "radius": "-1"})),
    ];
    for (label, values) in &chunkpos_range_closed {
        out.push(case(format!("chunkpos/range-closed/{label}"), "math", "chunkpos_range_closed", values.clone()));
    }
    out
}

fn constants() -> Vec<Value> {
    let mut out = Vec::new();
    for name in ["zero", "invalid_chunk_pos", "region_size", "region_max_index"] {
        out.push(case(format!("chunkpos/constants/{name}"), "math", "chunkpos_constant", json!({"name": name})));
    }
    let chunkpos_packed_accessors = [
        ("zero", "0000000000000000"),
        ("positive", "0000000200000001"),
        ("negative", "fffffffeffffffff"),
        ("x-min-z-max", "7fffffff80000000"),
        ("x-max-z-min", "800000007fffffff"),
        ("alternating", "aa55aa5555aa55aa"),
    ];
    for (label, packed) in &chunkpos_packed_accessors {
        out.push(case(
            format!("chunkpos/packed-accessors/{label}"),
            "math",
            "chunkpos_packed_accessors",
            json!({"packed": packed}),
        ));
    }
    let chunkpos_validity = [
        ("origin", json!({"x": "0", "z": "0"})),
        ("boundary", json!({"x": "2097061", "z": "-2097061"})),
        ("above-boundary", json!({"x": "2097062", "z": "0"})),
        ("negative-boundary", json!({"x": "-2097061", "z": "2097060"})),
        ("signed-minimum-quirk", json!({"x": "-2147483648", "z": "0"})),
        ("signed-maximum", json!({"x": "2147483647", "z": "-2147483648"})),
    ];
    for (label, values) in &chunkpos_validity {
        out.push(case(format!("chunkpos/validity/{label}"), "math", "chunkpos_validity", values.clone()));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [decode(), block_coordinates(), block_positions(), distances(), constants()].concat()
}
