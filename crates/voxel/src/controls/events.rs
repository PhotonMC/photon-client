use super::*;

impl Controls {
    /// A scripted press of the primary button on the confirmation, the death
    /// screen or the pause menu, at the mouse.
    pub(super) fn scripted_press(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        if let Some(confirm) = &self.confirm {
            if let Some(action) = confirm.clicked(mouse) {
                self.confirm_action(action, session, surface)?;
            }
        } else if let Some(screen) = self.options_screen.as_mut() {
            let input = screen.clicked(&assets.lang, self.gui_mouse);
            screen.released(&assets.lang);
            self.options_input(input, surface, assets);
        } else if let Some(action) = self.death.as_mut().and_then(|death| death.clicked(mouse)) {
            self.death_action(action, session, surface, assets)?;
        } else if let Some(action) = self.pause.as_ref().and_then(|pause| pause.clicked(mouse)) {
            self.pause_action(action, surface, assets);
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

    /// An event for the open confirmation: Escape answers no once it is
    /// awake.
    pub(super) fn confirm_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let Some(confirm) = self.confirm.as_mut() else {
            return Ok(());
        };
        let action = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => confirm.escape(),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                confirm.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                confirm.pressed()
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                confirm.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32))
            }
            _ => None,
        };
        match action {
            Some(action) => self.confirm_action(action, session, surface),
            None => Ok(()),
        }
    }

    /// An event for the open death screen: it takes no Escape.
    pub(super) fn death_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let Some(death) = self.death.as_mut() else {
            return Ok(());
        };
        let action = match event {
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                death.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                death.pressed()
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                death.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32))
            }
            _ => None,
        };
        match action {
            Some(action) => self.death_action(action, session, surface, assets),
            None => Ok(()),
        }
    }

    /// An event for the open pause menu.
    pub(super) fn pause_event(
        &mut self,
        event: glfw::WindowEvent,
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
                    self.pause_action(action, surface, assets);
                }
            }
            WindowEvent::Key(
                key @ (Key::Up | Key::Down | Key::Left | Key::Right),
                _,
                Action::Press | Action::Repeat,
                _,
            ) => {
                use voxel_render::gui::navigation::Direction;
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
                    self.pause_action(action, surface, assets);
                }
            }
            _ => {}
        }
    }

    /// `ChatComponent.openScreen`: the chat box opens on the draft a closed
    /// one left, or empty (a slash for a command), and the mouse is let go.
    /// `keyed` is whether a key opened it.
    pub(super) fn open_chat(
        &mut self,
        method: ChatMethod,
        keyed: bool,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let (initial, draft) = self.chat_drafts.open(method);
        let mut screen = ChatScreen::new(
            assets.creative.font.clone(),
            size,
            &initial,
            self.chat_history.clone(),
        );
        if draft {
            screen = screen.as_draft();
        }
        self.chat = Some(if keyed { screen } else { screen.without_key() });
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
    }

    /// `ChatScreen.removed`: the box's text is kept or dropped as a draft by
    /// the way it went.
    pub(super) fn leave_chat(&mut self, chat: &ChatScreen, reason: ExitReason) {
        self.chat_history = chat.history().to_vec();
        self.chat_drafts.removed(
            &chat.value(),
            chat.is_draft(),
            reason,
            self.options.save_chat_drafts,
        );
    }

    /// The chat box closes, keeping what was sent, and the mouse is grabbed.
    pub(super) fn close_chat(&mut self, surface: &mut WindowSurface, reason: ExitReason) {
        if let Some(chat) = self.chat.take() {
            self.leave_chat(&chat, reason);
        }
        self.chat_log.reset_scroll();
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// `ChatComponent.addRecentChat`'s second half: a command is kept in the
    /// history file (the last 50, no repeat of the last).
    pub(super) fn remember_command(&mut self, text: &str) {
        if !text.starts_with('/') || self.command_history.last().is_some_and(|last| last == text) {
            return;
        }
        if self.command_history.len() >= COMMAND_HISTORY_LIMIT {
            self.command_history.remove(0);
        }
        self.command_history.push(text.to_owned());
        write_command_history(&self.command_history);
    }

    /// `ClientPacketListener.sendChat` or `sendCommand` for a typed line.
    pub(super) fn send_chat(session: &mut PlaySession, text: &str) -> Result {
        match text.strip_prefix('/') {
            Some(command) => session.send_chat(command, true)?,
            None => session.send_chat(text, false)?,
        }
        Ok(())
    }

    /// The request the chat box's last edit asked for is sent, and an
    /// answer that came is handed to it.
    pub(super) fn send_suggestion_request(&mut self, session: &mut PlaySession) -> Result {
        let Some(chat) = self.chat.as_mut() else {
            return Ok(());
        };
        if let Some((id, text)) = chat.take_request() {
            session.request_suggestions(id, &text)?;
        }
        Ok(())
    }

    /// An answer to a suggestion request reaches the open chat box.
    pub fn receive_suggestions(&mut self, session: &mut PlaySession) {
        if let Some(chat) = self.chat.as_mut() {
            chat.set_commands(session.commands().cloned());
            chat.set_selectors(session.can_use_selectors());
            chat.set_names(
                session.online_player_names(),
                session.scoreboard().team_names(),
            );
            chat.set_selected_entities(session.selected_entities());
        }
        if let (Some(chat), Some(answer)) = (self.chat.as_mut(), session.take_suggestions()) {
            chat.suggested(answer.id, answer.start, answer.length, answer.matches);
        }
    }

    /// An event for the open chat box.
    pub(super) fn chat_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let Some(chat) = self.chat.as_mut() else {
            return Ok(());
        };
        chat.set_commands(session.commands().cloned());
        chat.set_selectors(session.can_use_selectors());
        chat.set_names(
            session.online_player_names(),
            session.scoreboard().team_names(),
        );
        chat.set_selected_entities(session.selected_entities());
        let size = surface.scale();
        chat.resize((size.gui_scaled_width(), size.gui_scaled_height()));
        if let glfw::WindowEvent::Key(glfw::Key::LeftShift | glfw::Key::RightShift, _, action, _) =
            event
        {
            self.shift_held = action != glfw::Action::Release;
        }
        match event {
            glfw::WindowEvent::Key(
                key,
                _,
                glfw::Action::Press | glfw::Action::Repeat,
                modifiers,
            ) => {
                let control = modifiers.contains(glfw::Modifiers::Control)
                    || modifiers.contains(glfw::Modifiers::Super);
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                match chat.key_pressed(key, (control, shift), &mut WindowClipboard(surface)) {
                    ChatKey::Close => self.close_chat(surface, ExitReason::Intentional),
                    ChatKey::Done => self.close_chat(surface, ExitReason::Done),
                    ChatKey::Send(text) => {
                        Self::send_chat(session, &text)?;
                        self.remember_command(&text);
                        self.close_chat(surface, ExitReason::Done);
                    }
                    ChatKey::Page(direction) => {
                        let page = self.chat_log.lines_per_page(true) as i32 - 1;
                        self.chat_log.scroll_by(direction * page, true);
                    }
                    ChatKey::None => {}
                }
                self.send_suggestion_request(session)?;
            }
            glfw::WindowEvent::Char(character) => {
                chat.char_typed(character);
                self.send_suggestion_request(session)?;
            }
            glfw::WindowEvent::Scroll(_, y) => {
                // `ChatScreen.mouseScrolled`: the list of matches first, then
                // the chat by a notch of seven lines.
                let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
                if !chat.mouse_scrolled(y, mouse) {
                    let notch = y.clamp(-1.0, 1.0);
                    let lines = if self.shift_held { notch } else { notch * 7.0 };
                    self.chat_log.scroll_by(lines as i32, true);
                }
            }
            glfw::WindowEvent::CursorPos(x, y) => {
                let (width, height) = surface.window_size();
                self.gui_mouse = (
                    x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
                    y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
                );
            }
            glfw::WindowEvent::MouseButton(
                glfw::MouseButton::Button1,
                glfw::Action::Press,
                modifiers,
            ) => {
                let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
                // `ChatScreen.mouseClicked`: the list of matches first, then
                // the style of the text under the mouse.
                if !chat.mouse_clicked(mouse) {
                    let shift = modifiers.contains(glfw::Modifiers::Shift);
                    let gui = (size.gui_scaled_width(), size.gui_scaled_height());
                    let font = chat.font();
                    if let Some(style) = self.chat_log.clicked_style(&font, gui, mouse, shift) {
                        self.style_clicked(&style, shift, session, surface, assets)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// `ChatScreen.handleComponentClicked`: what a click on text does. With
    /// Shift it types the text's insertion; without, its click event.
    pub(super) fn style_clicked(
        &mut self,
        style: &voxel_game::text::Style,
        insertions: bool,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use voxel_game::text::ClickEvent;
        if insertions {
            if let (Some(insertion), Some(chat)) = (&style.insertion, self.chat.as_mut()) {
                chat.insert_text(insertion, false);
            }
            return Ok(());
        }
        let Some(event) = style.click.as_deref() else {
            return Ok(());
        };
        match event {
            // `Commands.trimOptionalPrefix`, then the command as typed.
            ClickEvent::RunCommand(command) => {
                session.send_chat(command.strip_prefix('/').unwrap_or(command), true)?;
            }
            ClickEvent::SuggestCommand(command) => {
                if let Some(chat) = self.chat.as_mut() {
                    chat.insert_text(command, true);
                }
            }
            ClickEvent::CopyToClipboard(value) => surface.set_clipboard(value),
            ClickEvent::OpenUrl(url) => {
                // `clickUrlAction`, with the prompt on (the option is not
                // kept): an untrusted link asks first.
                let size = surface.scale();
                let gui = (size.gui_scaled_width(), size.gui_scaled_height());
                let screen =
                    ConfirmScreen::link(&assets.creative.font, &assets.lang, gui, url, false)
                        .with_background(voxel_render::gui::confirm::ConfirmBackground::Menu {
                            in_world: true,
                        });
                self.link_confirm = Some((screen, url.clone()));
            }
            ClickEvent::OpenFile(path) => open_uri(path),
            // Changing a page and the server's own actions have no screen
            // here to act on.
            ClickEvent::ChangePage(_) | ClickEvent::Custom(_) => {}
        }
        Ok(())
    }

    /// An event for the open link confirmation: Yes opens the link, Copy
    /// keeps it on the clipboard, No and Escape go back to the chat.
    pub(super) fn link_event(&mut self, event: glfw::WindowEvent, surface: &mut WindowSurface) {
        use glfw::{Action, Key, WindowEvent};
        if let WindowEvent::CursorPos(x, y) = event {
            self.move_gui_mouse((x, y), surface);
            return;
        }
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let Some((screen, url)) = self.link_confirm.as_mut() else {
            return;
        };
        let action = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => screen.escape(),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                screen.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                screen.pressed()
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                screen.clicked(mouse)
            }
            _ => None,
        };
        let Some(action) = action else {
            return;
        };
        match action {
            LinkAction::Yes => open_uri(url),
            LinkAction::Copy => surface.set_clipboard(url),
            LinkAction::No => {}
        }
        self.link_confirm = None;
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
            voxel_world::ClientPlayerGameMode::Adventure
                | voxel_world::ClientPlayerGameMode::Spectator
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
        let Some(open) = self.screen.as_mut() else {
            return Ok(());
        };
        let creative = session.game_mode() == voxel_world::ClientPlayerGameMode::Creative;
        let open_tab = open.as_creative_mut().map(|open| open.tab());
        if open_tab.is_some() != creative {
            self.creative_tab = open_tab.unwrap_or(self.creative_tab);
            self.screen = Some(assets.screen(creative, width, height, self.creative_tab));
        }
        Ok(())
    }

    /// A script's step on this tick: the inventory key, a click in the
    /// open screen, the mouse moved or text typed into it, a creative tab
    /// chosen or scrolled, and the attack and use keys. A scripted `attack`
    /// is a click on the tick; `mine` holds the key, and the same goes for
    /// `use` and `hold_use`.
    pub fn script_step(
        &mut self,
        held: HeldKeys,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if held.inventory {
            if self.screen.is_some() {
                close_inventory(session, surface, &mut self.screen, &mut self.creative_tab)?;
                surface.set_cursor_captured(false);
            } else {
                open_inventory(
                    surface,
                    &mut self.screen,
                    session.game_mode() == voxel_world::ClientPlayerGameMode::Creative,
                    self.creative_tab,
                    assets,
                );
            }
        }
        if let Some((slot, button, kind)) = held.click {
            match self.screen.as_mut() {
                Some(screen) => {
                    let action = screen.scripted_click(slot, button, kind);
                    run_actions(session, &mut self.screen, vec![action])?;
                }
                None => session.container_click(slot, button, kind)?,
            }
        }
        if let Some(mouse) = held.mouse {
            self.gui_mouse = mouse;
        }
        if held.escape {
            self.scripted_escape(session, surface, assets)?;
        }
        if held.chat {
            if self.chat.is_some() {
                self.close_chat(surface, ExitReason::Intentional);
            } else {
                self.open_chat(ChatMethod::Message, false, surface, assets);
            }
        }
        if let (Some(text), Some(chat)) = (held.typed, self.chat.as_mut()) {
            for character in text.chars() {
                chat.char_typed(character);
            }
        }
        self.send_suggestion_request(session)?;
        if held.complete
            && let Some(chat) = self.chat.as_mut()
        {
            let mut clipboard = WindowClipboard(surface);
            chat.key_pressed(glfw::Key::Tab, (false, false), &mut clipboard);
        }
        self.send_suggestion_request(session)?;
        if held.enter
            && let Some(chat) = self.chat.as_mut()
        {
            let mut clipboard = WindowClipboard(surface);
            if let ChatKey::Send(text) =
                chat.key_pressed(glfw::Key::Enter, (false, false), &mut clipboard)
            {
                Self::send_chat(session, &text)?;
            }
            self.close_chat(surface, ExitReason::Done);
        }
        if held.press.is_some() {
            self.scripted_press(session, surface, assets)?;
        }
        if let (Some((button, shift)), Some(screen)) = (held.press, self.screen.as_mut()) {
            let menu = session.active_menu().clone();
            let now = std::time::Instant::now();
            let mut actions = screen.mouse_clicked(self.gui_mouse, button, shift, now, &menu);
            actions.extend(screen.mouse_released(self.gui_mouse, button, shift, &menu));
            run_actions(session, &mut self.screen, actions)?;
        }
        if let (Some(text), Some(screen)) = (held.typed, self.screen.as_mut()) {
            for character in text.chars() {
                screen.char_typed(character);
            }
        }
        self.drain_screen_actions(session)?;
        if let Some(screen) = self
            .screen
            .as_mut()
            .and_then(|screen| screen.as_creative_mut())
        {
            if let Some(tab) = held.tab {
                screen.select_tab_index(tab);
            }
            if let Some(scroll) = held.scroll {
                screen.menu_mut().set_scroll(scroll as f32 / 100.0);
            }
        }
        script_key(&mut self.attack_key, held.attack, held.mine);
        script_key(&mut self.use_key, held.use_click, held.hold_use);
        script_key(&mut self.pick_key, held.pick, false);
        Ok(())
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
            let control_keys: [glfw::Key; 2] = if cfg!(target_os = "macos") {
                [glfw::Key::LeftSuper, glfw::Key::RightSuper]
            } else {
                [glfw::Key::LeftControl, glfw::Key::RightControl]
            };
            if control_keys.contains(&key) {
                self.control_held = action != glfw::Action::Release;
            }
            if matches!(key, glfw::Key::LeftControl | glfw::Key::RightControl) {
                self.ctrl_down = action != glfw::Action::Release;
            }
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
        if self.link_confirm.is_some() && for_screen {
            self.link_event(event, surface);
            return Ok(());
        }
        if self.chat.is_some() && for_screen {
            return self.chat_event(event, session, surface, assets);
        }
        if self.disconnected.is_some() && for_screen {
            self.disconnected_event(event, surface);
            return Ok(());
        }
        if self.confirm.is_some() && for_screen {
            return self.confirm_event(event, session, surface);
        }
        if self.death.is_some() && for_screen {
            return self.death_event(event, session, surface, assets);
        }
        if self.language_screen.is_some() && for_screen {
            self.language_event(event, surface, assets);
            return Ok(());
        }
        if (self.mouse_screen.is_some() || self.controls_screen.is_some()) && for_screen {
            self.controls_event(event, surface, assets);
            return Ok(());
        }
        if self.sound_screen.is_some() && for_screen {
            self.sound_options_event(event, surface, assets);
            return Ok(());
        }
        if self.chat_options_screen.is_some() && for_screen {
            self.chat_options_event(event, surface, assets);
            return Ok(());
        }
        if self.video_screen.is_some() && for_screen {
            self.video_event(event, surface, assets);
            return Ok(());
        }
        if self.options_screen.is_some() && for_screen {
            self.options_event(event, surface, assets);
            return Ok(());
        }
        if self.pause.is_some() && for_screen {
            self.pause_event(event, surface, assets);
            return Ok(());
        }
        if self.screen.is_some() && for_screen {
            let handled = self.screen_event(event, session, surface);
            self.drain_screen_actions(session)?;
            handled
        } else {
            self.world_event(event, session, surface, assets)
        }
    }

    /// The clicks the open screen queued while it took a key or a character.
    pub(super) fn drain_screen_actions(&mut self, session: &mut PlaySession) -> Result {
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
            ) => {
                let menu = session.active_menu().clone();
                let mods = KeyMods {
                    control: modifiers.contains(glfw::Modifiers::Control),
                    shift: modifiers.contains(glfw::Modifiers::Shift),
                    command: modifiers.contains(glfw::Modifiers::Super),
                };
                match screen.key_pressed(key, mods, &menu, &mut WindowClipboard(surface)) {
                    KeyResult::Close => {
                        close_inventory(
                            session,
                            surface,
                            &mut self.screen,
                            &mut self.creative_tab,
                        )?;
                        self.captured = true;
                        self.last_cursor = None;
                    }
                    KeyResult::Actions(actions) => {
                        run_actions(session, &mut self.screen, actions)?;
                    }
                }
            }
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
            glfw::WindowEvent::Scroll(_, y) => screen.mouse_scrolled(y),
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

    /// An event with no screen in the way: moving, looking, the hotbar, the
    /// attack and use keys, and the keys that open things.
    pub(super) fn world_event(
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
