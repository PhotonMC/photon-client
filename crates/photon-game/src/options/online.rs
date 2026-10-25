//! The online options that the online screen holds and `options.txt` keeps.

use super::json::{Json, write_string};

/// `PresenceSharing`: how much of the player's presence the friends see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceSharing {
    None,
    Limited,
    All,
}

impl PresenceSharing {
    pub const ALL: [Self; 3] = [Self::None, Self::Limited, Self::All];

    pub const fn serialized(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Limited => "limited",
            Self::All => "all",
        }
    }

    pub fn from_serialized(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.serialized() == name)
    }

    /// The caption's translation key, which is also the cycle button's value.
    pub const fn caption_key(self) -> &'static str {
        match self {
            Self::None => "options.sharePresence.none",
            Self::Limited => "options.sharePresence.limited",
            Self::All => "options.sharePresence.all",
        }
    }

    /// The tooltip's translation key.
    pub const fn tooltip_key(self) -> &'static str {
        match self {
            Self::None => "options.sharePresence.none.tooltip",
            Self::Limited => "options.sharePresence.limited.tooltip",
            Self::All => "options.sharePresence.all.tooltip",
        }
    }

    pub fn from_caption_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.caption_key() == key)
    }
}

/// An option of the group, by its name in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OnlineKey {
    RealmsNotifications,
    InGameNotification,
    SharePresence,
    AllowServerListing,
}

/// `realmsNotifications`, `inGameNotification` and `sharePresence`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnlineOptions {
    pub realms_notifications: bool,
    pub in_game_notification: bool,
    pub share_presence: PresenceSharing,
    /// `allowServerListing`: the server may list the player; the server is told.
    pub allow_server_listing: bool,
}

impl Default for OnlineOptions {
    fn default() -> Self {
        Self {
            realms_notifications: true,
            in_game_notification: false,
            share_presence: PresenceSharing::All,
            allow_server_listing: true,
        }
    }
}

impl OnlineOptions {
    /// A value the file holds; one of the wrong kind or an unknown name leaves the option.
    pub(super) fn read(&mut self, key: OnlineKey, json: &Json) {
        match (key, json) {
            (OnlineKey::RealmsNotifications, Json::Bool(value)) => {
                self.realms_notifications = *value;
            }
            (OnlineKey::InGameNotification, Json::Bool(value)) => {
                self.in_game_notification = *value;
            }
            (OnlineKey::AllowServerListing, Json::Bool(value)) => {
                self.allow_server_listing = *value;
            }
            (OnlineKey::SharePresence, Json::Text(name)) => {
                self.share_presence =
                    PresenceSharing::from_serialized(name).unwrap_or(self.share_presence);
            }
            _ => {}
        }
    }

    pub(super) fn write(&self, key: OnlineKey) -> String {
        match key {
            OnlineKey::RealmsNotifications => self.realms_notifications.to_string(),
            OnlineKey::InGameNotification => self.in_game_notification.to_string(),
            OnlineKey::AllowServerListing => self.allow_server_listing.to_string(),
            OnlineKey::SharePresence => write_string(self.share_presence.serialized()),
        }
    }
}
