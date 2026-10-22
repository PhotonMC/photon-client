//! The status exchange (`ServerStatusPacketListener`): a server's status text and its ping.

use std::net::SocketAddr;
use std::time::Duration;

use photon_protocol::handshake::ClientIntent;
use photon_protocol::phase::ConnectionProtocol;
use photon_protocol::play::*;

use super::record::{NoRecorder, Recorder};
use super::session::{connect_stream, read_packet, send_handshake_intention, send_packet};
use super::transport::Transport;

/// What a server answered to a status request and a ping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerStatus {
    pub status_response: String,
    pub pong_time: i64,
    /// From the ping request sent to its pong read, in milliseconds (`ServerData.ping`).
    pub round_trip_ms: i64,
}

/// Query a Java-protocol server's status and verify its ping echo.
pub fn query_status_and_ping(
    address: SocketAddr,
    timeout: Duration,
) -> Result<ServerStatus, ConnectionError> {
    query_status_and_ping_recorded(address, timeout, &mut NoRecorder)
}

/// The same, with a watcher of the packets that were received.
pub fn query_status_and_ping_recorded(
    address: SocketAddr,
    timeout: Duration,
    recorder: &mut dyn Recorder,
) -> Result<ServerStatus, ConnectionError> {
    if timeout.is_zero() {
        return Err(ConnectionError::InvalidArgument("timeout"));
    }

    let mut stream = connect_stream(address, timeout)?;
    send_handshake_intention(&mut stream, address, ClientIntent::Status)?;
    let status_response = request_status(&mut stream, recorder)?;
    let (pong_time, round_trip_ms) = ping(&mut stream, recorder)?;
    Ok(ServerStatus {
        status_response,
        pong_time,
        round_trip_ms,
    })
}

/// Read the next packet of the status phase, which must have the id wanted.
fn expect_packet(
    stream: &mut Transport,
    wanted: i32,
    recorder: &mut dyn Recorder,
) -> Result<Vec<u8>, ConnectionError> {
    let body = read_packet(stream, None)?;
    let (packet_id, _) = split_packet(&body)?;
    recorder.packet(ConnectionProtocol::Status, packet_id, body.len());
    if packet_id != wanted {
        return Err(ConnectionError::UnsupportedPacket {
            phase: ConnectionProtocol::Status,
            id: packet_id,
        });
    }
    Ok(body)
}

/// `ServerboundStatusRequestPacket`, and the status text the server answers with.
fn request_status(
    stream: &mut Transport,
    recorder: &mut dyn Recorder,
) -> Result<String, ConnectionError> {
    send_packet(
        stream,
        ConnectionProtocol::Status,
        STATUS_SERVERBOUND_REQUEST_PACKET_ID,
        |payload| {
            photon_protocol::status::ServerboundStatusRequestPacket::INSTANCE.encode(payload);
            Ok(())
        },
        None,
    )?;
    let body = expect_packet(stream, STATUS_CLIENTBOUND_RESPONSE_PACKET_ID, recorder)?;
    let (_, payload) = split_packet(&body)?;
    let mut input = payload;
    let status_units = photon_core::utf8::read_utf8(&mut input, STATUS_RESPONSE_MAX_UTF16_LENGTH)
        .map_err(|_| ConnectionError::Codec("status response"))?;
    String::from_utf16(&status_units).map_err(|_| ConnectionError::Codec("status response UTF-16"))
}

/// A ping, and the pong that must echo it: the echoed time, and the round trip in milliseconds.
fn ping(
    stream: &mut Transport,
    recorder: &mut dyn Recorder,
) -> Result<(i64, i64), ConnectionError> {
    let ping = photon_protocol::status::ServerboundPingRequestPacket::new(STATUS_PROBE_TIME);
    let ping_start = std::time::Instant::now();
    send_packet(
        stream,
        ConnectionProtocol::Status,
        STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID,
        |payload| {
            ping.encode(payload);
            Ok(())
        },
        None,
    )?;
    let body = expect_packet(stream, STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID, recorder)?;
    let (_, payload) = split_packet(&body)?;
    let mut input = payload;
    let pong = photon_protocol::status::ClientboundPongResponsePacket::decode(&mut input)
        .map_err(|_| ConnectionError::Codec("status pong"))?;
    if pong.time != STATUS_PROBE_TIME {
        return Err(ConnectionError::Codec("status pong echo"));
    }
    Ok((pong.time, ping_start.elapsed().as_millis() as i64))
}
