use super::*;

impl Controls {
    /// `DeathScreen.TitleConfirmScreen`, its buttons asleep for a second.
    pub(super) fn open_title_confirm(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        use photon_game::text::Component;
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let mut confirm = ConfirmScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            &Component::translatable("deathScreen.quit.confirm"),
            &Component::empty(),
            &Component::translatable("deathScreen.titleScreen"),
            &Component::translatable("deathScreen.respawn"),
        )
        .with_background(ConfirmBackground::Death);
        confirm.set_delay(20);
        self.confirm = Some(confirm);
    }

    /// What the confirmation's callback does: yes leaves; no respawns and
    /// closes the screens.
    pub(super) fn confirm_action(
        &mut self,
        action: ConfirmAction,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        if self.pack_prompt.is_some() {
            self.confirm = None;
            self.answer_pack_prompt(action == ConfirmAction::Yes, session);
            // `setScreen(null)`: the game has the mouse again unless something else holds it.
            if self.screen.is_none() && self.disconnected.is_none() {
                self.captured = true;
                surface.set_cursor_captured(true);
            }
            return Ok(());
        }
        match action {
            ConfirmAction::Yes => surface.request_close(),
            ConfirmAction::No => {
                session.respawn()?;
                self.close_death(surface);
            }
            // The death confirmation has no third button.
            ConfirmAction::Copy => {}
        }
        Ok(())
    }

    /// An event for the open confirmation: Escape answers no once it is
    /// awake.
    pub(super) fn confirm_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let Some(confirm) = self.confirm.as_mut() else {
            return Ok(());
        };
        let action = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => confirm.escape(),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                confirm.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                confirm.pressed()
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                confirm.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32))
            }
            _ => None,
        };
        match action {
            Some(action) => self.confirm_action(action, session, surface),
            None => Ok(()),
        }
    }
}
