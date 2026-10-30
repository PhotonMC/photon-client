//! The `bidi` family: the client's reordering of a line of text for right-to-left scripts.

use serde_json::{Value, json};

use crate::case::case;

/// Texts written with escapes so the source stays ASCII. Hebrew letters are U+05D0.., Arabic
/// letters U+0627.., and the digits are the European ones unless said otherwise.
const TEXTS: &[(&str, &str)] = &[
    ("empty", ""),
    ("latin", "Hello world"),
    ("hebrew", "\u{5E9}\u{5DC}\u{5D5}\u{5DD}"),
    ("hebrew-words", "\u{5E9}\u{5DC}\u{5D5}\u{5DD} \u{5E2}\u{5D5}\u{5DC}\u{5DD}"),
    ("arabic-isolated", "\u{627}"),
    ("arabic-word", "\u{645}\u{631}\u{62D}\u{628}\u{627}"),
    ("arabic-words", "\u{645}\u{631}\u{62D}\u{628}\u{627} \u{628}\u{627}\u{644}\u{639}\u{627}\u{644}\u{645}"),
    ("arabic-lam-alef", "\u{644}\u{627}"),
    ("arabic-lam-alef-word", "\u{62C}\u{644}\u{627}\u{644}"),
    ("arabic-harakat", "\u{643}\u{64E}\u{62A}\u{64E}\u{628}\u{64E}"),
    ("arabic-tatweel", "\u{628}\u{640}\u{628}"),
    ("arabic-digits", "\u{661}\u{662}\u{663}"),
    ("mixed-latin-hebrew", "abc \u{5D0}\u{5D1}\u{5D2} def"),
    ("mixed-hebrew-latin", "\u{5D0}\u{5D1}\u{5D2} abc \u{5D3}\u{5D4}"),
    ("mixed-arabic-latin", "Minecraft \u{645}\u{627}\u{64A}\u{646}\u{643}\u{631}\u{627}\u{641}\u{62A}"),
    ("hebrew-digits", "\u{5D0}\u{5D1} 123 \u{5D2}\u{5D3}"),
    ("latin-digits", "abc 123"),
    ("neutral-only", "... !?"),
    ("neutral-edges", " \u{5D0}\u{5D1} "),
    ("brackets-hebrew", "(\u{5D0}\u{5D1})"),
    ("brackets-latin", "(ab)"),
    ("brackets-mixed", "a (\u{5D0}\u{5D1}) b"),
    ("angle-quotes", "\u{5D0}<\u{5D1}>"),
    ("square-and-curly", "[\u{5D0}] {\u{5D1}}"),
    ("number-sign", "\u{5D0} -12.5% \u{5D1}"),
    ("embedding-controls", "a\u{202B}b\u{5D0}\u{202C}c"),
    ("isolates", "a\u{2067}b\u{5D0}\u{2069}c"),
    ("marks", "a\u{200F}b\u{200E}\u{5D0}"),
    ("surrogates", "\u{1F600}\u{5D0}\u{1F600}"),
    ("arabic-presentation-form", "\u{FE8D}\u{FEDF}"),
    ("persian-letters", "\u{67E}\u{686}\u{698}\u{6AF}"),
    ("section-sign", "a\u{A7}c\u{5D0}"),
];

/// The blocks the shaping pass looks at: Arabic, its supplements and the two presentation forms.
const SHAPED_BLOCKS: &[(u32, u32)] =
    &[(0x0600, 0x06FF), (0x0750, 0x077F), (0x08A0, 0x08FF), (0xFB50, 0xFDCF), (0xFDF0, 0xFDFF), (0xFE70, 0xFEFF)];

/// Neighbours that join (beh, a dual-joining letter) or ligate (lam) with the letter under test.
const BEH: char = '\u{628}';
const LAM: char = '\u{644}';

/// Explicit embeddings and overrides around Latin (a), Hebrew (h) and digits (1), alone and
/// nested, to pin the level the control characters themselves take.
const CONTROL_PIECES: &[(&str, &str)] = &[
    ("lre", "\u{202A}"),
    ("rle", "\u{202B}"),
    ("lro", "\u{202D}"),
    ("rlo", "\u{202E}"),
    ("pdf", "\u{202C}"),
    ("a", "a"),
    ("h", "\u{5D0}"),
    ("n", "1"),
    ("s", " "),
];

pub fn cases() -> Vec<Value> {
    let mut cases = text_cases();
    cases.extend(control_cases());
    cases.extend(letter_cases());
    cases
}

/// Every code point of the shaped blocks alone and between joining neighbours, so a missing or
/// wrong cell of the shaping table shows as its own row.
fn letter_cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for &(first, last) in SHAPED_BLOCKS {
        for code in first..=last {
            let Some(c) = char::from_u32(code) else { continue };
            let contexts = [
                ("alone", format!("{c}")),
                ("after-beh", format!("{BEH}{c}")),
                ("before-beh", format!("{c}{BEH}")),
                ("between-behs", format!("{BEH}{c}{BEH}")),
                ("after-lam", format!("{LAM}{c}")),
                ("after-beh-lam", format!("{BEH}{LAM}{c}")),
                ("between-lam-beh", format!("{LAM}{c}{BEH}")),
            ];
            for (context, text) in contexts {
                cases.push(reorder_case(&format!("letter/{code:04x}/{context}"), &text, false));
            }
        }
    }
    cases
}

/// Every sequence of up to four of the pieces that has a control in it.
fn control_cases() -> Vec<Value> {
    let mut cases = Vec::new();
    let mut indexes = vec![vec![]];
    for _ in 0..4 {
        let mut next = Vec::new();
        for prefix in &indexes {
            for piece in 0..CONTROL_PIECES.len() {
                let mut grown = prefix.clone();
                grown.push(piece);
                next.push(grown);
            }
        }
        for sequence in &next {
            let controls = sequence.iter().any(|&piece| piece < 5);
            if controls && sequence.len() >= 2 {
                let id: Vec<&str> = sequence.iter().map(|&piece| CONTROL_PIECES[piece].0).collect();
                let text: String = sequence.iter().map(|&piece| CONTROL_PIECES[piece].1).collect();
                for (direction, rtl) in [("ltr", false), ("rtl", true)] {
                    cases.push(reorder_case(&format!("control/{}/{direction}", id.join("-")), &text, rtl));
                }
            }
        }
        indexes = next;
    }
    cases
}

fn reorder_case(id: &str, text: &str, right_to_left: bool) -> Value {
    let units: String = text.encode_utf16().map(|unit| format!("{unit:04x}")).collect();
    case(format!("bidi/reorder/{id}"), "bidi", "reorder", json!({"utf16_hex": units, "right_to_left": right_to_left}))
}

fn text_cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, text) in TEXTS {
        for (direction, rtl) in [("ltr", false), ("rtl", true)] {
            cases.push(reorder_case(&format!("{label}/{direction}"), text, rtl));
        }
    }
    cases
}
