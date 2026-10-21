use super::*;
use photon_core::binary::Uuid;
use photon_game::chat_abilities::MessageSource;
use photon_game::text::Component;
use photon_gui::chat::Tag;
use photon_gui::screen::social::{Page, Roster, SocialEvent, SocialInteractionsScreen};

/// `CommonLinks.BLOCKING_HELP`.
const BLOCKING_HELP: &str = "https://aka.ms/javablocking";

impl Controls {
    /// `Gui.handleKeybinds` for `keySocialInteractions`: the screen opens over the players of the
    /// server, and the mouse is let go.
    pub(super) fn open_social(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let players = session.social_players();
        let person = |(id, name): (Uuid, String)| photon_gui::screen::social::SocialPlayer {
            id,
            name,
            recent_messages: false,
            draft_report: false,
        };
        let roster = Roster {
            // `ServerData.name` of a direct connection.
            server: assets.lang.text("selectServer.defaultName"),
            online: players.online.into_iter().map(person).collect(),
            gone: players.gone.into_iter().map(person).collect(),
            hidden: self.hidden_players.clone(),
            local: session.profile_id().unwrap_or(Uuid::new(0, 0)),
            reporting: false,
        };
        self.social = Some(SocialInteractionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            roster,
            Page::All,
        ));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// The screen closes and the mouse is grabbed again.
    fn close_social(&mut self, surface: &mut WindowSurface) {
        self.social = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// An event for the open social interactions screen.
    pub(super) fn social_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, MouseButton, WindowEvent};
        let font = &assets.creative.font;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            let mouse = self.gui_mouse;
            if let Some(screen) = self.social.as_mut() {
                screen.dragged(mouse, mouse.1 - before);
            }
            return;
        }
        let mouse = self.gui_mouse;
        let wheel = self.options.mouse_wheel_sensitivity;
        let control = self.mods.control;
        let social_key = |controls: &Controls, key, scancode| {
            let input = keybinds::input_key(key, scancode);
            controls
                .options
                .mappings_of(input)
                .any(|name| name == "key.socialInteractions")
        };
        let asked = match event {
            WindowEvent::Key(key, scancode, Action::Press | Action::Repeat, modifiers) => {
                let matches = social_key(self, key, scancode);
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                self.social.as_mut().and_then(|screen| {
                    screen.key_pressed(
                        key,
                        (control, shift),
                        matches,
                        font,
                        &mut WindowClipboard(surface),
                    )
                })
            }
            WindowEvent::Char(character) => {
                if let Some(screen) = self.social.as_mut() {
                    screen.char_typed(character, font);
                }
                None
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => self
                .social
                .as_mut()
                .and_then(|screen| screen.clicked(mouse, font)),
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                if let Some(screen) = self.social.as_mut() {
                    screen.released();
                }
                None
            }
            WindowEvent::Scroll(_, y) => {
                if let Some(screen) = self.social.as_mut() {
                    screen.scrolled(mouse, y * wheel);
                }
                None
            }
            _ => None,
        };
        match asked {
            Some(SocialEvent::Close) => self.close_social(surface),
            Some(SocialEvent::Hidden { id, name, hidden }) => {
                self.set_hidden(id, &name, hidden, assets);
            }
            Some(SocialEvent::BlockingHelp) => {
                let size = surface.scale();
                let gui = (size.gui_scaled_width(), size.gui_scaled_height());
                let screen = ConfirmScreen::link(font, &assets.lang, gui, BLOCKING_HELP, false)
                    .with_background(ConfirmBackground::Menu { in_world: true });
                self.link_confirm = Some((screen, ConfirmTarget::Link(BLOCKING_HELP.to_owned())));
            }
            // The report screen needs the online services.
            Some(SocialEvent::Report(_)) | None => {}
        }
    }

    /// `PlayerEntry.onHiddenOrShown`: the player is hidden from the chat or shown in it again, and
    /// the chat says so.
    fn set_hidden(&mut self, id: Uuid, name: &str, hidden: bool, assets: &ScreenAssets) {
        if hidden {
            if !self.hidden_players.contains(&id) {
                self.hidden_players.push(id);
            }
        } else {
            self.hidden_players.retain(|other| *other != id);
        }
        let key = if hidden {
            "gui.socialInteractions.hidden_in_chat"
        } else {
            "gui.socialInteractions.shown_in_chat"
        };
        let message = Component::translatable_with(key, vec![Component::literal(name.to_owned())]);
        self.chat_log.add_component(
            &message,
            &assets.lang,
            (
                MessageSource::SystemClient,
                None,
                Some(Tag::SystemSinglePlayer),
            ),
            &assets.creative.font,
            false,
        );
    }
}
