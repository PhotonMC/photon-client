use super::*;
use photon_gui::screen::confirm::ConfirmBackground;

/// `ClientCommonPacketListenerImpl.PackConfirmScreen`: whether the question about the server's
/// packs is up. What it asks is kept by the connection's pack source, which also takes the
/// answer.
#[derive(Debug, Clone, Copy, Default)]
pub struct PackPrompt;

impl Controls {
    /// The question the server's packs raised since the last frame: the player is asked.
    pub fn receive_pack_requests(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(prompt) = session.packs().take_prompt() else {
            return;
        };
        self.pack_prompt = Some(PackPrompt);
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.confirm = Some(
            ConfirmScreen::server_pack(
                &assets.creative.font,
                &assets.lang,
                size,
                prompt.required,
                prompt.text.as_ref(),
            )
            .with_background(ConfirmBackground::Menu { in_world: true }),
        );
        if let Some(chat) = self.chat.take() {
            self.leave_chat(&chat, ExitReason::Interrupted);
        }
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
    }

    /// What answering the prompt does (`PackConfirmScreen`'s callback): the packs are taken or
    /// declined, and a required one that is refused ends the connection.
    pub(super) fn answer_pack_prompt(&mut self, accepted: bool, session: &mut PlaySession) {
        if self.pack_prompt.take().is_none() {
            return;
        }
        if let Some(reason) = session.packs().answer_prompt(accepted).disconnect {
            session.disconnect(reason);
        }
    }
}
