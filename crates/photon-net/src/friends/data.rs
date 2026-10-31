//! What the friends service sends and takes: the list, the presence, the result codes and the
//! bodies of the requests.

use photon_core::binary::Uuid;
use photon_core::json::{self, JsonValue};

use crate::account::json_string;
use crate::account::parse_profile_id as parse_uuid;

/// `FriendDto`: a player on one of the lists.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Friend {
    pub id: Uuid,
    pub name: String,
}

/// `FriendData`: the friends and the requests to and from the player.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendData {
    pub friends: Vec<Friend>,
    pub incoming: Vec<Friend>,
    pub outgoing: Vec<Friend>,
}

/// `FriendsService.ResultCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCode {
    Success,
    Error,
    ServiceNotAvailable,
    TooManyRequests,
    Forbidden,
    UpgradeNeeded,
    ConnectionIssue,
    TemporaryUnavailable,
    UnknownProfile,
    Unauthorized,
    GenericError,
}

impl ResultCode {
    /// `YggdrasilFriendsService.handleHttpError`.
    pub fn for_status(status: u16) -> Self {
        match status {
            429 => Self::TooManyRequests,
            500.. => Self::ServiceNotAvailable,
            403 => Self::Forbidden,
            400 => Self::UnknownProfile,
            401 => Self::Unauthorized,
            _ => Self::Error,
        }
    }
}

/// `PresenceStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceStatus {
    Online,
    PlayingOffline,
    PlayingRealms,
    PlayingServer,
    PlayingHostedServer,
    Offline,
}

impl PresenceStatus {
    const NAMES: [(&'static str, Self); 6] = [
        ("ONLINE", Self::Online),
        ("PLAYING_OFFLINE", Self::PlayingOffline),
        ("PLAYING_REALMS", Self::PlayingRealms),
        ("PLAYING_SERVER", Self::PlayingServer),
        ("PLAYING_HOSTED_SERVER", Self::PlayingHostedServer),
        ("OFFLINE", Self::Offline),
    ];

    /// The constant's name, which is what the service reads and writes.
    pub fn name(self) -> &'static str {
        Self::NAMES
            .iter()
            .find(|(_, status)| *status == self)
            .map_or("OFFLINE", |(name, _)| name)
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::NAMES
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .map(|(_, status)| *status)
    }
}

/// `PresenceStatusDto`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresenceEntry {
    pub profile_id: Uuid,
    pub pmid: Option<Uuid>,
    pub status: PresenceStatus,
    /// The instant as the service wrote it.
    pub last_updated: Option<String>,
}

fn friend(value: &JsonValue) -> Option<Friend> {
    let JsonValue::Object(object) = value else {
        return None;
    };
    Some(Friend {
        id: parse_uuid(&object.text("profileId")?)?,
        name: object.text("name")?,
    })
}

/// One of the three lists of the answer: absent or not an array is the empty list, and an entry
/// that does not read is left out.
fn list(object: &json::JsonObject, key: &str) -> Vec<Friend> {
    object
        .array(key)
        .unwrap_or_default()
        .iter()
        .filter_map(friend)
        .collect()
}

/// `FriendsListResponse`: `None` when the body is not a JSON object.
pub fn parse_friends(body: &str) -> Option<FriendData> {
    let object = json::parse_object(body)?;
    Some(FriendData {
        friends: list(&object, "friends"),
        incoming: list(&object, "incomingRequests"),
        outgoing: list(&object, "outgoingRequests"),
    })
}

fn presence_entry(value: &JsonValue) -> Option<PresenceEntry> {
    let JsonValue::Object(object) = value else {
        return None;
    };
    Some(PresenceEntry {
        profile_id: parse_uuid(&object.text("profileId")?)?,
        pmid: object.text("pmid").and_then(|text| parse_uuid(&text)),
        status: PresenceStatus::from_name(&object.text("status")?)?,
        last_updated: object.text("lastUpdated"),
    })
}

/// `PresenceResponse`: `None` when the body is not a JSON object.
pub fn parse_presence(body: &str) -> Option<Vec<PresenceEntry>> {
    let object = json::parse_object(body)?;
    Some(
        object
            .array("presence")
            .unwrap_or_default()
            .iter()
            .filter_map(presence_entry)
            .collect(),
    )
}

/// What a friend action names.
pub(super) enum Target<'a> {
    Name(&'a str),
    Id(Uuid),
}

/// `FriendActionRequest`: the target, then `ADD` or `REMOVE`.
pub(super) fn action_body(target: &Target<'_>, add: bool) -> String {
    let target = match target {
        Target::Name(name) => format!("\"name\":{}", json_string(name)),
        Target::Id(id) => format!("\"profileId\":\"{}\"", id.to_string_java()),
    };
    let update = if add { "ADD" } else { "REMOVE" };
    format!("{{{target},\"updateType\":\"{update}\"}}")
}

/// `UserAttributesRequest` with the friends preferences only.
pub(super) fn settings_body(friends_list: bool, invites: bool) -> String {
    let toggle = |on: bool| if on { "ENABLED" } else { "DISABLED" };
    format!(
        "{{\"friendsPreferences\":{{\"friends\":\"{}\",\"acceptInvites\":\"{}\"}}}}",
        toggle(friends_list),
        toggle(invites)
    )
}

/// `PresenceRequest`.
pub(super) fn presence_body(status: PresenceStatus) -> String {
    format!("{{\"status\":\"{}\"}}", status.name())
}

/// What the account says about the friends list (`UserFlag.FRIENDS_ENABLED` and
/// `ACCEPT_FRIEND_INVITES`): both off when the service sent no friends preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preferences {
    pub friends_list: bool,
    pub accept_invites: bool,
}

/// `YggdrasilUserApiService.fetchProperties`, the friends part: `None` when the body is not a
/// JSON object.
pub fn parse_preferences(body: &str) -> Option<Preferences> {
    let object = json::parse_object(body)?;
    let on = |preferences: &json::JsonObject, key: &str| {
        preferences
            .text(key)
            .is_some_and(|value| value == "ENABLED")
    };
    Some(
        object
            .object("friendsPreferences")
            .map_or_else(Preferences::default, |preferences| Preferences {
                friends_list: on(preferences, "friends"),
                accept_invites: on(preferences, "acceptInvites"),
            }),
    )
}
