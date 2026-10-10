//! VarInt and VarLong codecs, packet framing, and the fixed-width common
//! packets.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::codec::fixed_hex;
use crate::case::case;

fn wire(id: String, operation: &str, fields: Value) -> Value {
    case(id, "wire", operation, fields)
}

fn varints(codec: &str, width: u32, limit: usize) -> Vec<Value> {
    let half = 1i128 << (width - 1);
    let mut values: BTreeSet<i128> = BTreeSet::from([0, 1, -1, -half, half - 1]);
    for shift in (7..width - 1).step_by(7) {
        for delta in [-1, 0, 1] {
            values.insert((1i128 << shift) + delta);
            values.insert(-((1i128 << shift) + delta));
        }
    }
    let mut cases: Vec<Value> = values
        .iter()
        .map(|value| {
            let fields = json!({"codec": codec, "value": value.to_string()});
            wire(format!("{codec}/write/{value}"), "write", fields)
        })
        .collect();
    // These byte sequences are authored independently of both
    // implementations.
    let mut reads: Vec<(String, String)> = [
        ("empty", ""),
        ("zero", "00"),
        ("one", "01"),
        ("boundary", "8001"),
        ("trailing", "012abb"),
        ("noncanonical-zero", "8000"),
        ("noncanonical-one", "8100"),
    ]
    .iter()
    .map(|(label, hex)| ((*label).to_owned(), (*hex).to_owned()))
    .collect();
    for count in 1..=limit + 1 {
        reads.push((format!("truncated-{count}"), "80".repeat(count)));
        reads.push((format!("terminated-{count}"), format!("{}00aa", "80".repeat(count))));
    }
    for last in [0u8, 1, 15, 16, 127, 128, 255] {
        let hex = format!("{}{last:02x}55", "ff".repeat(limit - 1));
        reads.push((format!("last-byte-{last:02x}"), hex));
    }
    cases.extend(
        reads
            .into_iter()
            .map(|(label, hex)| wire(format!("{codec}/read/{label}"), "read", json!({"codec": codec, "hex": hex}))),
    );
    cases
}

fn frames() -> Vec<Value> {
    let prepends = [
        ("empty", 0, 0),
        ("one-with-prefix", 1, 3),
        ("single-byte-header-limit", 127, 2),
        ("two-byte-header-start", 128, 1),
        ("two-byte-header-limit", 16383, 0),
        ("three-byte-header-start", 16384, 4),
        ("maximum", (1 << 21) - 1, 0),
        ("too-large", 1 << 21, 0),
    ];
    let mut cases: Vec<Value> = prepends
        .iter()
        .map(|(label, body_length, reader_skip)| {
            let fields = json!({"body_length": body_length, "reader_skip": reader_skip});
            wire(format!("frame/prepend/{label}"), "frame_prepend", fields)
        })
        .collect();
    let decodes = [
        ("exact", vec!["03aabbcc".to_owned()]),
        ("fragmented-header", vec!["80".to_owned(), format!("01{}", "aa".repeat(128))]),
        ("fragmented-body", vec!["02aa".to_owned(), "bb".to_owned()]),
        ("zero-length", vec!["00".to_owned()]),
        ("wide-length", vec!["808080".to_owned()]),
        ("incomplete-header", vec!["80".to_owned()]),
        ("incomplete-body", vec!["03aabb".to_owned()]),
        ("trailing-frame", vec!["01aa01bb".to_owned()]),
        ("successive-frames", vec!["01aa".to_owned(), "01bb".to_owned()]),
        ("three-byte-length", vec![format!("808001{}", "00".repeat(16384))]),
    ];
    cases.extend(
        decodes
            .into_iter()
            .map(|(label, chunks)| wire(format!("frame/decode/{label}"), "frame_decode", json!({ "chunks": chunks }))),
    );
    cases
}

fn common_packet(packet: &str, width: usize, pattern: i64) -> Vec<Value> {
    let bits = width * 8;
    let (minimum, maximum) = (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1);
    let writes = [
        ("zero", "0".to_owned(), ""),
        ("negative-one", "-1".to_owned(), ""),
        ("minimum", minimum.to_string(), "deadbeef"),
        ("maximum", maximum.to_string(), ""),
        ("pattern", pattern.to_string(), "aa55"),
    ];
    let base = |direction: &str| json!({"direction": direction, "packet": packet});
    let mut cases: Vec<Value> = writes
        .iter()
        .map(|(label, value, prefix)| {
            let mut fields = base("write");
            crate::case::merge(&mut fields, json!({"id": value, "prefix_hex": prefix}));
            wire(format!("common/{packet}/write/{label}"), "common_packet", fields)
        })
        .collect();
    let reads = [
        ("zero".to_owned(), fixed_hex(0, width)),
        ("negative-one".to_owned(), fixed_hex(-1, width)),
        ("minimum".to_owned(), fixed_hex(minimum as i64, width)),
        ("maximum".to_owned(), fixed_hex(maximum as i64, width)),
        ("trailing".to_owned(), format!("{}aabb", fixed_hex(pattern, width))),
        (format!("underflow-{}", width - 1), "00".repeat(width - 1)),
        ("empty".to_owned(), String::new()),
    ];
    cases.extend(reads.iter().map(|(label, hex)| {
        let mut fields = base("read");
        crate::case::merge(&mut fields, json!({ "hex": hex }));
        wire(format!("common/{packet}/read/{label}"), "common_packet", fields)
    }));
    cases
}

pub fn cases() -> Vec<Value> {
    let mut cases = varints("varint", 32, 5);
    cases.extend(varints("varlong", 64, 10));
    cases.extend(frames());
    for (packet, width, pattern) in [
        ("client-ping", 4, 16909060),
        ("server-pong", 4, 16909060),
        ("client-keepalive", 8, 72623859790382856),
        ("server-keepalive", 8, 72623859790382856),
    ] {
        cases.extend(common_packet(packet, width, pattern));
    }
    cases
}
