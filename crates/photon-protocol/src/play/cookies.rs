//! The play-phase packets about cookies, the transfer and the connection's own state.

use super::*;

/// `ClientboundStartConfigurationPacket`: the server moves the client back to configuration.
pub struct StartConfiguration;

impl Decode for StartConfiguration {
    fn decode(
        _payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        Ok(Self)
    }
}

/// A packet of `crate::cookie` that decodes itself from a payload.
macro_rules! cookie_packet {
    ($(#[$meta:meta])* $name:ident($ty:ty), $label:expr) => {
        $(#[$meta])*
        pub struct $name(pub $ty);

        impl Decode for $name {
            fn decode(
                payload: &[u8],
                _registries: &photon_data::version_registries::VersionRegistries,
            ) -> Result<Self, ConnectionError> {
                <$ty>::decode(&mut &payload[..])
                    .map(Self)
                    .map_err(|_| ConnectionError::Codec($label))
            }
        }
    };
}

cookie_packet!(
    /// `ClientboundCookieRequestPacket`.
    CookieRequest(crate::cookie::ClientboundCookieRequestPacket),
    "cookie request"
);
cookie_packet!(
    /// `ClientboundStoreCookiePacket`.
    StoreCookie(crate::cookie::ClientboundStoreCookiePacket),
    "store cookie"
);
cookie_packet!(
    /// `ClientboundCustomPayloadPacket`.
    CustomPayload(crate::cookie::ClientboundCustomPayloadPacket),
    "custom payload"
);
cookie_packet!(
    /// `ClientboundCustomReportDetailsPacket`.
    CustomReportDetails(crate::cookie::ClientboundCustomReportDetailsPacket),
    "custom report details"
);
cookie_packet!(
    /// `ClientboundTransferPacket`.
    Transfer(crate::cookie::ClientboundTransferPacket),
    "transfer"
);

/// `ClientboundServerLinksPacket`: the links as the server wrote them.
pub struct ServerLinksPacket(pub Vec<photon_core::server_links::UntrustedEntry>);

impl Decode for ServerLinksPacket {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        crate::server_links::parse_server_links(payload).map(Self)
    }
}

/// `ClientboundOpenSignEditorPacket`: the sign's position and which side is edited.
pub struct OpenSignEditor {
    pub pos: photon_core::binary::BlockPos,
    pub front: bool,
}

impl Decode for OpenSignEditor {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let pos = photon_core::binary::BlockPos::unpack(read_section_position(&mut input)? as u64);
        let front = read_bool(&mut input, "open sign editor")?;
        Ok(Self { pos, front })
    }
}

/// `ClientboundOpenBookPacket`: the hand the book is in.
pub struct OpenBook(pub i32);

impl Decode for OpenBook {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        read_varint(&mut input, "open book").map(Self)
    }
}
