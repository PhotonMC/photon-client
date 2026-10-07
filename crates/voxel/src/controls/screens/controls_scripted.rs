use super::*;

impl Controls {
    /// A scripted Escape: the confirmation answers no once awake; else the
    /// pause menu opens, or closes.
    pub(in super::super) fn scripted_escape(
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
}
