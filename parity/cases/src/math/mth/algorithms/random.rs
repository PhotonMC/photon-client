//! Cases for `random/sequence`, `random/bound-`, `random/gaussian`, `random/set-seed` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;

fn sequence() -> Vec<Value> {
    let mut out = Vec::new();
    for seed in [0, 1, -1, 12345, 9223372036854775807_i64] {
        out.push(case(
            format!("random/sequence/{}", seed),
            "math",
            "random",
            json!({
                "seed": (seed).to_string(),
                "steps": [
                    {"op": "nextInt"},
                    {"op": "nextFloat"},
                    {"op": "nextDouble"},
                    {"op": "nextLong"},
                    {"op": "nextBoolean"},
                ],
            }),
        ));
    }
    for bound in [1, 2, 16, 100] {
        out.push(case(
            format!("random/bound-{}", bound),
            "math",
            "random",
            json!({
                "seed": "42",
                "steps": [{"op": "nextIntBound", "bound": bound}, {"op": "nextIntBound", "bound": bound}],
            }),
        ));
    }
    for (label, seed) in
        [("zero", 0), ("one", 1), ("negative-one", -1), ("fixed", 12345), ("max", 9223372036854775807_i64)]
    {
        out.push(case(
            format!("random/gaussian/sequence/{}", label),
            "math",
            "random",
            json!({
                "seed": (seed).to_string(),
                "steps": [{"op": "nextGaussian"}, {"op": "nextGaussian"}, {"op": "nextGaussian"}],
            }),
        ));
    }
    out
}

fn interleaved() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "random/gaussian/interleaved",
        "math",
        "random",
        json!({
            "seed": "42",
            "steps": [
                {"op": "nextInt"},
                {"op": "nextGaussian"},
                {"op": "nextGaussian"},
                {"op": "nextDouble"},
                {"op": "nextGaussian"},
            ],
        }),
    ));
    out.push(case(
        "random/set-seed/gaussian-cache",
        "math",
        "random",
        json!({
            "seed": "12345",
            "steps": [
                {"op": "nextGaussian"},
                {"op": "setSeed", "seed": "12345"},
                {"op": "nextGaussian"},
                {"op": "nextGaussian"},
            ],
        }),
    ));
    out.push(case(
        "random/set-seed/reseeded-state",
        "math",
        "random",
        json!({
            "seed": "-1",
            "steps": [
                {"op": "nextInt"},
                {"op": "nextGaussian"},
                {"op": "setSeed", "seed": "0"},
                {"op": "nextDouble"},
                {"op": "nextGaussian"},
                {"op": "nextGaussian"},
            ],
        }),
    ));
    out
}

fn boundaries() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "random/set-seed/boundaries",
        "math",
        "random",
        json!({
            "seed": "0",
            "steps": [
                {"op": "setSeed", "seed": "9223372036854775807"},
                {"op": "nextInt"},
                {"op": "setSeed", "seed": "-9223372036854775808"},
                {"op": "nextLong"},
            ],
        }),
    ));
    out.push(case(
        "random/overloads/integer-ranges",
        "math",
        "random",
        json!({
            "seed": "314159",
            "steps": [
                {"op": "nextIntBetweenInclusive", "min": -10, "max": 10},
                {"op": "nextIntBetweenInclusive", "min": 5, "max": 5},
                {"op": "nextIntRange", "origin": -20, "bound": 20},
                {"op": "nextIntRange", "origin": 0, "bound": 1},
            ],
        }),
    ));
    out.push(case(
        "random/overloads/triangle",
        "math",
        "random",
        json!({
            "seed": "-314159",
            "steps": [
                {"op": "triangleDouble", "mean": f64_bits(-1.25), "spread": f64_bits(0.75)},
                {"op": "triangleFloat", "mean": f32_bits(0.25), "spread": f32_bits(1.5)},
            ],
        }),
    ));
    out
}

fn consume_count() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "random/overloads/consume-count",
        "math",
        "random",
        json!({
            "seed": "42",
            "steps": [
                {"op": "nextInt"},
                {"op": "consumeCount", "rounds": 3},
                {"op": "nextInt"},
                {"op": "consumeCount", "rounds": -2},
                {"op": "nextInt"},
            ],
        }),
    ));
    out.push(case(
        "random/fork/basic",
        "math",
        "random_fork",
        json!({
            "seed": "42",
            "prefix_steps": [],
            "child_steps": [{"op": "nextInt"}, {"op": "nextDouble"}, {"op": "nextLong"}],
            "parent_steps": [{"op": "nextInt"}, {"op": "nextLong"}],
        }),
    ));
    out.push(case(
        "random/fork/gaussian-parent",
        "math",
        "random_fork",
        json!({
            "seed": "12345",
            "prefix_steps": [{"op": "nextGaussian"}],
            "child_steps": [{"op": "nextGaussian"}, {"op": "nextGaussian"}],
            "parent_steps": [{"op": "nextGaussian"}, {"op": "nextInt"}],
        }),
    ));
    out
}

fn coordinates() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "random/positional/coordinates",
        "math",
        "random_positional",
        json!({
            "seed": "987654321",
            "factory_steps": [
                {"op": "at", "x": 0, "y": 0, "z": 0, "steps": [{"op": "nextInt"}, {"op": "nextDouble"}]},
                {
                    "op": "at",
                    "x": -2147483648,
                    "y": 2147483647,
                    "z": 2147483647,
                    "steps": [{"op": "nextInt"}, {"op": "nextLong"}],
                },
                {
                    "op": "at",
                    "x": 123456,
                    "y": -789,
                    "z": 654321,
                    "steps": [{"op": "nextGaussian"}, {"op": "nextGaussian"}],
                },
            ],
            "parent_steps": [{"op": "nextInt"}],
        }),
    ));
    out.push(case(
        "random/positional/hash",
        "math",
        "random_positional",
        json!({
            "seed": "-1",
            "factory_steps": [
                {"op": "fromHashOf", "name": "", "steps": [{"op": "nextInt"}]},
                {"op": "fromHashOf", "name": "minecraft:stone", "steps": [{"op": "nextInt"}, {"op": "nextDouble"}]},
                {"op": "fromHashOf", "name": "\u{e9}\u{1f600}", "steps": [{"op": "nextLong"}, {"op": "nextGaussian"}]},
            ],
            "parent_steps": [{"op": "nextLong"}],
        }),
    ));
    out
}

fn from_seed() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "random/positional/from-seed",
        "math",
        "random_positional",
        json!({
            "seed": "0",
            "factory_steps": [
                {"op": "fromSeed", "seed": "0", "steps": [{"op": "nextInt"}]},
                {"op": "fromSeed", "seed": "-1", "steps": [{"op": "nextLong"}]},
                {
                    "op": "fromSeed",
                    "seed": "9223372036854775807",
                    "steps": [{"op": "nextGaussian"}, {"op": "nextGaussian"}],
                },
            ],
            "parent_steps": [{"op": "nextInt"}],
        }),
    ));
    for (label, seed) in vec![
        ("zero", 0),
        ("one", 1),
        ("negative-one", -1),
        ("positive", 42),
        ("negative", -42),
        ("large-positive", 987654321),
        ("large-negative", -987654321),
        ("long-max", 9223372036854775807_i64),
        ("long-min", -9223372036854775808_i64),
        ("alternating-positive", 305419896),
        ("alternating-negative", -6148914691236517206_i64),
        ("high-bit", 4611686018427387904_i64),
    ] {
        out.push(case(
            format!("mth/create-insecure-uuid/{}", label),
            "math",
            "mth_create_insecure_uuid",
            json!({"seed": (seed).to_string()}),
        ));
    }
    out
}

fn murmur_hash3_mixer() -> Vec<Value> {
    let mut out = Vec::new();
    for (label, input_value) in vec![
        ("zero", 0),
        ("one", 1),
        ("negative-one", -1),
        ("i32-max", 2147483647),
        ("i32-min", -2147483648),
        ("high-bit-plus-one", -2147483647),
        ("low-31-bit", 1073741824),
        ("alternating-positive", 1431655765),
        ("alternating-negative", -1431655766),
        ("byte-alternating-positive", 16711935),
        ("byte-alternating-negative", -16711936),
        ("ascending-hex", 305419896),
        ("descending-hex", -2023406815),
        ("shift-16", 65536),
        ("shift-13", 8192),
        ("shift-1", 2),
        ("near-high-bit", 1073741823),
        ("near-low-bit", -1073741825),
        ("mixed-positive", 19088743),
        ("mixed-negative", -19088744),
        ("large-positive", 987654321),
        ("large-negative", -987654321),
        ("signed-pattern", -305419896),
        ("all-zero-high-byte", 16777215),
    ] {
        out.push(case(
            format!("mth/murmur-hash3-mixer/{}", label),
            "math",
            "mth_murmur_hash3_mixer",
            json!({"input": (input_value).to_string()}),
        ));
    }
    out
}

fn binary_search() -> Vec<Value> {
    let mut out = Vec::new();
    for (label, from_value, to_value, condition, threshold) in vec![
        ("empty", 0, 0, "ge", 0),
        ("single-true", 0, 1, "ge", 0),
        ("single-false", 0, 1, "ge", 1),
        ("single-gt-true", 0, 1, "gt", -1),
        ("basic-ge", 0, 16, "ge", 7),
        ("basic-gt", 0, 16, "gt", 7),
        ("threshold-at-to", -10, 10, "ge", 10),
        ("all-true", -10, 10, "ge", -20),
        ("negative-ge", -20, -5, "ge", -12),
        ("negative-gt", -20, -5, "gt", -12),
        ("cross-zero-ge", -10, 11, "ge", -1),
        ("cross-zero-gt", -10, 11, "gt", -1),
        ("offset-range", 5, 25, "ge", 13),
        ("descending-range", 25, 5, "ge", 13),
        ("max-empty", 2147483647, 2147483647, "ge", 2147483647),
        ("min-empty", -2147483648, -2147483648, "ge", -2147483648),
        ("full-range-overflow", -2147483648, 2147483647, "ge", 0),
        ("max-to-min-wrap", 2147483647, -2147483648, "gt", 2147483647),
        ("high-wrap-two", 2147483646, -2147483648, "gt", 2147483647),
        ("low-range-all-true", -2147483648, -2147483640, "ge", -2147483644),
        ("high-range-ge", 2147483630, 2147483647, "ge", 2147483640),
        ("high-range-gt", 2147483630, 2147483647, "gt", 2147483640),
        ("low-range-ge", -2147483648, -2147483630, "ge", -2147483640),
        ("low-range-gt", -2147483648, -2147483630, "gt", -2147483640),
    ] {
        out.push(case(
            format!("mth/binary-search/{}", label),
            "math",
            "mth_binary_search",
            json!({
                "from": (from_value).to_string(),
                "to": (to_value).to_string(),
                "condition": condition,
                "threshold": (threshold).to_string(),
            }),
        ));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [
        sequence(),
        interleaved(),
        boundaries(),
        consume_count(),
        coordinates(),
        from_seed(),
        murmur_hash3_mixer(),
        binary_search(),
    ]
    .concat()
}
