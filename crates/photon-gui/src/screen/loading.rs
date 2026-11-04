//! Photon's loading screen for a join from the menu: the game's name and description over a
//! banner, what the connection is doing, an indeterminate bar and Cancel. Later the banner shows
//! the images a server or the directory provides; until then it is an empty frame.

use crate::kit::{Painter, State, inside};
use crate::layout::Rect;
use crate::tokens::{CLASSIC, Skin};
use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

const PANEL: (i32, i32) = (240, 120);
const BANNER_HEIGHT: i32 = 50;
const BAR_HEIGHT: i32 = 4;
/// How long the bar's moving segment takes to cross, in milliseconds.
const SWEEP_MILLIS: u64 = 1600;

pub struct LoadingScreen {
    size: (i32, i32),
    skin: Skin,
    title: String,
    description: String,
    status: Vec<Run>,
    cancel_label: Vec<Run>,
}

impl LoadingScreen {
    /// A loading screen of a window `size` big for the game `title` with its `description`.
    pub fn new(lang: &Lang, size: (i32, i32), title: &str, description: &str) -> Self {
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        Self {
            size,
            skin: CLASSIC,
            title: title.to_owned(),
            description: description.to_owned(),
            status: runs("connect.connecting"),
            cancel_label: runs("gui.cancel"),
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The status text the key names.
    pub fn set_status(&mut self, lang: &Lang, key: &str) {
        self.status = lang.runs(&Component::translatable(key), &Style::default());
    }

    /// The card in the middle of the window.
    fn panel(&self) -> Rect {
        Rect {
            x: (self.size.0 - PANEL.0) / 2,
            y: (self.size.1 - PANEL.1) / 2 - 12,
            width: PANEL.0,
            height: PANEL.1,
        }
    }

    /// The Cancel button, under the card.
    fn cancel(&self) -> Rect {
        let panel = self.panel();
        Rect {
            x: self.size.0 / 2 - 40,
            y: panel.y + panel.height + 8,
            width: 80,
            height: 18,
        }
    }

    /// Whether a press at `mouse` is on Cancel.
    pub fn cancelled(&self, mouse: (i32, i32)) -> bool {
        inside(self.cancel(), mouse)
    }

    /// The moving segment of the bar for a moment: from the left end to the right, then again.
    fn sweep(track: Rect, millis: u64) -> Rect {
        let width = track.width / 3;
        let travel = track.width - width;
        let at = (millis % SWEEP_MILLIS) as i32 * travel / SWEEP_MILLIS as i32;
        Rect {
            x: track.x + at,
            width,
            ..track
        }
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        let millis = frame.millis;
        let mut painter = Painter {
            frame,
            atlas,
            font,
            skin: &self.skin,
        };
        let dim = (self.skin.bg & 0x00FF_FFFF) | 0x8000_0000;
        painter.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            dim,
        );
        let panel = self.panel();
        painter.panel(panel);
        let inner = crate::kit::shrink(panel, 2);
        self.draw_text(&mut painter, inner);
        let track = Rect {
            x: inner.x + 8,
            y: inner.y + inner.height - 22,
            width: inner.width - 16,
            height: BAR_HEIGHT,
        };
        painter.fill_rect(track, self.skin.line);
        painter.fill_rect(Self::sweep(track, millis), self.skin.ink);
        let status_width = font.width_of_runs(&self.status);
        let status_at = (inner.x + (inner.width - status_width) / 2, track.y + 8);
        painter
            .frame
            .styled_text(atlas, font, &self.status, status_at, self.skin.ink, true);
        let cancel = self.cancel();
        let state = if inside(cancel, mouse) {
            State::Hover
        } else {
            State::Normal
        };
        painter.button(cancel, &runs_text(&self.cancel_label), state);
    }

    /// The banner, the name and the description.
    fn draw_text(&self, painter: &mut Painter, inner: Rect) {
        let banner = Rect {
            height: BANNER_HEIGHT,
            ..inner
        };
        painter.fill_rect(banner, self.skin.bg | 0xFF00_0000);
        let text_y = banner.y + banner.height + 6;
        for (index, (text, colour)) in [
            (&self.title, self.skin.ink),
            (&self.description, self.skin.muted),
        ]
        .into_iter()
        .enumerate()
        {
            // Names and addresses come from operators: cut to the panel, not past it.
            let text = painter.fitted(text, inner.width - 8);
            let x = inner.x + (inner.width - painter.font.width(&text)) / 2;
            painter.label(&text, (x, text_y + 11 * index as i32), colour);
        }
    }
}

/// The plain text of runs.
fn runs_text(runs: &[Run]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> LoadingScreen {
        LoadingScreen::new(&Lang::default(), (427, 240), "Skyblock", "play.example.com")
    }

    #[test]
    fn cancel_is_under_the_card_and_nothing_else_cancels() {
        let screen = screen();
        let cancel = screen.cancel();
        assert!(cancel.y >= screen.panel().y + screen.panel().height);
        assert!(screen.cancelled((cancel.x + 5, cancel.y + 5)));
        assert!(!screen.cancelled((screen.panel().x + 5, screen.panel().y + 5)));
    }

    #[test]
    fn the_bar_segment_sweeps_across_and_starts_over() {
        let track = Rect {
            x: 10,
            y: 0,
            width: 90,
            height: 4,
        };
        assert_eq!(LoadingScreen::sweep(track, 0).x, 10);
        let late = LoadingScreen::sweep(track, SWEEP_MILLIS - 1);
        assert!(late.x + late.width <= track.x + track.width);
        assert_eq!(LoadingScreen::sweep(track, SWEEP_MILLIS).x, 10);
    }
}
