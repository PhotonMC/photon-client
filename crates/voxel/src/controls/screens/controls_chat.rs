use super::*;

impl Controls {
    /// An event for the open sound settings.
    pub(in super::super) fn sound_options_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(screen) = self.sound_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            self.take_sound_choices();
            return;
        }
        let Some(screen) = self.sound_screen.as_mut() else {
            return;
        };
        let mut done = false;
        match menu_key(&event) {
            Some(MenuKey::Tab(forward)) => screen.tab(forward),
            Some(MenuKey::Key(key, shift)) => done = screen.key_pressed(lang, key, shift).done,
            None => {}
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                done = screen.clicked(lang, self.gui_mouse, shift).done;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released(lang);
            }
            WindowEvent::Scroll(_, y) => {
                screen.scrolled(self.gui_mouse, y * self.options.mouse_wheel_sensitivity);
            }
            _ => {}
        }
        self.take_sound_choices();
        if done {
            self.close_sound_options();
        }
    }

    /// `ChatOptionsScreen`, over the options screen.
    pub(in super::super) fn open_chat_options(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        let scale = surface.scale();
        self.chat_options_screen = Some(ChatOptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            (scale.gui_scaled_width(), scale.gui_scaled_height()),
            true,
            &self.options,
        ));
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen,
    /// and the options are saved.
    pub(in super::super) fn close_chat_options(&mut self) {
        self.chat_options_screen = None;
        let _ = self.options.save(&options_path());
    }

    /// The chat options the screen shows are the options: the chat wraps
    /// its lines again when they changed (`rescaleChat`).
    pub(in super::super) fn take_chat_choices(&mut self, assets: &ScreenAssets) {
        let Some(screen) = self.chat_options_screen.as_ref() else {
            return;
        };
        let choices = screen.choices();
        let options = &mut self.options;
        options.chat_visibility = choices.visibility;
        options.save_chat_drafts = choices.save_drafts;
        options.chat_width = choices.settings.width;
        options.chat_scale = choices.settings.scale;
        options.chat_opacity = choices.settings.opacity;
        options.text_background_opacity = choices.settings.background_opacity;
        options.chat_line_spacing = choices.settings.line_spacing;
        options.chat_height_focused = choices.settings.height_focused;
        options.chat_height_unfocused = choices.settings.height_unfocused;
        self.chat_log
            .set_settings(choices.settings, &assets.creative.font);
    }

    /// An event for the open chat settings.
    pub(in super::super) fn chat_options_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(screen) = self.chat_options_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            self.take_chat_choices(assets);
            return;
        }
        let Some(screen) = self.chat_options_screen.as_mut() else {
            return;
        };
        let mut done = false;
        match menu_key(&event) {
            Some(MenuKey::Tab(forward)) => screen.tab(forward),
            Some(MenuKey::Key(key, shift)) => done = screen.key_pressed(lang, key, shift).done,
            None => {}
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                done = screen.clicked(lang, self.gui_mouse, shift).done;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released(lang);
            }
            WindowEvent::Scroll(_, y) => {
                screen.scrolled(self.gui_mouse, y * self.options.mouse_wheel_sensitivity);
            }
            _ => {}
        }
        self.take_chat_choices(assets);
        if done {
            self.close_chat_options();
        }
    }

    /// An event for the open video settings.
    pub(in super::super) fn video_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(video) = self.video_screen.as_mut() {
                video.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            return;
        }
        let Some(video) = self.video_screen.as_mut() else {
            return;
        };
        // `Minecraft.resizeGui`: a new GUI scale is the option's, and the
        // window answers with the size it gives the screen.
        let wheel = self.options.mouse_wheel_sensitivity;
        let options = &mut self.options;
        let mut resize = |scale: i32| {
            options.gui_scale = scale;
            surface.set_gui_scale_option(scale);
            let size = surface.scale();
            (size.gui_scaled_width(), size.gui_scaled_height())
        };
        let mut done = false;
        match menu_key(&event) {
            Some(MenuKey::Tab(forward)) => video.tab(forward),
            Some(MenuKey::Key(key, shift)) => {
                done = video.key_pressed(lang, key, shift, &mut resize).done;
            }
            None => {}
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                done = video.clicked(lang, self.gui_mouse, shift, &mut resize).done;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                video.released(lang);
            }
            WindowEvent::Scroll(_, y) => {
                video.scrolled(
                    lang,
                    self.gui_mouse,
                    y * wheel,
                    self.control_held,
                    &mut resize,
                );
            }
            _ => {}
        }
        let shown = video.values(Self::video_values(&self.options));
        self.options.attack_indicator = shown.attack_indicator;
        self.options.framerate_limit = shown.framerate_limit;
        self.options.enable_vsync = shown.vsync;
        self.options.inactivity_fps_limit = shown.inactivity_fps_limit;
        if done {
            self.close_video();
        }
    }

    /// An event for the open options screen.
    pub(in super::super) fn options_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, WindowEvent};
        let lang = &assets.lang;
        let Some(screen) = self.options_screen.as_mut() else {
            return;
        };
        let input = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => OptionsInput {
                action: Some(OptionsAction::Done),
                fov: None,
            },
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                screen.tab(!modifiers.contains(glfw::Modifiers::Shift));
                OptionsInput::default()
            }
            WindowEvent::Key(Key::Left, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(lang, SliderKey::Left)
            }
            WindowEvent::Key(Key::Right, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(lang, SliderKey::Right)
            }
            WindowEvent::Key(Key::Up, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(lang, SliderKey::Up)
            }
            WindowEvent::Key(Key::Down, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(lang, SliderKey::Down)
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                screen.key_pressed(lang, SliderKey::Select)
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                match self.options_screen.as_mut() {
                    Some(screen) => screen.dragged(lang, self.gui_mouse.0),
                    None => OptionsInput::default(),
                }
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                screen.clicked(lang, self.gui_mouse)
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                screen.released(lang);
                OptionsInput::default()
            }
            _ => OptionsInput::default(),
        };
        self.options_input(input, surface, assets);
    }

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
        self.sound_screen = None;
        self.controls_screen = None;
        self.language_screen = None;
        self.mouse_screen = None;
        self.death = None;
        self.confirm = None;
        self.screen = None;
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// An event for the disconnected screen: it takes no Escape, and its
    /// button leaves, there being no server list to go back to.
    pub(in super::super) fn disconnected_event(
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
                surface.request_close();
            }
            WindowEvent::CursorPos(x, y) => self.move_gui_mouse((x, y), surface),
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _)
                if screen.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32)) =>
            {
                surface.request_close();
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
                .map_or(0, voxel_world::ClientPlayerState::score);
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
            self.sound_screen = None;
            self.controls_screen = None;
            self.language_screen = None;
            self.mouse_screen = None;
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
    pub(in super::super) fn close_death(&mut self, surface: &mut WindowSurface) {
        self.death = None;
        self.confirm = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// What a death screen button does: "Respawn" asks the server; "Title
    /// Screen" leaves, after a confirmation unless the world is hardcore
    /// (`handleExitToTitleScreen`).
    pub(in super::super) fn death_action(
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

    /// `DeathScreen.TitleConfirmScreen`, its buttons asleep for a second.
    pub(in super::super) fn open_title_confirm(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        use voxel_game::text::Component;
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
    pub(in super::super) fn confirm_action(
        &mut self,
        action: ConfirmAction,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
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
}
