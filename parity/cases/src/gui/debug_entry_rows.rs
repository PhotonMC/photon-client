//! The debug overlay's entries that read the camera entity's surroundings, the clock and the
//! frame loop.

use serde_json::{Value, json};

use super::debug::{Place, row};
use crate::bits::f64_bits;

/// The biome entry: shown inside the build height, not at its edges outside, not with reduced debug
/// info.
pub(super) fn biome_rows() -> Vec<Value> {
    let row = |label: &str, place: Place, more: Value| {
        let mut options = json!({"extra": ["biome"]});
        if let (Some(options), Some(more)) = (options.as_object_mut(), more.as_object()) {
            options.extend(more.clone());
        }
        row(format!("debug-biome-{label}"), (place, (0.0, 0.0)), options)
    };
    vec![
        row("feet", (1.0, 64.0, 3.0), json!({})),
        row("lowest-block", (1.0, -64.0, 3.0), json!({})),
        row("below", (1.0, -64.5, 3.0), json!({})),
        row("highest-block", (1.0, 319.9, 3.0), json!({})),
        row("above", (1.0, 320.0, 3.0), json!({})),
        row("reduced", (1.0, 64.0, 3.0), json!({"reduced": true})),
        row("unloaded-chunk", (5000.5, 64.0, -5000.5), json!({})),
    ]
}

/// The day count entry: the overworld clock's ticks divided by the day's 24000, floored, from a
/// clock never set up through days in the hundreds of thousands, and not with reduced debug info.
pub(super) fn day_rows() -> Vec<Value> {
    [0_i64, 1, 23_999, 24_000, 24_001, 1_000_000, 100_000_000_000]
        .into_iter()
        .map(|ticks| {
            row(
                format!("debug-day-{ticks}"),
                ((1.0, 64.0, 3.0), (0.0, 0.0)),
                json!({"extra": ["day_count"], "day": ticks}),
            )
        })
        .chain([row(
            "debug-day-reduced".to_owned(),
            ((1.0, 64.0, 3.0), (0.0, 0.0)),
            json!({"extra": ["day_count"], "day": 48_000, "reduced": true}),
        )])
        .collect()
}

/// The light entry: each layer ahead, behind and level, the ends, and not with reduced debug info.
pub(super) fn light_rows() -> Vec<Value> {
    let lit = |label: &str, sky: i32, block: i32, reduced: bool| {
        row(
            format!("debug-light-{label}"),
            ((1.0, 64.0, 3.0), (0.0, 0.0)),
            json!({"extra": ["light_levels"], "sky": sky, "block": block, "reduced": reduced}),
        )
    };
    vec![
        lit("sky-ahead", 15, 3, false),
        lit("block-ahead", 2, 11, false),
        lit("level", 7, 7, false),
        lit("dark", 0, 0, false),
        lit("full", 15, 15, false),
        lit("reduced", 15, 3, true),
        // The groups and the plain lines of several entries together.
        row(
            "debug-light-with-biome-and-day".to_owned(),
            ((1.0, 64.0, 3.0), (0.0, 0.0)),
            json!({"extra": ["biome", "day_count", "light_levels"], "day": 72_000, "sky": 9, "block": 4}),
        ),
    ]
}

/// The GPU entry: whole and fractional shares, the edge of red, the roundings Java makes of halves
/// and of the double just under one, a share that is not a number, and reduced debug info.
pub(super) fn gpu_rows() -> Vec<Value> {
    let shares = [
        ("zero", 0.0),
        ("fraction", 42.3),
        ("half-up", 2.5),
        ("below-zero", -0.5),
        ("just-under-a-half", 0.499_999_999_999_999_94),
        ("near-full", 99.5),
        ("full", 100.0),
        ("over", 100.000_000_1),
        ("not-a-number", f64::NAN),
    ];
    let mut rows: Vec<Value> = shares
        .into_iter()
        .map(|(label, share)| {
            row(
                format!("debug-gpu-{label}"),
                ((1.0, 64.0, 3.0), (0.0, 0.0)),
                json!({"extra": ["gpu_utilization"], "gpu": f64_bits(share)}),
            )
        })
        .collect();
    rows.push(row(
        "debug-gpu-reduced".to_owned(),
        ((1.0, 64.0, 3.0), (0.0, 0.0)),
        json!({"extra": ["gpu_utilization"], "gpu": f64_bits(37.0), "reduced": true}),
    ));
    rows
}
