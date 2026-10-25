use super::*;
use photon_gui::options::online::{FriendFacts, OnlineOptionsScreen};

impl Controls {
    /// `OnlineOptionsScreen`, over the options screen. There is no account yet, so the friends
    /// list is off.
    pub(super) fn open_online(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.online_screen = Some(OnlineOptionsScreen::new(
            (&assets.creative.font, &assets.lang),
            Self::gui_size(surface),
            !self.title_host,
            (&self.options, FriendFacts::default()),
        ));
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen, and the options are
    /// saved.
    pub(super) fn close_online(&mut self) {
        self.online_screen = None;
        self.save_options();
    }

    /// The switches the screen shows are the options'.
    fn take_online_choices(&mut self) {
        let Some(screen) = self.online_screen.as_ref() else {
            return;
        };
        self.options.online = screen.choices(self.options.online);
    }

    /// An event for the open online settings.
    pub(super) fn online_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let lang = &assets.lang;
        if let glfw::WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(screen) = self.online_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            return;
        }
        let wheel = self.options.mouse_wheel_sensitivity;
        let Some(screen) = self.online_screen.as_mut() else {
            return;
        };
        let done = screen_input(screen, &event, (lang, self.gui_mouse, wheel));
        self.take_online_choices();
        if done {
            self.close_online();
        }
    }
}

/// What an event does on the screen: whether the screen is done.
fn screen_input(
    screen: &mut OnlineOptionsScreen,
    event: &glfw::WindowEvent,
    (lang, mouse, wheel): (&photon_game::text::Lang, (f64, f64), f64),
) -> bool {
    use glfw::{Action, Key, MouseButton, WindowEvent};
    match (menu_key(event), event) {
        (Some(MenuKey::Tab(forward)), _) => screen.tab(forward),
        (Some(MenuKey::Key(key, shift)), _) => return screen.key_pressed(lang, key, shift).done,
        (None, WindowEvent::Key(Key::Escape, _, Action::Press, _)) => return true,
        (None, WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers)) => {
            let shift = modifiers.contains(glfw::Modifiers::Shift);
            return screen.clicked(lang, mouse, shift).done;
        }
        (None, WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _)) => {
            screen.released(lang);
        }
        (None, WindowEvent::Scroll(_, y)) => screen.scrolled(mouse, y * wheel),
        _ => {}
    }
    false
}
