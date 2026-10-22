use super::*;

impl Controls {
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
                    self.mods.control,
                    &mut resize,
                );
            }
            _ => {}
        }
        let shown = video.values(Self::video_values(&self.options));
        self.take_video_values(shown);
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
        self.save_options();
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
            Some(OptionsAction::Accessibility) => self.open_accessibility(surface, assets),
            Some(OptionsAction::SkinCustomisation) => self.open_skin(surface, assets),
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

    /// The options the video screen handed back.
    fn take_video_values(&mut self, shown: VideoValues) {
        self.options.attack_indicator = shown.attack_indicator;
        self.options.framerate_limit = shown.framerate_limit;
        self.options.ambient_occlusion = shown.ambient_occlusion;
        self.options.enable_vsync = shown.vsync;
        self.options.inactivity_fps_limit = shown.inactivity_fps_limit;
        self.options.render_distance = shown.render_distance;
        self.options.menu_background_blurriness = shown.menu_blurriness;
    }

    /// The options the video screen shows, as the options hold them.
    pub(super) fn video_values(options: &photon_game::options::Options) -> VideoValues {
        VideoValues {
            attack_indicator: options.attack_indicator,
            framerate_limit: options.framerate_limit,
            vsync: options.enable_vsync,
            ambient_occlusion: options.ambient_occlusion,
            inactivity_fps_limit: options.inactivity_fps_limit,
            render_distance: options.render_distance,
            menu_blurriness: options.menu_background_blurriness,
        }
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen,
    /// and the options are saved.
    pub(super) fn close_video(&mut self) {
        self.video_screen = None;
        self.save_options();
    }

    /// `Options.save`, after which the session tells the server what changed.
    pub(crate) fn save_options(&mut self) {
        let _ = self.options.save(&options_path());
        self.options_saved = true;
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
        if let Some(pause) = self.pause.as_ref() {
            self.pause = Some(pause.resized(&assets.creative.font, &assets.lang, size));
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
        let control = self.mods.control;
        let wheel = self.options.mouse_wheel_sensitivity;
        let Some(screen) = self.language_screen.as_mut() else {
            return;
        };
        let mut input = None;
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                input = Some(photon_gui::options::language::LanguageInput {
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
            self.save_options();
        }
    }
}
