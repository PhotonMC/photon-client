//! The friends service of the connection's account.

use super::PlaySession;
use crate::friends::HttpTransport;
use crate::signin::Hosts;

impl PlaySession {
    /// How to reach the friends service as this connection's account, if it has one.
    pub fn friends_transport(&self) -> Option<HttpTransport> {
        let account = self.state.account.as_ref()?;
        Some(HttpTransport::for_account(
            account,
            &Hosts::from_environment().services,
        ))
    }
}
