//! The in-world game rules screen (`InWorldGameRulesScreen`), opened from the world settings: the
//! request for the server's values, the answer, and the changes it sends back.

use super::*;
use photon_game::text::Component;
use photon_gui::screen::confirm::ConfirmBackground;
use photon_gui::screen::game_rules::{GameRulesEvent, GameRulesScreen};

impl Controls {
    /// Whether the restrictions or the game rules are open over the world settings.
    pub fn has_world_overlay(&self) -> bool {
        self.restrictions.is_some() || self.game_rules.is_some()
    }

    /// The screen opens over the world settings; the request goes out with the next tick, which
    /// has the connection.
    pub(super) fn open_game_rules(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.game_rules = Some(GameRulesScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
        ));
        self.game_rules_request = true;
    }

    /// The request, the server's first answer, and the changes Done left: the connection's part
    /// of the screen. Without a screen an answer is dropped, as the reference ignores it.
    pub(super) fn game_rules_tick(
        &mut self,
        session: &mut PlaySession,
        assets: &ScreenAssets,
    ) -> Result {
        // Done closed the screen, and what it changed goes out afterwards.
        if let Some(changes) = self.game_rules_outbox.take() {
            session.set_game_rules(&changes)?;
        }
        let Some(screen) = self.game_rules.as_mut() else {
            self.game_rules_request = false;
            let _ = session.take_game_rule_values();
            return Ok(());
        };
        if std::mem::take(&mut self.game_rules_request) {
            session.request_game_rules()?;
        }
        if let Some(values) = session.take_game_rule_values() {
            screen.values_received(&values, &assets.creative.font, &assets.lang);
        }
        Ok(())
    }

    /// Losing the gamemaster permission closes the screen (`HasGamemasterPermissionReaction`).
    pub(super) fn drop_game_rules(&mut self) {
        self.game_rules = None;
        self.game_rules_confirm = None;
    }

    /// The question about pending changes (`ConfirmScreen` with the discard texts).
    fn open_discard_question(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let yes = Component::translatable("gui.yes");
        let no = Component::translatable("gui.no");
        self.game_rules_confirm = Some(
            ConfirmScreen::new(
                &assets.creative.font,
                &assets.lang,
                Self::gui_size(surface),
                &Component::translatable("editGamerule.inGame.discardChanges.title"),
                &Component::translatable("editGamerule.inGame.discardChanges.message"),
                (&yes, &no),
            )
            .with_background(ConfirmBackground::Menu { in_world: true }),
        );
    }

    /// What the screen asked for: Done sends what changed and closes, and the question opens when
    /// leaving would lose changes.
    fn game_rules_asked(
        &mut self,
        asked: Option<GameRulesEvent>,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        match asked {
            Some(GameRulesEvent::Close) => self.game_rules = None,
            Some(GameRulesEvent::Apply(changes)) => {
                if !changes.is_empty() {
                    self.game_rules_outbox = Some(changes);
                }
                self.game_rules = None;
            }
            Some(GameRulesEvent::ConfirmDiscard) => self.open_discard_question(surface, assets),
            None => {}
        }
    }

    /// An event for the open game rules screen, or for the question over it.
    pub(super) fn game_rules_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if self.game_rules_confirm.is_some() {
            return self.game_rules_question_event(event, surface);
        }
        use glfw::{Action, MouseButton, WindowEvent};
        let font = &assets.creative.font;
        let wheel = self.options.mouse_wheel_sensitivity;
        let control = self.mods.control;
        let mouse = self.gui_mouse;
        let Some(screen) = self.game_rules.as_mut() else {
            return;
        };
        let asked = match event {
            WindowEvent::CursorPos(x, y) => {
                let before = self.gui_mouse.1;
                self.move_gui_mouse((x, y), surface);
                let mouse = self.gui_mouse;
                if let Some(screen) = self.game_rules.as_mut() {
                    screen.dragged(mouse, mouse.1 - before);
                }
                None
            }
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                let mut clipboard = WindowClipboard(surface);
                screen.key_pressed(key, (control, shift), font, &mut clipboard)
            }
            WindowEvent::Char(character) => {
                screen.char_typed(character, font);
                None
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                screen.clicked(mouse, font)
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released();
                None
            }
            WindowEvent::Scroll(_, y) => {
                screen.scrolled(mouse, y * wheel);
                None
            }
            _ => None,
        };
        self.game_rules_asked(asked, surface, assets);
    }

    /// Yes discards the changes and leaves; No, or Escape, goes back to the list.
    fn game_rules_question_event(&mut self, event: glfw::WindowEvent, surface: &mut WindowSurface) {
        use glfw::{Action, Key, WindowEvent};
        let Some(question) = self.game_rules_confirm.as_mut() else {
            return;
        };
        let action = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => question.escape(),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                question.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                question.pressed()
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                question.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32))
            }
            _ => None,
        };
        match action {
            Some(ConfirmAction::Yes) => self.drop_game_rules(),
            Some(_) => self.game_rules_confirm = None,
            None => {}
        }
    }
}
