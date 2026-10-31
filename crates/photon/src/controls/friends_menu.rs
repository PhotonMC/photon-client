//! What the friends do with the pause menu under them and with the presence they post.

use photon_game::options::PresenceSharing;
use photon_net::friends::PresenceStatus;

use super::Controls;

impl Controls {
    /// The list did not turn on: back to the menu the question came from.
    pub(super) fn menu_back_after_failure(&mut self, failed: bool) {
        if failed && self.friends_from_pause && self.link_confirm.is_none() {
            self.restore_pause();
        }
    }

    /// The pause menu the friends were opened from comes back.
    pub(super) fn restore_pause(&mut self) {
        self.friends_from_pause = false;
        self.pause = self.pause_under.take();
    }

    /// `PresenceHandler.getPublicPresenceStatus`: what the player's setting lets the friends see.
    /// Photon only plays on servers, never a world of its own or a realm.
    pub(super) fn presence_status(&self) -> PresenceStatus {
        match self.options.online.share_presence {
            PresenceSharing::None => PresenceStatus::Offline,
            PresenceSharing::Limited => PresenceStatus::Online,
            PresenceSharing::All if self.title_host => PresenceStatus::Online,
            PresenceSharing::All => PresenceStatus::PlayingServer,
        }
    }
}
