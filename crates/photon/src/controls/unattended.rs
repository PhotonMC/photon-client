use super::*;
use photon_game::commands::{CommandCheck, CommandTree};
use photon_game::text::Component;
use photon_gui::screen::confirm::ConfirmBackground;

/// What an open confirmation over the chat is about.
pub enum ConfirmTarget {
    /// `ConfirmLinkScreen`: the address.
    Link(String),
    /// A command to run once the player agrees (`openCommandSendConfirmationWindow`).
    Run(String),
    /// A command only the chat box may send, offered for the clipboard.
    Copy(String),
}

impl Controls {
    /// `ClientPacketListener.sendUnattendedCommand`: a command the player did not type is sent
    /// when it checks out, and asked about when it does not. `command` has no slash.
    pub(super) fn send_unattended(
        &mut self,
        command: &str,
        session: &mut PlaySession,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let tree = session
            .commands()
            .cloned()
            .unwrap_or_else(|| std::sync::Arc::new(CommandTree::empty()));
        let (key, target) = match tree.check(command, session.can_use_selectors()) {
            CommandCheck::NoIssues => {
                session.send_chat(command, true)?;
                return Ok(());
            }
            CommandCheck::ParseErrors => (
                "multiplayer.confirm_command.parse_errors",
                ConfirmTarget::Run(command.to_owned()),
            ),
            CommandCheck::SignatureRequired => (
                "multiplayer.confirm_command.signature_required",
                ConfirmTarget::Copy(command.to_owned()),
            ),
            CommandCheck::PermissionsRequired => (
                "multiplayer.confirm_command.permissions_required",
                ConfirmTarget::Run(command.to_owned()),
            ),
        };
        self.open_command_confirm(key, target, command, surface, assets);
        Ok(())
    }

    /// `openSendConfirmationWindow`: the command in yellow in the message, and a way back.
    fn open_command_confirm(
        &mut self,
        key: &str,
        target: ConfirmTarget,
        command: &str,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        let translated = Component::translatable;
        let accept = match target {
            ConfirmTarget::Copy(_) => "chat.copy",
            _ => "multiplayer.confirm_command.run_command",
        };
        let back = if self.any_screen_open() {
            "gui.back"
        } else {
            "gui.cancel"
        };
        let message = Component::translatable_with(
            key,
            vec![Component::literal(command).with_colour("yellow")],
        );
        let screen = ConfirmScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            &translated("multiplayer.confirm_command.title"),
            &message,
            &translated(accept),
            &translated(back),
        )
        .with_background(ConfirmBackground::Menu { in_world: true });
        self.link_confirm = Some((screen, target));
    }

    /// The answer to a confirmation over the chat.
    pub(super) fn answer_confirm(
        &mut self,
        action: LinkAction,
        target: &ConfirmTarget,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        match (action, target) {
            (LinkAction::Yes, ConfirmTarget::Link(url)) => open_uri(url),
            (LinkAction::Copy, ConfirmTarget::Link(url)) => surface.set_clipboard(url),
            (LinkAction::Yes, ConfirmTarget::Run(command)) => session.send_chat(command, true)?,
            (LinkAction::Yes, ConfirmTarget::Copy(command)) => {
                surface.set_clipboard(&format!("/{command}"));
            }
            _ => {}
        }
        Ok(())
    }
}
