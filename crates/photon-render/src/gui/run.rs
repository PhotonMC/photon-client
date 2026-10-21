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
    pub underlined: bool,
    pub strikethrough: bool,
    /// Each glyph is drawn as a random one of the same width, a new one every frame.
    pub obfuscated: bool,
    /// Whether the run starts a piece of text, where the reference's character position is zero:
    /// an underline or strikethrough there reaches a unit further left. A string of `\u{a7}`
    /// codes is one piece; a component's literals are one each.
    pub piece_start: bool,
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
            underlined: self.underlined,
            strikethrough: self.strikethrough,
            obfuscated: self.obfuscated,
            piece_start: self.piece_start,
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
            underlined: false,
            strikethrough: false,
            obfuscated: false,
            piece_start: true,
            shadow: None,
            joins_previous: false,
        }
    }
}
