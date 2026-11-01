//! Where a join goes.

use std::net::SocketAddr;

use photon_game::text::Component;

/// An address already known, or a name the join thread looks up (`ConnectScreen.connect`
/// resolves it off the render thread).
pub(crate) enum Target {
    Address(SocketAddr),
    Name(String),
}

impl Target {
    pub(super) fn resolve(&self) -> Option<SocketAddr> {
        match self {
            Self::Address(address) => Some(*address),
            Self::Name(ip) => crate::menu::resolve(ip),
        }
    }
}

/// `ConnectScreen.UNKNOWN_HOST_MESSAGE`.
pub(super) fn unknown_host() -> Component {
    Component::translatable_with(
        "disconnect.genericReason",
        vec![Component::translatable("disconnect.unknownHost")],
    )
}
