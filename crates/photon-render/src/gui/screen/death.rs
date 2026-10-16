//! The death screen (`DeathScreen`): the title, what killed the player and
//! the score over a red gradient, and the buttons that wake after a second.

use photon_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// The buttons wake after this many ticks.
const WAKE_TICKS: i32 = 20;

/// What pressing a button asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathAction {
    /// "Respawn" (or "Spectate World" when hardcore).
    Respawn,
    /// "Title Screen".
    TitleScreen,
}

pub struct DeathScreen {
    arena: Arena,
    buttons: [(Widget, DeathAction); 2],
    title: Vec<Run>,
    cause: Option<Vec<Run>>,
    score: Vec<Run>,
    delay: i32,
    size: (i32, i32),
    /// Whether "Respawn" was pressed: its button stays asleep.
    pressed: bool,
    focus: Option<usize>,
}

/// `extractDeathBackground`: the red gradient over the whole screen, which
/// the confirmation of "Title Screen" shares.
pub fn draw_background(frame: &mut GuiFrame, atlas: &GuiAtlas, (width, height): (i32, i32)) {
    frame.fill_gradient(atlas, (0, 0, width, height), 0x6050_0000, 0xA080_3030);
}

fn runs(lang: &Lang, component: &Component) -> Vec<Run> {
    lang.runs(component, &Style::default())
}

impl DeathScreen {
    /// `DeathScreen` and `init`, for what killed the player (the server's
    /// message, if any) and the player's score.
    pub fn new(
        lang: &Lang,
        (width, height): (i32, i32),
        cause: Option<&Component>,
        hardcore: bool,
        score: i32,
    ) -> Self {
        let mut arena = Arena::default();
        let respawn_key = if hardcore {
            "deathScreen.spectate"
        } else {
            "deathScreen.respawn"
        };
        let mut button = |key: &str, y: i32| {
            let rect = arena.add(200, 20);
            arena.0[rect].x = width / 2 - 100;
            arena.0[rect].y = y;
            Widget {
                rect,
                active: false,
                kind: Kind::Button {
                    label: runs(lang, &Component::translatable(key)),
                },
            }
        };
        let respawn = button(respawn_key, height / 4 + 72);
        let title_screen = button("deathScreen.titleScreen", height / 4 + 96);
        let title_key = if hardcore {
            "deathScreen.title.hardcore"
        } else {
            "deathScreen.title"
        };
        let mut yellow = Component::literal(score.to_string());
        yellow.style.colour = photon_game::text::named_colour("yellow");
        Self {
            arena,
            buttons: [
                (respawn, DeathAction::Respawn),
                (title_screen, DeathAction::TitleScreen),
            ],
            title: runs(lang, &Component::translatable(title_key)),
            cause: cause.map(|cause| runs(lang, cause)),
            score: runs(
                lang,
                &Component::translatable_with("deathScreen.score.value", vec![yellow]),
            ),
            delay: 0,
            size: (width, height),
            pressed: false,
            focus: None,
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `tick`: the buttons wake on the twentieth.
    pub fn tick(&mut self) {
        self.delay += 1;
        if self.delay == WAKE_TICKS && !self.pressed {
            for (widget, _) in &mut self.buttons {
                widget.active = true;
            }
        }
    }

    /// The gradient, the buttons, then the title (twice as big), the cause
    /// and the score.
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let width = self.size.0;
        draw_background(frame, atlas, self.size);
        for (index, (widget, _)) in self.buttons.iter().enumerate() {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == Some(index);
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
        let middle = width / 2;
        let first = (frame.vertices.len(), frame.log.as_ref().map_or(0, Vec::len));
        let x = middle / 2 - font.width_of_runs(&self.title) / 2;
        frame.styled_text(atlas, font, &self.title, x, 30, 0xFFFF_FFFF, true);
        frame.scale_from(first, 2.0, (0.0, 0.0));
        if let Some(cause) = &self.cause {
            let x = middle - font.width_of_runs(cause) / 2;
            frame.styled_text(atlas, font, cause, x, 85, 0xFFFF_FFFF, true);
        }
        let x = middle - font.width_of_runs(&self.score) / 2;
        frame.styled_text(atlas, font, &self.score, x, 100, 0xFFFF_FFFF, true);
    }

    /// A press of the primary button at `mouse`.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<DeathAction> {
        let action = self
            .buttons
            .iter()
            .find(|(widget, _)| widget.is_active() && widget.contains(&self.arena, mouse))
            .map(|(_, action)| *action)?;
        self.pressed_action(action)
    }

    /// Tab: the focus moves to the next button.
    pub fn tab(&mut self, forward: bool) {
        let next = match (self.focus, forward) {
            (None | Some(1), true) => 0,
            (None | Some(0), false) => 1,
            (Some(_), _) => 1 - self.focus.unwrap_or(0),
        };
        self.focus = Some(next);
    }

    /// Enter or Space: the focused button.
    pub fn pressed(&mut self) -> Option<DeathAction> {
        let (widget, action) = self.buttons.get(self.focus?)?;
        if !widget.is_active() {
            return None;
        }
        let action = *action;
        self.pressed_action(action)
    }

    /// "Respawn" puts the button to sleep, as `button.active = false` does.
    fn pressed_action(&mut self, action: DeathAction) -> Option<DeathAction> {
        if action == DeathAction::Respawn {
            self.pressed = true;
            self.buttons[0].0.active = false;
        }
        Some(action)
    }
}
