use super::*;

impl Controls {
    /// `onDisconnect`: a lost connection puts the disconnected screen in the
    /// place of whatever was open.
    pub fn receive_disconnect(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(reason) = session.disconnected() else {
            return;
        };
        if self.disconnected.is_some() {
            return;
        }
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.disconnected = Some(DisconnectedScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            reason,
        ));
        self.chat = None;
        self.link_confirm = None;
        self.pause = None;
        self.options_screen = None;
        self.video_screen = None;
        self.chat_options_screen = None;
        self.accessibility_screen = None;
        self.skin_screen = None;
        self.social = None;
        self.debug_options = None;
        self.stats = None;
        self.advancements = None;
        self.sound_screen = None;
        self.controls_screen = None;
        self.language_screen = None;
        self.mouse_screen = None;
        self.key_binds_screen = None;
        self.death = None;
        self.confirm = None;
        self.screen = None;
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// An event for the disconnected screen: it takes no Escape, and its
    /// button leaves for the title screen (there is no server list yet).
    pub(super) fn disconnected_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
    ) {
        use glfw::{Action, Key, WindowEvent};
        let Some(screen) = self.disconnected.as_mut() else {
            return;
        };
        match event {
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, _) => screen.tab(),
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _)
                if screen.pressed() =>
            {
                self.leave = true;
            }
            WindowEvent::CursorPos(x, y) => self.move_gui_mouse((x, y), surface),
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _)
                if screen.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32)) =>
            {
                self.leave = true;
            }
            _ => {}
        }
    }

    /// `handlePlayerCombatKill`: a death the server told of opens the death
    /// screen, in the place of whatever was open; the player having health
    /// again closes it.
    pub fn receive_death(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if let Some(message) = session.take_death() {
            let size = surface.scale();
            let size = (size.gui_scaled_width(), size.gui_scaled_height());
            let score = session
                .world()
                .local_player()
                .map_or(0, photon_world::ClientPlayerState::score);
            self.death = Some(DeathScreen::new(
                &assets.lang,
                size,
                Some(&message),
                session.hardcore(),
                score,
            ));
            self.confirm = None;
            if let Some(chat) = self.chat.take() {
                // A death takes the box's place: its text is kept as a draft.
                self.leave_chat(&chat, ExitReason::Interrupted);
            }
            self.pause = None;
            self.options_screen = None;
            self.video_screen = None;
            self.chat_options_screen = None;
            self.accessibility_screen = None;
            self.skin_screen = None;
            self.social = None;
            self.debug_options = None;
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
        } else if self.death.is_some() && session.status().health > 0.0 {
            self.close_death(surface);
        }
    }

    /// The death screen goes, with the confirmation over it, and the mouse
    /// is grabbed again (`setScreen(null)`).
    pub(super) fn close_death(&mut self, surface: &mut WindowSurface) {
        self.death = None;
        self.confirm = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// What a death screen button does: "Respawn" asks the server; "Title
    /// Screen" leaves, after a confirmation unless the world is hardcore
    /// (`handleExitToTitleScreen`).
    pub(super) fn death_action(
        &mut self,
        action: DeathAction,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        match action {
            DeathAction::Respawn => session.respawn()?,
            DeathAction::TitleScreen if session.hardcore() => surface.request_close(),
            DeathAction::TitleScreen => self.open_title_confirm(surface, assets),
        }
        Ok(())
    }

    /// An event for the open death screen: it takes no Escape.
    pub(super) fn death_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let Some(death) = self.death.as_mut() else {
            return Ok(());
        };
        let action = match event {
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                death.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                death.pressed()
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                death.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32))
            }
            _ => None,
        };
        match action {
            Some(action) => self.death_action(action, session, surface, assets),
            None => Ok(()),
        }
    }
}
