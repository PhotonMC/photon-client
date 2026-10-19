//! The debug overlay (F3).

use serde_json::{Value, json};

use super::gui;
use crate::bits::{f32_bits, f64_bits};

type Place = (f64, f64, f64);

/// The debug overlay (F3): the default profile with the overlay up or down, with reduced debug
/// info, with the interface hidden, and the camera entity at places that round, wrap and
/// floor differently (ties at the third decimal, negative coordinates, the four facings, an angle
/// past a half turn).
pub fn hud_debug_overlay() -> Value {
    let mut rows = Vec::new();
    let places: [(Place, (f64, f64)); 7] = [
        ((1.0005, 64.0625, 2.5), (22.25, 22.25)),
        ((-0.5, -60.5, -15.999), (-135.0, -90.0)),
        ((100.125, 70.0, -300.9995), (190.5, 0.04)),
        ((0.0, 0.0, 0.0), (0.0, 0.0)),
        ((33.3333333, 12.12345678, 47.0), (-45.0, 45.0)),
        ((-1234.5678, 319.99999, 5000.0005), (359.9, -0.05)),
        ((15.99999, 15.0, 16.0), (720.0, 33.35)),
    ];
    for (index, ((x, y, z), (yaw, pitch))) in places.into_iter().enumerate() {
        for (overlay, reduced, hidden) in
            [(true, false, false), (false, false, false), (true, true, false), (true, false, true)]
        {
            if index > 2 && (!overlay || reduced || hidden) {
                continue;
            }
            rows.push(json!({
                "label": format!("debug-{index}-{overlay}-{reduced}-{hidden}"),
                "width": 427, "height": 240,
                "launched": "26.2",
                "overlay": overlay, "reduced": reduced, "hidden": hidden,
                "x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z),
                "yaw": f32_bits(yaw), "pitch": f32_bits(pitch),
            }));
        }
    }
    rows.push(json!({
        "label": "debug-small-window", "width": 160, "height": 90, "launched": "26.2",
        "overlay": true, "reduced": false, "hidden": false,
        "x": f64_bits(1.0), "y": f64_bits(2.0), "z": f64_bits(3.0),
        "yaw": f32_bits(0.0), "pitch": f32_bits(0.0),
    }));
    gui("hud-debug-overlay", "hud_debug_overlay", rows)
}

/// `extractLines` with lines in both columns: empty lines between, long lines that overlap in a narrow
/// window, and a column with nothing.
pub fn hud_debug_lines() -> Value {
    let long = "Facing: south (Towards positive Z) (22.3 / 22.3)";
    let rows = vec![
        json!({"label": "both", "width": 427, "height": 240,
            "left": ["Minecraft 26.2 (26.2/vanilla)", "", "XYZ: 1.001 / 64.06250 / 2.500", "Block: 1 64 2"],
            "right": ["Allocated: 12% 123/1024MB", "", "CPU: 8x some processor", "Display: 854x480"]}),
        json!({"label": "narrow-overlap", "width": 160, "height": 90,
            "left": [long, "", long], "right": [long, "", "Chunk: 0 4 0 [0 0 in r.0.0.mca]"]}),
        json!({"label": "right-only", "width": 427, "height": 240, "left": [], "right": ["a", "", "bb"]}),
        json!({"label": "left-only", "width": 427, "height": 240, "left": ["a", "", "bb"], "right": []}),
    ];
    gui("hud-debug-lines", "hud_debug_lines", rows)
}
