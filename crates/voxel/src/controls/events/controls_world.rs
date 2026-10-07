use super::*;

impl Controls {
    /// An event with no screen in the way: moving, looking, the hotbar, the
    /// attack and use keys, and the keys that open things.
    pub(in super::super) fn world_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let spectating = session.player().is_some_and(|player| player.spectating());
        match event {
            WindowEvent::Key(Key::E, _, Action::Press, _) if self.captured && !spectating => {
                open_inventory(
                    surface,
                    &mut self.screen,
                    session.game_mode() == voxel_world::ClientPlayerGameMode::Creative,
                    self.creative_tab,
                    assets,
                );
                self.captured = false;
                self.release_keys();
            }
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.open_pause(surface, assets);
            }
            WindowEvent::Key(Key::Tab, _, action, _) => {
                self.tab_held = action != Action::Release && self.captured;
            }
            WindowEvent::Key(Key::T, _, Action::Press, _) if self.captured => {
                self.open_chat(ChatMethod::Message, true, surface, assets);
            }
            WindowEvent::Key(Key::Slash, _, Action::Press, _) if self.captured => {
                self.open_chat(ChatMethod::Command, true, surface, assets);
            }
            WindowEvent::Key(Key::F5, _, Action::Press, _) => {
                self.camera_type = self.camera_type.next();
            }
            WindowEvent::Key(Key::F11, _, Action::Press, _) => surface.toggle_fullscreen(),
            WindowEvent::Scroll(x, y) if self.captured => {
                if !spectating
                    && let Some(slot) = self.scroll_wheel.select(
                        x * self.options.mouse_wheel_sensitivity,
                        y * self.options.mouse_wheel_sensitivity,
                        session.status().selected_slot,
                    )
                {
                    session.select_slot(slot);
                }
            }
            WindowEvent::Key(key, _, Action::Press, _)
                if self.captured && hotbar_key(key).is_some() =>
            {
                if !spectating && let Some(slot) = hotbar_key(key) {
                    session.select_slot(slot);
                }
            }
            WindowEvent::Key(key, _, action, _) if self.captured => {
                self.keys.set(key, action != Action::Release);
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) if self.captured => {
                self.attack_key.press();
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                self.attack_key.release();
            }
            WindowEvent::MouseButton(MouseButton::Button2, Action::Press, _) if self.captured => {
                self.use_key.press();
            }
            WindowEvent::MouseButton(MouseButton::Button2, Action::Release, _) => {
                self.use_key.release();
            }
            WindowEvent::MouseButton(MouseButton::Button3, Action::Press, _) if self.captured => {
                self.pick_key.press();
            }
            WindowEvent::MouseButton(MouseButton::Button3, Action::Release, _) => {
                self.pick_key.release();
            }
            WindowEvent::MouseButton(_, Action::Press, _) if !self.captured => {
                self.captured = true;
                self.last_cursor = None;
                surface.set_cursor_captured(true);
            }
            WindowEvent::CursorPos(x, y) => {
                if self.captured
                    && let Some((last_x, last_y)) = self.last_cursor
                {
                    let (turn_x, turn_y) = turn_by(&self.options, (x - last_x, y - last_y));
                    session.turn_player(turn_x, turn_y);
                }
                self.last_cursor = Some((x, y));
            }
            _ => {}
        }
        Ok(())
    }
}
