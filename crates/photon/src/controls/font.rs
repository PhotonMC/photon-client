use super::*;
use photon_gui::options::font::{FontChoices, FontOptionsScreen};

impl Controls {
    /// `FontOptionsScreen`, over the language screen.
    pub(super) fn open_font(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.font_screen = Some(FontOptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            &self.options,
        ));
    }

    /// `OptionsSubScreen.onClose` and `removed`: the options are saved, and the language screen
    /// it came from is set again, which builds its widgets anew: the search, the selection and the
    /// scroll start over.
    pub(super) fn close_font(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.font_screen = None;
        self.save_options();
        self.open_language(surface, assets);
    }

    /// The options the screen shows are the options'; a Unicode font that is forced or not
    /// changes the GUI scale (`Minecraft.resizeGui`).
    fn take_font_choices(&mut self, surface: &mut WindowSurface) {
        let Some(screen) = self.font_screen.as_ref() else {
            return;
        };
        let font = &mut self.options.font;
        let choices = screen.choices(FontChoices {
            force_unicode_font: font.force_unicode,
            japanese_glyph_variants: font.japanese_glyph_variants,
        });
        font.japanese_glyph_variants = choices.japanese_glyph_variants;
        if choices.force_unicode_font != font.force_unicode {
            font.force_unicode = choices.force_unicode_font;
            surface.set_enforce_unicode(choices.force_unicode_font);
        }
    }

    /// A key press the screen takes, or Tab: whether the screen is done.
    fn font_key(&mut self, event: &glfw::WindowEvent, lang: &photon_game::text::Lang) -> bool {
        let Some(screen) = self.font_screen.as_mut() else {
            return false;
        };
        match menu_key(event) {
            Some(MenuKey::Tab(forward)) => screen.tab(forward),
            Some(MenuKey::Key(key, shift)) => return screen.key_pressed(lang, key, shift).done,
            None => {}
        }
        false
    }

    /// An event for the open font settings.
    pub(super) fn font_event(
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
            if let Some(screen) = self.font_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            return;
        }
        let mut done = self.font_key(&event, lang);
        let Some(screen) = self.font_screen.as_mut() else {
            return;
        };
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
        self.take_font_choices(surface);
        if done {
            self.close_font(surface, assets);
        }
    }
}
