//! 26.1 (protocol 775) against 26.2.

use photon_core::binary::{ReadError, Uuid};

use crate::login::ClientboundLoginFinishedPacket;
use crate::play::{ConnectionError, LoginObservation};

/// 26.1's login-finished packet is the profile alone; 26.2 added the session id after it. The
/// client reads no session id from login, so it is left nil.
pub fn login_finished(input: &mut &[u8]) -> Result<ClientboundLoginFinishedPacket, ReadError> {
    let profile = ClientboundLoginFinishedPacket::decode_profile(input)?;
    Ok(ClientboundLoginFinishedPacket::new(
        profile,
        Uuid::new(0, 0),
    ))
}

/// 26.1's play login packet has no online-mode flag between the spawn info and the secure-chat
/// flag; 26.2 added it. The client treats the server as offline then (what it decides with the
/// flag is unchecked against 26.1's own client).
pub fn login_packet(input: &[u8]) -> Result<LoginObservation, ConnectionError> {
    crate::play::parse_login_fields(input, false)
}
