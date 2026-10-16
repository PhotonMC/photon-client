//! The text the draw layer takes: runs of text with the look they are drawn in. The screens turn a
//! text component's runs into these; the renderer reads nothing of a component but this.

/// One stretch of text and its look (`Style`, as far as drawing reads it).
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub text: String,
    /// `0xRRGGBB`, when the style names a colour.
    pub colour: Option<u32>,
    pub bold: bool,
    pub italic: bool,
    /// `0xAARRGGBB` of the shadow, when the style names one (zero alpha: none).
    pub shadow: Option<u32>,
    /// Whether the style equals the previous run's whole style, so the two are one stretch.
    pub joins_previous: bool,
}

impl TextRun {
    /// Text in no style of its own.
    pub fn plain(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            colour: None,
            bold: false,
            italic: false,
            shadow: None,
            joins_previous: false,
        }
    }
}
