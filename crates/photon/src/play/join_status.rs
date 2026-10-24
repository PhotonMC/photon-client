//! What a join shows of how far it has come.

use photon_net::connection::{LoginStage, stage};
use std::sync::atomic::{AtomicBool, Ordering};

/// How far a join has come: the login stage, then in the level waiting for it to load.
#[derive(Default)]
pub(crate) struct Status {
    loading: AtomicBool,
    pub(super) stage: LoginStage,
}

impl Status {
    pub(crate) fn loading(&self) -> bool {
        self.loading.load(Ordering::Relaxed)
    }

    pub(super) fn set_loading(&self) {
        self.loading.store(true, Ordering::Relaxed);
    }

    /// The text the connect screen shows for the login stage
    /// (`ClientHandshakePacketListenerImpl.State`).
    pub(crate) fn message_key(&self) -> &'static str {
        match self.stage.0.load(Ordering::Relaxed) {
            stage::AUTHORIZING => "connect.authorizing",
            stage::ENCRYPTING => "connect.encrypting",
            stage::JOINING => "connect.joining",
            _ => "connect.connecting",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_login_stage_has_its_message() {
        let status = Status::default();
        assert_eq!(status.message_key(), "connect.connecting");
        for (reached, key) in [
            (stage::AUTHORIZING, "connect.authorizing"),
            (stage::ENCRYPTING, "connect.encrypting"),
            (stage::JOINING, "connect.joining"),
        ] {
            status.stage.0.store(reached, Ordering::Relaxed);
            assert_eq!(status.message_key(), key);
        }
    }
}
