//! Text measured in the default font, where the unifont is the fallback of the codepoints the
//! bitmap fonts lack.

use serde_json::{Value, json};

use super::gui;

const TEXTS: [(&str, &str); 16] = [
    ("latin", "Hello"),
    ("cyrillic", "\u{41F}\u{440}\u{438}\u{432}\u{435}\u{442}"),
    // Inside the first size override: every ideograph is the 16 columns of the override.
    ("han-override", "\u{6F22}\u{5B57}"),
    // Outside every override: the glyph is as wide as its ink.
    ("han-radical", "\u{2E80}\u{2E81}"),
    ("hiragana", "\u{3042}\u{3044}\u{3046}"),
    ("katakana", "\u{30AB}\u{30BF}\u{30AB}\u{30CA}"),
    // The syllables' override leaves a column out on the left.
    ("hangul-syllables", "\u{D55C}\u{AD6D}\u{C5B4}"),
    ("hangul-jamo", "\u{1100}\u{1161}\u{11A8}"),
    ("fullwidth", "\u{FF21}\u{FF42}\u{FF43}"),
    // Past the overrides' last range, so measured by ink.
    ("halfwidth-katakana", "\u{FF71}\u{FF72}"),
    // Beyond the basic plane.
    ("emoji", "\u{1F600}"),
    ("linear-b", "\u{10000}\u{10001}"),
    // Spaces with no ink: the whole bitmap, 8 or 16 columns.
    ("no-ink", "\u{A0}\u{3000}\u{2003}"),
    ("mixed", "a\u{3042}b\u{6F22}c \u{41F}\u{440}\u{438}\u{432}\u{435}\u{442}"),
    ("accents", "\u{E9}\u{FC}\u{F1}\u{DF}\u{E7}\u{F8}"),
    ("arabic", "\u{627}\u{644}\u{639}"),
];

pub fn font_width() -> Value {
    let rows: Vec<Value> = TEXTS.iter().map(|(label, text)| json!({"label": label, "text": text})).collect();
    gui("font-width", "font_width", rows)
}
