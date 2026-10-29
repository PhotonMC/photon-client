//! Connection phase transitions and the synchronous transport state machine.

use super::transport::Transport;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use photon_core::wire;
use photon_protocol::configuration::{ClientInformation, ServerboundClientInformationPacket};
use photon_protocol::frame::{FrameError, MAX_VARINT21_BYTES};
use photon_protocol::handshake::{ClientIntent, ClientIntentionPacket};
use photon_protocol::login::ClientboundLoginFinishedPacket;
use photon_protocol::phase::ConnectionProtocol;

use photon_protocol::play::*;

pub fn send_configuration_start(
    stream: &mut Transport,
    information: &ClientInformation,
    compression_threshold: Option<usize>,
) -> Result<(), ConnectionError> {
    send_packet(
        stream,
        ConnectionProtocol::Configuration,
        CONFIG_CUSTOM_PAYLOAD_PACKET_ID,
        |output| {
            let id = photon_core::identifier::Identifier::parse("minecraft:brand")
                .map_err(|_| ConnectionError::Codec("brand identifier"))?;
            photon_core::identifier::write_identifier(&id, output)
                .map_err(|_| ConnectionError::Codec("brand identifier"))?;
            photon_core::utf8::write_utf8(
                output,
                &"vanilla".encode_utf16().collect::<Vec<_>>(),
                32_767,
            )
            .map_err(|_| ConnectionError::Codec("brand payload"))?;
            Ok(())
        },
        compression_threshold,
    )?;

    send_packet(
        stream,
        ConnectionProtocol::Configuration,
        CONFIG_CLIENT_INFORMATION_PACKET_ID,
        |output| {
            ServerboundClientInformationPacket::new(information.clone())
                .encode(output)
                .map_err(|_| ConnectionError::Codec("client information"))
        },
        compression_threshold,
    )
}

pub fn connect_stream(
    address: SocketAddr,
    timeout: Duration,
) -> Result<Transport, ConnectionError> {
    let stream = Transport::from(TcpStream::connect_timeout(&address, timeout)?);
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.set_nodelay(true)?;
    Ok(stream)
}

pub fn send_handshake_intention(
    stream: &mut Transport,
    address: SocketAddr,
    intent: ClientIntent,
) -> Result<(), ConnectionError> {
    send_status_handshake(stream, (&address.ip().to_string(), address.port()), intent)
}

/// The handshake of a status request, which announces the client's own protocol, as the reference
/// does.
pub fn send_status_handshake(
    stream: &mut Transport,
    (host, port): (&str, u16),
    intent: ClientIntent,
) -> Result<(), ConnectionError> {
    send_handshake_to(
        stream,
        (host, port),
        (intent, photon_core::version::Version::NEWEST),
    )
}

/// The handshake for a host name as given (`ClientIntentionPacket`'s host is what the player typed,
/// which a proxy may route by) and a port.
pub fn send_handshake_to(
    stream: &mut Transport,
    (host, port): (&str, u16),
    (intent, version): (ClientIntent, photon_core::version::Version),
) -> Result<(), ConnectionError> {
    let intention = ClientIntentionPacket::new(
        version.protocol(),
        host.encode_utf16().collect(),
        i32::from(port),
        intent,
    );
    send_packet(
        stream,
        ConnectionProtocol::Handshaking,
        LOGIN_INTENTION_PACKET_ID,
        |payload| {
            intention
                .encode(payload)
                .map_err(|_| ConnectionError::Codec("handshake"))
        },
        None,
    )
}

/// A packet as it goes on the wire: its id and payload, framed and, past the threshold,
/// compressed.
pub fn frame_packet<F>(
    packet_id: i32,
    encode_payload: F,
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
{
    let mut body = Vec::new();
    wire::write_varint(packet_id, &mut body);
    encode_payload(&mut body)?;
    encode_packet_frame(&body, compression_threshold)
}

pub fn send_packet<F>(
    stream: &mut Transport,
    _phase: ConnectionProtocol,
    packet_id: i32,
    encode_payload: F,
    compression_threshold: Option<usize>,
) -> Result<(), ConnectionError>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
{
    let framed = frame_packet(packet_id, encode_payload, compression_threshold)?;
    stream.write_all(&framed)?;
    stream.flush()?;
    Ok(())
}

pub fn read_packet(
    stream: &mut Transport,
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    let frame = read_frame(stream)?;
    decode_packet_frame(&frame, compression_threshold)
}

fn read_frame(stream: &mut Transport) -> Result<Vec<u8>, ConnectionError> {
    let mut header = Vec::with_capacity(MAX_VARINT21_BYTES);
    for _ in 0..MAX_VARINT21_BYTES {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte)?;
        header.push(byte[0]);
        if byte[0] & 0x80 == 0 {
            let mut input = header.as_slice();
            let length = wire::read_varint(&mut input)?;
            if length <= 0 {
                return Err(ConnectionError::Frame(if length == 0 {
                    FrameError::ZeroLength
                } else {
                    FrameError::PacketTooLarge
                }));
            }
            let length = usize::try_from(length)
                .map_err(|_| ConnectionError::Frame(FrameError::PacketTooLarge))?;
            if length > MAX_READABLE_FRAME_LENGTH {
                return Err(ConnectionError::Frame(FrameError::PacketTooLarge));
            }
            let mut body = vec![0_u8; length];
            stream.read_exact(&mut body)?;
            return Ok(body);
        }
    }
    Err(ConnectionError::Frame(FrameError::LengthTooWide))
}

/// The login-finished packet as `version` sends it.
pub(super) fn decode_login_finished(
    payload: &[u8],
    version: photon_core::version::Version,
) -> Result<ClientboundLoginFinishedPacket, ConnectionError> {
    let mut input = payload;
    ClientboundLoginFinishedPacket::decode_for(&mut input, version)
        .map_err(|_| ConnectionError::Codec("login finished"))
}
