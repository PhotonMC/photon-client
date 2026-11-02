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
    /// The address and the host name the handshake announces for it (none: the address's own).
    pub(super) fn resolve(&self) -> Option<(SocketAddr, Option<String>)> {
        match self {
            Self::Address(address) => Some((*address, None)),
            Self::Name(ip) => {
                crate::menu::resolve_named(ip).map(|(address, host)| (address, Some(host)))
            }
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
