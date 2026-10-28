//! How a clientbound play packet parses. Parsing is pure and happens on the reader thread; what a
//! packet does to the client's state is the listener's (`connection::play`).

use super::*;

/// What a packet that does not parse leaves behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnError {
    /// The connection fails with the error.
    Fail,
    /// The packet is dropped, as the reference drops what it cannot place; the reason is logged.
    Ignore(&'static str),
    /// The packet is dropped and the session warns, with this label and the reason.
    Warn(&'static str),
}

/// A packet that parses itself from its payload (the bytes after the packet id).
pub trait Decode: Sized {
    const ON_ERROR: OnError = OnError::Fail;

    fn decode(
        payload: &[u8],
        registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError>;
}

/// A packet type that holds what a parse function returns.
macro_rules! wrapped_packet {
    ($(#[$meta:meta])* $name:ident($ty:ty) = $parse:path $(, $on_error:expr)?) => {
        $(#[$meta])*
        pub struct $name(pub $ty);

        impl Decode for $name {
            $(const ON_ERROR: OnError = $on_error;)?

            fn decode(
                payload: &[u8],
                _registries: &photon_data::version_registries::VersionRegistries,
            ) -> Result<Self, ConnectionError> {
                $parse(payload).map(Self)
            }
        }
    };
}
pub(crate) use wrapped_packet;

/// A packet type that holds what a parse function returns, which needs the session's registries
/// to turn the ids it reads into canonical ones.
macro_rules! wrapped_wire_packet {
    ($(#[$meta:meta])* $name:ident($ty:ty) = $parse:path $(, $on_error:expr)?) => {
        $(#[$meta])*
        pub struct $name(pub $ty);

        impl Decode for $name {
            $(const ON_ERROR: OnError = $on_error;)?

            fn decode(
                payload: &[u8],
                registries: &photon_data::version_registries::VersionRegistries,
            ) -> Result<Self, ConnectionError> {
                $parse(payload, registries).map(Self)
            }
        }
    };
}
pub(crate) use wrapped_wire_packet;
