use super::*;

impl Controls {
    /// `PauseScreen`: the menu opens and the mouse is let go. The game goes
    /// on behind it, as on a server.
    pub(super) fn open_pause(&mut self, surface: &mut WindowSurface, assets: &ScreenAssets) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.pause = Some(PauseScreen::new(&assets.creative.font, &assets.lang, size));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// The menu closes and the mouse is grabbed again.
    pub(super) fn close_pause(&mut self, surface: &mut WindowSurface) {
        self.pause = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// What a pause menu button does: "Back to Game" closes the menu and
    /// "Disconnect" leaves; the screens the others open are not built.
    pub(super) fn pause_action(
        &mut self,
        action: PauseAction,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        match action {
            PauseAction::Resume => self.close_pause(surface),
            PauseAction::Disconnect => surface.request_close(),
            PauseAction::Options => self.open_options(surface, assets),
            _ => {}
        }
    }
}
