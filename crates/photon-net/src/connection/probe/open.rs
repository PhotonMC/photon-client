use super::*;

/// Log in and configure, leaving the connection at the start of play.
///
/// Returns the stream and the compression threshold the server chose. The
/// login and configuration packets are recorded in `trace`, and the world in
/// it takes the tags the server sends.
pub(crate) fn open_play_connection(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
    select_known_packs: bool,
    information: &crate::configuration::ClientInformation,
    trace: &mut PlayConnectionTrace,
) -> Result<(TcpStream, Option<usize>), ConnectionError> {
    let mut stream = connect_stream(address, timeout)?;
    let host = trace
        .handshake_host
        .clone()
        .unwrap_or_else(|| address.ip().to_string());
    let intent = if trace.transferring {
        ClientIntent::Transfer
    } else {
        ClientIntent::Login
    };
    send_handshake_to(&mut stream, (&host, address.port()), intent)?;

    let compression_threshold = log_in(&mut stream, (username, timeout), information, trace)?;
    configure_connection(
        &mut stream,
        compression_threshold,
        (timeout, select_known_packs),
        trace,
    )?;
    Ok((stream, compression_threshold))
}

/// Send the login hello and run the login exchange up to the server's finish, which is
/// acknowledged and followed by the client's configuration start. The compression threshold the
/// server chose is returned.
fn log_in(
    stream: &mut TcpStream,
    (username, timeout): (&str, Duration),
    information: &crate::configuration::ClientInformation,
    trace: &mut PlayConnectionTrace,
) -> Result<Option<usize>, ConnectionError> {
    let hello = ServerboundHelloPacket::new(username.encode_utf16().collect(), Uuid::new(0, 1));
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
                let finished = ClientboundLoginFinishedPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login finished"))?;
                trace.login_finished = true;
                trace.profile_id = Some(finished.game_profile.id);

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
                // `handleCustomQuery`: nothing to answer with.
                let query = crate::cookie::ClientboundCustomQueryPacket::decode(&mut &payload[..])
                    .map_err(|_| ConnectionError::Codec("login custom query"))?;
                send_packet(
                    stream,
                    ConnectionProtocol::Login,
                    LOGIN_SERVERBOUND_CUSTOM_QUERY_ANSWER_PACKET_ID,
                    |output| {
                        crate::cookie::encode_unanswered_query(query.transaction_id, output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            LOGIN_CLIENTBOUND_COOKIE_REQUEST_PACKET_ID => {
                answer_cookie_request(
                    stream,
                    (
                        ConnectionProtocol::Login,
                        LOGIN_SERVERBOUND_COOKIE_RESPONSE_PACKET_ID,
                    ),
                    (payload, compression_threshold),
                    &trace.cookies,
                )?;
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
    Ok(compression_threshold)
}

/// Run the configuration exchange up to the server's finish, which is answered: registries, tags
/// and features are taken into `trace`, known packs are answered and keep-alives are returned.
/// The connection is in the play phase after it.
#[expect(
    clippy::too_many_lines,
    reason = "the configuration exchange as one sequence"
)]
pub(crate) fn configure_connection(
    stream: &mut TcpStream,
    compression_threshold: Option<usize>,
    (timeout, select_known_packs): (Duration, bool),
    trace: &mut PlayConnectionTrace,
) -> Result<(), ConnectionError> {
    let configuration_started = Instant::now();
    loop {
        let body = read_packet(stream, compression_threshold)?;
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
                trace
                    .registry_entry_names
                    .entry(packet.registry.as_string())
                    .or_default()
                    .extend(packet.entries.iter().map(|entry| entry.id.as_string()));
                if packet.registry.as_string() == "minecraft:dimension_type" {
                    for entry in packet.entries {
                        let bounds = entry
                            .dimension_type_bounds()
                            .map_err(|_| ConnectionError::Codec("dimension type registry"))?;
                        let environment = entry
                            .dimension_type_environment()
                            .map_err(|_| ConnectionError::Codec("dimension type environment"))?;
                        trace.dimension_type_registry_bounds.push(bounds);
                        trace.dimension_type_registry_environments.push(environment);
                    }
                }
            }
            CONFIG_UPDATE_TAGS_PACKET_ID => {
                let mut input = payload;
                let packet = photon_core::tags::ClientboundUpdateTagsPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("update tags"))?;
                trace.world.apply_update_tags_packet(&packet);
                trace.update_tags_packet_count += 1;
            }
            CONFIG_ENABLED_FEATURES_PACKET_ID => {
                let mut input = payload;
                let packet = crate::configuration::ClientboundUpdateEnabledFeaturesPacket::decode(
                    &mut input,
                )
                .map_err(|_| ConnectionError::Codec("enabled features"))?;
                trace.enabled_feature_count = packet.features.len();
                trace
                    .world
                    .set_minecart_improvements(packet.features.iter().any(|feature| {
                        feature.namespace() == "minecraft"
                            && feature.path() == "minecart_improvements"
                    }));
            }
            CONFIG_SELECT_KNOWN_PACKS_PACKET_ID => {
                let mut input = payload;
                let packs = KnownPack::decode_list(&mut input, usize::MAX)
                    .map_err(|_| ConnectionError::Codec("clientbound known packs"))?;
                trace.known_pack_count = packs.len();
                let selected_packs = if select_known_packs {
                    packs
                } else {
                    Vec::new()
                };
                trace.selected_known_pack_count = selected_packs.len();

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
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration keep alive"))?;
                send_packet(
                    stream,
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
                    stream,
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
                    stream,
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
                    stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_FINISH_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                trace.configuration_finished = true;
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
                    &trace.cookies,
                )?;
            }
            CONFIG_CLIENTBOUND_STORE_COOKIE_PACKET_ID => {
                store_cookie(trace, payload)?;
            }
            CONFIG_CLIENTBOUND_CUSTOM_PAYLOAD_PACKET_ID => {
                take_custom_payload(trace, payload)?;
            }
            CONFIG_CLIENTBOUND_CUSTOM_REPORT_DETAILS_PACKET_ID => {
                take_report_details(trace, payload)?;
            }
            2 => return Err(ConnectionError::ServerDisconnected),
            CONFIG_CLIENTBOUND_TRANSFER_PACKET_ID => {
                // `handleTransfer`: the connection is let go and another is made.
                let transfer = crate::cookie::ClientboundTransferPacket::decode(&mut &payload[..])
                    .map_err(|_| ConnectionError::Codec("transfer"))?;
                return Err(ConnectionError::Transfer {
                    host: transfer.host,
                    port: transfer.port,
                });
            }
            9 => {
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
    Ok(())
}

/// Answer a cookie the server asked for with what the client keeps under that key, or nothing.
pub(crate) fn answer_cookie_request(
    stream: &mut TcpStream,
    (phase, response_id): (ConnectionProtocol, i32),
    (payload, compression_threshold): (&[u8], Option<usize>),
    cookies: &crate::cookie::CookieStore,
) -> Result<(), ConnectionError> {
    let request = crate::cookie::ClientboundCookieRequestPacket::decode(&mut &payload[..])
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
    trace: &mut PlayConnectionTrace,
    payload: &[u8],
) -> Result<(), ConnectionError> {
    let packet = crate::cookie::ClientboundStoreCookiePacket::decode(&mut &payload[..])
        .map_err(|_| ConnectionError::Codec("store cookie"))?;
    trace.cookies.store(&packet.key, packet.payload);
    Ok(())
}

/// `handleCustomPayload`: the brand is kept, other payloads are not acted on.
pub(crate) fn take_custom_payload(
    trace: &mut PlayConnectionTrace,
    payload: &[u8],
) -> Result<(), ConnectionError> {
    let packet = crate::cookie::ClientboundCustomPayloadPacket::decode(&mut &payload[..])
        .map_err(|_| ConnectionError::Codec("custom payload"))?;
    if let Some(brand) = packet.brand() {
        trace.server_brand = Some(brand);
    }
    Ok(())
}

/// `handleCustomReportDetails`.
pub(crate) fn take_report_details(
    trace: &mut PlayConnectionTrace,
    payload: &[u8],
) -> Result<(), ConnectionError> {
    let packet = crate::cookie::ClientboundCustomReportDetailsPacket::decode(&mut &payload[..])
        .map_err(|_| ConnectionError::Codec("custom report details"))?;
    trace.custom_report_details = packet.details;
    Ok(())
}
