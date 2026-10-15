//! Connection phase transitions and the synchronous transport state machine.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use photon_core::wire;
use photon_protocol::configuration::{ClientInformation, ServerboundClientInformationPacket};
use photon_protocol::frame::{FrameError, MAX_VARINT21_BYTES};
use photon_protocol::handshake::{ClientIntent, ClientIntentionPacket};
use photon_protocol::phase::ConnectionProtocol;

use photon_protocol::play::*;

pub fn send_configuration_start(
    stream: &mut TcpStream,
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
) -> Result<TcpStream, ConnectionError> {
    let stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.set_nodelay(true)?;
    Ok(stream)
}

pub fn send_handshake_intention(
    stream: &mut TcpStream,
    address: SocketAddr,
    intent: ClientIntent,
) -> Result<(), ConnectionError> {
    send_handshake_to(stream, (&address.ip().to_string(), address.port()), intent)
}

/// The handshake for a host name as given (`ClientIntentionPacket`'s host is what the player typed,
/// which a proxy may route by) and a port.
pub fn send_handshake_to(
    stream: &mut TcpStream,
    (host, port): (&str, u16),
    intent: ClientIntent,
) -> Result<(), ConnectionError> {
    let intention = ClientIntentionPacket::new(
        PROTOCOL_VERSION,
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

pub fn send_packet<F>(
    stream: &mut TcpStream,
    _phase: ConnectionProtocol,
    packet_id: i32,
    encode_payload: F,
    compression_threshold: Option<usize>,
) -> Result<(), ConnectionError>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
{
    let mut body = Vec::new();
    wire::write_varint(packet_id, &mut body);
    encode_payload(&mut body)?;
    let framed = encode_packet_frame(&body, compression_threshold)?;
    stream.write_all(&framed)?;
    stream.flush()?;
    Ok(())
}

pub fn read_packet(
    stream: &mut TcpStream,
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    let frame = read_frame(stream)?;
    decode_packet_frame(&frame, compression_threshold)
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, ConnectionError> {
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
