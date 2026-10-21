//! The line a restricted chat shows over its messages, and the click on it.

use super::*;
use crate::screen::confirm::head_by_width;
use crate::{FontRuns, FrameRuns};
use photon_game::text::ClickEvent;

/// `ChatComponent.GO_TO_RESTRICTIONS_SCREEN`: the click that opens the restrictions.
pub const GO_TO_RESTRICTIONS_ID: &str = "minecraft:internal/go_to_restrictions_screen";

impl ChatLog {
    /// Whether the line is up: an open chat of a player with restrictions
    /// (`DisplayMode.FOREGROUND_RESTRICTED`).
    pub(super) fn prompt_up(&self, focused: bool) -> bool {
        focused && self.abilities.has_any_restrictions()
    }

    /// `chat_screen.restricted` as red, underlined text that opens the restrictions, clipped to
    /// the chat's width (and then with a hover of its own text) when it does not fit.
    pub(super) fn prompt_runs(&self, font: &Font, lang: &Lang, max_width: i32) -> Vec<Run> {
        let style = Style {
            colour: photon_game::text::named_colour("red"),
            underlined: Some(true),
            click: Some(Box::new(ClickEvent::Custom(
                GO_TO_RESTRICTIONS_ID.into(),
                None,
            ))),
            ..Style::default()
        };
        let text = Component::translatable("chat_screen.restricted");
        let runs = lang.runs(&text, &style);
        if font.width_of_runs(&runs) <= max_width {
            return runs;
        }
        let hover = Style {
            hover: Some(Box::new(HoverEvent::ShowText(Box::new(text.clone())))),
            ..style
        };
        let runs = lang.runs(&text, &hover);
        let ellipsis = lang.runs(&Component::literal("..."), &Style::default());
        let room = max_width - font.width_of_runs(&ellipsis);
        let mut clipped = head_by_width(&runs, font, room);
        clipped.extend(ellipsis);
        clipped
    }

    /// Where the line is: its band over the oldest message shown.
    pub(super) fn prompt_band(&self, shown: usize) -> i32 {
        -((shown as i32 + 1) * self.entry_height())
    }

    /// The band behind the line.
    pub(super) fn prompt_background(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (chat_bottom, shown): (i32, usize),
        (max_width, focused): (i32, bool),
    ) {
        if !self.prompt_up(focused) {
            return;
        }
        let top = chat_bottom + self.prompt_band(shown);
        let alpha = ((self.settings.background_opacity as f32 * 255.0) as u32) << 24;
        frame.fill(
            atlas,
            (-2, top, max_width + 4 + 4, top + self.entry_height()),
            alpha,
        );
    }

    /// The line's text row.
    fn prompt_top(&self, chat_bottom: i32, shown: usize) -> i32 {
        let spacing = self.settings.line_spacing;
        let message_y = (8.0 * (spacing + 1.0) - 4.0 * spacing + 0.5).floor() as i32;
        chat_bottom + self.prompt_band(shown) + self.entry_height() - message_y - 1
    }

    /// Draws the line, and gives back its style when the open chat's mouse is over it.
    pub(super) fn prompt_text(
        &self,
        (frame, atlas, font, lang): (&mut GuiFrame, &GuiAtlas, &Font, &Lang),
        (chat_bottom, shown, max_width): (i32, usize, i32),
        (text_opacity, hover): (f32, Option<(f32, f32)>),
        focused: bool,
    ) -> Option<Style> {
        if !self.prompt_up(focused) {
            return None;
        }
        let runs = self.prompt_runs(font, lang, max_width);
        let top = self.prompt_top(chat_bottom, shown);
        let colour = ((text_opacity * 255.0) as u32) << 24 | 0x00FF_FFFF;
        frame.styled_text(atlas, font, &runs, 0, top, colour, true);
        crate::text_hit::style_under(font, &runs, (0, top), hover?).cloned()
    }

    /// The style of the line under a press, if the line is up.
    pub(super) fn prompt_style(
        &self,
        (font, lang): (&Font, &Lang),
        (chat_bottom, shown, max_width): (i32, usize, i32),
        point: (f32, f32),
    ) -> Option<Style> {
        if !self.prompt_up(true) {
            return None;
        }
        let runs = self.prompt_runs(font, lang, max_width);
        let top = self.prompt_top(chat_bottom, shown);
        crate::text_hit::style_under(font, &runs, (0, top), point).cloned()
    }
}
