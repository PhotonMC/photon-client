//! The text the draw layer takes: runs of text with the look they are drawn in. The screens turn a
//! text component's runs into these; the renderer reads nothing of a component but this.

/// One stretch of text and its look (`Style`, as far as drawing reads it).
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun<'a> {
    pub text: std::borrow::Cow<'a, str>,
    /// `0xRRGGBB`, when the style names a colour.
    pub colour: Option<u32>,
    pub bold: bool,
    pub italic: bool,
    /// `0xAARRGGBB` of the shadow, when the style names one (zero alpha: none).
    pub shadow: Option<u32>,
    /// Whether the style equals the previous run's whole style, so the two are one stretch.
    pub joins_previous: bool,
}

impl<'a> TextRun<'a> {
    /// The run with a text of its own, for one that outlives what it was made from.
    pub fn into_owned(self) -> TextRun<'static> {
        TextRun {
            text: std::borrow::Cow::Owned(self.text.into_owned()),
            colour: self.colour,
            bold: self.bold,
            italic: self.italic,
            shadow: self.shadow,
            joins_previous: self.joins_previous,
        }
    }

    /// Text in no style of its own.
    pub fn plain(text: &'a str) -> Self {
        Self {
            text: text.into(),
            colour: None,
            bold: false,
            italic: false,
            shadow: None,
            joins_previous: false,
        }
    }
}
