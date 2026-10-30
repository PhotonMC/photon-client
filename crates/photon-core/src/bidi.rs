//! The reordering the client applies to a line of text before drawing it
//! (`FormattedBidiReorder`): Arabic letters in their joined forms, the Unicode bidirectional
//! algorithm at the default level the language asks for, and mirrored brackets in right-to-left
//! runs. The text comes back in the order it is drawn, left to right.

mod shaping;

use std::sync::atomic::{AtomicBool, Ordering};

use unicode_bidi::{BidiClass, BidiInfo, Level, bidi_class};
use unicode_bidi_mirroring::get_mirrored;

static DEFAULT_RIGHT_TO_LEFT: AtomicBool = AtomicBool::new(false);

/// Whether the language the client shows reads right to left (`Language.isDefaultRightToLeft`):
/// the direction of a line without a strong character.
pub fn default_right_to_left() -> bool {
    DEFAULT_RIGHT_TO_LEFT.load(Ordering::Relaxed)
}

/// The language changed (`LanguageInfo.bidirectional` of the one chosen).
pub fn set_default_right_to_left(right_to_left: bool) {
    DEFAULT_RIGHT_TO_LEFT.store(right_to_left, Ordering::Relaxed);
}

/// Whether drawing `text` can differ from reading it: a language that reads right to left reorders
/// even a line of neutrals, and every right-to-left script, Arabic letter and formatting mark is
/// above U+058F.
pub fn needs_reorder(text: &str, default_rtl: bool) -> bool {
    default_rtl || text.chars().any(|c| c >= '\u{590}')
}

/// The text as it is drawn: shaped, then in visual order. `default_rtl` is the level a paragraph
/// without a strong character gets (the language's direction); a paragraph with one takes that
/// character's direction.
pub fn reorder(text: &str, default_rtl: bool) -> String {
    let units: Vec<()> = vec![(); text.encode_utf16().count()];
    reorder_tagged(text, &units, default_rtl)
        .into_iter()
        .map(|(c, ())| c)
        .collect()
}

/// [`reorder`] for text whose UTF-16 units each carry a tag (the style of a character). The
/// shaped text is shorter when letters ligate, and the reference gives each character of it the
/// tag at its own position, not at the position of the character it came from; so do we.
pub fn reorder_tagged<T: Copy>(text: &str, tags: &[T], default_rtl: bool) -> Vec<(char, T)> {
    let shaped = shaping::shape(text);
    let mut drawn = Vec::with_capacity(shaped.len());
    let info = BidiInfo::new(&shaped, paragraph_level(&shaped, default_rtl));
    let mut unit = 0;
    for paragraph in &info.paragraphs {
        let line = paragraph.range.clone();
        let mut levels = info.reordered_levels_per_char(paragraph, line.clone());
        let chars: Vec<char> = shaped[line].chars().collect();
        explicit_controls_follow_next(&chars, &mut levels, paragraph.level);
        let starts: Vec<usize> = chars
            .iter()
            .scan(unit, |at, c| {
                let start = *at;
                *at += c.len_utf16();
                Some(start)
            })
            .collect();
        unit += chars.iter().map(|c| c.len_utf16()).sum::<usize>();
        for index in BidiInfo::reorder_visual(&levels) {
            let c = chars[index];
            let c = if levels[index].is_rtl() {
                get_mirrored(c).unwrap_or(c)
            } else {
                c
            };
            if let Some(&tag) = tags.get(starts[index]).or(tags.last()) {
                drawn.push((c, tag));
            }
        }
    }
    drawn
}

/// The level every paragraph starts at: `None` lets the first strong character decide, which is
/// what the default levels ask for unless the text has none and the language reads right to left.
fn paragraph_level(text: &str, default_rtl: bool) -> Option<Level> {
    if !default_rtl || has_strong(text) {
        return None;
    }
    Some(Level::rtl())
}

fn has_strong(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(bidi_class(c), BidiClass::L | BidiClass::R | BidiClass::AL))
}

/// The explicit embedding and override characters sit at the level of the character after them,
/// and a trailing one at the paragraph's: that is where the reference's runs put them, not where
/// the algorithm leaves the removed characters.
fn explicit_controls_follow_next(chars: &[char], levels: &mut [Level], paragraph: Level) {
    let mut next = paragraph;
    for (index, &c) in chars.iter().enumerate().rev() {
        if matches!(
            bidi_class(c),
            BidiClass::LRE | BidiClass::RLE | BidiClass::LRO | BidiClass::RLO | BidiClass::PDF
        ) {
            levels[index] = next;
        } else {
            next = levels[index];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::reorder;

    #[test]
    fn latin_text_keeps_its_order() {
        assert_eq!(reorder("Hello world", false), "Hello world");
    }

    #[test]
    fn hebrew_runs_are_reversed() {
        assert_eq!(
            reorder("\u{5D0}\u{5D1}\u{5D2}", false),
            "\u{5D2}\u{5D1}\u{5D0}"
        );
    }

    #[test]
    fn brackets_mirror_inside_right_to_left_runs() {
        assert_eq!(reorder("(\u{5D0}\u{5D1})", false), "(\u{5D1}\u{5D0})");
    }

    #[test]
    fn lam_alef_joins_into_one_ligature_and_letters_take_joined_forms() {
        assert_eq!(reorder("\u{644}\u{627}", false), "\u{FEFB}");
        assert_eq!(
            reorder("\u{628}\u{628}\u{628}", false),
            "\u{FE90}\u{FE92}\u{FE91}"
        );
    }

    #[test]
    fn a_paragraph_without_strong_characters_takes_the_default_direction() {
        assert_eq!(reorder("1 2", false), "1 2");
        assert_eq!(reorder("... !?", true), "?! ...");
    }
}
