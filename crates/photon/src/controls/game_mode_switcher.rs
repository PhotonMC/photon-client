use super::*;
use photon_gui::screen::game_mode::GameModeSwitcherScreen;

/// `debug.prefix` and a message: `KeyboardHandler.debugFeedbackComponent`.
pub(super) fn debug_feedback(key: &str) -> photon_game::text::Component {
    use photon_game::text::{Component, Style};
    let prefix = Component::translatable("debug.prefix").with_style(&Style {
        colour: photon_game::text::named_colour("yellow"),
        bold: Some(true),
        ..Style::default()
    });
    Component::empty()
        .append(prefix)
        .append(Component::literal(" "))
        .append(Component::translatable(key))
}

impl Controls {
    /// F4 with F3 held, in the world: `KeyboardHandler.handleDebugKeys` opens the switcher for a
    /// player who may change game modes, and says why not for one who may not.
    pub(super) fn debug_switch_game_mode(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if !session.may_change_game_mode() {
            self.debug_say("debug.gamemodes.error", assets);
            return;
        }
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.game_mode_switcher = Some(GameModeSwitcherScreen::new(
            GameModeSwitcherScreen::default_selected(
                session.previous_game_mode(),
                session.game_mode(),
            ),
            size,
        ));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        let (w, h) = surface.window_size();
        surface.set_cursor_position(f64::from(w) / 2.0, f64::from(h) / 2.0);
        self.release_keys();
    }

    /// An event for the open switcher: F4 steps to the next mode, letting go of F3 switches to the
    /// one picked and closes it, Escape opens the pause menu instead.
    pub(super) fn game_mode_switcher_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let Some(screen) = self.game_mode_switcher.as_mut() else {
            return Ok(());
        };
        match event {
            WindowEvent::Key(Key::F4, _, Action::Press | Action::Repeat, _) => screen.next_mode(),
            WindowEvent::Key(Key::F3, _, Action::Release, _) => {
                let picked = screen.selected();
                // `switchToHoveredGameMode`: another mode than the one played in now.
                if picked != session.game_mode() && session.may_change_game_mode() {
                    session.change_game_mode(picked)?;
                }
                self.close_game_mode_switcher(surface);
            }
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.game_mode_switcher = None;
                self.open_pause(surface, assets);
            }
            WindowEvent::CursorPos(x, y) => self.move_gui_mouse((x, y), surface),
            _ => {}
        }
        Ok(())
    }

    /// The screen closes and the mouse is grabbed again.
    pub(super) fn close_game_mode_switcher(&mut self, surface: &mut WindowSurface) {
        self.game_mode_switcher = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }
}
