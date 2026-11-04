//! The question a link or a command asks over the screen: a vanilla `ConfirmScreen`, or Photon's
//! own `NoticeScreen` for the questions Photon's links ask over a game. They answer the same
//! way, so what holds one does not care which it is.

use photon_game::text::Lang;
use photon_gui::screen::confirm::{ConfirmAction, ConfirmScreen};
use photon_gui::screen::notice::{Answer, NoticeScreen};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

pub(crate) enum Question {
    Confirm(ConfirmScreen),
    Notice(NoticeScreen),
}

impl From<ConfirmScreen> for Question {
    fn from(screen: ConfirmScreen) -> Self {
        Self::Confirm(screen)
    }
}

impl From<NoticeScreen> for Question {
    fn from(screen: NoticeScreen) -> Self {
        Self::Notice(screen)
    }
}

fn action(answer: Answer) -> ConfirmAction {
    match answer {
        Answer::Yes => ConfirmAction::Yes,
        Answer::Back => ConfirmAction::No,
    }
}

impl Question {
    pub(crate) fn size(&self) -> (i32, i32) {
        match self {
            Self::Confirm(screen) => screen.size(),
            Self::Notice(screen) => screen.size(),
        }
    }

    /// The question again for another size.
    pub(crate) fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        match self {
            Self::Confirm(screen) => Self::Confirm(screen.resized(font, lang, size)),
            Self::Notice(screen) => Self::Notice(screen.resized(font, lang, size)),
        }
    }

    /// Puts the buttons back to sleep where a delay is still running after a resize.
    pub(crate) fn rest(&mut self) {
        // A notice keeps its delay through `resized`.
        if let Self::Confirm(screen) = self {
            screen.rest();
        }
    }

    pub(crate) fn tick(&mut self) {
        match self {
            Self::Confirm(screen) => screen.tick(),
            Self::Notice(screen) => screen.tick(),
        }
    }

    pub(crate) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
        blurriness: i32,
    ) {
        match self {
            Self::Confirm(screen) => screen.draw(frame, (atlas, font), mouse, blurriness),
            Self::Notice(screen) => screen.draw(frame, atlas, font, mouse),
        }
    }

    pub(crate) fn clicked(&self, mouse: (i32, i32)) -> Option<ConfirmAction> {
        match self {
            Self::Confirm(screen) => screen.clicked(mouse),
            Self::Notice(screen) => screen.answer_clicked(mouse).map(action),
        }
    }

    pub(crate) fn tab(&mut self, forward: bool) {
        match self {
            Self::Confirm(screen) => screen.tab(forward),
            Self::Notice(screen) => screen.cycle(),
        }
    }

    pub(crate) fn pressed(&self) -> Option<ConfirmAction> {
        match self {
            Self::Confirm(screen) => screen.pressed(),
            Self::Notice(screen) => screen.answer_pressed().map(action),
        }
    }

    pub(crate) fn escape(&self) -> Option<ConfirmAction> {
        match self {
            Self::Confirm(screen) => screen.escape(),
            Self::Notice(screen) => screen.escape().then_some(ConfirmAction::No),
        }
    }
}
