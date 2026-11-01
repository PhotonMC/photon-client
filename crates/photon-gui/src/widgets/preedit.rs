//! The overlay an input method's composition text is drawn in (`IMEPreeditOverlay`): a small panel
//! under the text being edited (over it when there is no room below) holding the text being
//! composed, its focused block underlined, with a blinking caret.

use crate::{FontRuns, FrameRuns};
use photon_game::input::event::PreeditEvent;
use photon_game::text::{Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `SEPARATION_FROM_INPUT`.
const SEPARATION: i32 = 4;
/// The height of the line drawn (`Font.lineHeight`).
const LINE: i32 = 9;
/// `TextCursorUtils.isCursorVisible`: on for 300 ms, off for 300.
pub fn cursor_visible(millis: u64) -> bool {
    (millis / 300).is_multiple_of(2)
}

/// One composition in progress, with where its input is.
#[derive(Debug, Clone)]
pub struct PreeditOverlay {
    runs: Vec<Run>,
    width: i32,
    caret: i32,
    opened_at: u64,
    input_height: i32,
}

impl PreeditOverlay {
    /// `new IMEPreeditOverlay(event, font, inputHeight)` at the time `now` (milliseconds).
    pub fn new(event: &PreeditEvent, font: &Font, input_height: i32, now: u64) -> Self {
        let runs = Self::runs(event);
        let before = (event.caret_position.max(0) as usize).min(event.full_text.len());
        Self {
            width: font.width_of_runs(&runs),
            caret: font.width(&String::from_utf16_lossy(&event.full_text[..before])),
            runs,
            opened_at: now,
            input_height,
        }
    }

    /// `PreeditEvent.toFormattedText(underlined)`, black: the focused block underlined, or the
    /// only block whatever the focus.
    fn runs(event: &PreeditEvent) -> Vec<Run> {
        let run = |text: &[u16], underlined: bool| Run {
            text: String::from_utf16_lossy(text),
            style: Style {
                colour: Some(0),
                underlined: underlined.then_some(true),
                ..Style::default()
            },
        };
        if let [only] = event.blocks.as_slice() {
            return vec![run(only, true)];
        }
        event
            .blocks
            .iter()
            .enumerate()
            .map(|(index, block)| run(block, index as i32 == event.focused_block))
            .collect()
    }

    /// `extractRenderState`: the panel, the text and the caret, kept inside the `screen`, for an
    /// input whose cursor is at `input` (`updateInputPosition`).
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        screen: (i32, i32),
        input: (i32, i32),
    ) {
        let mut left = input.0;
        let mut right = left + self.width;
        if right > screen.0 {
            left = screen.0 - self.width;
            right = left + self.width;
        }
        let input_bottom = input.1 + self.input_height;
        let mut bottom = input_bottom + SEPARATION + LINE;
        if bottom > screen.1 {
            bottom = input.1 - SEPARATION - LINE;
        }
        let top = bottom - LINE;
        frame.sprite(
            atlas,
            "widget/preedit",
            left - 5,
            top - 5,
            right - left + 10,
            bottom - top + 10,
        );
        frame.styled_text(atlas, font, &self.runs, (left, top), 0xFF00_0000, false);
        if cursor_visible(frame.millis.saturating_sub(self.opened_at)) {
            let x = left + self.caret;
            frame.fill(atlas, (x, top - 1, x + 1, top + LINE + 1), 0xFF00_0000);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caret_blinks_every_three_hundred_milliseconds() {
        assert!(cursor_visible(0));
        assert!(cursor_visible(299));
        assert!(!cursor_visible(300));
        assert!(cursor_visible(600));
    }
}
