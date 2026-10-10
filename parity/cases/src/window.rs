//! The `window` family: `Window` GUI scale selection and scaled-size
//! rounding.
//!
//! `calculateScale` walks the scale up while both axes still leave at least
//! the 320x240 base area, then rounds odd scales up when unicode is enforced.
//! `setGuiScale` divides the framebuffer by the scale as a double and rounds
//! up whenever the division left a remainder, so the interesting inputs are
//! the ones that divide exactly, the ones that do not, and the scale
//! boundaries on each axis.

use serde_json::{Value, json};

const MIN_SIZES: &[(&str, i32)] = &[
    ("zero", 0),
    ("one", 1),
    ("negative", -1),
    ("min-int", i32::MIN),
    ("default-width", 854),
    ("default-height", 480),
    ("max-int", i32::MAX),
];

/// Label, framebuffer width and height, max scale, enforce unicode.
const SCALES: &[(&str, i32, i32, i32, bool)] = &[
    ("auto-1920x1080", 1920, 1080, 0, false),
    ("auto-2560x1440", 2560, 1440, 0, false),
    ("auto-3840x2160", 3840, 2160, 0, false),
    ("auto-1366x768", 1366, 768, 0, false),
    ("auto-800x600", 800, 600, 0, false),
    ("auto-1024x768", 1024, 768, 0, false),
    // The exact base area and its neighbours drive the loop guard.
    ("base-exact", 320, 240, 0, false),
    ("base-minus-one-x", 319, 240, 0, false),
    ("base-minus-one-y", 320, 239, 0, false),
    ("base-double", 640, 480, 0, false),
    ("base-double-minus-one", 639, 479, 0, false),
    // Degenerate framebuffers still have to produce a usable scale.
    ("tiny-1x1", 1, 1, 0, false),
    ("tiny-2x2", 2, 2, 0, false),
    ("narrow-320x2160", 320, 2160, 0, false),
    ("wide-3840x240", 3840, 240, 0, false),
    // Capped scales stop the loop before the area guard does.
    ("cap-1", 1920, 1080, 1, false),
    ("cap-2", 1920, 1080, 2, false),
    ("cap-3", 1920, 1080, 3, false),
    ("cap-4", 1920, 1080, 4, false),
    ("cap-above-auto", 1920, 1080, 9, false),
    // Unicode rounds an odd result up, including past the cap.
    ("unicode-even-result", 1920, 1080, 0, true),
    ("unicode-odd-result", 1000, 700, 0, true),
    ("unicode-cap-1", 1920, 1080, 1, true),
    ("unicode-cap-3", 1920, 1080, 3, true),
    ("unicode-base-exact", 320, 240, 0, true),
    ("unicode-tiny", 1, 1, 0, true),
    // Non-divisible sizes exercise the scaled-size rounding branch.
    ("remainder-1001x701", 1001, 701, 0, false),
    ("remainder-1920x1081", 1920, 1081, 0, false),
    ("remainder-cap-3", 1001, 701, 3, false),
    ("remainder-cap-7", 1001, 701, 7, false),
    ("divides-exactly", 1280, 720, 4, false),
];

pub fn cases() -> Vec<Value> {
    let min_sizes: Vec<Value> = MIN_SIZES
        .iter()
        .map(|(label, requested)| json!({"label": label, "requested": requested}))
        .collect();
    let scales: Vec<Value> = SCALES
        .iter()
        .map(|(label, width, height, max_scale, unicode)| {
            json!({
                "label": label, "framebuffer_width": width, "framebuffer_height": height,
                "max_scale": max_scale, "enforce_unicode": unicode,
            })
        })
        .collect();
    vec![
        json!({"case_id": "window/base-size", "family": "window", "operation": "base_size"}),
        json!({
            "case_id": "window/min-size", "family": "window", "operation": "min_size",
            "rows": min_sizes,
        }),
        json!({
            "case_id": "window/gui-scale", "family": "window", "operation": "gui_scale",
            "rows": scales,
        }),
    ]
}
