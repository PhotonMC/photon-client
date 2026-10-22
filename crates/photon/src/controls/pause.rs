use super::*;

impl Controls {
    /// `PauseScreen`: the menu opens and the mouse is let go. The game goes
    /// on behind it, as on a server.
    pub(super) fn open_pause(&mut self, surface: &mut WindowSurface, assets: &ScreenAssets) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let custom = self.pause_additions.as_ref().map(|dialog| {
            let common = dialog.common();
            common
                .external_title
                .clone()
                .unwrap_or_else(|| common.title.clone())
        });
        self.pause = Some(PauseScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            custom.as_ref(),
        ));
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
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        match action {
            PauseAction::Resume => self.close_pause(surface),
            PauseAction::Disconnect => self.leave = true,
            PauseAction::Options => self.open_options(surface, assets),
            PauseAction::Statistics => self.open_stats(surface, assets),
            PauseAction::Advancements => self.open_advancements(session, surface, assets),
            // `minecraft.player.connection.showDialog(dialog, this)`: over the menu.
            PauseAction::Custom => {
                if let Some(dialog) = self.pause_additions.clone() {
                    let dialog = dialog.resolved(session.server_links());
                    self.show_dialog(dialog, surface, assets);
                }
            }
            _ => {}
        }
    }
}
