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
        if self.escape_light_screen(session, surface) {
            return Ok(());
        }
        if let Some(confirm) = &self.confirm {
            if let Some(action) = confirm.escape() {
                self.confirm_action(action, session, surface)?;
            }
        } else if self.key_binds_screen.is_some() {
            self.close_key_binds();
        } else if self.mouse_screen.is_some() || self.controls_screen.is_some() {
            self.close_controls();
        } else if self.sound_screen.is_some() {
            self.close_sound_options();
        } else if self.skin_screen.is_some() {
            self.close_skin();
        } else if self.close_chat_settings() {
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
        self.scripted_screens(held, session, surface, assets)?;
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
        self.scripted_toggles(held);
        if held.chat {
            if self.chat.is_some() {
                self.close_chat(surface, ExitReason::Intentional);
            } else {
                self.open_chat(ChatMethod::Message, false, surface, assets);
            }
        }
        self.scripted_typing(held.typed);
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

    /// Scripted text typed into the chat box, the command block's box or the container screen.
    fn scripted_typing(&mut self, typed: Option<&str>) {
        for character in typed.unwrap_or_default().chars() {
            if let Some(chat) = self.chat.as_mut() {
                chat.char_typed(character);
            }
            if let Some(edit) = self.command_block.as_mut() {
                edit.screen.char_typed(character);
            }
            if let Some(edit) = self.structure_block.as_mut() {
                edit.screen.char_typed(character);
            }
            if let Some(screen) = self.screen.as_mut() {
                screen.char_typed(character);
            }
        }
    }

    /// A scripted press on the advancements, the statistics or the command block screen. Whether
    /// one of them was up to take it.
    fn scripted_press_own(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        if self.advancements.is_some() {
            self.advancements_pressed(session, surface, assets);
        } else if self.stats.is_some() {
            self.stats_pressed((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32));
        } else if self.command_block.is_some() {
            let press = glfw::WindowEvent::MouseButton(
                glfw::MouseButton::Button1,
                glfw::Action::Press,
                glfw::Modifiers::empty(),
            );
            self.command_block_event(press, session, surface, assets)?;
        } else if self.structure_block.is_some() {
            let press = glfw::WindowEvent::MouseButton(
                glfw::MouseButton::Button1,
                glfw::Action::Press,
                glfw::Modifiers::empty(),
            );
            self.structure_block_event(press, session, surface, assets)?;
        } else {
            return Ok(false);
        }
        Ok(true)
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
        if self.scripted_press_own(session, surface, assets)? {
            return Ok(());
        }
        if let Some(confirm) = &self.confirm {
            if let Some(action) = confirm.clicked(mouse) {
                self.confirm_action(action, session, surface)?;
            }
        } else if self.accessibility_screen.is_some() {
            self.accessibility_pressed(surface, assets);
        } else if let Some(screen) = self.options_screen.as_mut() {
            let input = screen.clicked(&assets.lang, self.gui_mouse);
            screen.released(&assets.lang);
            self.options_input(input, surface, assets);
        } else if let Some(action) = self.death.as_mut().and_then(|death| death.clicked(mouse)) {
            self.death_action(action, session, surface, assets)?;
        } else if let Some(action) = self.pause.as_ref().and_then(|pause| pause.clicked(mouse)) {
            self.pause_action(action, session, surface, assets);
        } else if self
            .chat
            .as_ref()
            .is_some_and(|chat| chat.leave_bed_clicked(mouse))
        {
            session.send_stop_sleeping()?;
        }
        Ok(())
    }
}

impl Controls {
    /// A scripted Escape in the advancements or the language screen. Whether there was one.
    fn escape_light_screen(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> bool {
        if self.advancements.is_some() {
            self.close_advancements(session, surface);
        } else if self.language_screen.is_some() {
            self.language_screen = None;
        } else {
            return false;
        }
        true
    }

    /// The scripted keys that open or close a screen: the inventory and the advancements.
    fn scripted_screens(
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
        if held.advancements && self.advancements.is_none() {
            self.open_advancements(session, surface, assets);
        }
        Ok(())
    }
}

impl Controls {
    /// The scripted F1, F2 and F3 presses of a tick.
    fn scripted_toggles(&mut self, held: HeldKeys) {
        self.hud_hidden ^= held.toggle_gui;
        self.screenshot_pending |= held.screenshot;
        if held.debug_overlay {
            self.debug.entries.toggle_overlay();
        }
    }
}
