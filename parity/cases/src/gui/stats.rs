//! The statistics screen.

use serde_json::{Value, json};

use super::gui;

/// `StatFormatter`: the default's grouping, the tenths, and the distance and the time over their
/// unit thresholds, with values on the half and on the ties of two decimals.
pub fn stat_format() -> Value {
    let rows = vec![
        json!({"label": "default", "formatter": "default", "values": [
            0, 1, 12, 999, 1000, 1001, 12345, 999_999, 1_000_000, 123_456_789, 2_147_483_647, -1, -999, -1000, -2_147_483_648_i64]}),
        json!({"label": "divide-by-ten", "formatter": "divide_by_ten", "values": [
            0, 1, 2, 3, 5, 9, 10, 15, 25, 35, 45, 100, 125, 1234, 12345, 2_147_483_647, -1, -25]}),
        json!({"label": "distance", "formatter": "distance", "values": [
            0, 1, 49, 50, 51, 99, 100, 101, 149, 150, 151, 999, 1000, 12345, 49999, 50000, 50001, 50049, 50050, 99999,
            100_000, 100_001, 123_449, 123_450, 123_451, 1_234_500, 1_234_501, 2_147_483_647, -1, -100_000]}),
        json!({"label": "time", "formatter": "time", "values": [
            0, 1, 2, 3, 19, 20, 21, 599, 600, 601, 1199, 1200, 1201, 1220, 3000, 36_000, 36_001, 72_000, 72_001, 100_000,
            1_728_000, 1_728_001, 1_728_020, 2_000_000, 31_536_000, 315_360_000, 631_152_000, 631_152_001, 1_000_000_000,
            2_147_483_647, -1, -1200]}),
        json!({"label": "ties", "formatter": "distance", "values": [
            // 12.345 km (the exact binary value is just above the tie), 1.005 m, 0.125 km steps.
            1_234_500, 100_500, 112_500, 62_500, 50_001]}),
    ];
    gui("stat-format", "stat_format", rows)
}
