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

/// What a question's buttons answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Yes,
    Back,
}

pub struct NoticeScreen {
    size: (i32, i32),
    heading: Vec<Run>,
    reason: Vec<Vec<Run>>,
    label: Vec<Run>,
    panel: Rect,
    button: Rect,
    focused: bool,
    /// The second button of a question, and whether it has the focus (the first is the "no").
    yes: Option<(Vec<Run>, Rect)>,
    yes_focused: bool,
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
            yes: None,
            yes_focused: false,
        }
    }

    /// A question: the same screen with a second button. The first button, which Escape and a
    /// stray Enter choose, says no.
    pub fn asking(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        (heading, reason, no, yes): (&Component, &Component, &Component, &Component),
    ) -> Self {
        let mut screen = Self::new(font, lang, size, (heading, reason, no));
        let width = (screen.panel.width - 3 * PAD) / 2;
        let y = screen.button.y;
        screen.button = Rect {
            x: screen.panel.x + PAD,
            width,
            ..screen.button
        };
        let rect = Rect {
            x: screen.panel.x + PAD + width + PAD,
            y,
            width,
            height: BUTTON.1,
        };
        screen.yes = Some((lang.runs(yes, &Style::default()), rect));
        screen.focused = true;
        screen
    }

    /// A press at the mouse of a question: which button it was on.
    pub fn answer_clicked(&self, mouse: (i32, i32)) -> Option<Answer> {
        let answer = match &self.yes {
            Some((_, rect)) if inside(*rect, mouse) => Answer::Yes,
            _ if inside(self.button, mouse) => Answer::Back,
            _ => return None,
        };
        crate::ui_sound::play_down_sound();
        Some(answer)
    }

    /// Enter or Space on a question: the focused button, which starts as the "no".
    pub fn answer_pressed(&self) -> Option<Answer> {
        crate::ui_sound::play_down_sound();
        Some(if self.yes_focused {
            Answer::Yes
        } else {
            Answer::Back
        })
    }

    /// Tab on a question moves the focus to the other button.
    pub fn cycle(&mut self) {
        if self.yes.is_some() {
            self.yes_focused = !self.yes_focused;
        }
        self.focused = true;
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
        let lit = |rect: Rect, focused: bool| {
            if inside(rect, mouse) || focused {
                State::Hover
            } else {
                State::Normal
            }
        };
        let text = |runs: &[Run]| runs.iter().map(|run| run.text.as_str()).collect::<String>();
        let back_focused = self.focused && !self.yes_focused;
        painter.button(
            self.button,
            &text(&self.label),
            lit(self.button, back_focused),
        );
        if let Some((label, rect)) = &self.yes {
            painter.button(*rect, &text(label), lit(*rect, self.yes_focused));
        }
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

    fn question() -> NoticeScreen {
        let (font, lang) = (Font::default(), Lang::default());
        NoticeScreen::asking(
            &font,
            &lang,
            (427, 240),
            (
                &Component::literal("Sign in?"),
                &Component::literal("As Miner"),
                &Component::literal("Cancel"),
                &Component::literal("Sign in"),
            ),
        )
    }

    #[test]
    fn a_question_answers_by_the_button_clicked() {
        let ask = question();
        let yes = ask.yes.as_ref().expect("yes").1;
        assert_eq!(ask.answer_clicked(middle(yes)), Some(Answer::Yes));
        assert_eq!(ask.answer_clicked(middle(ask.button)), Some(Answer::Back));
        assert_eq!(ask.answer_clicked((0, 0)), None);
        assert!(yes.x >= ask.button.x + ask.button.width);
        assert!(yes.x + yes.width <= ask.panel.x + ask.panel.width);
    }

    #[test]
    fn a_stray_enter_on_a_question_says_no_and_only_tab_then_enter_says_yes() {
        let mut ask = question();
        assert_eq!(ask.answer_pressed(), Some(Answer::Back));
        assert!(ask.escape());
        ask.cycle();
        assert_eq!(ask.answer_pressed(), Some(Answer::Yes));
        ask.cycle();
        assert_eq!(ask.answer_pressed(), Some(Answer::Back));
    }
}
