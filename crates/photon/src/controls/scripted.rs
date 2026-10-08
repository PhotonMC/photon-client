use super::*;

impl Controls {
    /// A scripted Escape: the confirmation answers no once awake; else the
    /// pause menu opens, or closes.
    pub(super) fn scripted_escape(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if let Some(confirm) = &self.confirm {
            if let Some(action) = confirm.escape() {
                self.confirm_action(action, session, surface)?;
            }
        } else if self.language_screen.is_some() {
            self.language_screen = None;
        } else if self.key_binds_screen.is_some() {
            self.close_key_binds();
        } else if self.mouse_screen.is_some() || self.controls_screen.is_some() {
            self.close_controls();
        } else if self.sound_screen.is_some() {
            self.close_sound_options();
        } else if self.chat_options_screen.is_some() {
            self.close_chat_options();
        } else if self.video_screen.is_some() {
            self.close_video();
        } else if self.options_screen.is_some() {
            self.close_options();
        } else if self.pause.is_some() {
            self.close_pause(surface);
        } else {
            self.open_pause(surface, assets);
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
                    creative_tags(session),
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
}
