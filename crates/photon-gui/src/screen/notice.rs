//! Photon's own notice screen: a heading, a reason that is wrapped, and one button back. The
//! menu shows it when a join fails (a launch link, a directory card or the server list).

use crate::chat::split;
use crate::kit::{Painter, State, inside};
use crate::layout::Rect;
use crate::tokens::CLASSIC;
use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// The panel is at most this wide, in units.
const PANEL_WIDTH: i32 = 260;
const BUTTON: (i32, i32) = (150, 20);
const PAD: i32 = 10;

pub struct NoticeScreen {
    size: (i32, i32),
    heading: Vec<Run>,
    reason: Vec<Vec<Run>>,
    label: Vec<Run>,
    panel: Rect,
    button: Rect,
    focused: bool,
    /// What it was made for, to make it again for another size.
    made_for: (Component, Component, Component),
}

impl NoticeScreen {
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        (heading, reason, back): (&Component, &Component, &Component),
    ) -> Self {
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let (heading_runs, reason_runs, label) = (runs(heading), runs(reason), runs(back));
        let width = PANEL_WIDTH.min(size.0 - 2 * PAD).max(40);
        let lines = if reason_runs.iter().all(|run| run.text.is_empty()) {
            Vec::new()
        } else {
            split(&reason_runs, width - 2 * PAD, font)
        };
        let height = PAD + 9 + PAD + lines.len() as i32 * 9 + PAD + BUTTON.1 + PAD;
        let panel = Rect {
            x: (size.0 - width) / 2,
            y: ((size.1 - height) / 2).max(0),
            width,
            height,
        };
        let button = Rect {
            x: panel.x + (width - BUTTON.0.min(width - 2 * PAD)) / 2,
            y: panel.y + height - PAD - BUTTON.1,
            width: BUTTON.0.min(width - 2 * PAD),
            height: BUTTON.1,
        };
        Self {
            size,
            heading: heading_runs,
            reason: lines,
            label,
            panel,
            button,
            focused: false,
            made_for: (heading.clone(), reason.clone(), back.clone()),
        }
    }

    /// The screen again for a screen of another size.
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        let (heading, reason, back) = &self.made_for;
        Self::new(font, lang, size, (heading, reason, back))
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let skin = CLASSIC;
        let mut painter = Painter {
            frame,
            atlas,
            font,
            skin: &skin,
        };
        let (width, height) = self.size;
        let veil = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        painter.fill_rect(veil, skin.bg | 0xFF00_0000);
        painter.panel(self.panel);
        let middle = self.panel.x + self.panel.width / 2;
        let centred = |runs: &[Run]| middle - font.width_of_runs(runs) / 2;
        let mut y = self.panel.y + PAD;
        painter.frame.styled_text(
            atlas,
            font,
            &self.heading,
            (centred(&self.heading), y),
            skin.ink,
            false,
        );
        y += 9 + PAD;
        for line in &self.reason {
            painter
                .frame
                .styled_text(atlas, font, line, (centred(line), y), skin.muted, false);
            y += 9;
        }
        let state = if inside(self.button, mouse) || self.focused {
            State::Hover
        } else {
            State::Normal
        };
        let label: String = self.label.iter().map(|run| run.text.as_str()).collect();
        painter.button(self.button, &label, state);
    }

    /// A press of the primary button: whether it was on the button.
    pub fn clicked(&self, mouse: (i32, i32)) -> bool {
        let hit = inside(self.button, mouse);
        if hit {
            crate::ui_sound::play_down_sound();
        }
        hit
    }

    /// Tab focuses the button, the only one.
    pub fn tab(&mut self) {
        self.focused = true;
    }

    /// Enter or Space on the focused button.
    pub fn pressed(&self) -> bool {
        if self.focused {
            crate::ui_sound::play_down_sound();
        }
        self.focused
    }

    /// Escape goes where the button goes.
    pub fn escape(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(size: (i32, i32), reason: &str) -> NoticeScreen {
        let (font, lang) = (Font::default(), Lang::default());
        NoticeScreen::new(
            &font,
            &lang,
            size,
            (
                &Component::literal("Heading"),
                &Component::literal(reason),
                &Component::literal("Back"),
            ),
        )
    }

    fn middle(rect: Rect) -> (i32, i32) {
        (rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    #[test]
    fn the_button_is_pressed_by_a_click_on_it_and_by_enter_only_when_focused() {
        let mut notice = screen((427, 240), "No");
        assert!(notice.clicked(middle(notice.button)));
        assert!(!notice.clicked((0, 0)));
        assert!(!notice.pressed());
        notice.tab();
        assert!(notice.pressed());
        assert!(notice.escape());
    }

    #[test]
    fn the_panel_and_its_button_stay_on_a_narrow_screen() {
        // The test font has no glyphs, so wrapping is not tested here (a live capture showed it).
        for width in [427, 120, 60] {
            let notice = screen((width, 240), "A reason");
            assert!(notice.panel.x >= 0 && notice.panel.x + notice.panel.width <= width);
            assert!(inside(notice.panel, middle(notice.button)));
            assert!(notice.button.width <= notice.panel.width);
        }
    }
}
