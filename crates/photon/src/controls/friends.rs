//! The friends overlay and the state behind it (`Minecraft.toggleFriendsScreen`,
//! `FriendsOverlayScreen`, `PlayerSocialManager`).
//!
//! Not done: the confirmation that turns the friends list on, the pause and title menus' friends
//! buttons, and the entries' buttons.

use super::*;
use crate::social::Social;
use photon_game::text::{Component, Lang};
use photon_gui::screen::friends::{FriendsEvent, FriendsOverlay, FriendsView, Listing};
use photon_net::friends::{FriendData, PresenceStatus, ResultCode, State};

impl Controls {
    /// The state of the account's friends, started once the connection has an account.
    pub(crate) fn start_friends(&mut self, session: &PlaySession) {
        if self.friends_list.is_none()
            && let Some(transport) = session.friends_transport()
        {
            self.friends_list = Some(Social::start(transport, std::time::Instant::now()));
        }
    }

    /// One game tick of the friends: the answers of the service, the update of the lists, and the
    /// overlay following them. The toasts the changes call for are returned.
    pub(crate) fn friends_tick(&mut self, lang: &Lang) -> Vec<Component> {
        let Some(social) = self.friends_list.as_mut() else {
            return Vec::new();
        };
        let open = self.friends.is_some();
        social.tick(std::time::Instant::now(), (open, true), presence_status());
        let mut toasts = Vec::new();
        while let Some(notice) = social.take_toast() {
            toasts.push(notice_title(&notice));
        }
        while let Some(code) = social.take_result() {
            if code != ResultCode::Success {
                toasts.push(Component::translatable("gui.friends.toast.generic_error"));
            }
        }
        if social.take_changed() || open {
            let view = self.friends_view(lang);
            if let (Some(overlay), Some(view)) = (self.friends.as_mut(), view) {
                overlay.set_view(lang, view);
            }
        }
        toasts
    }

    /// What the overlay shows of the lists now.
    fn friends_view(&self, lang: &Lang) -> Option<FriendsView> {
        let social = self.friends_list.as_ref()?;
        let data = social.lists();
        let names = |friends: &[photon_net::friends::Friend]| -> Vec<String> {
            friends.iter().map(|friend| friend.name.clone()).collect()
        };
        let (friends, pending) = match social.state() {
            State::Loading => (Listing::Loading, Listing::Loading),
            State::Success => (listing(names(&data.friends)), listing(pending_names(data))),
            failed => {
                let message = lang.runs(&error_message(failed), &Default::default());
                (Listing::Error(message.clone()), Listing::Error(message))
            }
        };
        Some(FriendsView {
            friends,
            pending,
            requests: data.incoming.len(),
            received: data.incoming.len(),
            profile_name: self.profile_name.clone(),
            ready: social.state() == State::Success,
            sending: social.sending(),
        })
    }

    /// `Minecraft.toggleFriendsScreen`: the overlay closes when it is open (unless its box is
    /// being typed in), and opens over the game when the friends list is on.
    pub(super) fn toggle_friends(&mut self, surface: &mut WindowSurface, assets: &ScreenAssets) {
        if let Some(overlay) = self.friends.as_ref() {
            if !overlay.typing() {
                self.close_friends(surface);
            }
            return;
        }
        if self.any_screen_open() || !self.friends_list.as_ref().is_some_and(Social::list_enabled) {
            return;
        }
        let size = Self::gui_size(surface);
        let Some(view) = self.friends_view(&assets.lang) else {
            return;
        };
        if let Some(social) = self.friends_list.as_mut() {
            social.want_presence();
        }
        self.friends = Some(FriendsOverlay::new(&assets.lang, size, view));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    fn close_friends(&mut self, surface: &mut WindowSurface) {
        self.friends = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// An event for the open friends overlay.
    pub(super) fn friends_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, MouseButton, WindowEvent};
        let font = &assets.creative.font;
        if let WindowEvent::CursorPos(x, y) = event {
            self.move_gui_mouse((x, y), surface);
            return;
        }
        let mouse = self.gui_mouse;
        let control = self.mods.control;
        let asked = match event {
            WindowEvent::Key(key, scancode, Action::Press | Action::Repeat, modifiers) => {
                let input = keybinds::input_key(key, scancode);
                let matches = self
                    .options
                    .mappings_of(input)
                    .any(|name| name == "key.friends");
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                self.friends.as_mut().and_then(|screen| {
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
                if let Some(screen) = self.friends.as_mut() {
                    screen.char_typed(character, font);
                }
                None
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => self
                .friends
                .as_mut()
                .and_then(|screen| screen.clicked(mouse, font)),
            _ => None,
        };
        match asked {
            Some(FriendsEvent::Close) => self.close_friends(surface),
            Some(FriendsEvent::Send(name)) => {
                if let Some(social) = self.friends_list.as_mut() {
                    social.act(crate::social::Action::Send(name));
                }
                if let Some(screen) = self.friends.as_mut() {
                    screen.clear_box(font);
                }
            }
            // Copying the name is the next slice.
            Some(FriendsEvent::CopyName) | None => {}
        }
    }
}

/// Presence changes the account's state on the service, so it is posted only when asked for.
fn presence_status() -> Option<PresenceStatus> {
    std::env::var_os("PHOTON_FRIENDS_PRESENCE")
        .is_some_and(|value| value == "1")
        .then_some(PresenceStatus::Online)
}

fn listing(names: Vec<String>) -> Listing {
    if names.is_empty() {
        Listing::Empty
    } else {
        Listing::Entries(names)
    }
}

/// The received requests, then the sent ones.
fn pending_names(data: &FriendData) -> Vec<String> {
    data.incoming
        .iter()
        .chain(&data.outgoing)
        .map(|friend| friend.name.clone())
        .collect()
}

/// `FriendsOverlayScreen.refreshLists`'s message for a state that is no list.
fn error_message(state: State) -> Component {
    let key = match state {
        State::UpgradeNeeded => "gui.friends.error.upgrade_needed",
        State::ConnectionIssue => "gui.friends.error.connection_issue",
        State::TemporaryUnavailable => "gui.friends.error.temporary_unavailable",
        State::UserMayLackActiveProfile => "gui.friends.error.user_may_lack_active_profile",
        State::Unauthorized => "gui.friends.error.unauthorized",
        State::Loading | State::GenericError | State::Success => "gui.friends.error.generic",
    };
    Component::translatable(key)
}

/// The title of a friend toast (`PlayerSocialManager`'s `SystemToast.addOrUpdate`).
fn notice_title(notice: &photon_net::friends::Notice) -> Component {
    use photon_net::friends::Notice;
    let (key, friend) = match notice {
        Notice::FriendAdded(friend) => ("gui.friends.toast.friend_added.message", friend),
        Notice::RequestAccepted(friend) => ("gui.friends.toast.request_accepted.message", friend),
        Notice::RequestReceived(friend) => ("gui.friends.toast.request_received.message", friend),
        Notice::RequestSent(friend) => ("gui.friends.toast.request_sent.message", friend),
    };
    Component::translatable_with(key, vec![Component::literal(friend.name.clone())])
}
