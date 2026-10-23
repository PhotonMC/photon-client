//! Where the statuses of the server's resource packs are sent: the connection's own socket, in
//! the phase it is in.

use photon_protocol::play::{
    CONFIG_SERVERBOUND_RESOURCE_PACK_PACKET_ID, PLAY_SERVERBOUND_RESOURCE_PACK_PACKET_ID,
    encode_resource_pack,
};

use super::session::send_packet;
use super::transport::Transport;
use crate::server_packs::PackSink;
use photon_protocol::phase::ConnectionProtocol;

/// A sink for the phase `phase` of the connection `stream` (a clone of it is kept). It writes
/// from the thread that reports, which the transport lets happen beside the connection's own
/// writer: each packet is written whole. The configuration sink also ends the connection when
/// the player's answer must, which wakes the configuration that is waiting on it.
pub(super) fn sink_for(
    stream: &Transport,
    phase: ConnectionProtocol,
    compression_threshold: Option<usize>,
) -> std::io::Result<PackSink> {
    let packet_id = match phase {
        ConnectionProtocol::Play => PLAY_SERVERBOUND_RESOURCE_PACK_PACKET_ID,
        _ => CONFIG_SERVERBOUND_RESOURCE_PACK_PACKET_ID,
    };
    let sender = std::sync::Mutex::new(stream.try_clone()?);
    let sink = PackSink::new(move |id, action| {
        let mut stream = sender
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sent = send_packet(
            &mut stream,
            phase,
            packet_id,
            |output| {
                encode_resource_pack(id, action, output);
                Ok(())
            },
            compression_threshold,
        );
        if let Err(error) = sent {
            log::debug!("pack status not sent: {error:?}");
        }
    });
    if phase == ConnectionProtocol::Play {
        return Ok(sink);
    }
    let ender = stream.try_clone()?;
    Ok(sink.with_abort(move || ender.shutdown()))
}
