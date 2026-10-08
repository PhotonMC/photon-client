use super::*;

impl Controls {
    /// An event for the open sound settings.
    pub(super) fn sound_options_event(
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
    pub(super) fn open_chat_options(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
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
    pub(super) fn close_chat_options(&mut self) {
        self.chat_options_screen = None;
        self.save_options();
    }

    /// The chat options the screen shows are the options: the chat wraps
    /// its lines again when they changed (`rescaleChat`).
    pub(super) fn take_chat_choices(&mut self, assets: &ScreenAssets) {
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
    pub(super) fn chat_options_event(
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

    /// `ControlsScreen`, over the options screen.
    pub(super) fn open_controls(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.controls_screen = Some(ControlsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
        ));
    }

    /// `MouseSettingsScreen`, over the controls screen.
    pub(super) fn open_mouse_settings(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.mouse_screen = Some(MouseSettingsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            &self.options,
        ));
    }

    /// The mouse options the screen shows are the options.
    pub(super) fn take_mouse_choices(&mut self) {
        if let Some(screen) = self.mouse_screen.as_ref() {
            let choices = screen.choices();
            self.options.sensitivity = choices.sensitivity;
            self.options.mouse_wheel_sensitivity = choices.wheel_sensitivity;
            self.options.invert_mouse_x = choices.invert_x;
            self.options.invert_mouse_y = choices.invert_y;
        }
    }

    /// An event for the open mouse settings or the controls screen under
    /// them.
    pub(super) fn controls_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        let wheel = self.options.mouse_wheel_sensitivity;
        let mut done = false;
        let mut open_mouse = false;
        let mut open_key_binds = false;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            let (mouse, dy) = (self.gui_mouse, self.gui_mouse.1 - before);
            if let Some(screen) = self.mouse_screen.as_mut() {
                screen.dragged(lang, mouse, dy);
            } else if let Some(screen) = self.controls_screen.as_mut() {
                screen.dragged(lang, mouse, dy);
            }
            self.take_mouse_choices();
            return;
        }
        let mouse = self.gui_mouse;
        if let Some(key) = menu_key(&event) {
            if let Some(screen) = self.mouse_screen.as_mut() {
                match key {
                    MenuKey::Tab(forward) => screen.tab(forward),
                    MenuKey::Key(key, shift) => done = screen.key_pressed(lang, key, shift).done,
                }
            } else if let Some(screen) = self.controls_screen.as_mut() {
                match key {
                    MenuKey::Tab(forward) => screen.tab(forward),
                    MenuKey::Key(key, shift) => {
                        let input = screen.key_pressed(lang, key, shift);
                        done = input.done;
                        open_mouse = input.mouse_settings;
                        open_key_binds = input.key_binds;
                    }
                }
            }
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                if let Some(screen) = self.mouse_screen.as_mut() {
                    done = screen.clicked(lang, mouse, shift).done;
                } else if let Some(screen) = self.controls_screen.as_mut() {
                    let input = screen.clicked(lang, mouse, shift);
                    done = input.done;
                    open_mouse = input.mouse_settings;
                    open_key_binds = input.key_binds;
                }
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                if let Some(screen) = self.mouse_screen.as_mut() {
                    screen.released(lang);
                } else if let Some(screen) = self.controls_screen.as_mut() {
                    screen.released(lang);
                }
            }
            WindowEvent::Scroll(_, y) => {
                if let Some(screen) = self.mouse_screen.as_mut() {
                    screen.scrolled(mouse, y * wheel);
                } else if let Some(screen) = self.controls_screen.as_mut() {
                    screen.scrolled(mouse, y * wheel);
                }
            }
            _ => {}
        }
        self.take_mouse_choices();
        if open_mouse {
            self.open_mouse_settings(surface, assets);
        } else if open_key_binds {
            self.open_key_binds(surface, assets);
        } else if done {
            self.close_controls();
        }
    }

    /// Done or Escape on the mouse settings goes back to the controls
    /// screen, on that to the options; each saves the options.
    pub(super) fn close_controls(&mut self) {
        if self.mouse_screen.take().is_none() {
            self.controls_screen = None;
        }
        self.save_options();
    }

    /// `SoundOptionsScreen`, over the options screen.
    pub(super) fn open_sound_options(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.sound_screen = Some(SoundOptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            &self.options,
        ));
    }

    /// `OptionsSubScreen.onClose`: back to the options screen, and the
    /// options are saved.
    pub(super) fn close_sound_options(&mut self) {
        self.sound_screen = None;
        self.save_options();
    }

    /// The sound options the screen shows are the options.
    pub(super) fn take_sound_choices(&mut self) {
        let Some(screen) = self.sound_screen.as_ref() else {
            return;
        };
        let choices = screen.choices();
        let options = &mut self.options;
        options.sound_volumes = choices.volumes;
        options.show_subtitles = choices.show_subtitles;
        options.directional_audio = choices.directional_audio;
        options.music_frequency = choices.music_frequency;
        options.music_toast = choices.music_toast;
    }
}
