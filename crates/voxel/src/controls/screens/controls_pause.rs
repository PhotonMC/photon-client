use super::*;

impl Controls {
    pub fn new(camera_type: CameraType, captured: bool) -> Self {
        let command_history = read_command_history();
        let options = load_options();
        let chat_settings = ChatSettings::from(&options);
        Self {
            keys: HeldKeys::default(),
            attack_key: AttackKey::default(),
            use_key: AttackKey::default(),
            pick_key: AttackKey::default(),
            ctrl_down: false,
            scroll_wheel: ScrollWheel::default(),
            camera_type,
            captured,
            last_cursor: None,
            screen: None,
            server_container: None,
            creative_tab: voxel_game::creative_menu::default_tab(),
            chat: None,
            pause: None,
            options_screen: None,
            video_screen: None,
            chat_options_screen: None,
            sound_screen: None,
            controls_screen: None,
            language_screen: None,
            language_request: None,
            last_press: None,
            mouse_screen: None,
            death: None,
            confirm: None,
            link_confirm: None,
            disconnected: None,
            options,
            chat_history: command_history.clone(),
            chat_drafts: ChatDrafts::default(),
            command_history,
            shift_held: false,
            control_held: false,
            chat_log: ChatLog::with_settings(chat_settings),
            tab_held: false,
            gui_mouse: (0.0, 0.0),
            active_button: None,
        }
    }

    /// `PauseScreen`: the menu opens and the mouse is let go. The game goes
    /// on behind it, as on a server.
    pub(in super::super) fn open_pause(
        &mut self,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.pause = Some(PauseScreen::new(&assets.creative.font, &assets.lang, size));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// The menu closes and the mouse is grabbed again.
    pub(in super::super) fn close_pause(&mut self, surface: &mut WindowSurface) {
        self.pause = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// What a pause menu button does: "Back to Game" closes the menu and
    /// "Disconnect" leaves; the screens the others open are not built.
    pub(in super::super) fn pause_action(
        &mut self,
        action: PauseAction,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        match action {
            PauseAction::Resume => self.close_pause(surface),
            PauseAction::Disconnect => surface.request_close(),
            PauseAction::Options => self.open_options(surface, assets),
            _ => {}
        }
    }

    /// `OptionsScreen`, over the pause menu, in a world.
    pub(in super::super) fn open_options(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.options_screen = Some(OptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            self.options.fov,
            true,
        ));
    }

    /// `OptionsScreen.onClose` and `removed`: back to the pause menu, and
    /// the options are saved.
    pub(in super::super) fn close_options(&mut self) {
        self.options_screen = None;
        let _ = self.options.save(&options_path());
    }

    /// What an input to the options screen asks for: a new field of view
    /// takes effect at once; Done leaves; Video opens the video settings;
    /// the screens the others open are not built.
    pub(in super::super) fn options_input(
        &mut self,
        input: OptionsInput,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        if let Some(fov) = input.fov {
            self.options.fov = fov;
        }
        match input.action {
            Some(OptionsAction::Done) => self.close_options(),
            Some(OptionsAction::Video) => self.open_video(surface, assets),
            Some(OptionsAction::Chat) => self.open_chat_options(surface, assets),
            Some(OptionsAction::Sounds) => self.open_sound_options(surface, assets),
            Some(OptionsAction::Controls) => self.open_controls(surface, assets),
            Some(OptionsAction::Language) => self.open_language(surface, assets),
            _ => {}
        }
    }

    /// `VideoSettingsScreen`, over the options screen.
    pub(in super::super) fn open_video(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let scale = surface.scale();
        self.video_screen = Some(VideoSettingsScreen::new(
            &assets.creative.font,
            &assets.lang,
            (scale.gui_scaled_width(), scale.gui_scaled_height()),
            true,
            self.options.gui_scale,
            scale.calculate_scale(0, false),
            Self::video_values(&self.options),
        ));
    }

    /// The options the video screen shows, as the options hold them.
    pub(in super::super) fn video_values(options: &voxel_game::options::Options) -> VideoValues {
        VideoValues {
            attack_indicator: options.attack_indicator,
            framerate_limit: options.framerate_limit,
            vsync: options.enable_vsync,
            inactivity_fps_limit: options.inactivity_fps_limit,
        }
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen,
    /// and the options are saved.
    pub(in super::super) fn close_video(&mut self) {
        self.video_screen = None;
        let _ = self.options.save(&options_path());
    }

    pub(in super::super) fn gui_size(surface: &WindowSurface) -> (i32, i32) {
        let scale = surface.scale();
        (scale.gui_scaled_width(), scale.gui_scaled_height())
    }

    /// `LanguageSelectScreen`, over the options screen.
    pub(in super::super) fn open_language(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(source) = assets.languages.as_ref() else {
            return;
        };
        self.language_screen = Some(LanguageSelectScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            source.languages().into_iter().collect(),
            &self.options.language_code,
        ));
    }

    /// The language the player chose, once.
    pub fn take_language_request(&mut self) -> Option<String> {
        self.language_request.take()
    }

    /// The screens that hold their words are made again in the language
    /// now loaded.
    pub fn rebuild_menus(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let size = Self::gui_size(surface);
        if self.pause.is_some() {
            self.pause = Some(PauseScreen::new(&assets.creative.font, &assets.lang, size));
        }
        if self.options_screen.is_some() {
            self.open_options(surface, assets);
        }
    }

    /// An event for the open language screen.
    pub(in super::super) fn language_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let font = &assets.creative.font;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            let mouse = self.gui_mouse;
            if let Some(screen) = self.language_screen.as_mut() {
                screen.dragged(mouse, mouse.1 - before);
            }
            return;
        }
        let mouse = self.gui_mouse;
        let control = self.control_held;
        let wheel = self.options.mouse_wheel_sensitivity;
        let Some(screen) = self.language_screen.as_mut() else {
            return;
        };
        let mut input = None;
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                input = Some(voxel_render::gui::language_select::LanguageInput {
                    done: true,
                    language: None,
                });
            }
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                input = Some(screen.key_pressed(
                    font,
                    key,
                    (control, shift),
                    &mut WindowClipboard(surface),
                ));
            }
            WindowEvent::Char(character) => screen.char_typed(font, character),
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                // The window's own double click timing is not kept: two
                // presses at the same place within 250 ms.
                let now = Instant::now();
                let double = self.last_press.is_some_and(|(at, place)| {
                    now - at < Duration::from_millis(250)
                        && place == (mouse.0 as i32, mouse.1 as i32)
                });
                self.last_press = Some((now, (mouse.0 as i32, mouse.1 as i32)));
                input = Some(screen.clicked(font, mouse, double));
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => screen.released(),
            WindowEvent::Scroll(_, y) => screen.scrolled(mouse, y * wheel),
            _ => {}
        }
        if let Some(input) = input.filter(|input| input.done) {
            self.language_screen = None;
            if let Some(code) = input.language {
                self.options.language_code.clone_from(&code);
                self.language_request = Some(code);
            }
            let _ = self.options.save(&options_path());
        }
    }

    /// `ControlsScreen`, over the options screen.
    pub(in super::super) fn open_controls(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        self.controls_screen = Some(ControlsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
        ));
    }

    /// `MouseSettingsScreen`, over the controls screen.
    pub(in super::super) fn open_mouse_settings(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        self.mouse_screen = Some(MouseSettingsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            &self.options,
        ));
    }

    /// The mouse options the screen shows are the options.
    pub(in super::super) fn take_mouse_choices(&mut self) {
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
    pub(in super::super) fn controls_event(
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
        } else if done {
            self.close_controls();
        }
    }

    /// Done or Escape on the mouse settings goes back to the controls
    /// screen, on that to the options; each saves the options.
    pub(in super::super) fn close_controls(&mut self) {
        if self.mouse_screen.take().is_none() {
            self.controls_screen = None;
        }
        let _ = self.options.save(&options_path());
    }

    /// `SoundOptionsScreen`, over the options screen.
    pub(in super::super) fn open_sound_options(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
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
    pub(in super::super) fn close_sound_options(&mut self) {
        self.sound_screen = None;
        let _ = self.options.save(&options_path());
    }

    /// The sound options the screen shows are the options.
    pub(in super::super) fn take_sound_choices(&mut self) {
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
