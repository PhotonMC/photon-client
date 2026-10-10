//! The `utf8` family: Java's modified-length UTF-8 string codec.

use std::collections::BTreeSet;

use serde_json::{Value, json};

/// UTF-16 units keep unpaired surrogates intact without relying on JSON text
/// decoding.
const WRITES: &[(&str, &str)] = &[
    ("empty", ""),
    ("ascii", "0041007a"),
    ("nul", "0000"),
    ("turkish", "01300131015f"),
    ("combining", "00650301"),
    ("bmp-widths", "007f008007ff0800ffff"),
    ("supplementary", "d83dde80"),
    ("high-alone", "d800"),
    ("low-alone", "dc00"),
    ("high-ascii", "d8000041"),
    ("high-high", "d800d801"),
    ("low-high", "dc00d800"),
    ("high-high-low", "d800d801dc00"),
    ("high-bmp", "d8000800"),
    ("pair-low", "d83dde80dc00"),
];

/// Independent byte inputs, not encoder output or calculated expectations.
const PAYLOADS: &[(&str, &str)] = &[
    ("empty", ""),
    ("nul", "00"),
    ("ascii", "417a"),
    ("two-byte", "c480"),
    ("three-byte", "e0a080"),
    ("supplementary", "f09f9a80"),
    ("unicode-max", "f48fbfbf"),
    ("combining", "65cc81"),
    ("continuation", "80bf"),
    ("overlong-two", "c080"),
    ("overlong-three", "e08080"),
    ("overlong-four", "f0808080"),
    ("surrogate", "eda080"),
    ("out-of-range", "f4908080"),
    ("invalid-leaders", "f580ff"),
    ("truncated-two", "c2"),
    ("truncated-three", "e282"),
    ("truncated-four", "f09f9a"),
    ("invalid-middle", "e241a1"),
    ("invalid-fourth", "f09f9a41"),
    ("invalid-second", "c241"),
    ("replacement", "efbfbd"),
];

const FRAMED: &[(&str, i32, &str)] = &[
    ("zero-limit-empty", 0, "0055aa"),
    ("zero-limit-nonempty", 0, "014155aa"),
    ("negative-length", 8, "ffffffff0f55aa"),
    ("encoded-limit-before-availability", 1, "04"),
    ("encoded-limit-with-payload", 1, "044142434455aa"),
    ("missing-payload", 8, "01"),
    ("short-payload", 8, "034142"),
    ("decoded-limit-after-payload", 1, "0341424355aa"),
    ("noncanonical-zero", 8, "800055aa"),
    ("noncanonical-one", 8, "81004155aa"),
    ("prefix-overflow-to-zero", 8, "808080801055aa"),
];

pub fn cases() -> Vec<Value> {
    let mut cases = write_cases();
    cases.extend(read_cases());
    cases
}

fn write(id: &str, limit: usize, units: &str) -> Value {
    json!({
        "case_id": format!("utf8/write/{id}"), "family": "utf8", "operation": "write",
        "max_length": limit, "prefix_hex": "a55a", "utf16_hex": units,
    })
}

fn read(id: &str, limit: i32, framed: &str) -> Value {
    json!({
        "case_id": format!("utf8/read/{id}"), "family": "utf8", "operation": "read",
        "max_length": limit, "prefix_hex": "a55a", "hex": framed,
    })
}

fn write_cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, units) in WRITES {
        let length = units.len() / 4;
        let limits = BTreeSet::from([length.saturating_sub(1), length, length + 1]);
        for limit in limits {
            cases.push(write(&format!("{label}/limit-{limit}"), limit, units));
        }
    }
    // Large values straddle the real default limit and VarInt prefix boundaries.
    for length in [127, 128, 32766, 32767, 32768] {
        let id = format!("ascii-length-{length}");
        cases.push(write(&id, 32767, &"0061".repeat(length)));
    }
    let units = "0800".repeat(32767);
    cases.push(write("three-byte-default-limit", 32767, &units));
    cases
}

fn read_cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, payload) in PAYLOADS {
        // All of these payload sizes fit in a single literal length byte.
        let framed = format!("{:02x}{payload}55aa", payload.len() / 2);
        for limit in [1, 2, 8] {
            cases.push(read(&format!("{label}/limit-{limit}"), limit, &framed));
        }
    }
    for (label, limit, framed) in FRAMED {
        cases.push(read(label, *limit, framed));
    }
    let long = [
        ("ascii-127", format!("7f{}55aa", "61".repeat(127))),
        ("ascii-128", format!("8001{}55aa", "61".repeat(128))),
        ("default-limit", format!("ffff01{}55aa", "61".repeat(32767))),
        ("default-limit-exceeded", format!("808002{}55aa", "61".repeat(32768))),
        ("encoded-default-limit", format!("fdff05{}55aa", "e0a080".repeat(32767))),
        ("encoded-default-limit-exceeded", "feff05".to_owned()),
    ];
    for (label, framed) in long {
        cases.push(read(label, 32767, &framed));
    }
    for count in 0..7 {
        cases.push(read(&format!("prefix-truncated-{count}"), 8, &"80".repeat(count)));
    }
    cases.push(read("prefix-too-big-terminated", 8, "80808080800055aa"));
    cases
}
