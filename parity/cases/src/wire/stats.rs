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
        // 26.3 registers `sleep_in_straw_bed` after `sleep_in_bed` (60): the ids after it move up
        // by one there, and the registry has one more entry at the end.
        ("custom-before-straw-bed", frame(&[(8, 60, 4)])),
        ("custom-straw-bed", frame(&[(8, 61, 4)])),
        ("custom-after-straw-bed", frame(&[(8, 62, 4)])),
        ("custom-newest-last", frame(&[(8, 77, 5)])),
        ("custom-past-the-registry", frame(&[(8, 78, 5)])),
        ("custom-negative", frame(&[(8, -1, 5)])),
        // The values of the other types are ids of the block, item and entity type registries.
        ("mined-high-block", frame(&[(0, 1000, 6)])),
        ("crafted-high-item", frame(&[(1, 1000, 7)])),
        ("used-higher-item", frame(&[(2, 1300, 8)])),
        ("killed-high-entity", frame(&[(6, 150, 9)])),
        ("killed-by-late-entity", frame(&[(7, 158, 9)])),
        ("killed-past-the-registry", frame(&[(6, 5000, 9)])),
        // The block and item registries answer air for an id they do not have, as the entity
        // type registry answers the pig.
        ("mined-past-the-registry", frame(&[(0, 5000, 6)])),
        ("crafted-past-the-registry", frame(&[(1, 5000, 7)])),
        ("used-past-the-registry", frame(&[(2, 6000, 8)])),
        ("picked-up-negative-item", frame(&[(4, -1, 3)])),
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
