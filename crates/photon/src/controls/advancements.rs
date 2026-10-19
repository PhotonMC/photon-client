use super::*;
use photon_gui::screen::advancements::{AdvancementsOut, AdvancementsScreen};

/// The advancements screen and what the controls keep of it.
pub struct OpenAdvancements {
    pub screen: AdvancementsScreen,
    /// Whether the game was captured when it opened (the key), so closing grabs the mouse again;
    /// from the pause menu the menu is what is left.
    from_game: bool,
    /// The primary button is down: the pointer drags the tree.
    dragging: bool,
}

impl Controls {
    /// The keys that open a screen over the game: the advancements, the social interactions and
    /// the chat; and the cinematic camera's. Whether the mapping was one of them.
    pub(super) fn screen_key(
        &mut self,
        name: &str,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> bool {
        match name {
            "key.smoothCamera" => self.turning.smooth_camera = !self.turning.smooth_camera,
            "key.advancements" => self.open_advancements(session, surface, assets),
            "key.socialInteractions" => self.open_social(session, surface, assets),
            "key.chat" => self.open_chat(ChatMethod::Message, true, surface, assets),
            "key.command" => self.open_chat(ChatMethod::Command, true, surface, assets),
            _ => return false,
        }
        true
    }

    /// `Gui.handleKeybinds` for `keyAdvancements`, and the pause menu's Advancements: the screen
    /// listens to the advancements, and the server is told the tab shown.
    pub(super) fn open_advancements(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let fonts = (&*assets.creative.font, &*assets.lang);
        let (screen, out) = AdvancementsScreen::new(session.advancements_mut(), fonts, size, true);
        let from_game = self.captured;
        self.advancements = Some(OpenAdvancements {
            screen,
            from_game,
            dragging: false,
        });
        Self::tell_server(session, &out);
        if from_game {
            self.captured = false;
            self.last_cursor = None;
            surface.set_cursor_captured(false);
            self.release_keys();
        }
    }

    fn tell_server(session: &mut PlaySession, out: &AdvancementsOut) {
        let Some(tab) = out.opened_tab.as_deref() else {
            return;
        };
        if let Err(error) = session.send_seen_advancements(Some(tab)) {
            log::warn!("could not tell the server of the advancements tab: {error:?}");
        }
    }

    /// `AdvancementsScreen.removed`: the screen stops listening and the server is told it closed.
    /// Whether the game was captured when it opened comes back.
    pub(super) fn drop_advancements(&mut self, session: &mut PlaySession) -> Option<bool> {
        let mut open = self.advancements.take()?;
        open.screen.close(session.advancements_mut());
        if let Err(error) = session.send_seen_advancements(None) {
            log::warn!("could not tell the server the advancements closed: {error:?}");
        }
        Some(open.from_game)
    }

    /// The screen is left for the game or the pause menu: the mouse is grabbed again if the key
    /// had let it go.
    pub(super) fn close_advancements(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) {
        if self.drop_advancements(session) == Some(true) {
            self.captured = true;
            self.last_cursor = None;
            surface.set_cursor_captured(true);
        }
    }

    /// The tick: what the server changed shows on the screen, and the pointer is over the tree.
    pub(super) fn advancements_tick(&mut self, session: &mut PlaySession, assets: &ScreenAssets) {
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        if let Some(open) = self.advancements.as_mut() {
            let fonts = (&*assets.creative.font, &*assets.lang);
            open.screen.sync(session.advancements_mut(), fonts);
            open.screen.tick(mouse);
        }
    }

    /// An event for the open advancements screen: Escape and Done leave it, the primary button
    /// chooses a tab or drags the tree, and the wheel scrolls it.
    pub(super) fn advancements_event(
        &mut self,
        event: glfw::WindowEvent,
        (session, surface, assets): (&mut PlaySession, &mut WindowSurface, &ScreenAssets),
    ) -> Result {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.close_advancements(session, surface);
            }
            WindowEvent::CursorPos(x, y) => {
                let before = self.gui_mouse;
                self.move_gui_mouse((x, y), surface);
                let delta = (self.gui_mouse.0 - before.0, self.gui_mouse.1 - before.1);
                if let Some(open) = self.advancements.as_mut().filter(|open| open.dragging) {
                    open.screen.dragged(delta);
                }
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                self.advancements_pressed(session, surface, assets);
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                if let Some(open) = self.advancements.as_mut() {
                    open.dragging = false;
                    open.screen.released();
                }
            }
            WindowEvent::Scroll(x, y) => {
                let sensitivity = self.options.mouse_wheel_sensitivity;
                if let Some(open) = self.advancements.as_mut() {
                    open.screen.scrolled((x * sensitivity, y * sensitivity));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// A press of the primary button: a tab is chosen, Done leaves, and the tree can be dragged.
    pub(super) fn advancements_pressed(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let fonts = (&*assets.creative.font, &*assets.lang);
        let Some(open) = self.advancements.as_mut() else {
            return;
        };
        open.dragging = true;
        let out = open
            .screen
            .clicked(session.advancements_mut(), fonts, mouse);
        Self::tell_server(session, &out);
        if out.close {
            self.close_advancements(session, surface);
        }
    }
}
