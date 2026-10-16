use super::*;
use photon_game::text::Component;
use photon_gui::screen::confirm::ConfirmBackground;
use photon_net::connection::{PackAction, PackRequest};

/// `ClientCommonPacketListenerImpl.PackConfirmScreen`: the packs the server asked for that the
/// player has not been answered for, and whether the latest of them is required.
#[derive(Debug, Clone, Default)]
pub struct PackPrompt {
    requests: Vec<PackRequest>,
    required: bool,
    prompt: Option<Component>,
}

impl PackPrompt {
    /// `addOrUpdatePackPrompt`: a pack asked for while the screen is up joins it.
    fn add(&mut self, request: PackRequest) {
        self.required = request.required;
        self.prompt.clone_from(&request.prompt);
        self.requests.push(request);
    }
}

impl Controls {
    /// The packs the server asked for since the last frame: the player is asked about them.
    pub fn receive_pack_requests(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let requests = session.take_pack_requests();
        if requests.is_empty() {
            return;
        }
        let prompt = self.pack_prompt.get_or_insert_with(PackPrompt::default);
        for request in requests {
            prompt.add(request);
        }
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.confirm = Some(
            ConfirmScreen::server_pack(
                &assets.creative.font,
                &assets.lang,
                size,
                prompt.required,
                prompt.prompt.as_ref(),
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

    /// What answering the prompt does (`PackConfirmScreen`'s callback). There is nothing yet to
    /// download a pack with, so a pack that is accepted is answered as accepted and then as one
    /// that could not be downloaded; one that is refused is answered so, and a required one
    /// ends the connection.
    pub(super) fn answer_pack_prompt(
        &mut self,
        accepted: bool,
        session: &mut PlaySession,
    ) -> Result {
        let Some(prompt) = self.pack_prompt.take() else {
            return Ok(());
        };
        if !accepted && prompt.required {
            session.disconnect(Component::translatable(
                "multiplayer.requiredTexturePrompt.disconnect",
            ));
        }
        for request in &prompt.requests {
            if accepted {
                session.answer_resource_pack(request.id, PackAction::Accepted)?;
                session.answer_resource_pack(request.id, PackAction::FailedDownload)?;
            } else {
                session.answer_resource_pack(request.id, PackAction::Declined)?;
            }
        }
        Ok(())
    }
}
