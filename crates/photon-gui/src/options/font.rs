//! The font settings screen (`FontOptionsScreen`): the Unicode font for all text, and the Japanese
//! forms of the shared Han glyphs. Both are the options'.

use photon_game::options::Options;
use photon_game::text::Lang;

use crate::options::list::{SubScreen, SubScreenInput, boolean, boolean_tip};
use crate::widgets::cycle::CycleButton;
use crate::widgets::slider::SliderKey;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// What the screen finds its widgets by.
const FORCE_UNICODE: u8 = 1;
const JAPANESE_VARIANTS: u8 = 2;

/// What the screen's buttons say now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontChoices {
    pub force_unicode_font: bool,
    pub japanese_glyph_variants: bool,
}

pub struct FontOptionsScreen {
    screen: SubScreen,
}

impl FontOptionsScreen {
    /// `init`, with the options as they are.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        options: &Options,
    ) -> Self {
        let screen = SubScreen::new(font, lang, size, "options.font.title", in_world, |list| {
            list.add_small(
                lang,
                vec![
                    (
                        FORCE_UNICODE,
                        boolean("options.forceUnicodeFont", options.font.force_unicode),
                    ),
                    (
                        JAPANESE_VARIANTS,
                        boolean_tip(
                            "options.japaneseGlyphVariants",
                            options.font.japanese_glyph_variants,
                            "options.japaneseGlyphVariants.tooltip",
                        ),
                    ),
                ],
            );
        });
        Self { screen }
    }

    fn flag(&self, tag: u8, fallback: bool) -> bool {
        self.screen
            .list
            .find(tag)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
            .map_or(fallback, |button| *button.value())
    }

    /// What the buttons say now, for the options to take.
    pub fn choices(&self, fallback: FontChoices) -> FontChoices {
        FontChoices {
            force_unicode_font: self.flag(FORCE_UNICODE, fallback.force_unicode_font),
            japanese_glyph_variants: self.flag(JAPANESE_VARIANTS, fallback.japanese_glyph_variants),
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.screen.size()
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.screen.resize(size);
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        self.screen.draw(frame, atlas, font, mouse);
    }

    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> SubScreenInput {
        self.screen.clicked(lang, mouse, shift)
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) {
        self.screen.dragged(lang, mouse, dy);
    }

    /// Tab (and Shift+Tab).
    pub fn tab(&mut self, forward: bool) {
        self.screen.tab(forward);
    }

    /// An arrow, Enter or Space.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> SubScreenInput {
        self.screen.key_pressed(lang, key, shift)
    }

    pub fn released(&mut self, lang: &Lang) {
        self.screen.released(lang);
    }

    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        self.screen.list.scrolled(mouse, scroll_y);
    }

    pub fn scroll_amount(&self) -> f64 {
        self.screen.list.scroll_amount()
    }
}
