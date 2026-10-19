use super::*;
use photon_gui::screen::stats::{StatsAction, StatsScreen};

impl Controls {
    /// The tick of the screens that watch the game: the in-bed chat box, and the statistics.
    pub fn screens_tick(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        self.bed_tick(session, surface, assets);
        self.stats_tick(session, assets);
    }

    /// `PauseScreen`'s Statistics: the screen opens over the pause menu.
    pub(super) fn open_stats(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let none = std::collections::HashMap::new();
        self.stats = Some(StatsScreen::new(&assets.lang, size, true, &none));
    }

    /// `StatsScreen.init` asks the server for the statistics once, and `handleAwardStats` builds
    /// the lists in the tick the answer was handled (with the screen up).
    fn stats_tick(&mut self, session: &mut PlaySession, assets: &ScreenAssets) {
        let mut updated = session.take_stats_updated();
        let Some(stats) = self.stats.as_mut() else {
            return;
        };
        if stats.take_request() {
            // What arrived before the request is not its answer: the screen did not exist then.
            updated = false;
            if let Err(error) = session.request_stats() {
                log::warn!("could not ask for the statistics: {error:?}");
            }
        }
        if updated {
            stats.on_stats_updated(&assets.lang, session.stats());
        }
        stats.set_stats(session.stats());
    }

    /// A press of the primary button on the statistics screen: Done leaves it.
    pub(super) fn stats_pressed(&mut self, mouse: (i32, i32)) {
        if self.stats.as_mut().and_then(|stats| stats.clicked(mouse)) == Some(StatsAction::Close) {
            self.stats = None;
        }
    }

    /// An event for the open statistics screen: Done and Escape go back to the pause menu, the
    /// mouse chooses a tab and the wheel scrolls.
    pub(super) fn stats_event(
        &mut self,
        event: glfw::WindowEvent,
        _session: &mut PlaySession,
        surface: &mut WindowSurface,
        _assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        if self.stats.is_none() {
            return Ok(());
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => self.stats = None,
            WindowEvent::CursorPos(x, y) => self.move_gui_mouse((x, y), surface),
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                self.stats_pressed(mouse);
            }
            WindowEvent::Scroll(_, y) => {
                let delta = y * self.options.mouse_wheel_sensitivity;
                self.stats
                    .iter_mut()
                    .for_each(|stats| stats.scrolled(mouse, delta));
            }
            _ => {}
        }
        Ok(())
    }
}
