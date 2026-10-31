//! The friends overlay and the state behind it (`Minecraft.toggleFriendsScreen`,
//! `FriendsOverlayScreen`, `PlayerSocialManager`).
//!
//! Not done: the confirmation that turns the friends list on, the pause and title menus' friends
//! buttons, and the entries' buttons.

use super::*;
use crate::social::{Action, Social};
use photon_game::text::{Component, Lang, Run, Style};
use photon_gui::options::online::FriendFacts;
use photon_gui::screen::friends::{
    Entry, EntryKind, FriendsEvent, FriendsOverlay, FriendsView, Listing,
};
use photon_net::friends::{Friend, FriendData, PresenceStatus, ResultCode, State};

impl Controls {
    /// The state of the account's friends, started once the connection has an account.
    pub(crate) fn start_friends(&mut self, session: &PlaySession) {
        if self.friends_list.is_none()
            && let Some(transport) = session.friends_transport()
        {
            self.friends_list = Some(Social::start(transport, std::time::Instant::now()));
        }
    }

    /// The state of the friends of the title screen, from the account resolved there.
    pub(crate) fn start_friends_for(&mut self, account: &photon_net::account::Account) {
        if self.friends_list.is_some() {
            return;
        }
        let host = photon_net::signin::Hosts::from_environment().services;
        let transport = photon_net::friends::HttpTransport::for_account(account, &host);
        self.friends_list = Some(Social::start(transport, std::time::Instant::now()));
        self.profile_name.clone_from(&account.name);
    }

    /// One game tick of the friends: the answers of the service, the update of the lists, and the
    /// overlay following them. The toasts the changes call for are returned.
    pub(crate) fn friends_tick(&mut self, lang: &Lang) -> Vec<Component> {
        let Some(social) = self.friends_list.as_mut() else {
            return Vec::new();
        };
        let open = self.friends.is_some();
        let notify = self.options.online.in_game_notification;
        social.tick(std::time::Instant::now(), (open, notify), presence_status());
        if let Some(screen) = self.online_screen.as_mut() {
            let facts = FriendFacts {
                enabled: social.list_enabled(),
                allow_requests: social.allow_requests(),
            };
            screen.set_friends(lang, facts);
        }
        let from_online = self.online_screen.is_some();
        let (mut toasts, turned_on) = collect_toasts(social);
        toasts.append(&mut self.pending_toasts);
        if turned_on {
            // From the online settings the list only turns on; the key opens the overlay.
            self.friends_wanted = !from_online;
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
        let (friends, pending) = match social.state() {
            State::Loading => (Listing::Loading, Listing::Loading),
            State::Success => (
                listing(friend_entries(social, data, lang)),
                listing(request_entries(data)),
            ),
            failed => {
                let message = lang.runs(&error_message(failed), &Default::default());
                (Listing::Error(message.clone()), Listing::Error(message))
            }
        };
        Some(FriendsView {
            friends,
            pending,
            requests: data.incoming.len(),
            profile_name: self.profile_name.clone(),
            ready: social.state() == State::Success,
            sending: social.sending(),
        })
    }

    /// `Minecraft.toggleFriendsScreen`: the overlay closes when it is open (unless its box is
    /// being typed in), and opens over the game when the friends list is on.
    pub(crate) fn toggle_friends(&mut self, surface: &mut WindowSurface, assets: &ScreenAssets) {
        if let Some(overlay) = self.friends.as_ref() {
            if !overlay.typing() {
                self.close_friends(surface, assets);
            }
            return;
        }
        if self.any_screen_open() {
            return;
        }
        if !self.friends_list.as_ref().is_some_and(Social::list_enabled) {
            self.ask_friends_list(surface, assets);
            self.captured = false;
            self.last_cursor = None;
            surface.set_cursor_captured(false);
            self.release_keys();
            return;
        }
        let size = Self::gui_size(surface);
        let Some(view) = self.friends_view(&assets.lang) else {
            return;
        };
        if let Some(social) = self.friends_list.as_mut() {
            social.want_presence();
        }
        self.friends_from_pause = false;
        self.friends = Some(FriendsOverlay::new(
            &assets.lang,
            &assets.creative.font,
            size,
            view,
        ));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// `confirmFriendsListEnabled` for a list that is off: the screen that asks to turn it on.
    pub(super) fn ask_friends_list(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        if self.friends_list.is_none() {
            return;
        }
        let link = Component::translatable("options.friendsList.confirm.message.link").with_style(
            &Style {
                underlined: Some(true),
                colour: Some(0x5555FF),
                ..Style::default()
            },
        );
        let title =
            Component::translatable("options.friendsList.confirm.title").with_style(&Style {
                underlined: Some(true),
                ..Style::default()
            });
        let message =
            Component::translatable_with("options.friendsList.confirm.message", vec![link]);
        let buttons = (
            &Component::translatable("options.friendsList.confirm.turnOn"),
            &Component::translatable("options.friendsList.confirm.turnOff"),
        );
        let screen = ConfirmScreen::friends(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            (&title, &message),
            buttons,
        )
        .with_background(ConfirmBackground::Menu {
            in_world: !self.title_host,
        });
        self.link_confirm = Some((screen, ConfirmTarget::FriendsList));
    }

    /// The answer of a confirmation of the friends: Turn ON asks the service, Remove removes the
    /// friend; Turn OFF goes back to the game (`setScreen(lastScreen)`) unless another screen
    /// is under it.
    pub(super) fn answer_friends_confirm(
        &mut self,
        action: LinkAction,
        target: &ConfirmTarget,
        surface: &mut WindowSurface,
    ) {
        match (action, target) {
            (LinkAction::Yes, ConfirmTarget::RemoveFriend(id)) => self.remove_friend(*id),
            (LinkAction::Yes, _) => self.turn_on_friends(),
            (_, ConfirmTarget::FriendsList) if self.online_screen.is_none() && !self.title_host => {
                self.captured = true;
                self.last_cursor = None;
                surface.set_cursor_captured(true);
            }
            _ => {}
        }
    }

    /// The answer Turn ON: the account's friends flags are changed (`applyFriendSettings`), and
    /// the overlay opens once the service said yes.
    pub(super) fn turn_on_friends(&mut self) {
        if let Some(social) = self.friends_list.as_ref() {
            social.update_settings(true, true);
        }
    }

    /// Whether the overlay should open now that the list was turned on.
    pub(crate) fn take_friends_wanted(&mut self) -> bool {
        std::mem::take(&mut self.friends_wanted)
    }

    /// The overlay closes: the pause menu it was opened from is what is left, otherwise the
    /// game takes the mouse again.
    fn close_friends(&mut self, surface: &mut WindowSurface, assets: &ScreenAssets) {
        self.friends = None;
        if std::mem::take(&mut self.friends_from_pause) {
            self.open_pause(surface, assets);
            return;
        }
        if self.title_host {
            return;
        }
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
            self.drag_friends((x, y), surface);
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
            WindowEvent::Scroll(_, y) => {
                let wheel = self.options.mouse_wheel_sensitivity;
                if let Some(screen) = self.friends.as_mut() {
                    screen.scrolled(mouse, y * wheel);
                }
                None
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                if let Some(screen) = self.friends.as_mut() {
                    screen.released();
                }
                None
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => self
                .friends
                .as_mut()
                .and_then(|screen| screen.clicked(mouse, font)),
            _ => None,
        };
        self.friends_asked(asked, surface, assets);
    }

    /// The cursor moved: the scroller follows it while it is held.
    fn drag_friends(&mut self, at: (f64, f64), surface: &WindowSurface) {
        let before = self.gui_mouse.1;
        self.move_gui_mouse(at, surface);
        let (now, dy) = (self.gui_mouse.1, self.gui_mouse.1 - before);
        if let Some(screen) = self.friends.as_mut() {
            screen.dragged(now, dy);
        }
    }

    /// What the overlay asked the game for.
    fn friends_asked(
        &mut self,
        asked: Option<FriendsEvent>,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let font = &assets.creative.font;
        match asked {
            Some(FriendsEvent::Close) => self.close_friends(surface, assets),
            Some(FriendsEvent::Send(name)) => self.send_request(&name, font),
            Some(FriendsEvent::CopyName) => {
                use photon_gui::widgets::edit_box::Clipboard;
                WindowClipboard(surface).set(&self.profile_name);
            }
            Some(FriendsEvent::OpenLink) => self.ask_account_link(surface, assets),
            Some(FriendsEvent::Remove(id)) => self.ask_remove_friend(id, surface, assets),
            Some(FriendsEvent::Accept(id)) => self.act_friend(Action::Accept(id)),
            Some(FriendsEvent::Decline(id)) => self.act_friend(Action::Decline(id)),
            Some(FriendsEvent::Revoke(id)) => self.act_friend(Action::Revoke(id)),
            None => {}
        }
    }

    /// `AddFriendWidget.sendFriendRequest`: a name that cannot be asked is told why in a toast.
    fn send_request(&mut self, name: &str, font: &photon_render::gui::font::Font) {
        let Some(social) = self.friends_list.as_mut() else {
            return;
        };
        if let Some(reason) = invalid_reason(name, &self.profile_name, social.lists()) {
            self.pending_toasts.push(reason);
            return;
        }
        social.act(Action::Send(name.to_owned()));
        if let Some(screen) = self.friends.as_mut() {
            screen.clear_box(font);
        }
    }

    /// `PrivacyConfirmLinkScreen.confirmLinkNow`: the account page is asked about, over the overlay.
    fn ask_account_link(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let url = super::online::PRIVACY_AND_ONLINE_SETTINGS;
        let screen = ConfirmScreen::link(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            url,
            true,
        )
        .with_background(ConfirmBackground::Menu { in_world: true });
        self.link_confirm = Some((screen, ConfirmTarget::Link(url.to_owned())));
    }

    fn act_friend(&mut self, action: Action) {
        if let Some(social) = self.friends_list.as_mut() {
            social.act(action);
        }
    }

    /// The answer Remove of the confirmation of `FriendEntry.confirmRemoveFriend`.
    pub(super) fn remove_friend(&mut self, id: photon_core::binary::Uuid) {
        if let Some(screen) = self.friends.as_mut() {
            screen.removing(id);
        }
        self.act_friend(Action::Remove(id));
    }

    /// `FriendEntry.confirmRemoveFriend`: the popup over the overlay.
    fn ask_remove_friend(
        &mut self,
        id: photon_core::binary::Uuid,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        let text = |key: &str| Component::translatable(key);
        let screen = ConfirmScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            &text("gui.friends.confirm_title"),
            &text("gui.friends.confirm_unfriend"),
            (&text("gui.remove"), &text("gui.cancel")),
        )
        .with_background(ConfirmBackground::Menu { in_world: true });
        self.link_confirm = Some((screen, ConfirmTarget::RemoveFriend(id)));
    }
}

/// `PresenceHandler` shares that the player is online while the overlay is open.
fn presence_status() -> Option<PresenceStatus> {
    Some(PresenceStatus::Online)
}

/// The toasts the answers of the service call for, and whether the list was turned on
/// (`PlayerSocialManager.showFailureToast`).
fn collect_toasts(social: &mut Social) -> (Vec<Component>, bool) {
    let mut toasts = Vec::new();
    let mut turned_on = false;
    while let Some(code) = social.take_settings_result() {
        turned_on |= code == ResultCode::Success;
        toasts.extend(failure_title(code));
    }
    while let Some(notice) = social.take_toast() {
        toasts.push(notice_title(&notice));
    }
    while let Some(code) = social.take_result() {
        toasts.extend(failure_title(code));
    }
    (toasts, turned_on)
}

/// The title of the toast a failed action shows; some codes show none.
fn failure_title(code: ResultCode) -> Option<Component> {
    let key = match code {
        ResultCode::TooManyRequests => "gui.friends.error.rateLimited.message",
        ResultCode::UnknownProfile => "gui.friends.error.user_may_lack_active_profile",
        ResultCode::Unauthorized => "gui.friends.error.unauthorized",
        ResultCode::Forbidden => "gui.friends.error.forbidden.message",
        ResultCode::ServiceNotAvailable => "gui.friends.error.unavailable.message",
        ResultCode::Error => "gui.friends.error.failed.message",
        ResultCode::Success
        | ResultCode::UpgradeNeeded
        | ResultCode::ConnectionIssue
        | ResultCode::TemporaryUnavailable
        | ResultCode::GenericError => return None,
    };
    Some(Component::translatable(key))
}

/// `AddFriendWidget.getInvalidInputReason`: why a request to `name` is not sent.
fn invalid_reason(name: &str, own: &str, data: &FriendData) -> Option<Component> {
    let has = |list: &[Friend]| {
        list.iter()
            .any(|friend| friend.name.eq_ignore_ascii_case(name))
    };
    let with_name =
        |key: &str| Component::translatable_with(key, vec![Component::literal(name.to_owned())]);
    if own.eq_ignore_ascii_case(name) {
        Some(Component::translatable(
            "gui.friends.validation.cannot_add_self",
        ))
    } else if has(&data.friends) {
        Some(with_name("gui.friends.validation.already_friend"))
    } else if has(&data.outgoing) {
        Some(with_name("gui.friends.validation.already_outgoing"))
    } else if has(&data.incoming) {
        Some(with_name("gui.friends.validation.already_incoming"))
    } else {
        None
    }
}

fn listing(entries: Vec<Entry>) -> Listing {
    if entries.is_empty() {
        Listing::Empty
    } else {
        Listing::Entries(entries)
    }
}

/// The friends with their presence (`FriendEntry.presenceStatusComponent`), the ones in a game
/// first (`presenceStatusSortOrder`).
fn friend_entries(social: &Social, data: &FriendData, lang: &Lang) -> Vec<Entry> {
    let mut entries: Vec<(u8, Entry)> = data
        .friends
        .iter()
        .map(|friend| {
            let status = social.presence_of(friend.id);
            (
                sort_order(status),
                entry(friend, EntryKind::Friend, presence_runs(status, lang)),
            )
        })
        .collect();
    entries.sort_by_key(|(order, _)| *order);
    entries.into_iter().map(|(_, entry)| entry).collect()
}

/// The received requests, then the sent ones.
fn request_entries(data: &FriendData) -> Vec<Entry> {
    let both = [
        (&data.incoming, EntryKind::Incoming),
        (&data.outgoing, EntryKind::Outgoing),
    ];
    both.into_iter()
        .flat_map(|(list, kind)| {
            list.iter()
                .map(move |friend| entry(friend, kind, Vec::new()))
        })
        .collect()
}

fn entry(friend: &Friend, kind: EntryKind, status: Vec<Run>) -> Entry {
    Entry {
        id: friend.id,
        name: friend.name.clone(),
        kind,
        status,
    }
}

fn sort_order(status: PresenceStatus) -> u8 {
    match status {
        PresenceStatus::PlayingHostedServer => 0,
        PresenceStatus::PlayingServer => 1,
        PresenceStatus::PlayingRealms => 2,
        PresenceStatus::PlayingOffline => 3,
        PresenceStatus::Online => 4,
        PresenceStatus::Offline => 5,
    }
}

/// The status line of a friend: green when online, grey when not.
fn presence_runs(status: PresenceStatus, lang: &Lang) -> Vec<Run> {
    let key = format!(
        "gui.friends.presence.status.{}",
        status.name().to_ascii_lowercase()
    );
    let colour = if status == PresenceStatus::Offline {
        0xA0A0A0
    } else {
        0x00FF00
    };
    let style = Style {
        colour: Some(colour),
        ..Style::default()
    };
    lang.runs(
        &Component::translatable(key).with_style(&style),
        &Style::default(),
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::binary::Uuid;

    fn friend(name: &str) -> Friend {
        Friend {
            id: Uuid::new(0, 1),
            name: name.to_owned(),
        }
    }

    #[test]
    fn a_request_is_refused_for_yourself_and_for_names_on_a_list() {
        let data = FriendData {
            friends: vec![friend("Alex")],
            incoming: vec![friend("Bob")],
            outgoing: vec![friend("Cleo")],
        };
        for name in ["steve", "ALEX", "bob", "Cleo"] {
            assert!(invalid_reason(name, "Steve", &data).is_some(), "{name}");
        }
        assert!(invalid_reason("Dana", "Steve", &data).is_none());
    }

    #[test]
    fn only_some_failures_show_a_toast() {
        assert!(failure_title(ResultCode::Success).is_none());
        assert!(failure_title(ResultCode::GenericError).is_none());
        for code in [
            ResultCode::Error,
            ResultCode::Forbidden,
            ResultCode::TooManyRequests,
            ResultCode::ServiceNotAvailable,
        ] {
            assert!(failure_title(code).is_some());
        }
    }
}
