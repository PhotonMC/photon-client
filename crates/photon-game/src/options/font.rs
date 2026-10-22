//! The font options that the font screen holds and `options.txt` keeps.

/// `forceUnicodeFont` and `japaneseGlyphVariants`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FontOptions {
    /// The Unicode font for all text; the GUI scale stays even.
    pub force_unicode: bool,
    /// The Japanese forms of the shared Han glyphs.
    pub japanese_glyph_variants: bool,
}
