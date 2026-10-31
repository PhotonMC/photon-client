//! `YggdrasilFriendsService`: the calls, the caches that answer an unchanged list, the cooldown
//! between list requests and the poll intervals the service asks for.

use std::time::{Duration, Instant};

use photon_core::binary::Uuid;
use photon_core::json;

use super::data::{
    FriendData, PresenceEntry, PresenceStatus, ResultCode, Target, action_body, parse_friends,
    parse_presence, presence_body, settings_body,
};
use super::transport::{Method, Reply, Request, Transport, TransportError};

/// Between two list requests: the cache answers in between.
const REQUEST_COOLDOWN: Duration = Duration::from_secs(10);

/// Why a call produced no usable body.
enum Failure {
    /// The service answered with an error status.
    Http(u16),
    /// The service could not be reached.
    Unreachable,
    /// The answer was not what the call expects.
    Unreadable,
}

pub struct FriendsService<T: Transport> {
    transport: T,
    friends_etag: Option<String>,
    friends_cache: FriendData,
    presence_etag: Option<String>,
    presence_cache: Vec<PresenceEntry>,
    friends_poll: Option<Duration>,
    presence_poll: Option<Duration>,
    cooldown_until: Option<Instant>,
}

impl<T: Transport> FriendsService<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            friends_etag: None,
            friends_cache: FriendData::default(),
            presence_etag: None,
            presence_cache: Vec::new(),
            friends_poll: None,
            presence_poll: None,
            cooldown_until: None,
        }
    }

    /// `getFriendsPollInterval`.
    pub fn friends_poll_interval(&self) -> Option<Duration> {
        self.friends_poll
    }

    /// `getPresencePollInterval`.
    pub fn presence_poll_interval(&self) -> Option<Duration> {
        self.presence_poll
    }

    fn send(&self, request: &Request) -> Result<Reply, Failure> {
        let reply = self
            .transport
            .send(request)
            .map_err(|TransportError| Failure::Unreachable)?;
        match reply.status {
            400.. => Err(Failure::Http(reply.status)),
            _ => Ok(reply),
        }
    }

    /// `getFriendData`: the lists, from the cache while the cooldown runs, and the code.
    pub fn friend_data(&mut self, now: Instant) -> (ResultCode, FriendData) {
        if self.cooldown_until.is_some_and(|until| now <= until) {
            return (ResultCode::Success, self.friends_cache.clone());
        }
        self.cooldown_until = Some(now + REQUEST_COOLDOWN);
        let code = match self.request_friend_data() {
            Ok(()) => ResultCode::Success,
            Err(Failure::Http(status)) => ResultCode::for_status(status),
            Err(_) => ResultCode::Error,
        };
        (code, self.friends_cache.clone())
    }

    fn request_friend_data(&mut self) -> Result<(), Failure> {
        let reply = self.send(&Request {
            method: Method::Get,
            path: "/friends",
            body: None,
            etag: self.friends_etag.clone(),
        })?;
        if reply.retry_after.is_some() {
            self.friends_poll = reply.retry_after;
        }
        // A 304 or an empty body leaves the cache as it is.
        if reply.status != 304 && !reply.body.is_empty() {
            self.friends_cache = parse_friends(&reply.body).ok_or(Failure::Unreadable)?;
            self.friends_etag = reply.etag;
        }
        Ok(())
    }

    /// `removeFriend`.
    pub fn remove_friend(&mut self, id: Uuid, now: Instant) -> ResultCode {
        self.friend_action(&Target::Id(id), false, now)
    }

    /// `acceptIncomingFriendRequest`.
    pub fn accept_incoming(&mut self, id: Uuid, now: Instant) -> ResultCode {
        self.friend_action(&Target::Id(id), true, now)
    }

    /// `declineIncomingFriendRequest`.
    pub fn decline_incoming(&mut self, id: Uuid, now: Instant) -> ResultCode {
        self.friend_action(&Target::Id(id), false, now)
    }

    /// `sendFriendRequest(String)`.
    pub fn send_request(&mut self, name: &str, now: Instant) -> ResultCode {
        self.friend_action(&Target::Name(name), true, now)
    }

    /// `sendFriendRequest(UUID)`.
    pub fn send_request_to(&mut self, id: Uuid, now: Instant) -> ResultCode {
        self.friend_action(&Target::Id(id), true, now)
    }

    /// `revokeOutgoingFriendRequest`.
    pub fn revoke_outgoing(&mut self, id: Uuid, now: Instant) -> ResultCode {
        self.friend_action(&Target::Id(id), false, now)
    }

    fn friend_action(&mut self, target: &Target<'_>, add: bool, now: Instant) -> ResultCode {
        let request = Request {
            method: Method::Put,
            path: "/friends",
            body: Some(action_body(target, add)),
            etag: None,
        };
        match self.send(&request).and_then(|reply| reply_friends(&reply)) {
            Ok(Some(data)) => {
                self.cooldown_until = Some(now + REQUEST_COOLDOWN);
                self.friends_etag = None;
                self.friends_cache = data;
                ResultCode::Success
            }
            Ok(None) => ResultCode::Success,
            Err(failure) => action_failure(&failure),
        }
    }

    /// `updateFriendSettings`: the friends list and the friend invites on or off.
    pub fn update_settings(&mut self, friends_list: bool, invites: bool) -> ResultCode {
        let request = Request {
            method: Method::Post,
            path: "/player/attributes",
            body: Some(settings_body(friends_list, invites)),
            etag: None,
        };
        match self.send(&request).and_then(|reply| readable(&reply.body)) {
            Ok(()) => ResultCode::Success,
            Err(failure) => action_failure(&failure),
        }
    }

    /// `presence`: the friends' presence after posting the player's own, the cached one when the
    /// service says nothing new or cannot be reached.
    pub fn presence(&mut self, status: PresenceStatus) -> Vec<PresenceEntry> {
        let request = Request {
            method: Method::Post,
            path: "/presence",
            body: Some(presence_body(status)),
            etag: self.presence_etag.clone(),
        };
        let Ok(reply) = self.send(&request) else {
            return self.presence_cache.clone();
        };
        if reply.retry_after.is_some() {
            self.presence_poll = reply.retry_after;
        }
        if reply.status != 304 && !reply.body.is_empty() {
            let Some(entries) = parse_presence(&reply.body) else {
                return self.presence_cache.clone();
            };
            self.presence_cache = entries;
        }
        // A 304 keeps the etag that was sent; any other answer brings its own, or none.
        if reply.status != 304 {
            self.presence_etag = reply.etag;
        }
        self.presence_cache.clone()
    }
}

/// The lists of a friend action's answer: `None` for an empty answer.
fn reply_friends(reply: &Reply) -> Result<Option<FriendData>, Failure> {
    if reply.body.is_empty() {
        return Ok(None);
    }
    parse_friends(&reply.body)
        .map(Some)
        .ok_or(Failure::Unreadable)
}

/// A body that is empty or JSON.
fn readable(body: &str) -> Result<(), Failure> {
    if body.is_empty() || json::parse(body).is_ok() {
        Ok(())
    } else {
        Err(Failure::Unreadable)
    }
}

/// An action that could not reach the service is `SERVICE_NOT_AVAILABLE`, one that could not
/// read its answer is `ERROR`.
fn action_failure(failure: &Failure) -> ResultCode {
    match failure {
        Failure::Http(status) => ResultCode::for_status(*status),
        Failure::Unreachable => ResultCode::ServiceNotAvailable,
        Failure::Unreadable => ResultCode::Error,
    }
}
