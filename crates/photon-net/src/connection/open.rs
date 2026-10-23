use super::transport::Transport;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use photon_core::binary::Uuid;
use photon_protocol::handshake::ClientIntent;
use photon_protocol::login::{ClientboundLoginFinishedPacket, ServerboundHelloPacket};
use photon_protocol::phase::ConnectionProtocol;
use photon_protocol::play::*;
use photon_protocol::resources::KnownPack;
use photon_protocol::{common, configuration, cookie, login, registry_data};

use super::encrypt;
use super::pack_sink::sink_for;
use super::record::Recorder;
use super::session::*;
use super::state::ConnectionState;
use crate::server_packs::PackUser;

/// Log in and configure, leaving the connection at the start of play.
///
/// Returns the stream and the compression threshold the server chose. What the server tells the
/// client to keep goes into `state`, and what happens is told to `recorder`.
pub fn open_play_connection(
    (address, username): (SocketAddr, &str),
    timeout: Duration,
    select_known_packs: bool,
    information: &configuration::ClientInformation,
    (state, recorder): (&mut ConnectionState, &mut dyn Recorder),
) -> Result<(Transport, Option<usize>), ConnectionError> {
    let mut stream = connect_stream(address, timeout)?;
    state.packs.set_user(PackUser {
        name: username.to_owned(),
        id: state
            .account
            .as_ref()
            .map_or(Uuid::new(0, 1), |account| account.profile_id),
    });
    state
        .packs
        .configure_for_server(sink_for(&stream, ConnectionProtocol::Configuration, None)?);
    let host = state
        .handshake_host
        .clone()
        .unwrap_or_else(|| address.ip().to_string());
    let intent = if state.transferring {
        ClientIntent::Transfer
    } else {
        ClientIntent::Login
    };
    send_handshake_to(&mut stream, (&host, address.port()), intent)?;

    let compression_threshold = log_in(
        &mut stream,
        (username, timeout),
        information,
        (state, recorder),
    )?;
    configure_connection(
        &mut stream,
        compression_threshold,
        (timeout, select_known_packs),
        (state, recorder),
    )?;
    Ok((stream, compression_threshold))
}

/// Send the login hello and run the login exchange up to the server's finish, which is
/// acknowledged and followed by the client's configuration start. The compression threshold the
/// server chose is returned.
fn log_in(
    stream: &mut Transport,
    (username, timeout): (&str, Duration),
    information: &configuration::ClientInformation,
    (state, recorder): (&mut ConnectionState, &mut dyn Recorder),
) -> Result<Option<usize>, ConnectionError> {
    let profile_id = state
        .account
        .as_ref()
        .map_or(Uuid::new(0, 1), |account| account.profile_id);
    let hello = ServerboundHelloPacket::new(username.encode_utf16().collect(), profile_id);
    send_packet(
        stream,
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

    loop {
        let body = read_packet(stream, compression_threshold)?;
        let (packet_id, payload) = split_packet(&body)?;
        recorder.packet(ConnectionProtocol::Login, packet_id, body.len());

        match packet_id {
            LOGIN_COMPRESSION_PACKET_ID => {
                let mut input = payload;
                let packet = login::ClientboundLoginCompressionPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login compression"))?;
                compression_threshold = if packet.threshold < 0 {
                    None
                } else {
                    Some(packet.threshold as usize)
                };
            }
            LOGIN_FINISHED_PACKET_ID => {
                let mut input = payload;
                let finished = ClientboundLoginFinishedPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login finished"))?;
                recorder.login_finished();
                remember_profile(state, &finished.game_profile);

                send_packet(
                    stream,
                    ConnectionProtocol::Login,
                    LOGIN_ACKNOWLEDGED_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                send_configuration_start(stream, information, compression_threshold)?;
                break;
            }
            LOGIN_CLIENTBOUND_CUSTOM_QUERY_PACKET_ID => {
                answer_custom_query(stream, payload, compression_threshold)?;
            }
            LOGIN_CLIENTBOUND_COOKIE_REQUEST_PACKET_ID => {
                answer_cookie_request(
                    stream,
                    (
                        ConnectionProtocol::Login,
                        LOGIN_SERVERBOUND_COOKIE_RESPONSE_PACKET_ID,
                    ),
                    (payload, compression_threshold),
                    &state.cookies,
                )?;
            }
            LOGIN_CLIENTBOUND_DISCONNECT_PACKET_ID => return Err(login_disconnect(payload)),
            LOGIN_CLIENTBOUND_HELLO_PACKET_ID => {
                let hello = login::ClientboundHelloPacket::decode(&mut &payload[..])
                    .map_err(|_| ConnectionError::Codec("login hello"))?;
                encrypt::answer_hello(
                    stream,
                    &hello,
                    (state.account.as_ref(), timeout),
                    compression_threshold,
                )?;
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
    Ok(compression_threshold)
}

/// The statuses of the server's packs go to the server over `stream` in `phase` from now on.
fn route_pack_statuses(
    stream: &Transport,
    phase: ConnectionProtocol,
    compression_threshold: Option<usize>,
    state: &ConnectionState,
) -> Result<(), ConnectionError> {
    state
        .packs
        .set_sink(sink_for(stream, phase, compression_threshold)?);
    Ok(())
}

/// The next packet of the configuration. When the connection ended on the player's answer (a
/// pack the server requires was refused), that is the reason it ended.
fn read_configuration_packet(
    stream: &mut Transport,
    compression_threshold: Option<usize>,
    state: &ConnectionState,
) -> Result<Vec<u8>, ConnectionError> {
    read_packet(stream, compression_threshold).map_err(|error| {
        state.packs.take_disconnect().map_or(error, |reason| {
            ConnectionError::Disconnected(Box::new(reason))
        })
    })
}

/// `ClientboundDisconnectPacket`: the reason goes to the disconnected screen.
fn config_disconnect(payload: &[u8]) -> ConnectionError {
    let mut input = payload;
    match read_disconnect_reason(&mut input) {
        Ok(reason) => ConnectionError::Disconnected(Box::new(reason)),
        Err(error) => error,
    }
}

/// `ClientboundLoginDisconnectPacket`: the reason is JSON text (`ByteBufCodecs.lenientJson`).
fn login_disconnect(payload: &[u8]) -> ConnectionError {
    let mut input = payload;
    let reason = photon_core::utf8::read_utf8(&mut input, 262_144)
        .ok()
        .map(|units| String::from_utf16_lossy(&units))
        .and_then(|text| photon_core::json::parse(&text).ok())
        .and_then(|json| photon_game::text::Component::from_json(&json));
    reason.map_or(ConnectionError::Codec("login disconnect"), |reason| {
        ConnectionError::Disconnected(Box::new(reason))
    })
}

/// `handleCustomQuery`: nothing to answer with.
fn answer_custom_query(
    stream: &mut Transport,
    payload: &[u8],
    compression_threshold: Option<usize>,
) -> Result<(), ConnectionError> {
    let query = cookie::ClientboundCustomQueryPacket::decode(&mut &payload[..])
        .map_err(|_| ConnectionError::Codec("login custom query"))?;
    send_packet(
        stream,
        ConnectionProtocol::Login,
        LOGIN_SERVERBOUND_CUSTOM_QUERY_ANSWER_PACKET_ID,
        |output| {
            cookie::encode_unanswered_query(query.transaction_id, output);
            Ok(())
        },
        compression_threshold,
    )
}

/// Run the configuration exchange up to the server's finish, which is answered: registries, tags
/// and features are taken into `state`, known packs are answered and keep-alives are returned.
/// The connection is in the play phase after it.
pub fn configure_connection(
    stream: &mut Transport,
    compression_threshold: Option<usize>,
    (timeout, select_known_packs): (Duration, bool),
    (state, recorder): (&mut ConnectionState, &mut dyn Recorder),
) -> Result<(), ConnectionError> {
    let configuration_started = Instant::now();
    route_pack_statuses(
        stream,
        ConnectionProtocol::Configuration,
        compression_threshold,
        state,
    )?;
    loop {
        let body = read_configuration_packet(stream, compression_threshold, state)?;
        let (packet_id, payload) = split_packet(&body)?;
        recorder.packet(ConnectionProtocol::Configuration, packet_id, body.len());

        match packet_id {
            CONFIG_REGISTRY_DATA_PACKET_ID => {
                let mut input = payload;
                let packet = registry_data::ClientboundRegistryDataPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("registry data"))?;
                recorder.registry_data(packet.entries.len());
                state
                    .registry_entry_names
                    .entry(packet.registry.as_string())
                    .or_default()
                    .extend(packet.entries.iter().map(|entry| entry.id.as_string()));
                if packet.registry.as_string() == "minecraft:dialog" {
                    state
                        .dialog_registry
                        .extend(packet.entries.iter().map(|entry| {
                            entry
                                .data
                                .as_ref()
                                .and_then(photon_game::dialog::Dialog::from_nbt)
                        }));
                }
                if packet.registry.as_string() == "minecraft:dimension_type" {
                    for entry in packet.entries {
                        let bounds = entry
                            .dimension_type_bounds()
                            .map_err(|_| ConnectionError::Codec("dimension type registry"))?;
                        let environment = entry
                            .dimension_type_environment()
                            .map_err(|_| ConnectionError::Codec("dimension type environment"))?;
                        state.dimension_type_registry_bounds.push(bounds);
                        state.dimension_type_registry_environments.push(environment);
                    }
                }
            }
            CONFIG_UPDATE_TAGS_PACKET_ID => {
                let mut input = payload;
                let packet = photon_core::tags::ClientboundUpdateTagsPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("update tags"))?;
                state.tags.push(packet);
                recorder.update_tags();
            }
            CONFIG_ENABLED_FEATURES_PACKET_ID => {
                let mut input = payload;
                let packet =
                    configuration::ClientboundUpdateEnabledFeaturesPacket::decode(&mut input)
                        .map_err(|_| ConnectionError::Codec("enabled features"))?;
                recorder.enabled_features(packet.features.len());
                state.minecart_improvements = Some(packet.features.iter().any(|feature| {
                    feature.namespace() == "minecraft" && feature.path() == "minecart_improvements"
                }));
            }
            CONFIG_SELECT_KNOWN_PACKS_PACKET_ID => {
                let mut input = payload;
                let packs = KnownPack::decode_list(&mut input, usize::MAX)
                    .map_err(|_| ConnectionError::Codec("clientbound known packs"))?;
                let offered = packs.len();
                let selected_packs = if select_known_packs {
                    packs
                } else {
                    Vec::new()
                };
                recorder.known_packs(offered, selected_packs.len());

                send_packet(
                    stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_SERVERBOUND_SELECT_KNOWN_PACKS_PACKET_ID,
                    |output| {
                        KnownPack::encode_list(&selected_packs, output, MAX_KNOWN_PACKS)
                            .map_err(|_| ConnectionError::Codec("serverbound known packs"))
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration keep alive"))?;
                send_packet(
                    stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_KEEP_ALIVE_PACKET_ID,
                    |output| {
                        common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_PONG_PACKET_ID => {
                let mut input = payload;
                let packet = common::ClientboundPingPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration ping"))?;
                send_packet(
                    stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_PONG_PACKET_ID,
                    |output| {
                        common::ServerboundPongPacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_CODE_OF_CONDUCT_PACKET_ID => {
                let mut input = payload;
                configuration::ClientboundCodeOfConductPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("code of conduct"))?;
                send_packet(
                    stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_ACCEPT_CODE_OF_CONDUCT_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
            }
            CONFIG_FINISH_PACKET_ID => {
                configuration::ClientboundFinishConfigurationPacket::decode(&mut &payload[..])
                    .map_err(|_| ConnectionError::Codec("finish configuration"))?;
                send_packet(
                    stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_FINISH_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                recorder.configuration_finished();
                route_pack_statuses(
                    stream,
                    ConnectionProtocol::Play,
                    compression_threshold,
                    state,
                )?;
                break;
            }
            CONFIG_CLIENTBOUND_COOKIE_REQUEST_PACKET_ID => {
                answer_cookie_request(
                    stream,
                    (
                        ConnectionProtocol::Configuration,
                        CONFIG_SERVERBOUND_COOKIE_RESPONSE_PACKET_ID,
                    ),
                    (payload, compression_threshold),
                    &state.cookies,
                )?;
            }
            CONFIG_CLIENTBOUND_STORE_COOKIE_PACKET_ID => {
                store_cookie(state, payload)?;
            }
            CONFIG_CLIENTBOUND_CUSTOM_PAYLOAD_PACKET_ID => {
                take_custom_payload(state, payload)?;
            }
            CONFIG_CLIENTBOUND_CUSTOM_REPORT_DETAILS_PACKET_ID | RESET_CHAT | SERVER_LINKS => {
                take_server_notes(state, (packet_id, payload))?;
            }
            CONFIG_CLIENTBOUND_DISCONNECT_PACKET_ID => return Err(config_disconnect(payload)),
            CONFIG_CLIENTBOUND_TRANSFER_PACKET_ID => {
                // `handleTransfer`: the connection is let go and another is made.
                let transfer = cookie::ClientboundTransferPacket::decode(&mut &payload[..])
                    .map_err(|_| ConnectionError::Codec("transfer"))?;
                return Err(ConnectionError::Transfer {
                    host: transfer.host,
                    port: transfer.port,
                });
            }
            CONFIG_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID => {
                state.packs.push(parse_resource_pack_push(payload)?);
            }
            CONFIG_CLIENTBOUND_RESOURCE_PACK_POP_PACKET_ID => {
                state.packs.pop(parse_resource_pack_pop(payload)?);
            }
            _ => {}
        }

        // A pack the player is asked about or that is on its way keeps the server waiting; that
        // is not the configuration stalling.
        if configuration_started.elapsed() > timeout && state.packs.is_idle() {
            return Err(ConnectionError::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "configuration timeout",
            )));
        }
    }
    Ok(())
}

/// Answer a cookie the server asked for with what the client keeps under that key, or nothing.
pub(crate) fn answer_cookie_request(
    stream: &mut Transport,
    (phase, response_id): (ConnectionProtocol, i32),
    (payload, compression_threshold): (&[u8], Option<usize>),
    cookies: &cookie::CookieStore,
) -> Result<(), ConnectionError> {
    let request = cookie::ClientboundCookieRequestPacket::decode(&mut &payload[..])
        .map_err(|_| ConnectionError::Codec("cookie request"))?;
    let answer = cookies.answer(&request);
    send_packet(
        stream,
        phase,
        response_id,
        |output| {
            answer
                .encode(output)
                .map_err(|_| ConnectionError::Codec("cookie response"))
        },
        compression_threshold,
    )
}

/// `handleStoreCookie`.
pub(crate) fn store_cookie(
    state: &mut ConnectionState,
    payload: &[u8],
) -> Result<(), ConnectionError> {
    let packet = cookie::ClientboundStoreCookiePacket::decode(&mut &payload[..])
        .map_err(|_| ConnectionError::Codec("store cookie"))?;
    state.cookies.store(&packet.key, packet.payload);
    Ok(())
}

/// `handleCustomPayload`: the brand is kept, other payloads are not acted on.
pub(crate) fn take_custom_payload(
    state: &mut ConnectionState,
    payload: &[u8],
) -> Result<(), ConnectionError> {
    let packet = cookie::ClientboundCustomPayloadPacket::decode(&mut &payload[..])
        .map_err(|_| ConnectionError::Codec("custom payload"))?;
    if let Some(brand) = packet.brand() {
        state.server_brand = Some(brand);
    }
    Ok(())
}

const SERVER_LINKS: i32 = CONFIG_CLIENTBOUND_SERVER_LINKS_PACKET_ID;
const RESET_CHAT: i32 = CONFIG_CLIENTBOUND_RESET_CHAT_PACKET_ID;

/// The configuration packets that leave a note on the state: the report details, the links the
/// server offers (`handleServerLinks`) and the reset of the chat (`handleResetChat`).
fn take_server_notes(
    state: &mut ConnectionState,
    (packet_id, payload): (i32, &[u8]),
) -> Result<(), ConnectionError> {
    if packet_id == RESET_CHAT {
        state.chat_reset = true;
        return Ok(());
    }
    if packet_id == SERVER_LINKS {
        let links = photon_protocol::server_links::parse_server_links(payload)?;
        state.server_links = photon_game::server_links::ServerLinks::trusted(links);
        return Ok(());
    }
    take_report_details(state, payload)
}

/// `handleCustomReportDetails`.
pub(crate) fn take_report_details(
    state: &mut ConnectionState,
    payload: &[u8],
) -> Result<(), ConnectionError> {
    let packet = cookie::ClientboundCustomReportDetailsPacket::decode(&mut &payload[..])
        .map_err(|_| ConnectionError::Codec("custom report details"))?;
    state.custom_report_details = packet.details;
    Ok(())
}

/// What the server's profile says of the player: the id, and the name as the server spells it.
fn remember_profile(state: &mut ConnectionState, profile: &photon_protocol::login::GameProfile) {
    state.profile_id = Some(profile.id);
    state.profile_name = Some(String::from_utf16_lossy(&profile.name));
}
