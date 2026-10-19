use super::*;

impl Controls {
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

    /// Whether a screen is up over the game (`Gui.screen != null`): any of the screens and
    /// dialogs the controls hold.
    pub(crate) fn any_screen_open(&self) -> bool {
        [
            self.screen.is_some(),
            self.chat.is_some(),
            self.pause.is_some(),
            self.death.is_some(),
            self.credits.is_some(),
            self.social.is_some(),
            self.stats.is_some(),
            self.advancements.is_some(),
            self.game_mode_switcher.is_some(),
            self.sign_edit.is_some(),
            self.book_edit.is_some(),
            self.confirm.is_some(),
            self.link_confirm.is_some(),
            self.dialog.is_some(),
            self.dialog_warning.is_some(),
            self.pack_prompt.is_some(),
            self.disconnected.is_some(),
        ]
        .contains(&true)
    }

    /// `Gui.tick`'s part for the bed: a sleeper with no screen up gets the in-bed chat box, and
    /// that box ends with the sleep (`InBedChatScreen.onPlayerWokeUp`): what was typed goes on in
    /// the ordinary chat box, or the box closes.
    pub fn bed_tick(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let sleeping = session
            .world()
            .local_player()
            .is_some_and(|player| player.is_sleeping());
        if let Some(chat) = self.chat.as_ref().filter(|chat| chat.is_in_bed()) {
            if !sleeping {
                let (text, draft) = (chat.value(), chat.is_draft());
                if !draft && !text.is_empty() {
                    self.close_chat(surface, ExitReason::Done);
                    self.open_chat_with(&text, surface, assets);
                } else {
                    self.close_chat(surface, ExitReason::Interrupted);
                }
            }
        } else if !self.any_screen_open() && sleeping {
            self.open_chat(ChatMethod::Message, false, surface, assets);
            if let Some(chat) = self.chat.take() {
                self.chat = Some(chat.in_bed(&assets.lang));
            }
        }
    }

    /// An ordinary chat box that holds `text` (`new ChatScreen(text, false)`).
    fn open_chat_with(&mut self, text: &str, surface: &mut WindowSurface, assets: &ScreenAssets) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let screen = ChatScreen::new(
            assets.creative.font.clone(),
            size,
            text,
            self.chat_history.clone(),
        );
        self.chat = Some(screen.without_key());
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

    /// What a key in the chat box asks for.
    fn chat_key(
        &mut self,
        key: ChatKey,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        if self.chat.as_ref().is_some_and(|chat| chat.is_in_bed()) {
            return self.bed_chat_key(key, session);
        }
        match key {
            ChatKey::Close => self.close_chat(surface, ExitReason::Intentional),
            ChatKey::Done => self.close_chat(surface, ExitReason::Done),
            ChatKey::Send(text) => {
                Self::send_chat(session, &text)?;
                self.remember_command(&text);
                self.close_chat(surface, ExitReason::Done);
            }
            ChatKey::Page(direction) => self.page_chat(direction),
            ChatKey::None => {}
        }
        Ok(())
    }

    /// A key in the in-bed chat box: the box stays up until the server has the player out of
    /// bed, so Escape asks for that and a sent line only empties the box.
    fn bed_chat_key(&mut self, key: ChatKey, session: &mut PlaySession) -> Result {
        match key {
            ChatKey::Close => session.send_stop_sleeping()?,
            ChatKey::Done | ChatKey::Send(_) => {
                if let Some(chat) = self.chat.as_mut() {
                    chat.clear();
                }
                if let ChatKey::Send(text) = key {
                    Self::send_chat(session, &text)?;
                    self.remember_command(&text);
                }
                self.chat_log.reset_scroll();
            }
            ChatKey::Page(direction) => self.page_chat(direction),
            ChatKey::None => {}
        }
        Ok(())
    }

    /// Page Up and Page Down: the log scrolls a page, less a line.
    fn page_chat(&mut self, direction: i32) {
        let page = self.chat_log.lines_per_page(true) as i32 - 1;
        self.chat_log.scroll_by(direction * page, true);
    }

    /// `ChatScreen.mouseClicked`: the list of matches first, then the style of the text under the
    /// mouse, then the widgets (the leave-bed button).
    fn chat_pressed(
        &mut self,
        modifiers: glfw::Modifiers,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let Some(chat) = self.chat.as_mut() else {
            return Ok(());
        };
        if chat.mouse_clicked(mouse) {
            return Ok(());
        }
        let size = surface.scale();
        let shift = modifiers.contains(glfw::Modifiers::Shift);
        let gui = (size.gui_scaled_width(), size.gui_scaled_height());
        let bed = chat.leave_bed_clicked(mouse);
        if let Some(style) =
            self.chat_log
                .clicked_style(&chat.font(), &assets.lang, gui, mouse, shift)
        {
            // `ChatScreen.handleComponentClicked`: the queue line is the chat's own, not the server's.
            if !shift
                && matches!(style.click.as_deref(), Some(photon_game::text::ClickEvent::Custom(id, _))
                    if id == photon_gui::chat::QUEUE_EXPAND_ID)
            {
                self.chat_log.request_queue_accept();
                return Ok(());
            }
            self.style_clicked(&style, shift, session, surface, assets)?;
        } else if bed {
            session.send_stop_sleeping()?;
        }
        Ok(())
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
            self.mods.shift = action != glfw::Action::Release;
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
                let pressed =
                    chat.key_pressed(key, (control, shift), &mut WindowClipboard(surface));
                self.chat_key(pressed, session, surface)?;
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
                    let lines = if self.mods.shift { notch } else { notch * 7.0 };
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
                self.chat_pressed(modifiers, session, surface, assets)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// `ChatScreen.handleComponentClicked`: what a click on text does. With
    /// Shift it types the text's insertion; without, its click event.
    pub(super) fn style_clicked(
        &mut self,
        style: &photon_game::text::Style,
        insertions: bool,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if insertions {
            if let (Some(insertion), Some(chat)) = (&style.insertion, self.chat.as_mut()) {
                chat.insert_text(insertion, false);
            }
            return Ok(());
        }
        let Some(event) = style.click.as_deref() else {
            return Ok(());
        };
        self.click_event(event, session, surface, assets)
    }

    /// What a click event does (`Screen.handleClickEvent`): a command is sent, a link asked
    /// about, and the server's own actions answered.
    pub(super) fn click_event(
        &mut self,
        event: &photon_game::text::ClickEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use photon_game::text::ClickEvent;
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
                        .with_background(photon_gui::screen::confirm::ConfirmBackground::Menu {
                            in_world: true,
                        });
                self.link_confirm = Some((screen, url.clone()));
            }
            ClickEvent::OpenFile(path) => open_uri(path),
            ClickEvent::Custom(id, payload) => {
                session.send_custom_click_action(id, payload.as_ref())?;
            }
            ClickEvent::ShowDialog(dialog) => {
                if let Some(dialog) = self.resolve_dialog(dialog, session) {
                    self.show_dialog(dialog, surface, assets);
                }
            }
            // Changing a page has no book here to act on.
            ClickEvent::ChangePage(_) => {}
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
}
