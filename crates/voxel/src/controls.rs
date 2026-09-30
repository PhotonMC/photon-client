//! The player's hands on the client: the keys and the mouse, a script's
//! steps, and the inventory screen they may be typing into.

use voxel_game::attack::AttackKey;
use voxel_game::hotbar::ScrollWheel;
use voxel_net::connection::PlaySession;
use voxel_render::gui::screen::{KeyMods, KeyResult};
use voxel_render::window::WindowSurface;

use crate::screens::{
    OpenScreen, ScreenAssets, WindowClipboard, close_inventory, open_inventory, run_actions,
};
use crate::script::HeldKeys;
use crate::view::CameraType;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

/// What the player is holding and pointing at between events.
pub struct Controls {
    pub keys: HeldKeys,
    /// Attack clicks waiting for the next tick, and `Minecraft.missTime`:
    /// ticks a missed attack keeps the next one waiting.
    pub attack_key: AttackKey,
    /// The use key keeps its clicks and held state the same way.
    pub use_key: AttackKey,
    scroll_wheel: ScrollWheel,
    /// F5 cycles first person, third person from behind and from the front.
    pub camera_type: CameraType,
    /// Whether the mouse is grabbed to look around.
    pub captured: bool,
    last_cursor: Option<(f64, f64)>,
    /// The open inventory screen, if any.
    pub screen: OpenScreen,
    /// The id of the container the server opened, while its screen is up.
    server_container: Option<i32>,
    /// The creative screen's tab, kept for the next time it opens.
    pub creative_tab: usize,
    /// The mouse in GUI units.
    pub gui_mouse: (f64, f64),
    /// The button a drag is held with (`MouseHandler.activeButton`).
    active_button: Option<i32>,
}

impl Controls {
    pub fn new(camera_type: CameraType, captured: bool) -> Self {
        Self {
            keys: HeldKeys::default(),
            attack_key: AttackKey::default(),
            use_key: AttackKey::default(),
            scroll_wheel: ScrollWheel::default(),
            camera_type,
            captured,
            last_cursor: None,
            screen: None,
            server_container: None,
            creative_tab: voxel_game::creative_menu::default_tab(),
            gui_mouse: (0.0, 0.0),
            active_button: None,
        }
    }

    /// Let go of every key, as the mouse is freed.
    fn release_keys(&mut self) {
        self.keys = HeldKeys::default();
        self.attack_key.release();
        self.use_key.release();
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
        // A container with no screen here is closed at once, so the server
        // does not wait on a window the player cannot see.
        if session.open_container_unsupported() {
            session.close_inventory()?;
        }
        match (self.server_container, session.open_container_info()) {
            (None, Some((id, kind, title))) => {
                self.server_container = Some(id);
                self.screen = Some(assets.container_screen(kind, &title, width, height));
                self.release_keys();
                surface.set_cursor_captured(false);
                let (w, h) = surface.window_size();
                surface.set_cursor_position(f64::from(w) / 2.0, f64::from(h) / 2.0);
                return Ok(());
            }
            (Some(open), Some((id, kind, title))) if open != id => {
                self.server_container = Some(id);
                self.screen = Some(assets.container_screen(kind, &title, width, height));
                return Ok(());
            }
            (Some(_), None) => {
                // The server closed it, or the player did.
                self.server_container = None;
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
        if self.screen.is_some() && for_screen {
            let handled = self.screen_event(event, session, surface);
            self.drain_screen_actions(session)?;
            handled
        } else {
            self.world_event(event, session, surface, assets)
        }
    }

    /// The clicks the open screen queued while it took a key or a character.
    fn drain_screen_actions(&mut self, session: &mut PlaySession) -> Result {
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
    fn screen_event(
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
    fn world_event(
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
                if self.captured {
                    self.captured = false;
                    surface.set_cursor_captured(false);
                    self.release_keys();
                } else {
                    surface.request_close();
                }
            }
            WindowEvent::Key(Key::F5, _, Action::Press, _) => {
                self.camera_type = self.camera_type.next();
            }
            WindowEvent::Key(Key::F11, _, Action::Press, _) => surface.toggle_fullscreen(),
            WindowEvent::Scroll(x, y) if self.captured => {
                if !spectating
                    && let Some(slot) =
                        self.scroll_wheel
                            .select(x, y, session.status().selected_slot)
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
            WindowEvent::MouseButton(_, Action::Press, _) if !self.captured => {
                self.captured = true;
                self.last_cursor = None;
                surface.set_cursor_captured(true);
            }
            WindowEvent::CursorPos(x, y) => {
                if self.captured
                    && let Some((last_x, last_y)) = self.last_cursor
                    && let Some(player) = session.player_mut()
                {
                    // The default sensitivity, 0.5, makes the scale exactly one.
                    let sensitivity = 0.5 * 0.6 + 0.2_f64;
                    let scaled = sensitivity * sensitivity * sensitivity * 8.0;
                    player.turn((x - last_x) * scaled, (y - last_y) * scaled);
                }
                self.last_cursor = Some((x, y));
            }
            _ => {}
        }
        Ok(())
    }
}

/// A scripted key: a click on this tick, and whether it is held.
fn script_key(key: &mut AttackKey, click: bool, hold: bool) {
    if click {
        key.press();
    }
    if !hold {
        key.release();
    } else if !key.is_down() {
        key.press();
        key.take_clicks();
    }
}

/// The number key for a hotbar slot, 1 to 9.
fn hotbar_key(key: glfw::Key) -> Option<i32> {
    use glfw::Key;
    [
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
    ]
    .iter()
    .position(|candidate| *candidate == key)
    .map(|slot| slot as i32)
}
