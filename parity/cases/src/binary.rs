//! The `binary` family: the fixed-layout codecs of `FriendlyByteBuf`.

use serde_json::{Value, json};

use crate::bits::vector_bits;
use crate::case::{case, merge};

const LP_VEC3_READS: &[(&str, &str)] = &[
    ("empty", ""),
    ("zero-early", "00aabb"),
    ("header-only", "01"),
    ("header-short", "0100"),
    ("unit-payload", "010000000000aabb"),
    ("continuation-no-scale", "040000000000"),
    ("continuation-payload", "04000000000001aabb"),
    ("continuation-varint-truncated", "04000000000080"),
    ("too-big-scale", "040000000000808080808080"),
];

const SPECIAL: (&str, &str, &str) = ("7ff8000000000002", "7ff0000000000000", "fff0000000000000");

const UUIDS: &[(&str, &str, &str)] = &[
    ("zero", "0000000000000000", "0000000000000000"),
    ("max", "ffffffffffffffff", "ffffffffffffffff"),
    ("sample", "0011223344556677", "8899aabbccddeeff"),
    ("alternating", "f0f0f0f0f0f0f0f0", "0f0f0f0f0f0f0f0f"),
    ("version4", "123e4567e89b12d3", "a456426655440000"),
    ("high-bit", "8000000000000000", "0000000000000001"),
    ("low-bit", "0000000000000001", "8000000000000000"),
    ("deadbeef", "deadbeefdeadbeef", "cafebabecafebabe"),
];

const BLOCK_POSITIONS: &[(&str, i64, i64, i64)] = &[
    ("origin", 0, 0, 0),
    ("unit", 1, 2, 3),
    ("negative", -1, -1, -1),
    ("max-corner", 33554431, 2047, 33554431),
    ("min-corner", -33554432, -2048, -33554432),
    ("wrap-x", 33554432, 0, 0),
    ("wrap-y-high", 0, 2048, 0),
    ("wrap-y-low", 0, -2049, 0),
    ("border", 30000000, 100, -30000000),
    ("i32-max", 2147483647, 2147483647, 2147483647),
    ("i32-min", -2147483648, -2147483648, -2147483648),
    ("mixed", 123456, -789, 654321),
];

const BLOCK_PAYLOADS: &[(&str, &str)] = &[
    ("zero", "0000000000000000"),
    ("ones", "ffffffffffffffff"),
    ("sample", "0040000030020000"),
    ("max-corner", "7fffffdffffff7ff"),
    ("high-bit", "8000000000000000"),
    ("deadbeef", "deadbeefcafebabe"),
];

const SECTIONS: &[(&str, i64, i64, i64)] = &[
    ("origin", 0, 0, 0),
    ("unit", 1, 2, 3),
    ("negative", -1, -1, -1),
    ("max-corner", 2097151, 524287, 2097151),
    ("min-corner", -2097152, -524288, -2097152),
    ("wrap-x", 2097152, 0, 0),
    ("wrap-y-high", 0, 524288, 0),
    ("wrap-y-low", 0, -524289, 0),
    ("wrap-z", 0, 0, 2097152),
    ("i32-max", 2147483647, 2147483647, 2147483647),
    ("i32-min", -2147483648, -2147483648, -2147483648),
    ("mixed", 123456, -789, 654321),
];

const SECTION_PAYLOADS: &[(&str, &str)] = &[
    ("zero", "0000000000000000"),
    ("ones", "ffffffffffffffff"),
    ("sample", "0008420000c00030"),
    ("high-bit", "8000000000000000"),
];

const CHUNKS: &[(&str, i64, i64)] = &[
    ("origin", 0, 0),
    ("unit", 1, 2),
    ("negative", -1, -2),
    ("alternating", 1437226410, -1437226411),
    ("i32-max-min", 2147483647, -2147483648),
    ("i32-min-max", -2147483648, 2147483647),
];

const CHUNK_PAYLOADS: &[(&str, &str)] = &[
    ("zero", "0000000000000000"),
    ("unit", "0000000200000001"),
    ("negative", "fffffffeffffffff"),
    ("alternating", "aa55aa5555aa55aa"),
    ("i32-max-min", "800000007fffffff"),
    ("sample", "deadbeefcafebabe"),
];

const LONG_ARRAY_READS: &[(&str, &str)] = &[
    ("empty", "0055aa"),
    ("one", "01010000000000000055aa"),
    ("two", "0200112233445566778899aabbccddeeff55aa"),
    ("short-payload", "02001122334455667755aa"),
    ("missing-payload", "01"),
    ("encoded-limit", "04"),
    ("negative-length", "ffffffff0f55aa"),
    ("noncanonical-zero", "800055aa"),
    ("prefix-overflow-to-zero", "808080801055aa"),
];

const BITSETS: &[(&str, &[u32])] = &[
    ("empty", &[]),
    ("bit-zero", &[0]),
    ("byte-boundary", &[7, 8]),
    ("word-boundary", &[63, 64, 65]),
    ("sample", &[1, 65, 127]),
    ("trailing-zero", &[0, 64]),
];

const BITSET_READS: &[(&str, &str)] = &[
    ("empty", "0055aa"),
    ("bit-zero", "01010000000000000055aa"),
    ("two-words", "020000000000000001010000000000000055aa"),
    ("short-payload", "02001122334455667755aa"),
    ("negative-length", "ffffffff0f55aa"),
];

const FIXED_READS: &[(&str, u32, &str)] = &[
    ("size-zero-empty", 0, "55aa"),
    ("size-zero-ignored", 0, "ff55aa"),
    ("size-one-empty", 1, "0055aa"),
    ("size-eight", 8, "ff55aa"),
    ("size-nine", 9, "ffff55aa"),
    ("size-sixteen", 16, "010255aa"),
    ("short-payload", 9, "ff"),
    ("missing-payload", 8, ""),
];

const FIXED_WRITES: &[(&str, u32, &[u32])] = &[
    ("size-zero-empty", 0, &[]),
    ("size-zero-overflow", 0, &[0]),
    ("size-eight", 8, &[0, 7]),
    ("size-eight-overflow", 8, &[8]),
    ("size-nine", 9, &[8]),
    ("size-sixteen", 16, &[0, 15]),
];

/// A `codec` case `binary/<codec>-<operation>/<label>` with `fields`.
fn codec(operation: &str, codec: &str, label: &str, fields: Value) -> Value {
    let kind = codec.replace('_', "-");
    let id = format!("binary/{kind}-{operation}/{label}");
    let mut case = case(id, "binary", operation, json!({"codec": codec, "prefix_hex": "a55a"}));
    merge(&mut case, fields);
    case
}

fn read(name: &str, label: &str, hex: &str) -> Value {
    codec("read", name, label, json!({ "hex": hex }))
}

fn special() -> Value {
    json!({"x": SPECIAL.0, "y": SPECIAL.1, "z": SPECIAL.2})
}

pub fn cases() -> Vec<Value> {
    [vec3_cases(), position_cases(), array_cases()].concat()
}

fn vec3_cases() -> Vec<Value> {
    let lp_writes = [
        ("zero", vector_bits(0.0, 0.0, 0.0)),
        ("below-min", vector_bits(0.00001, -0.00002, 0.00003)),
        ("unit-scale", vector_bits(1.0, -2.0, 3.0)),
        ("partial-scale", vector_bits(4.0, -8.0, 12.0)),
        ("sanitize", special()),
        ("large-clamp", vector_bits(3.4359738366e10, -3.4359738366e10, 0.0)),
    ];
    let mut cases: Vec<Value> =
        lp_writes.into_iter().map(|(label, values)| codec("write", "lp_vec3", label, values)).collect();
    cases.extend(LP_VEC3_READS.iter().map(|(label, hex)| read("lp_vec3", label, hex)));
    let sample = vector_bits(1.25, -2.5, 3.75);
    let writes = [
        ("zero", vector_bits(0.0, 0.0, 0.0)),
        ("sample", sample.clone()),
        ("signed-zero", json!({"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"})),
        ("subnormal-and-large", json!({"x": "0000000000000001", "y": "7fefffffffffffff", "z": "8000000000000001"})),
        ("special", special()),
    ];
    cases.extend(writes.into_iter().map(|(label, values)| codec("write", "vec3", label, values)));
    let payload: String = ["x", "y", "z"].iter().filter_map(|axis| sample[axis].as_str()).collect();
    for length in [0, 7, 8, 15, 16, 23, 24] {
        let suffix = if length % 8 == 0 { "55aa" } else { "" };
        let framed = format!("{}{suffix}", &payload[..length * 2]);
        cases.push(read("vec3", &format!("truncated-{length}"), &framed));
    }
    let framed = format!("{}{}{}55aa", SPECIAL.0, SPECIAL.1, SPECIAL.2);
    cases.push(read("vec3", "special", &framed));
    cases
}

fn coordinates(x: i64, y: Option<i64>, z: i64) -> Value {
    let mut fields = json!({"x": x.to_string(), "z": z.to_string()});
    if let Some(y) = y {
        merge(&mut fields, json!({"y": y.to_string()}));
    }
    fields
}

fn truncated(name: &str, payload: &str) -> Vec<Value> {
    (0..8)
        .map(|length| {
            let framed = format!("{}55aa", &payload[..length * 2]);
            read(name, &format!("truncated-{length}"), &framed)
        })
        .collect()
}

fn position_cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, most, least) in UUIDS {
        let fields = json!({"most_hex": most, "least_hex": least});
        cases.push(codec("write", "uuid", label, fields));
    }
    let sample = "00112233445566778899aabbccddeeff";
    for length in [0, 1, 7, 8, 9, 15, 16] {
        let framed = format!("{}55aa", &sample[..length * 2]);
        cases.push(read("uuid", &format!("truncated-{length}"), &framed));
    }
    cases.push(read("uuid", "exact", sample));
    cases.push(read("uuid", "all-ones", "ffffffffffffffffffffffffffffffff55aa"));
    let full = "0011223344556677";
    for (label, x, y, z) in BLOCK_POSITIONS {
        cases.push(codec("write", "blockpos", label, coordinates(*x, Some(*y), *z)));
    }
    for (label, payload) in BLOCK_PAYLOADS {
        cases.push(read("blockpos", label, &format!("{payload}55aa")));
    }
    cases.extend(truncated("blockpos", full));
    for (label, x, y, z) in SECTIONS {
        cases.push(codec("write", "sectionpos", label, coordinates(*x, Some(*y), *z)));
    }
    for (label, payload) in SECTION_PAYLOADS {
        cases.push(read("sectionpos", label, &format!("{payload}55aa")));
    }
    cases.extend(truncated("sectionpos", full));
    for (label, x, z) in CHUNKS {
        cases.push(codec("write", "chunkpos", label, coordinates(*x, None, *z)));
    }
    for (label, payload) in CHUNK_PAYLOADS {
        cases.push(read("chunkpos", label, &format!("{payload}55aa")));
    }
    cases.extend(truncated("chunkpos", "deadbeefcafebabe"));
    cases
}

fn long_arrays() -> Vec<(&'static str, Vec<String>)> {
    let texts = |values: &[i64]| values.iter().map(i64::to_string).collect();
    vec![
        ("empty", vec![]),
        ("zero", texts(&[0])),
        ("sample", texts(&[0, 1, -1])),
        ("extremes", texts(&[i64::MIN, i64::MAX])),
        ("powers", texts(&[1 << 7, -(1 << 7), 1 << 35, -(1 << 35)])),
        ("deadbeef", texts(&[-2401053089206453570, 2309737966])),
    ]
}

fn array_cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, values) in long_arrays() {
        cases.push(codec("write", "longarray", label, json!({ "values": values })));
    }
    for (label, framed) in LONG_ARRAY_READS {
        cases.push(read("longarray", label, framed));
    }
    cases.push(read("longarray", "ascii-127", &format!("7f{}55aa", "00".repeat(127))));
    for count in 0..7 {
        let label = format!("prefix-truncated-{count}");
        cases.push(read("longarray", &label, &"80".repeat(count)));
    }
    cases.push(read("longarray", "prefix-too-big", "80808080800055aa"));
    for (label, bits) in BITSETS {
        cases.push(codec("write", "bitset", label, json!({ "bits": bits })));
    }
    for (label, framed) in BITSET_READS {
        cases.push(read("bitset", label, framed));
    }
    for (label, size, framed) in FIXED_READS {
        let fields = json!({"size": size, "hex": framed});
        cases.push(codec("read", "fixedbitset", label, fields));
    }
    for (label, size, bits) in FIXED_WRITES {
        let fields = json!({"size": size, "bits": bits});
        cases.push(codec("write", "fixedbitset", label, fields));
    }
    cases
}
