use super::*;
use photon_game::commands::{CommandCheck, CommandTree};
use photon_game::dialog::Next;
use photon_gui::screen::confirm::ConfirmBackground;

/// What an open confirmation over the chat is about.
pub enum ConfirmTarget {
    /// `ConfirmLinkScreen`: the address.
    Link(String),
    /// A command to run once the player agrees (`openCommandSendConfirmationWindow`), and the
    /// dialog screen to leave for afterwards.
    Run(String, Option<Next>),
    /// A command only the chat box may send, offered for the clipboard.
    Copy(String, Option<Next>),
    /// The same, offered for the chat box with the game as the screen after.
    Suggest(String),
}

impl Controls {
    /// `ClientPacketListener.sendUnattendedCommand`: a command the player did not type is sent
    /// when it checks out, and asked about when it does not. `command` has no slash, and `after`
    /// is the screen a dialog leaves for (none: the chat stays).
    pub(super) fn send_unattended(
        &mut self,
        command: &str,
        after: Option<Next>,
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
                self.leave_for(after, surface, assets);
                return Ok(());
            }
            CommandCheck::ParseErrors => (
                "multiplayer.confirm_command.parse_errors",
                ConfirmTarget::Run(command.to_owned(), after),
            ),
            CommandCheck::SignatureRequired => (
                "multiplayer.confirm_command.signature_required",
                self.signed_target(command, after),
            ),
            CommandCheck::PermissionsRequired => (
                "multiplayer.confirm_command.permissions_required",
                ConfirmTarget::Run(command.to_owned(), after),
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
        let accept = match target {
            ConfirmTarget::Copy(..) => "chat.copy",
            ConfirmTarget::Suggest(_) => "multiplayer.confirm_command.suggest_command",
            _ => "multiplayer.confirm_command.run_command",
        };
        let screen = ConfirmScreen::command(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            (key, command, accept),
            self.any_screen_open(),
        )
        .with_background(ConfirmBackground::Menu { in_world: true });
        self.link_confirm = Some((screen, target));
    }

    /// `openSignedCommandSendConfirmationWindow`: with no screen to go back to and a player who
    /// may send commands the command is offered for the chat box, else for the clipboard.
    fn signed_target(&mut self, command: &str, after: Option<Next>) -> ConfirmTarget {
        let screen_after = match after {
            None | Some(Next::Same | Next::Waiting) => true,
            Some(Next::Previous) => self.screen_open_below_dialog(),
        };
        if !screen_after && self.chat_log.abilities().can_send_commands() {
            ConfirmTarget::Suggest(command.to_owned())
        } else {
            ConfirmTarget::Copy(command.to_owned(), after)
        }
    }

    /// Whether a screen other than the dialog is open (`previousScreen != null`).
    fn screen_open_below_dialog(&mut self) -> bool {
        let dialog = self.dialog.take();
        let open = self.any_screen_open();
        self.dialog = dialog;
        open
    }

    /// `setScreen(screenAfterCommand)` for a dialog's button.
    fn leave_for(&mut self, after: Option<Next>, surface: &WindowSurface, assets: &ScreenAssets) {
        if let Some(next) = after {
            self.leave_dialog(next, surface, assets);
        }
    }

    /// The answer to a confirmation over the chat.
    pub(super) fn answer_confirm(
        &mut self,
        action: LinkAction,
        target: &ConfirmTarget,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        match (action, target) {
            (LinkAction::Yes, ConfirmTarget::Link(url)) => open_uri(url),
            (LinkAction::Copy, ConfirmTarget::Link(url)) => surface.set_clipboard(url),
            (LinkAction::Yes, ConfirmTarget::Run(command, after)) => {
                session.send_chat(command, true)?;
                self.leave_for(*after, surface, assets);
            }
            (LinkAction::Yes, ConfirmTarget::Copy(command, after)) => {
                surface.set_clipboard(&format!("/{command}"));
                self.leave_for(*after, surface, assets);
            }
            (LinkAction::Yes, ConfirmTarget::Suggest(command)) => {
                self.dialog = None;
                self.open_chat(ChatMethod::Command, true, surface, assets);
                if let Some(chat) = self.chat.as_mut() {
                    chat.insert_text(command, false);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
