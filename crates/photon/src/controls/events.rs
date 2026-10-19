use super::*;

impl Controls {
    /// An event with no screen in the way: moving, looking, the hotbar, the
    /// attack and use keys, and the keys that open things.
    pub(super) fn world_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let spectating = session.player().is_some_and(|player| player.spectating());
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.open_pause(surface, assets);
            }
            // `key.debug.switchGameMode` with the debug modifier held.
            WindowEvent::Key(Key::F4, _, Action::Press, _) if self.debug.down => {
                self.debug_switch_game_mode(session, surface, assets);
            }
            WindowEvent::Key(key, scancode, action, _) => {
                let input = keybinds::input_key(key, scancode);
                self.mapped_input(input, action, (session, surface, assets), spectating);
            }
            WindowEvent::MouseButton(_, Action::Press, _) if !self.captured => {
                self.captured = true;
                self.last_cursor = None;
                surface.set_cursor_captured(true);
            }
            WindowEvent::MouseButton(button, action, _) => {
                let input = photon_data::keys::InputKey::Mouse(button as i32);
                self.mapped_input(input, action, (session, surface, assets), spectating);
            }
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
            WindowEvent::CursorPos(x, y) => {
                if self.captured
                    && let Some((last_x, last_y)) = self.last_cursor
                {
                    self.turning.moved((x - last_x, y - last_y));
                }
                self.last_cursor = Some((x, y));
            }
            _ => {}
        }
        Ok(())
    }

    /// An event for the settings screens (and the social interactions and pause screens), the
    /// first of them open that takes it; the event comes back when none is open.
    fn settings_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
        for_screen: bool,
    ) -> std::result::Result<(), glfw::WindowEvent> {
        if !for_screen {
            return Err(event);
        }
        if self.language_screen.is_some() {
            self.language_event(event, surface, assets);
            return Ok(());
        }
        if self.key_binds_screen.is_some() {
            self.key_binds_event(event, surface, assets);
            return Ok(());
        }
        if self.mouse_screen.is_some() || self.controls_screen.is_some() {
            self.controls_event(event, surface, assets);
            return Ok(());
        }
        if self.sound_screen.is_some() {
            self.sound_options_event(event, surface, assets);
            return Ok(());
        }
        if self.skin_screen.is_some() {
            self.skin_event(event, surface, assets);
            return Ok(());
        }
        if self.accessibility_screen.is_some() {
            self.accessibility_event(event, surface, assets);
            return Ok(());
        }
        if self.chat_options_screen.is_some() {
            self.chat_options_event(event, surface, assets);
            return Ok(());
        }
        if self.video_screen.is_some() {
            self.video_event(event, surface, assets);
            return Ok(());
        }
        if self.options_screen.is_some() {
            self.options_event(event, surface, assets);
            return Ok(());
        }
        if self.social.is_some() {
            self.social_event(event, surface, assets);
            return Ok(());
        }
        if self.pause.is_some() {
            self.pause_event(event, session, surface, assets);
            return Ok(());
        }
        Err(event)
    }

    /// A key or mouse button, by the mappings it is bound to (`KeyMapping.set` and `click`): the
    /// keys held for moving, the attack, use and pick keys, and the keys that open things.
    fn mapped_input(
        &mut self,
        input: photon_data::keys::InputKey,
        action: glfw::Action,
        (session, surface, assets): (&mut PlaySession, &mut WindowSurface, &ScreenAssets),
        spectating: bool,
    ) {
        let down = action != glfw::Action::Release;
        let press = action == glfw::Action::Press;
        let names: Vec<&'static str> = self.options.mappings_of(input).collect();
        for name in names {
            let held = |key: &mut AttackKey, captured: bool| {
                if press && captured {
                    key.press();
                } else if !down {
                    key.release();
                }
            };
            match name {
                "key.attack" => held(&mut self.attack_key, self.captured),
                "key.use" => held(&mut self.use_key, self.captured),
                "key.pickItem" => held(&mut self.pick_key, self.captured),
                "key.inventory" if press && self.captured && !spectating => {
                    open_inventory(
                        surface,
                        &mut self.screen,
                        creative_tags(session),
                        self.creative_tab,
                        assets,
                    );
                    self.captured = false;
                    self.turning.inventory_opened = true;
                    self.release_keys();
                }
                screen
                    if press
                        && self.captured
                        && self.screen_key(screen, session, surface, assets) => {}
                "key.togglePerspective" if press => self.camera_type = self.camera_type.next(),
                "key.fullscreen" if press => surface.toggle_fullscreen(),
                "key.screenshot" if press => self.screenshot_pending = true,
                "key.toggleGui" if press && self.captured => self.hud_hidden = !self.hud_hidden,
                "key.playerlist" => self.tab_held = down && self.captured,
                hotbar if press && self.captured && hotbar.starts_with("key.hotbar.") => {
                    let slot = hotbar["key.hotbar.".len()..]
                        .parse::<i32>()
                        .map(|slot| slot - 1);
                    if let (false, Ok(slot)) = (spectating, slot) {
                        session.select_slot(slot);
                    }
                }
                moving if self.captured || !down => self.keys.set_mapping(moving, down),
                _ => {}
            }
        }
    }

    /// Let go of every key, as the mouse is freed.
    pub(super) fn release_keys(&mut self) {
        self.keys = HeldKeys::default();
        self.tab_held = false;
        self.attack_key.release();
        self.use_key.release();
        self.pick_key.release();
    }

    /// `containerTick`: a container the server opens or closes opens or
    /// closes its screen, and each inventory screen hands over to the other
    /// when the game mode changes under it. Neither closes the container.
    pub fn container_tick(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let size = surface.scale();
        let (width, height) = (size.gui_scaled_width(), size.gui_scaled_height());
        // `Player.mayBuild`: not in adventure or spectator.
        let may_build = !matches!(
            session.game_mode(),
            photon_world::ClientPlayerGameMode::Adventure
                | photon_world::ClientPlayerGameMode::Spectator
        );
        // A container with no screen here is closed at once, so the server
        // does not wait on a window the player cannot see.
        if session.open_container_unsupported() {
            session.close_inventory()?;
        }
        match (self.server_container, session.open_container_info()) {
            (None, Some((id, kind, title))) => {
                self.server_container = Some(id);
                self.screen = Some(assets.container_screen(kind, &title, width, height, may_build));
                self.release_keys();
                surface.set_cursor_captured(false);
                let (w, h) = surface.window_size();
                surface.set_cursor_position(f64::from(w) / 2.0, f64::from(h) / 2.0);
                return Ok(());
            }
            (Some(open), Some((id, kind, title))) if open != id => {
                self.server_container = Some(id);
                self.screen = Some(assets.container_screen(kind, &title, width, height, may_build));
                return Ok(());
            }
            (Some(_), None) => {
                // The server closed it, or the player did.
                self.server_container = None;
                session.clear_recipe_ghost();
                if self.screen.take().is_some() {
                    surface.set_cursor_captured(true);
                }
                return Ok(());
            }
            (Some(_), Some(_)) => return Ok(()),
            (None, None) => {}
        }
        let Some(open) = self.screen.as_mut().filter(|open| open.is_container()) else {
            return Ok(());
        };
        let creative = creative_tags(session);
        let open_tab = open.as_creative_mut().map(|open| open.tab());
        if open_tab.is_some() != creative.is_some() {
            self.creative_tab = open_tab.unwrap_or(self.creative_tab);
            self.screen = Some(assets.screen(creative, width, height, self.creative_tab));
        }
        Ok(())
    }

    /// The screens and dialogs over the game that take an event whole, in the order they stack:
    /// what handled it, or the event back.
    fn modal_event(
        &mut self,
        event: glfw::WindowEvent,
        (session, surface, assets): (&mut PlaySession, &mut WindowSurface, &ScreenAssets),
        for_screen: bool,
    ) -> std::result::Result<Result, glfw::WindowEvent> {
        if !for_screen {
            return Err(event);
        }
        if self.game_mode_switcher.is_some() {
            return Ok(self.game_mode_switcher_event(event, session, surface, assets));
        }
        if self.link_confirm.is_some() {
            self.link_event(event, surface);
            return Ok(Ok(()));
        }
        if self.dialog_warning.is_some() {
            self.dialog_warning_event(event, session, surface);
            return Ok(Ok(()));
        }
        if self.dialog.is_some() {
            return Ok(self.dialog_event(event, session, surface, assets));
        }
        if self.chat.is_some() {
            return Ok(self.chat_event(event, session, surface, assets));
        }
        if self.disconnected.is_some() {
            self.disconnected_event(event, surface);
            return Ok(Ok(()));
        }
        Err(event)
    }

    /// The screens that fill the game's place, and the statistics over the pause menu.
    fn full_screen_event(
        &mut self,
        event: glfw::WindowEvent,
        (session, surface, assets): (&mut PlaySession, &mut WindowSurface, &ScreenAssets),
        for_screen: bool,
    ) -> std::result::Result<Result, glfw::WindowEvent> {
        if !for_screen {
            return Err(event);
        }
        if self.confirm.is_some() {
            return Ok(self.confirm_event(event, session, surface));
        }
        if self.credits.is_some() {
            return Ok(self.credits_event(event, session, surface));
        }
        if self.death.is_some() {
            return Ok(self.death_event(event, session, surface, assets));
        }
        if self.sign_edit.is_some() {
            return Ok(self.sign_event(event, session, surface, assets));
        }
        if self.book_edit.is_some() {
            return Ok(self.book_event(event, session, surface, assets));
        }
        if self.advancements.is_some() {
            return Ok(self.advancements_event(event, (session, surface, assets)));
        }
        if self.stats.is_some() {
            return Ok(self.stats_event(event, session, surface, assets));
        }
        Err(event)
    }

    /// One window event. An open screen takes the keys and the mouse.
    pub fn handle(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if let glfw::WindowEvent::Key(key, _, action, _) = event {
            self.track_modifiers(key, action, session.reduced_debug_info());
        }
        if let Some(screen) = self.screen.as_mut() {
            let size = surface.scale();
            screen.resize(size.gui_scaled_width(), size.gui_scaled_height());
        }
        let for_screen = matches!(
            event,
            glfw::WindowEvent::Key(..)
                | glfw::WindowEvent::Char(_)
                | glfw::WindowEvent::MouseButton(..)
                | glfw::WindowEvent::Scroll(..)
                | glfw::WindowEvent::CursorPos(..)
        );
        let event = match self.modal_event(event, (session, surface, assets), for_screen) {
            Ok(done) => return done,
            Err(event) => event,
        };
        let event = match self.full_screen_event(event, (session, surface, assets), for_screen) {
            Ok(done) => return done,
            Err(event) => event,
        };
        let event = match self.settings_event(event, session, surface, assets, for_screen) {
            Ok(()) => return Ok(()),
            Err(event) => event,
        };
        if self.screen.is_some() && for_screen {
            let handled = self.screen_event(event, session, surface);
            self.drain_screen_actions(session)?;
            handled
        } else {
            self.world_event(event, session, surface, assets)
        }
    }

    /// The clicks the open screen queued while it took a key or a character.
    pub(crate) fn drain_screen_actions(&mut self, session: &mut PlaySession) -> Result {
        let Some(screen) = self.screen.as_mut() else {
            return Ok(());
        };
        let actions = screen.drain_actions();
        if actions.is_empty() {
            return Ok(());
        }
        run_actions(session, &mut self.screen, actions)
    }

    /// An event for the open screen.
    pub(super) fn screen_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        let Some(screen) = self.screen.as_mut() else {
            return Ok(());
        };
        match event {
            glfw::WindowEvent::Key(
                key,
                _,
                glfw::Action::Press | glfw::Action::Repeat,
                modifiers,
            ) => self.screen_key_press(key, modifiers, session, surface)?,
            glfw::WindowEvent::Key(..) => screen.key_released(),
            glfw::WindowEvent::Char(character) => screen.char_typed(character),
            glfw::WindowEvent::MouseButton(button, action, modifiers) => {
                let button = button as i32;
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                let menu = session.active_menu().clone();
                let actions = if action == glfw::Action::Press {
                    self.active_button = Some(button);
                    let now = std::time::Instant::now();
                    screen.mouse_clicked(self.gui_mouse, button, shift, now, &menu)
                } else {
                    self.active_button = None;
                    screen.mouse_released(self.gui_mouse, button, shift, &menu)
                };
                run_actions(session, &mut self.screen, actions)?;
                // A button of the screen closed it: the mouse is grabbed again.
                if self.screen.is_none() {
                    surface.set_cursor_captured(true);
                    self.captured = true;
                    self.last_cursor = None;
                }
            }
            glfw::WindowEvent::Scroll(_, y) => {
                let menu = session.active_menu().clone();
                screen.mouse_scrolled(y, &menu);
            }
            glfw::WindowEvent::CursorPos(x, y) => {
                // `MouseHandler.onMove`: screen coordinates to GUI units.
                let size = surface.scale();
                let (width, height) = surface.window_size();
                self.gui_mouse = (
                    x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
                    y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
                );
                if self.active_button.is_some() {
                    let menu = session.active_menu().clone();
                    screen.mouse_dragged(self.gui_mouse, &menu);
                }
                self.last_cursor = None;
            }
            _ => {}
        }
        Ok(())
    }

    /// The mouse in GUI units, from a cursor position in pixels.
    pub(super) fn move_gui_mouse(&mut self, (x, y): (f64, f64), surface: &WindowSurface) {
        let size = surface.scale();
        let (width, height) = surface.window_size();
        self.gui_mouse = (
            x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
            y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
        );
    }

    /// An event for the open pause menu.
    pub(super) fn pause_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, WindowEvent};
        let Some(pause) = self.pause.as_mut() else {
            return;
        };
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => self.close_pause(surface),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                pause.tab(!modifiers.contains(glfw::Modifiers::Shift));
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                if let Some(action) = pause.pressed() {
                    self.pause_action(action, session, surface, assets);
                }
            }
            WindowEvent::Key(
                key @ (Key::Up | Key::Down | Key::Left | Key::Right),
                _,
                Action::Press | Action::Repeat,
                _,
            ) => {
                use photon_gui::widgets::navigation::Direction;
                pause.arrow(match key {
                    Key::Up => Direction::Up,
                    Key::Down => Direction::Down,
                    Key::Left => Direction::Left,
                    _ => Direction::Right,
                });
            }
            WindowEvent::CursorPos(x, y) => {
                pause.mouse_input();
                let size = surface.scale();
                let (width, height) = surface.window_size();
                self.gui_mouse = (
                    x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
                    y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
                );
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
                if let Some(action) = pause.clicked(mouse) {
                    self.pause_action(action, session, surface, assets);
                }
            }
            _ => {}
        }
    }
}
