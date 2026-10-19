use std::collections::HashMap;
use std::sync::OnceLock;

use super::*;

mod blocks;
mod chat;
mod connection;
mod containers;
mod emitters;
mod entity;
mod player;
mod recipes;
mod session;
mod world;

/// A clientbound play packet, parsed (the reference's `Packet`): what it does to the client is
/// its `handle`, the listener's `handleX` of the reference.
pub(super) trait ClientboundPacket: Send {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError>;

    /// Whether this is a bundle's delimiter, which `Connection` joins into one packet.
    fn is_bundle_delimiter(&self) -> bool {
        false
    }

    /// Whether loading may be over once the packet is handled.
    fn checks_loading(&self) -> bool {
        true
    }
}

/// A packet as the reader thread hands it over; the order the packets came in is kept, so an
/// error surfaces where it always did.
pub(super) type Incoming = Box<dyn ClientboundPacket>;

/// Parses the packet with this id from its payload.
type Decoder = fn(i32, &[u8]) -> Result<Incoming, ConnectionError>;

/// The packets of the families that are typed, by packet id.
fn decoders() -> &'static HashMap<i32, Decoder> {
    static DECODERS: OnceLock<HashMap<i32, Decoder>> = OnceLock::new();
    DECODERS.get_or_init(|| {
        [
            world::DECODERS,
            connection::DECODERS,
            chat::DECODERS,
            recipes::DECODERS,
            player::DECODERS,
            containers::DECODERS,
            blocks::DECODERS,
            entity::DECODERS,
            session::DECODERS,
            &[(PLAY_CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID, |_, _| {
                Ok(Box::new(BundleDelimiter))
            })],
        ]
        .concat()
        .into_iter()
        .collect()
    })
}

/// Parse the packet in `body` (its id, then its payload). A packet the client has no use for is
/// dropped, as the reference drops one it does not know.
pub(super) fn decode_packet(body: &[u8]) -> Result<Incoming, ConnectionError> {
    let (packet_id, payload) = split_packet(body)?;
    match decoders().get(&packet_id) {
        Some(decode) => decode(packet_id, payload),
        None => Ok(Box::new(Ignored)),
    }
}

/// The decoder of a packet type, as its table row holds it.
fn typed<P>(_packet_id: i32, payload: &[u8]) -> Result<Incoming, ConnectionError>
where
    P: Decode + ClientboundPacket + 'static,
{
    match P::decode(payload) {
        Ok(packet) => Ok(Box::new(packet)),
        Err(error) => match P::ON_ERROR {
            OnError::Fail => Err(error),
            OnError::Ignore(name) => {
                log::debug!("ignored {name} packet: {error}");
                Ok(Box::new(Ignored))
            }
            OnError::Warn(label) => Ok(Box::new(Malformed(format!("{label}: {error:?}")))),
        },
    }
}

/// A packet that did not parse and ends the connection, with its error, when its turn comes: what
/// went before it has been applied, as it always was.
struct Failed(ConnectionError);

impl ClientboundPacket for Failed {
    fn handle(self: Box<Self>, _session: &mut PlaySession) -> Result<(), ConnectionError> {
        Err(self.0)
    }
}

/// The packet that stands for a failure to parse one.
pub(super) fn failed(error: ConnectionError) -> Incoming {
    Box::new(Failed(error))
}

/// `ClientboundBundleDelimiterPacket`: it only marks where a bundle begins and ends.
struct BundleDelimiter;

impl ClientboundPacket for BundleDelimiter {
    fn handle(self: Box<Self>, _session: &mut PlaySession) -> Result<(), ConnectionError> {
        Ok(())
    }

    fn is_bundle_delimiter(&self) -> bool {
        true
    }
}

/// A packet that did not parse and is dropped.
struct Ignored;

impl ClientboundPacket for Ignored {
    fn handle(self: Box<Self>, _session: &mut PlaySession) -> Result<(), ConnectionError> {
        Ok(())
    }
}

/// A packet that did not parse: the session warns, once for each message.
struct Malformed(String);

impl ClientboundPacket for Malformed {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.warn(self.0);
        Ok(())
    }
}

impl PlaySession {
    /// A packet from the reader thread, applied to the client's state.
    pub(super) fn handle_incoming(&mut self, packet: Incoming) -> Result<(), ConnectionError> {
        let checks_loading = packet.checks_loading();
        self.link
            .traffic
            .received_packet(packet.is_bundle_delimiter());
        packet.handle(self)?;
        if checks_loading {
            self.maybe_finish_loading()?;
        }
        Ok(())
    }

    /// A packet's bytes, parsed and applied: the way a test hands one over.
    #[cfg(test)]
    pub(super) fn handle_bytes(&mut self, body: &[u8]) -> Result<(), ConnectionError> {
        self.handle_incoming(decode_packet(body)?)
    }
}
