//! `RemoteFriendListUpdateHandler`'s decisions: which state an answer puts the list in, when the
//! next update is due, and which toasts a change of the lists calls for. The thread that runs the
//! updates is the caller's.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use super::data::{Friend, FriendData, ResultCode};

/// How long a list stays current when the service gave no hint (a minute).
const FOREGROUND_INTERVAL: Duration = Duration::from_secs(60);
/// A list that is not in front of the player is updated this many times less often.
const BACKGROUND_MULTIPLIER: u32 = 5;

/// `RemoteFriendListUpdateHandler.State`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Loading,
    UpgradeNeeded,
    ConnectionIssue,
    UserMayLackActiveProfile,
    Unauthorized,
    TemporaryUnavailable,
    GenericError,
    Success,
}

impl State {
    /// `mapResultCodeToState`.
    pub fn for_code(code: ResultCode) -> Self {
        match code {
            ResultCode::TemporaryUnavailable
            | ResultCode::Forbidden
            | ResultCode::ServiceNotAvailable
            | ResultCode::TooManyRequests => Self::TemporaryUnavailable,
            ResultCode::ConnectionIssue => Self::ConnectionIssue,
            ResultCode::UpgradeNeeded => Self::UpgradeNeeded,
            ResultCode::UnknownProfile => Self::UserMayLackActiveProfile,
            ResultCode::Unauthorized => Self::Unauthorized,
            ResultCode::GenericError | ResultCode::Error => Self::GenericError,
            ResultCode::Success => Self::Success,
        }
    }
}

/// The toast a change of the lists shows (`FriendToast`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    FriendAdded(Friend),
    RequestAccepted(Friend),
    RequestReceived(Friend),
    RequestSent(Friend),
}

/// What an applied answer asks of the caller.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Applied {
    pub notices: Vec<Notice>,
    /// The screens that show the lists have to refresh.
    pub notify: bool,
}

pub struct FriendListUpdates {
    state: State,
    latest: FriendData,
    known: FriendData,
    last_update: Option<Instant>,
}

impl Default for FriendListUpdates {
    fn default() -> Self {
        Self::new()
    }
}

impl FriendListUpdates {
    pub fn new() -> Self {
        Self {
            state: State::Loading,
            latest: FriendData::default(),
            known: FriendData::default(),
            last_update: None,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// `getLatestFriendData`.
    pub fn latest(&self) -> &FriendData {
        &self.latest
    }

    /// `runBackgroundTick`'s test: the first update at once, then one every interval, which is
    /// the service's hint (or a minute), and five times that while the overlay is not shown.
    pub fn is_due(&self, now: Instant, hint: Option<Duration>, overlay_open: bool) -> bool {
        let interval = hint.unwrap_or(FOREGROUND_INTERVAL);
        let interval = if overlay_open {
            interval
        } else {
            interval * BACKGROUND_MULTIPLIER
        };
        self.last_update
            .is_none_or(|last| now.duration_since(last) >= interval)
    }

    /// `runUpdateFriendDataInternal` after the call: the state, the lists when it worked and the
    /// toasts of what changed. `toasts` is false in a world with in-game notifications off.
    pub fn apply(
        &mut self,
        (code, data): (ResultCode, FriendData),
        now: Instant,
        toasts: bool,
    ) -> Applied {
        let previous = self.state;
        self.state = State::for_code(code);
        self.last_update = Some(now);
        if code != ResultCode::Success {
            return Applied {
                notices: Vec::new(),
                notify: true,
            };
        }
        let (notices, changed) = self.detect_changes(&data, previous, toasts);
        self.latest = data;
        Applied {
            notices,
            notify: changed || previous != self.state,
        }
    }

    /// `detectChangesAndShowToast`.
    fn detect_changes(
        &mut self,
        data: &FriendData,
        previous: State,
        toasts: bool,
    ) -> (Vec<Notice>, bool) {
        let known = std::mem::replace(&mut self.known, data.clone());
        if previous != State::Success {
            return (Vec::new(), true);
        }
        let mut notices = Vec::new();
        if toasts {
            notices = notices_for(&known, data);
        }
        let changed = as_set(&known.friends) != as_set(&data.friends)
            || as_set(&known.incoming) != as_set(&data.incoming)
            || as_set(&known.outgoing) != as_set(&data.outgoing);
        (notices, changed)
    }
}

fn as_set(list: &[Friend]) -> HashSet<&Friend> {
    list.iter().collect()
}

fn notices_for(known: &FriendData, now: &FriendData) -> Vec<Notice> {
    let mut notices: Vec<Notice> = now
        .friends
        .iter()
        .filter(|friend| !known.friends.contains(friend))
        .map(|friend| {
            if known.outgoing.contains(friend) || known.incoming.contains(friend) {
                Notice::RequestAccepted(friend.clone())
            } else {
                Notice::FriendAdded(friend.clone())
            }
        })
        .collect();
    notices.extend(
        new_requests(&known.incoming, &now.incoming, &now.friends)
            .map(|friend| Notice::RequestReceived(friend.clone())),
    );
    notices.extend(
        new_requests(&known.outgoing, &now.outgoing, &now.friends)
            .map(|friend| Notice::RequestSent(friend.clone())),
    );
    notices
}

/// The requests that were not there before and did not become friendships.
fn new_requests<'a>(
    known: &'a [Friend],
    now: &'a [Friend],
    friends: &'a [Friend],
) -> impl Iterator<Item = &'a Friend> {
    now.iter()
        .filter(move |friend| !known.contains(friend) && !friends.contains(friend))
}
