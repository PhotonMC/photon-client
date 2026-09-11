//! A small synchronous client connection used by the first Pumpkin integration probe.
//!
//! This module owns the byte pipeline up to the first play-world packets. It does not
//! implement a renderer or a client world; its purpose is to make the phase transition
//! and the first world delivery observable without turning the test into a manual launch.

use std::fmt;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::binary::Uuid;
use crate::configuration::{
    ChatVisibility, ClientInformation, MainHand, ParticleStatus, ServerboundClientInformationPacket,
};
use crate::frame::{self, FrameError, MAX_FRAME_LENGTH, MAX_VARINT21_BYTES};
use crate::handshake::{ClientIntent, ClientIntentionPacket};
use crate::login::{ClientboundLoginFinishedPacket, ServerboundHelloPacket};
use crate::protocol::ConnectionProtocol;
use crate::resources::KnownPack;
use crate::wire;

const PROTOCOL_VERSION: i32 = 776;
const LOGIN_INTENTION_PACKET_ID: i32 = 0;
const LOGIN_HELLO_PACKET_ID: i32 = 0;
const LOGIN_ACKNOWLEDGED_PACKET_ID: i32 = 3;
const LOGIN_FINISHED_PACKET_ID: i32 = 2;
const LOGIN_COMPRESSION_PACKET_ID: i32 = 3;

const CONFIG_CLIENT_INFORMATION_PACKET_ID: i32 = 0;
const CONFIG_CUSTOM_PAYLOAD_PACKET_ID: i32 = 2;
const CONFIG_FINISH_PACKET_ID: i32 = 3;
const CONFIG_KEEP_ALIVE_PACKET_ID: i32 = 4;
const CONFIG_PONG_PACKET_ID: i32 = 5;
const CONFIG_REGISTRY_DATA_PACKET_ID: i32 = 7;
const CONFIG_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 14;
const CONFIG_SERVERBOUND_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 7;
const CONFIG_ACCEPT_CODE_OF_CONDUCT_PACKET_ID: i32 = 9;
const CONFIG_ENABLED_FEATURES_PACKET_ID: i32 = 12;
const CONFIG_UPDATE_TAGS_PACKET_ID: i32 = 13;
const CONFIG_CODE_OF_CONDUCT_PACKET_ID: i32 = 19;

// The virtual bundle packet is not registered in the Java dispatch table; the delimiter
// occupies id 0 and shifts the listed play clientbound packet ids by one.
const PLAY_CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID: i32 = 0;
const PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
const PLAY_CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
const PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID: i32 = 43;
const PLAY_CLIENTBOUND_CHUNK_PACKET_ID: i32 = 45;
const PLAY_CLIENTBOUND_LOGIN_PACKET_ID: i32 = 49;
const PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID: i32 = 11;
const PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID: i32 = 28;

const MAX_PACKET_UNCOMPRESSED_LENGTH: usize = 8 * 1024 * 1024;
const MAX_KNOWN_PACKS: usize = 64;
const MAX_READABLE_FRAME_LENGTH: usize = MAX_FRAME_LENGTH;
const WORLD_READY_DRAIN_TIMEOUT: Duration = Duration::from_millis(250);

/// Errors raised while driving the initial real-server client session.
#[derive(Debug)]
pub enum ConnectionError {
    Io(io::Error),
    Frame(FrameError),
    Wire(wire::ReadError),
    Compression(String),
    Codec(&'static str),
    UnsupportedPacket { phase: ConnectionProtocol, id: i32 },
    ServerDisconnected,
    InvalidArgument(&'static str),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Frame(error) => write!(formatter, "frame error: {error:?}"),
            Self::Wire(error) => write!(formatter, "wire error: {error:?}"),
            Self::Compression(error) => write!(formatter, "compression error: {error}"),
            Self::Codec(name) => write!(formatter, "packet codec failed: {name}"),
            Self::UnsupportedPacket { phase, id } => {
                write!(formatter, "unsupported packet id {id} in {phase:?}")
            }
            Self::ServerDisconnected => formatter.write_str("server disconnected"),
            Self::InvalidArgument(name) => write!(formatter, "invalid argument: {name}"),
        }
    }
}

impl std::error::Error for ConnectionError {}

impl From<io::Error> for ConnectionError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FrameError> for ConnectionError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}

impl From<wire::ReadError> for ConnectionError {
    fn from(error: wire::ReadError) -> Self {
        Self::Wire(error)
    }
}

/// One received packet after the outer frame and optional compression were removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketObservation {
    pub phase: ConnectionProtocol,
    pub id: i32,
    pub body_length: usize,
}

/// The observable result of the initial real Pumpkin session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayConnectionTrace {
    pub packets: Vec<PacketObservation>,
    pub compression_threshold: Option<usize>,
    pub login_finished: bool,
    pub configuration_finished: bool,
    pub enabled_feature_count: usize,
    pub known_pack_count: usize,
    pub registry_packet_count: usize,
    pub registry_entry_count: usize,
    pub update_tags_packet_count: usize,
    pub play_login: bool,
    pub chunk_packet_count: usize,
    pub chunk_batch_finished_count: usize,
}

/// Connect to a Java-protocol server and drive the session until the first play chunk.
///
/// The server must use offline authentication, as the project deliberately does not
/// implement Mojang's encryption/authentication exchange in this first slice.
pub fn connect_and_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    if username.is_empty() || username.encode_utf16().count() > crate::login::MAX_NAME_LENGTH {
        return Err(ConnectionError::InvalidArgument("username"));
    }
    if timeout.is_zero() {
        return Err(ConnectionError::InvalidArgument("timeout"));
    }

    let mut stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.set_nodelay(true)?;

    let host = address.ip().to_string();
    let intention = ClientIntentionPacket::new(
        PROTOCOL_VERSION,
        host.encode_utf16().collect(),
        i32::from(address.port()),
        ClientIntent::Login,
    );
    send_packet(
        &mut stream,
        ConnectionProtocol::Handshaking,
        LOGIN_INTENTION_PACKET_ID,
        |payload| {
            intention
                .encode(payload)
                .map_err(|_| ConnectionError::Codec("handshake"))
        },
        None,
    )?;

    let hello = ServerboundHelloPacket::new(username.encode_utf16().collect(), Uuid::new(0, 1));
    send_packet(
        &mut stream,
        ConnectionProtocol::Login,
        LOGIN_HELLO_PACKET_ID,
        |payload| {
            hello
                .encode(payload)
                .map_err(|_| ConnectionError::Codec("login hello"))
        },
        None,
    )?;

    let started = Instant::now();
    let mut compression_threshold = None;
    let mut trace = PlayConnectionTrace::default();

    loop {
        let body = read_packet(&mut stream, compression_threshold)?;
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Login,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            LOGIN_COMPRESSION_PACKET_ID => {
                let mut input = payload;
                let packet = crate::login::ClientboundLoginCompressionPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login compression"))?;
                compression_threshold = if packet.threshold < 0 {
                    None
                } else {
                    Some(packet.threshold as usize)
                };
                trace.compression_threshold = compression_threshold;
            }
            LOGIN_FINISHED_PACKET_ID => {
                let mut input = payload;
                ClientboundLoginFinishedPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login finished"))?;
                trace.login_finished = true;

                send_packet(
                    &mut stream,
                    ConnectionProtocol::Login,
                    LOGIN_ACKNOWLEDGED_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                send_configuration_start(&mut stream, compression_threshold)?;
                break;
            }
            0 => return Err(ConnectionError::ServerDisconnected),
            1 => {
                return Err(ConnectionError::UnsupportedPacket {
                    phase: ConnectionProtocol::Login,
                    id: packet_id,
                });
            }
            _ => {}
        }

        if started.elapsed() > timeout {
            return Err(ConnectionError::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "login timeout",
            )));
        }
    }

    let configuration_started = Instant::now();
    loop {
        let body = read_packet(&mut stream, compression_threshold)?;
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Configuration,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            CONFIG_REGISTRY_DATA_PACKET_ID => {
                let mut input = payload;
                let packet =
                    crate::registry_data::ClientboundRegistryDataPacket::decode(&mut input)
                        .map_err(|_| ConnectionError::Codec("registry data"))?;
                trace.registry_packet_count += 1;
                trace.registry_entry_count += packet.entries.len();
            }
            CONFIG_UPDATE_TAGS_PACKET_ID => {
                let mut input = payload;
                crate::tags::ClientboundUpdateTagsPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("update tags"))?;
                trace.update_tags_packet_count += 1;
            }
            CONFIG_ENABLED_FEATURES_PACKET_ID => {
                let mut input = payload;
                let packet = crate::configuration::ClientboundUpdateEnabledFeaturesPacket::decode(
                    &mut input,
                )
                .map_err(|_| ConnectionError::Codec("enabled features"))?;
                trace.enabled_feature_count = packet.features.len();
            }
            CONFIG_SELECT_KNOWN_PACKS_PACKET_ID => {
                let mut input = payload;
                let packs = KnownPack::decode_list(&mut input, usize::MAX)
                    .map_err(|_| ConnectionError::Codec("clientbound known packs"))?;
                trace.known_pack_count = packs.len();

                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_SERVERBOUND_SELECT_KNOWN_PACKS_PACKET_ID,
                    |output| {
                        KnownPack::encode_list(&packs, output, MAX_KNOWN_PACKS)
                            .map_err(|_| ConnectionError::Codec("serverbound known packs"))
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration keep alive"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_KEEP_ALIVE_PACKET_ID,
                    |output| {
                        crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_PONG_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundPingPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration ping"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_PONG_PACKET_ID,
                    |output| {
                        crate::common::ServerboundPongPacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_CODE_OF_CONDUCT_PACKET_ID => {
                let mut input = payload;
                crate::configuration::ClientboundCodeOfConductPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("code of conduct"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_ACCEPT_CODE_OF_CONDUCT_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
            }
            CONFIG_FINISH_PACKET_ID => {
                crate::configuration::ClientboundFinishConfigurationPacket::decode(
                    &mut &payload[..],
                )
                .map_err(|_| ConnectionError::Codec("finish configuration"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_FINISH_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                trace.configuration_finished = true;
                break;
            }
            2 => return Err(ConnectionError::ServerDisconnected),
            9 | 11 => {
                return Err(ConnectionError::UnsupportedPacket {
                    phase: ConnectionProtocol::Configuration,
                    id: packet_id,
                });
            }
            _ => {}
        }

        if configuration_started.elapsed() > timeout {
            return Err(ConnectionError::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "configuration timeout",
            )));
        }
    }

    let play_started = Instant::now();
    let mut chunk_batch_started = false;
    let mut world_ready_drain_started = None;
    while play_started.elapsed() <= timeout {
        let body = match read_packet(&mut stream, compression_threshold) {
            Ok(body) => body,
            Err(ConnectionError::Io(error))
                if world_ready_drain_started.is_some()
                    && matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
            {
                return Ok(trace);
            }
            Err(error) => return Err(error),
        };
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Play,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            PLAY_CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID => {
                chunk_batch_started = true;
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID => {
                let mut input = payload;
                wire::read_varint(&mut input)
                    .map_err(|_| ConnectionError::Codec("chunk batch finished"))?;
                trace.chunk_batch_finished_count += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID,
                    |output| {
                        // This is the initial value used by Java's chunk batch calculator:
                        // 7,000,000 ns / 2,000,000 ns per chunk.
                        output.extend_from_slice(&3.5_f32.to_be_bytes());
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            PLAY_CLIENTBOUND_LOGIN_PACKET_ID => trace.play_login = true,
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => trace.chunk_packet_count += 1,
            PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("play keep alive"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID,
                    |output| {
                        crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            PLAY_CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID => {}
            _ => {}
        }

        if trace.play_login
            && trace.chunk_packet_count > 0
            && (!chunk_batch_started || trace.chunk_batch_finished_count > 0)
            && world_ready_drain_started.is_none()
        {
            world_ready_drain_started = Some(Instant::now());
            stream.set_read_timeout(Some(WORLD_READY_DRAIN_TIMEOUT))?;
        }

        if world_ready_drain_started
            .is_some_and(|started| started.elapsed() >= WORLD_READY_DRAIN_TIMEOUT)
        {
            return Ok(trace);
        }
    }

    Err(ConnectionError::Io(io::Error::new(
        io::ErrorKind::TimedOut,
        "play world did not arrive before timeout",
    )))
}

fn send_configuration_start(
    stream: &mut TcpStream,
    compression_threshold: Option<usize>,
) -> Result<(), ConnectionError> {
    send_packet(
        stream,
        ConnectionProtocol::Configuration,
        CONFIG_CUSTOM_PAYLOAD_PACKET_ID,
        |output| {
            let id = crate::identifier::Identifier::parse("minecraft:brand")
                .map_err(|_| ConnectionError::Codec("brand identifier"))?;
            crate::identifier::write_identifier(&id, output)
                .map_err(|_| ConnectionError::Codec("brand identifier"))?;
            crate::utf8::write_utf8(
                output,
                &"vanilla".encode_utf16().collect::<Vec<_>>(),
                32_767,
            )
            .map_err(|_| ConnectionError::Codec("brand payload"))?;
            Ok(())
        },
        compression_threshold,
    )?;

    let information = ClientInformation::new(
        "en_us".encode_utf16().collect(),
        12,
        ChatVisibility::Full,
        true,
        127,
        MainHand::Right,
        false,
        true,
        ParticleStatus::All,
    );
    send_packet(
        stream,
        ConnectionProtocol::Configuration,
        CONFIG_CLIENT_INFORMATION_PACKET_ID,
        |output| {
            ServerboundClientInformationPacket::new(information)
                .encode(output)
                .map_err(|_| ConnectionError::Codec("client information"))
        },
        compression_threshold,
    )
}

fn send_packet<F>(
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

fn read_packet(
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

fn encode_packet_frame(
    body: &[u8],
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    if body.len() > MAX_PACKET_UNCOMPRESSED_LENGTH {
        return Err(ConnectionError::Frame(FrameError::PacketTooLarge));
    }

    let mut payload = Vec::new();
    match compression_threshold {
        Some(threshold) if body.len() < threshold => {
            wire::write_varint(0, &mut payload);
            payload.extend_from_slice(body);
        }
        Some(_) => {
            wire::write_varint(
                i32::try_from(body.len())
                    .map_err(|_| ConnectionError::Frame(FrameError::PacketTooLarge))?,
                &mut payload,
            );
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body)?;
            payload.extend_from_slice(
                &encoder
                    .finish()
                    .map_err(|error| ConnectionError::Compression(error.to_string()))?,
            );
        }
        None => payload.extend_from_slice(body),
    }
    frame::prepend(&payload).map_err(ConnectionError::from)
}

fn decode_packet_frame(
    frame: &[u8],
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    let Some(_) = compression_threshold else {
        return Ok(frame.to_vec());
    };

    let mut input = frame;
    let uncompressed_length = wire::read_varint(&mut input)?;
    if uncompressed_length < 0 {
        return Err(ConnectionError::Compression(
            "negative uncompressed length".to_owned(),
        ));
    }
    if uncompressed_length == 0 {
        return Ok(input.to_vec());
    }

    let uncompressed_length = usize::try_from(uncompressed_length)
        .map_err(|_| ConnectionError::Compression("invalid uncompressed length".to_owned()))?;
    if uncompressed_length > MAX_PACKET_UNCOMPRESSED_LENGTH {
        return Err(ConnectionError::Compression(
            "uncompressed packet exceeds 8 MiB".to_owned(),
        ));
    }

    let decoder = ZlibDecoder::new(input);
    let mut output = Vec::with_capacity(uncompressed_length);
    decoder
        .take((uncompressed_length as u64) + 1)
        .read_to_end(&mut output)?;
    if output.len() != uncompressed_length {
        return Err(ConnectionError::Compression(format!(
            "declared {uncompressed_length} bytes, decoded {}",
            output.len()
        )));
    }
    Ok(output)
}

fn split_packet(body: &[u8]) -> Result<(i32, &[u8]), ConnectionError> {
    let mut input = body;
    let packet_id = wire::read_varint(&mut input)?;
    Ok((packet_id, input))
}

#[cfg(test)]
mod tests {
    use super::{decode_packet_frame, encode_packet_frame};

    #[test]
    fn compression_uses_java_boundary_and_round_trips() {
        let below = vec![0x5a; 255];
        let at = vec![0xa5; 256];

        let below_frame = encode_packet_frame(&below, Some(256)).expect("frame below threshold");
        let mut below_header = below_frame.as_slice();
        crate::wire::read_varint(&mut below_header).expect("below frame header");
        assert_eq!(below_header[0], 0x00);
        assert_eq!(decode_packet_frame(below_header, Some(256)).unwrap(), below);

        let at_frame = encode_packet_frame(&at, Some(256)).expect("frame at threshold");
        let mut at_header = at_frame.as_slice();
        crate::wire::read_varint(&mut at_header).expect("at frame header");
        assert_ne!(at_header[0], 0x00);
        assert_eq!(decode_packet_frame(at_header, Some(256)).unwrap(), at);
    }
}
