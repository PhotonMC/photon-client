use super::*;
use photon_render::gui::options::skin::SkinCustomizationScreen;

impl Controls {
    /// `SkinCustomizationScreen`, over the options screen.
    pub(super) fn open_skin(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.skin_screen = Some(SkinCustomizationScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            &self.options,
        ));
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen, and the options are
    /// saved, which tells the server.
    pub(super) fn close_skin(&mut self) {
        self.skin_screen = None;
        self.save_options();
    }

    /// The parts and the hand the screen shows are the options'.
    fn take_skin_choices(&mut self) {
        let Some(screen) = self.skin_screen.as_ref() else {
            return;
        };
        let choices = screen.choices();
        self.options.model_parts = choices.model_parts;
        self.options.main_hand = choices.main_hand;
    }

    /// An event for the open skin customization.
    pub(super) fn skin_event(
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
            if let Some(screen) = self.skin_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            return;
        }
        let Some(screen) = self.skin_screen.as_mut() else {
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
        self.take_skin_choices();
        if done {
            self.close_skin();
        }
    }
}
