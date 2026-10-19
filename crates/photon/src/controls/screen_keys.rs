use super::*;

impl Controls {
    /// The modifier keys held (control, the command key on macOS, F3 for the debug keys).
    pub(super) fn track_modifiers(
        &mut self,
        key: glfw::Key,
        action: glfw::Action,
        reduced_debug_info: bool,
    ) {
        let control_keys: [glfw::Key; 2] = if cfg!(target_os = "macos") {
            [glfw::Key::LeftSuper, glfw::Key::RightSuper]
        } else {
            [glfw::Key::LeftControl, glfw::Key::RightControl]
        };
        if control_keys.contains(&key) {
            self.mods.control = action != glfw::Action::Release;
        }
        if matches!(key, glfw::Key::LeftControl | glfw::Key::RightControl) {
            self.mods.ctrl = action != glfw::Action::Release;
        }
        // `key.debug.modifier`.
        if key == glfw::Key::F3 {
            self.debug.key(action);
        } else if action != glfw::Action::Release && self.debug_key_unconsumed(key) {
            self.debug.combination(key, reduced_debug_info);
        }
    }

    /// Whether a key pressed for the debug keys reaches `handleDebugKeys` before the screens have
    /// answered: Java runs it for every key the open screen did not consume. Escape belongs to
    /// the screens, and a container screen answers for itself in `screen_key_press`.
    fn debug_key_unconsumed(&self, key: glfw::Key) -> bool {
        if !self.any_screen_open() {
            return true;
        }
        key != glfw::Key::Escape && self.screen.is_none()
    }

    /// A key pressed for the open screen: what the screen answers is done, and a key it did not
    /// consume counts as a debug combination.
    pub(super) fn screen_key_press(
        &mut self,
        key: glfw::Key,
        modifiers: glfw::Modifiers,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        let Some(screen) = self.screen.as_mut() else {
            return Ok(());
        };
        let menu = session.active_menu().clone();
        let mods = KeyMods {
            control: modifiers.contains(glfw::Modifiers::Control),
            shift: modifiers.contains(glfw::Modifiers::Shift),
            command: modifiers.contains(glfw::Modifiers::Super),
        };
        let result = screen.key_pressed(key, mods, &menu, &mut WindowClipboard(surface));
        if !matches!(result, KeyResult::Close | KeyResult::Consumed(_)) && key != glfw::Key::Escape
        {
            self.debug.combination(key, session.reduced_debug_info());
        }
        match result {
            KeyResult::Close => {
                close_inventory(session, surface, &mut self.screen, &mut self.creative_tab)?;
                self.captured = true;
                self.last_cursor = None;
            }
            KeyResult::Actions(actions) | KeyResult::Consumed(actions) => {
                run_actions(session, &mut self.screen, actions)?;
            }
        }
        Ok(())
    }
}
