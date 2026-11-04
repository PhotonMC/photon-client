use super::*;

/// `CommonLinks.SNAPSHOT_BUGS_FEEDBACK` and `CommonLinks.RELEASE_FEEDBACK` (the build is a release).
const SNAPSHOT_BUGS_FEEDBACK: &str = "https://aka.ms/snapshotbugs?ref=game";
const RELEASE_FEEDBACK: &str = "https://aka.ms/javafeedback?ref=game";

impl Controls {
    /// `ConfirmLinkScreen.confirmLink`: the address is asked about over the pause menu.
    fn ask_about_link(&mut self, url: &str, surface: &WindowSurface, assets: &ScreenAssets) {
        let screen = ConfirmScreen::link(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            url,
            true,
        )
        .with_background(ConfirmBackground::Menu { in_world: true });
        self.link_confirm = Some((screen.into(), ConfirmTarget::Link(url.to_owned())));
    }

    /// A link handed over during the game (over the pause menu, which opens for it): the question
    /// whether to leave for another game, or why the link did nothing.
    pub(crate) fn ask_about_game(
        &mut self,
        prompt: crate::launch_link::InPlay,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use crate::launch_link::InPlay;
        use photon_game::text::Component;
        if self.pause.is_none() {
            self.open_pause(surface, assets);
        }
        let lang = &assets.lang;
        let text = |key: &str| Component::literal(lang.text(key));
        let worded = |key: &str, name: &str| Component::literal(lang.text(key).replace("%s", name));
        let (title, message, buttons, target) = match prompt {
            InPlay::Ask(address, name) => (
                text("photon.link.leave_title"),
                worded("photon.link.leave_ask", &name),
                Some(("gui.yes", "gui.no")),
                ConfirmTarget::Game(address, name),
            ),
            InPlay::AskSignIn(state, name) => (
                text("photon.signin.heading"),
                worded("photon.signin.ask", &name),
                Some(("photon.signin.confirm", "photon.signin.cancel")),
                ConfirmTarget::SignIn(state),
            ),
            InPlay::Say(title, key) => (text(title), text(key), None, ConfirmTarget::Notice),
        };
        let (font, size) = (&assets.creative.font, Self::gui_size(surface));
        // The questions Photon's links ask are Photon's own screen, over the dimmed game; a notice
        // keeps the confirmation screen's one button.
        let screen: Question = match buttons {
            Some((yes, no)) => {
                let mut question = NoticeScreen::asking(
                    font,
                    lang,
                    size,
                    (&title, &message, &text(no), &text(yes)),
                )
                .over_game();
                // A question that opens by itself, over whatever the player was doing, wakes
                // after a second.
                question.set_delay(20);
                question.into()
            }
            None => ConfirmScreen::notice(font, lang, size, (&title, &message), &text("gui.back"))
                .with_background(ConfirmBackground::Menu { in_world: true })
                .into(),
        };
        self.link_confirm = Some((screen, target));
    }

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
        let screen = PauseScreen::new(&assets.creative.font, &assets.lang, size, custom.as_ref());
        // The friends button works once the connection has an account to ask the service as.
        self.pause = Some(if self.friends_list.is_some() {
            screen
        } else {
            screen.without_services(&assets.lang)
        });
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

    /// What a pause menu button does.
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
            PauseAction::PlayerReporting => self.open_social(session, surface, assets),
            PauseAction::ReportBugs | PauseAction::Feedback => {
                let url = match action {
                    PauseAction::ReportBugs => SNAPSHOT_BUGS_FEEDBACK,
                    _ => RELEASE_FEEDBACK,
                };
                self.ask_about_link(url, surface, assets);
            }
            PauseAction::Friends => self.open_friends_from_pause(surface, assets),
            PauseAction::Custom => self.show_pause_additions(session, surface, assets),
        }
    }

    /// The overlay replaces the menu (`Minecraft.toggleFriendsScreen` from the pause screen) but
    /// has it under it (`backgroundScreen`); closing it returns to the menu.
    pub(super) fn open_friends_from_pause(
        &mut self,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        self.pause_under = self.pause.take();
        self.friends_from_pause = true;
        self.toggle_friends(surface, assets);
        // Neither the overlay nor its question opened: the menu stays.
        if self.friends.is_none() && !self.question_open() {
            self.restore_pause();
        }
    }

    // `minecraft.player.connection.showDialog(dialog, this)`: over the menu.
    fn show_pause_additions(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if let Some(dialog) = self.pause_additions.clone() {
            let dialog = dialog.resolved(session);
            self.show_dialog(dialog, surface, assets);
        }
    }
}
