//! The rectangles of underlined and struck-through text (`Font.PreparedTextBuilder.accept`).

use crate::gui::TextRun;
use crate::gui::font::Font;

/// One rectangle a text draws besides its glyphs: where, in what colour, and in which shadow
/// colour (zero for none, otherwise drawn a unit right and down first).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextEffect {
    pub rect: (f32, f32, f32, f32),
    pub colour: u32,
    pub shadow_colour: u32,
}

/// A run's colour, and its shadow's (zero when the text has none): the style's colour over the
/// call's alpha, and a quarter of it unless the style names one.
pub(crate) fn paint(run: &TextRun, default_colour: u32, shadow: bool) -> (u32, u32) {
    let colour = run
        .colour
        .map_or(default_colour, |rgb| (default_colour & 0xFF00_0000) | rgb);
    // `ARGB.scaleRGB(colour, 0.25)`.
    let quarter = [16, 8, 0]
        .map(|shift| ((((colour >> shift) & 0xFF) as f32 * 0.25) as u32) << shift)
        .into_iter()
        .fold(colour & 0xFF00_0000, |sum, channel| sum | channel);
    // A style that names its shadow colour has that one; a clear one has none.
    let shadow_colour = run.shadow.unwrap_or(quarter);
    (
        colour,
        if shadow && shadow_colour >> 24 != 0 {
            shadow_colour
        } else {
            0
        },
    )
}

/// The effects of `runs` drawn from `origin`, in the order the reference adds them: for each
/// glyph its strikethrough, then its underline. Each reaches from the glyph's pen to its advance,
/// and a unit further left at a piece's first character.
pub fn effects(
    font: &Font,
    runs: &[TextRun],
    (x, y): (f32, f32),
    (default_colour, shadow): (u32, bool),
) -> Vec<TextEffect> {
    let mut found = Vec::new();
    let mut pen = x;
    for run in runs {
        let (colour, shadow_colour) = paint(run, default_colour, shadow);
        for (position, codepoint) in run.text.chars().enumerate() {
            let Some(glyph) = font.glyph(codepoint) else {
                continue;
            };
            let advance = glyph.advance(run.bold);
            let left = if position == 0 && run.piece_start {
                pen - 1.0
            } else {
                pen
            };
            let bars = [(run.strikethrough, 4.5), (run.underlined, 9.0)];
            for (_, bottom) in bars.into_iter().filter(|(on, _)| *on) {
                found.push(TextEffect {
                    rect: (left, y + bottom - 1.0, pen + advance, y + bottom),
                    colour,
                    shadow_colour,
                });
            }
            pen += advance;
        }
    }
    found
}

/// `Font.getGlyph`: an obfuscated character other than a space is drawn as a random glyph of the
/// same width, a new one every frame. Where is advances is the character's own.
pub(crate) fn obfuscated(
    font: &Font,
    run: &TextRun,
    codepoint: char,
    glyph: crate::gui::font::Glyph,
) -> crate::gui::font::Glyph {
    use std::sync::atomic::{AtomicU32, Ordering};
    static STATE: AtomicU32 = AtomicU32::new(0x9E37_79B9);
    if !run.obfuscated || codepoint == ' ' {
        return glyph;
    }
    // A xorshift step; the picks need to differ, not to be good random numbers.
    let mut roll = STATE.load(Ordering::Relaxed);
    roll ^= roll << 13;
    roll ^= roll >> 17;
    roll ^= roll << 5;
    STATE.store(roll, Ordering::Relaxed);
    font.random_glyph(glyph.advance(false).ceil() as i32, roll)
        .unwrap_or(glyph)
}
