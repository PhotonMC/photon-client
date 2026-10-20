//! The debug options screen (`DebugOptionsScreen`), opened with F3+F6: its events, and the entry
//! list it changes.

use super::*;
use photon_gui::screen::debug_options::{DebugOptionsEvent, DebugOptionsScreen};

impl Controls {
    /// F3+F6: the screen closes if it is open, and opens over the game otherwise. (Another screen
    /// in the way, which the reference closes first, is not handled.)
    pub(super) fn toggle_debug_options(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if self.debug_options.is_some() {
            self.close_debug_options(surface);
        } else if !self.any_screen_open() {
            self.open_debug_options(session, surface, assets);
        }
    }

    /// `new DebugOptionsScreen()`: the screen opens over the entries, and the mouse is let go.
    fn open_debug_options(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let scale = surface.scale();
        let size = (scale.gui_scaled_width(), scale.gui_scaled_height());
        self.debug_options = Some(DebugOptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            &self.debug.entries,
            session.reduced_debug_info(),
        ));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// The screen closes and the mouse is grabbed again.
    fn close_debug_options(&mut self, surface: &mut WindowSurface) {
        self.debug_options = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// An event for the open debug options screen.
    pub(super) fn debug_options_event(
        &mut self,
        event: glfw::WindowEvent,
        (session, surface, assets): (&mut PlaySession, &mut WindowSurface, &ScreenAssets),
    ) -> Result {
        use glfw::{Action, MouseButton, WindowEvent};
        let font = &assets.creative.font;
        let mouse = self.gui_mouse;
        let Some(screen) = self.debug_options.as_mut() else {
            return Ok(());
        };
        let asked = match event {
            WindowEvent::CursorPos(x, y) => {
                let before = self.gui_mouse.1;
                self.move_gui_mouse((x, y), surface);
                let mouse = self.gui_mouse;
                if let Some(screen) = self.debug_options.as_mut() {
                    screen.dragged(mouse, mouse.1 - before);
                }
                None
            }
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                return self.debug_options_key(key, modifiers, (session, surface, assets));
            }
            WindowEvent::Char(character) => {
                screen.char_typed(character, font);
                None
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                screen.clicked(mouse, font)
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released();
                None
            }
            WindowEvent::Scroll(_, y) => {
                screen.scrolled(mouse, y * self.options.mouse_wheel_sensitivity);
                None
            }
            _ => None,
        };
        self.debug_options_asked(asked, surface);
        Ok(())
    }

    /// A key for the open screen: a debug key with F3 held is answered as in the world, and the
    /// rows show what it changed; the rest goes to the screen.
    fn debug_options_key(
        &mut self,
        key: glfw::Key,
        modifiers: glfw::Modifiers,
        (session, surface, assets): (&mut PlaySession, &mut WindowSurface, &ScreenAssets),
    ) -> Result {
        if self.debug.down && self.run_debug_key(key, (session, surface, assets))? {
            if let Some(screen) = self.debug_options.as_mut() {
                screen.refresh(&self.debug.entries);
            }
            return Ok(());
        }
        let keys = (
            self.mods.control,
            modifiers.contains(glfw::Modifiers::Shift),
        );
        let font = &assets.creative.font;
        let asked = self
            .debug_options
            .as_mut()
            .and_then(|screen| screen.key_pressed(key, keys, font, &mut WindowClipboard(surface)));
        self.debug_options_asked(asked, surface);
        Ok(())
    }

    /// What the screen asked: the entry list takes the status or the profile and saves, as the
    /// buttons' actions do (`setStatus` saves itself; the profile buttons save after loading).
    fn debug_options_asked(
        &mut self,
        asked: Option<DebugOptionsEvent>,
        surface: &mut WindowSurface,
    ) {
        match asked {
            Some(DebugOptionsEvent::Close) => self.close_debug_options(surface),
            Some(DebugOptionsEvent::Status { id, status }) => {
                self.debug.entries.set_status(id, status);
                self.save_debug_profile();
            }
            Some(DebugOptionsEvent::Profile(profile)) => {
                self.debug.entries.load_profile(profile);
                self.save_debug_profile_now();
            }
            None => {}
        }
    }

    /// A scripted press of the primary button at the mouse.
    pub(super) fn scripted_debug_options_press(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let press = glfw::WindowEvent::MouseButton(
            glfw::MouseButton::Button1,
            glfw::Action::Press,
            glfw::Modifiers::empty(),
        );
        self.debug_options_event(press, (session, surface, assets))
    }
}
