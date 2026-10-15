//! Cases for `character-event`, `preedit-event`, `input-with-modifiers`.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;

const CHARACTERS: [(&str, u32); 10] = [
    ("nul", 0),
    ("space", 32),
    ("ascii", 65),
    ("newline", 10),
    ("delete", 127),
    ("section-sign", 167),
    ("accented", 233),
    ("surrogate", 55296),
    ("supplementary", 128512),
    ("maximum", 1114111),
];

/// A preedit string: its codepoints, block sizes, focused block and caret.
type Preedit = (&'static str, &'static [i64], &'static [i64], i64, i64);

const PREEDITS: [Preedit; 8] = [
    ("empty", &[], &[0], 0, 0),
    ("single-block", &[65, 66], &[2], 0, 1),
    ("multiple-blocks", &[65, 66, 67], &[1, 2], 1, 2),
    ("supplementary-caret", &[128512, 233, 66], &[2, 1], 0, 1),
    ("block-boundary-caret", &[65, 128512, 66], &[1, 2], 1, 1),
    ("end-caret", &[65, 128512], &[2], 0, 2),
    ("negative-block-size", &[65], &[-1, 1], 1, 0),
    ("ignored-trailing-codepoints", &[65, 66, 67], &[1], 0, 2),
];

/// A key press: label, key, scancode, modifiers, mouse button and mouse position.
type KeyPress = (&'static str, i64, i64, i64, i64, f64, f64);

const KEY_PRESSES: [KeyPress; 20] = [
    ("enter-selection", 257, 1, 0, 0, 1.25, -2.5),
    ("space-selection", 32, 2, 0, 1, 0.0, 0.0),
    ("keypad-enter-selection", 335, 3, 0, 2, -0.0, 3.5),
    ("escape", 256, 4, 0, 3, 4.5, -0.0),
    ("left", 263, 5, 0, 4, 5.5, 6.5),
    ("right", 262, 6, 0, 5, 7.5, 8.5),
    ("up", 265, 7, 0, 6, 9.5, 10.5),
    ("down", 264, 8, 0, 7, 11.5, 12.5),
    ("cycle-focus", 258, 9, 0, 0, 13.5, 14.5),
    ("digit-zero", 48, 10, 0, 1, 15.5, 16.5),
    ("digit-nine", 57, 11, 0, 2, 17.5, 18.5),
    ("non-digit", 58, 12, 0, 3, 19.5, 20.5),
    ("select-all", 65, 13, 2, 4, 21.5, 22.5),
    ("copy", 67, 14, 2, 5, 23.5, 24.5),
    ("paste", 86, 15, 2, 6, 25.5, 26.5),
    ("cut", 88, 16, 2, 7, 27.5, 28.5),
    ("shortcut-with-shift", 65, 17, 3, 0, 29.5, 30.5),
    ("modifier-bits", 0, 18, 15, 1, 31.5, 32.5),
    ("super-only", 65, 19, 8, 2, 33.5, 34.5),
    ("integer-underflow", -2147483648, 20, 0, 3, 35.5, 36.5),
];

fn character_events() -> Vec<Value> {
    CHARACTERS
        .iter()
        .map(|(label, codepoint)| {
            case(format!("character-event/{label}"), "math", "character_event", json!({"codepoint": codepoint}))
        })
        .collect()
}

fn preedit_events() -> Vec<Value> {
    PREEDITS
        .iter()
        .map(|(label, codepoints, block_sizes, focused_block, caret)| {
            case(
                format!("preedit-event/{label}"),
                "math",
                "preedit_event",
                json!({
                    "codepoints": codepoints,
                    "block_sizes": block_sizes,
                    "focused_block": focused_block,
                    "caret": caret,
                }),
            )
        })
        .collect()
}

fn key_presses() -> Vec<Value> {
    KEY_PRESSES
        .iter()
        .map(|(label, key, scancode, modifiers, mouse_button, mouse_x, mouse_y)| {
            case(
                format!("input-with-modifiers/{label}"),
                "math",
                "input_with_modifiers",
                json!({
                    "key": key,
                    "scancode": scancode,
                    "modifiers": modifiers,
                    "mouse_button": mouse_button,
                    "mouse_x": f64_bits(*mouse_x),
                    "mouse_y": f64_bits(*mouse_y),
                }),
            )
        })
        .collect()
}

pub fn cases() -> Vec<Value> {
    [character_events(), preedit_events(), key_presses()].concat()
}
