use super::*;

/// `glfwGetKeyName`: the name the keyboard layout gives a printable key, by its key code or its
/// scancode; none for another key, or a mouse button.
pub(crate) fn system_key_name(key: photon_data::keys::InputKey) -> Option<String> {
    use photon_data::keys::InputKey;
    let (code, scancode) = match key {
        InputKey::Keysym(code) => (code, -1),
        InputKey::Scancode(scancode) => (-1, scancode),
        InputKey::Mouse(_) => return None,
    };
    // SAFETY: GLFW is initialised on this thread, which runs the window's events; it takes any
    // key code and scancode, and its string is copied before it is asked again.
    unsafe { glfw::string_from_nullable_c_str(glfw::ffi::glfwGetKeyName(code, scancode)) }
}

/// `InputConstants.getKey`: a key by its code, or by its scancode if it has none.
pub(super) fn input_key(key: glfw::Key, scancode: glfw::Scancode) -> photon_data::keys::InputKey {
    if key == glfw::Key::Unknown {
        photon_data::keys::InputKey::Scancode(scancode)
    } else {
        photon_data::keys::InputKey::Keysym(key as i32)
    }
}

impl Controls {
    /// The attack, use, pick and drop keys as `handleKeybinds` spends them in one tick.
    pub(crate) fn game_keys(&mut self, grabbed: bool) -> photon_game::game_mode::Keys {
        photon_game::game_mode::Keys {
            attack_clicks: self.attack_key.take_clicks(),
            attack_held: self.attack_key.is_down() && grabbed,
            use_clicks: self.use_key.take_clicks(),
            use_held: self.use_key.is_down(),
            pick_clicks: self.pick_key.take_clicks(),
            drop_clicks: self.drop_key.take_clicks(),
            control_down: self.mods.ctrl,
        }
    }

    /// `KeyBindsScreen`, over the controls screen.
    pub(super) fn open_key_binds(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.key_binds_screen = Some(KeyBindsScreen::new(
            (&assets.creative.font, &assets.lang),
            Self::gui_size(surface),
            !self.title_host,
            self.options.key_bindings(),
            system_key_name,
        ));
    }

    /// Done or Escape: back to the controls screen, and the options are saved.
    pub(super) fn close_key_binds(&mut self) {
        self.key_binds_screen = None;
        self.save_options();
    }

    /// An event for the open key binds screen; the keys it binds are the options' at once
    /// (`KeyMapping.setKey`).
    pub(super) fn key_binds_event(
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
            if let Some(screen) = self.key_binds_screen.as_mut() {
                screen.dragged(self.gui_mouse, self.gui_mouse.1 - before);
            }
            return;
        }
        let mouse = self.gui_mouse;
        let wheel = self.options.mouse_wheel_sensitivity;
        let menu = menu_key(&event);
        let Some(screen) = self.key_binds_screen.as_mut() else {
            return;
        };
        let input = match event {
            WindowEvent::Key(key, scancode, Action::Press | Action::Repeat, _) => {
                let input = input_key(key, scancode);
                let kind = match (key, menu) {
                    (Key::Escape, _) => BindKey::Escape,
                    (_, Some(MenuKey::Tab(forward))) => BindKey::Tab(forward),
                    (_, Some(MenuKey::Key(key, _))) => BindKey::Menu(key),
                    _ => BindKey::Other,
                };
                screen.key_pressed(lang, input, kind)
            }
            WindowEvent::MouseButton(button, Action::Press, _) => {
                screen.clicked(lang, mouse, button as i32)
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released();
                return;
            }
            WindowEvent::Scroll(_, y) => {
                screen.scrolled(mouse, y * wheel);
                return;
            }
            _ => return,
        };
        let keys = screen.bindings().to_vec();
        self.options.set_key_bindings(&keys);
        if input.done {
            self.close_key_binds();
        }
    }
}
