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
    /// What a plain notice was made for, to make it again for another size.
    made_for: (Component, Component, Component),
    /// The label of a question's second button, to make the question again for another size.
    yes_label: Option<Component>,
    /// `delayTicker`: a question that opens by itself answers nothing until this runs out.
    delay: i32,
    /// How opaque the wall behind the screen is: a game or the panorama can show through.
    veil: u32,
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
            made_for: (heading.clone(), reason.clone(), back.clone()),
            yes_label: None,
            delay: 0,
            veil: 0xFF00_0000,
        }
    }

    /// A plain notice again for a screen of another size.
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        let (heading, reason, back) = &self.made_for;
        let mut screen = match &self.yes_label {
            Some(yes) => Self::asking(font, lang, size, (heading, reason, back, yes)),
            None => Self::new(font, lang, size, (heading, reason, back)),
        };
        // A question made again for another size is as asleep as it was.
        screen.delay = self.delay;
        screen.veil = self.veil;
        screen
    }

    /// The buttons sleep, and Escape is shut, for `ticks`: a question that opens over a game
    /// must not be answered by a click the player was already making.
    pub fn set_delay(&mut self, ticks: i32) {
        self.delay = ticks;
    }

    /// The screen shows the game through a dim, not a plain wall.
    #[must_use]
    pub fn over_game(mut self) -> Self {
        self.veil = 0xC000_0000;
        self
    }

    /// The screen shows the title panorama through the same dim the loading screen has.
    #[must_use]
    pub fn over_panorama(mut self) -> Self {
        self.veil = 0x8000_0000;
        self
    }

    /// A tick: the delay runs down.
    pub fn tick(&mut self) {
        self.delay = (self.delay - 1).max(0);
    }

    fn awake(&self) -> bool {
        self.delay == 0
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
        screen.yes_label = Some(yes.clone());
        screen.focused = true;
        screen
    }

    /// A press at the mouse of a question: which button it was on.
    pub fn answer_clicked(&self, mouse: (i32, i32)) -> Option<Answer> {
        if !self.awake() {
            return None;
        }
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
        if !self.awake() {
            return None;
        }
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
        painter.fill_rect(veil, (skin.bg & 0x00FF_FFFF) | self.veil);
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
        self.draw_buttons(&mut painter, mouse);
    }

    /// The button, and a question's second one; asleep they are greyed.
    fn draw_buttons(&self, painter: &mut Painter, mouse: (i32, i32)) {
        let awake = self.awake();
        let lit = |rect: Rect, focused: bool| {
            if !awake {
                State::Disabled
            } else if inside(rect, mouse) || focused {
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
        self.awake()
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
    fn a_notice_made_again_for_another_size_keeps_its_words_in_the_new_size() {
        let (font, lang) = (Font::default(), Lang::default());
        let notice = screen((427, 240), "A reason").resized(&font, &lang, (200, 120));
        assert_eq!(notice.size(), (200, 120));
        assert!(inside(notice.panel, middle(notice.button)));
        assert_eq!(notice.made_for.1, Component::literal("A reason"));
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
    fn a_question_over_a_game_answers_nothing_until_its_delay_runs_out() {
        let (font, lang) = (Font::default(), Lang::default());
        let mut ask = question();
        ask.set_delay(2);
        let yes = ask.yes.as_ref().expect("yes").1;
        assert_eq!(ask.answer_clicked(middle(yes)), None);
        assert_eq!(ask.answer_pressed(), None);
        assert!(!ask.escape());
        // Made again for another size, it is still asleep, and still a question.
        let mut again = ask.resized(&font, &lang, (300, 200));
        assert_eq!(again.answer_pressed(), None);
        again.tick();
        again.tick();
        let yes = again.yes.as_ref().expect("still a question").1;
        assert_eq!(again.answer_clicked(middle(yes)), Some(Answer::Yes));
        assert!(again.escape());
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
