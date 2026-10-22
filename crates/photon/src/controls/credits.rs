use super::*;

impl Controls {
    /// `handleGameEvent` for `WIN_GAME`: the credits open, with the end poem, in the place of
    /// whatever was open.
    pub fn receive_win_game(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if !session.take_win_game() {
            return;
        }
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.credits = Some(CreditsScreen::new(
            &assets.creative.font,
            &assets.assets_root,
            size,
            true,
            session.local_name(),
        ));
        if let Some(chat) = self.chat.take() {
            self.leave_chat(&chat, ExitReason::Interrupted);
        }
        self.confirm = None;
        self.death = None;
        self.pause = None;
        self.options_screen = None;
        self.video_screen = None;
        self.chat_options_screen = None;
        self.accessibility_screen = None;
        self.skin_screen = None;
        self.font_screen = None;
        self.drop_light_screens();
        self.stats = None;
        self.drop_advancements(session);
        self.sound_screen = None;
        self.controls_screen = None;
        self.language_screen = None;
        self.mouse_screen = None;
        self.key_binds_screen = None;
        self.screen = None;
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// `WinScreen.tick`: the roll that has run past its end asks to respawn.
    pub fn credits_tick(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        if self.credits.as_ref().is_some_and(CreditsScreen::tick) {
            self.close_credits(session, surface)?;
        }
        Ok(())
    }

    /// `WinScreen.respawn`: the server is asked to bring the player back, and the screen goes.
    fn close_credits(&mut self, session: &mut PlaySession, surface: &mut WindowSurface) -> Result {
        session.respawn()?;
        self.credits = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
        Ok(())
    }

    /// An event for the credits: Escape closes them (`onClose`), the speed keys steer the roll.
    pub(super) fn credits_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let Some(credits) = self.credits.as_mut() else {
            return Ok(());
        };
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press | Action::Repeat, _) => {
                return self.close_credits(session, surface);
            }
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, _) => {
                credits.key_pressed(key as i32);
            }
            WindowEvent::Key(key, _, Action::Release, _) => credits.key_released(key as i32),
            _ => {}
        }
        Ok(())
    }
}
