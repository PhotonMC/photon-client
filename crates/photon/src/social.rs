//! `PlayerSocialManager` and its presence handler, for the friends overlay: the friend lists the
//! service gave, the account's friends flags, and the thread that talks to the service. The game
//! thread sends commands and reads what came back once a tick.

mod worker;

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use photon_net::friends::{
    FriendData, FriendListUpdates, FriendsService, Notice, Preferences, PresenceEntry,
    PresenceStatus, ResultCode, State, Transport,
};

pub(crate) use worker::{Action, Command, Event};

/// `PresenceHandler.PRESENCE_UPDATE_INTERVAL`, and how many times it is stretched at most.
const PRESENCE_INTERVAL: Duration = Duration::from_secs(60);
const MAX_PRESENCE_MULTIPLIER: u32 = 5;

pub(crate) struct Social {
    commands: Sender<Command>,
    events: Receiver<Event>,
    updates: FriendListUpdates,
    /// `Minecraft.friendsEnabled()` and `allowFriendRequests()`, once the account has answered.
    preferences: Option<Preferences>,
    list_enabled: bool,
    allow_requests: bool,
    refreshing: bool,
    friends_hint: Option<Duration>,
    presence_hint: Option<Duration>,
    presence: Vec<PresenceEntry>,
    last_presence: Instant,
    /// `tryUpdatePresence`: the overlay asks for a fresh presence.
    update_presence: bool,
    toasts: VecDeque<Notice>,
    /// The lists or the presence changed since a screen last looked.
    changed: bool,
    /// What the last call that changed the account's settings or lists said.
    results: VecDeque<ResultCode>,
    /// Actions sent whose answer has not come yet.
    in_flight: u32,
    /// What `updateFriendSettings` answered, oldest first.
    settings_results: VecDeque<ResultCode>,
}

impl Social {
    /// The state of an account whose service is reached through `transport`; the thread ends when
    /// this is dropped.
    pub(crate) fn start<T: Transport + Send + 'static>(transport: T, now: Instant) -> Self {
        let (commands, command_rx) = channel();
        let (event_tx, events) = channel();
        let service = FriendsService::new(transport);
        let spawned = std::thread::Builder::new()
            .name("Friends List".to_owned())
            .spawn(move || worker::run(service, &command_rx, &event_tx));
        let social = Self::with_channels(commands, events, now);
        if spawned.is_ok() {
            social.send(Command::Preferences);
        }
        social
    }

    fn with_channels(commands: Sender<Command>, events: Receiver<Event>, now: Instant) -> Self {
        Self {
            commands,
            events,
            updates: FriendListUpdates::new(),
            preferences: None,
            list_enabled: false,
            allow_requests: false,
            refreshing: false,
            friends_hint: None,
            presence_hint: None,
            presence: Vec::new(),
            last_presence: now,
            update_presence: true,
            toasts: VecDeque::new(),
            changed: false,
            results: VecDeque::new(),
            in_flight: 0,
            settings_results: VecDeque::new(),
        }
    }

    fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    /// `isFriendListEnabled`.
    pub(crate) fn list_enabled(&self) -> bool {
        self.list_enabled
    }

    /// `getFriendListState`.
    pub(crate) fn state(&self) -> State {
        self.updates.state()
    }

    /// `getFriends`, `getIncomingRequests` and `getOutgoingRequests`.
    pub(crate) fn lists(&self) -> &FriendData {
        self.updates.latest()
    }

    /// `tryUpdatePresence`.
    pub(crate) fn want_presence(&mut self) {
        self.update_presence = true;
    }

    /// Whether the lists or the presence changed since this was last asked.
    pub(crate) fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// The toasts the changes of the lists call for, oldest first.
    pub(crate) fn take_toast(&mut self) -> Option<Notice> {
        self.toasts.pop_front()
    }

    /// The code of a finished action or settings call, oldest first.
    pub(crate) fn take_result(&mut self) -> Option<ResultCode> {
        self.results.pop_front()
    }

    /// `updateFriendSettings`.
    pub(crate) fn update_settings(&self, friends_list: bool, invites: bool) {
        self.send(Command::Settings(friends_list, invites));
    }

    /// The code of a finished settings call, oldest first.
    pub(crate) fn take_settings_result(&mut self) -> Option<ResultCode> {
        self.settings_results.pop_front()
    }

    /// An action is on its way (`AddFriendWidget.State.SENDING`).
    pub(crate) fn sending(&self) -> bool {
        self.in_flight > 0
    }

    /// `PlayerSocialManager`'s actions.
    pub(crate) fn act(&mut self, action: Action) {
        self.in_flight += 1;
        self.send(Command::Action(action));
    }

    /// One game tick: what the thread answered, the next update of the lists when it is due and
    /// the next presence post. `overlay_open` is `screen instanceof FriendsOverlayScreen`;
    /// `toasts` is false in a world with in-game notifications off; `status` is the presence to
    /// share, or none to post nothing.
    pub(crate) fn tick(
        &mut self,
        now: Instant,
        (overlay_open, toasts): (bool, bool),
        status: Option<PresenceStatus>,
    ) {
        while let Ok(event) = self.events.try_recv() {
            self.receive(event, now, toasts);
        }
        if !self.list_enabled {
            return;
        }
        if !self.refreshing && self.updates.is_due(now, self.friends_hint, overlay_open) {
            self.refreshing = true;
            self.send(Command::Refresh);
        }
        if let Some(status) = status
            && overlay_open
            && self.presence_due(now)
        {
            self.update_presence = false;
            self.last_presence = now;
            self.send(Command::Presence(status));
        }
    }

    /// `PresenceHandler.shouldRefreshPresence`: only while there are friends to show.
    fn presence_due(&self, now: Instant) -> bool {
        if self.updates.latest().friends.is_empty() {
            return false;
        }
        let since = now.duration_since(self.last_presence);
        let interval = self.presence_hint.unwrap_or(PRESENCE_INTERVAL);
        (self.update_presence && since >= interval) || since >= interval * MAX_PRESENCE_MULTIPLIER
    }

    fn receive(&mut self, event: Event, now: Instant, toasts: bool) {
        match event {
            Event::Preferences(preferences) => {
                self.preferences = Some(preferences);
                self.list_enabled = preferences.friends_list;
                self.allow_requests = preferences.accept_invites;
            }
            Event::List { code, data, hint } => {
                self.refreshing = false;
                self.friends_hint = hint;
                let applied = self.updates.apply((code, data), now, toasts);
                self.toasts.extend(applied.notices);
                self.changed |= applied.notify;
            }
            Event::Action(code) => {
                self.in_flight = self.in_flight.saturating_sub(1);
                self.results.push_back(code);
            }
            Event::Settings {
                code,
                friends_list,
                invites,
            } => {
                if code == ResultCode::Success {
                    self.list_enabled = friends_list;
                    self.allow_requests = invites;
                }
                self.settings_results.push_back(code);
            }
            Event::Presence { entries, hint } => {
                self.presence_hint = hint;
                self.changed |= self.presence != entries;
                self.presence = entries;
            }
        }
    }
}

#[cfg(test)]
mod tests;
