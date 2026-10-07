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
    pub(super) fn open_pause(&mut self, surface: &mut WindowSurface, assets: &ScreenAssets) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.pause = Some(PauseScreen::new(&assets.creative.font, &assets.lang, size));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// The menu closes and the mouse is grabbed again.
    pub(super) fn close_pause(&mut self, surface: &mut WindowSurface) {
        self.pause = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// What a pause menu button does: "Back to Game" closes the menu and
    /// "Disconnect" leaves; the screens the others open are not built.
    pub(super) fn pause_action(
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
    pub(super) fn open_options(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
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
    pub(super) fn close_options(&mut self) {
        self.options_screen = None;
        let _ = self.options.save(&options_path());
    }

    /// What an input to the options screen asks for: a new field of view
    /// takes effect at once; Done leaves; Video opens the video settings;
    /// the screens the others open are not built.
    pub(super) fn options_input(
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
    pub(super) fn open_video(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
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
    pub(super) fn video_values(options: &voxel_game::options::Options) -> VideoValues {
        VideoValues {
            attack_indicator: options.attack_indicator,
            framerate_limit: options.framerate_limit,
            vsync: options.enable_vsync,
            inactivity_fps_limit: options.inactivity_fps_limit,
        }
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen,
    /// and the options are saved.
    pub(super) fn close_video(&mut self) {
        self.video_screen = None;
        let _ = self.options.save(&options_path());
    }

    pub(super) fn gui_size(surface: &WindowSurface) -> (i32, i32) {
        let scale = surface.scale();
        (scale.gui_scaled_width(), scale.gui_scaled_height())
    }

    /// `LanguageSelectScreen`, over the options screen.
    pub(super) fn open_language(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
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
    pub(super) fn language_event(
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
    pub(super) fn close_controls(&mut self) {
        if self.mouse_screen.take().is_none() {
            self.controls_screen = None;
        }
        let _ = self.options.save(&options_path());
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
        let _ = self.options.save(&options_path());
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
        let _ = self.options.save(&options_path());
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

    /// An event for the open video settings.
    pub(super) fn video_event(
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
    pub(super) fn options_event(
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

    /// `DeathScreen.TitleConfirmScreen`, its buttons asleep for a second.
    pub(super) fn open_title_confirm(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
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
    pub(super) fn confirm_action(
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

    /// A scripted Escape: the confirmation answers no once awake; else the
    /// pause menu opens, or closes.
    pub(super) fn scripted_escape(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if let Some(confirm) = &self.confirm {
            if let Some(action) = confirm.escape() {
                self.confirm_action(action, session, surface)?;
            }
        } else if self.language_screen.is_some() {
            self.language_screen = None;
        } else if self.mouse_screen.is_some() || self.controls_screen.is_some() {
            self.close_controls();
        } else if self.sound_screen.is_some() {
            self.close_sound_options();
        } else if self.chat_options_screen.is_some() {
            self.close_chat_options();
        } else if self.video_screen.is_some() {
            self.close_video();
        } else if self.options_screen.is_some() {
            self.close_options();
        } else if self.pause.is_some() {
            self.close_pause(surface);
        } else {
            self.open_pause(surface, assets);
        }
        Ok(())
    }
}
