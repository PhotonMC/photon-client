//! The award statistics packet.

use serde_json::{Value, json};

use super::codec::varint_hex;
use crate::case::case;

/// A packet of the statistics given: the count, then each one's type id, value id and number.
fn frame(stats: &[(i64, i64, i64)]) -> String {
    let mut hex = varint_hex(stats.len() as i64);
    for (stat_type, value, amount) in stats {
        hex += &format!("{}{}{}", varint_hex(*stat_type), varint_hex(*value), varint_hex(*amount));
    }
    hex
}

pub fn cases() -> Vec<Value> {
    // Stat types: mined 0, crafted 1, used 2, broken 3, picked up 4, dropped 5, killed 6, killed
    // by 7, custom 8.
    let reads = [
        ("empty", frame(&[])),
        ("mined", frame(&[(0, 1, 12)])),
        (
            "one-of-each",
            frame(&[
                (0, 2, 1),
                (1, 3, 2),
                (2, 4, 3),
                (3, 5, 4),
                (4, 6, 5),
                (5, 7, 6),
                (6, 8, 7),
                (7, 9, 8),
                (8, 10, 9),
            ]),
        ),
        ("custom-zero", frame(&[(8, 0, 0)])),
        ("custom-last", frame(&[(8, 76, 2_147_483_647)])),
        ("big-numbers", frame(&[(1, 100, 300), (1, 101, 16_384), (1, 102, 2_097_152)])),
        ("negative-number", frame(&[(8, 1, -1)])),
        ("duplicate", frame(&[(2, 4, 1), (2, 4, 5)])),
        ("shuffled", frame(&[(8, 5, 1), (0, 9, 2), (8, 2, 3), (0, 1, 4)])),
        ("unknown-type", frame(&[(9, 1, 1)])),
        ("negative-type", frame(&[(-1, 1, 1)])),
        ("truncated-count", varint_hex(2)),
        ("truncated-entry", format!("{}{}", varint_hex(1), varint_hex(0))),
    ];
    reads
        .into_iter()
        .map(|(label, hex)| case(format!("game/award-stats/{label}"), "wire", "award_stats", json!({"hex": hex})))
        .collect()
}
