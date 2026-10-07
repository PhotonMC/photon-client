use super::*;

pub(super) fn send_world_chunk_unload_step(
    stream: &mut TcpStream,
    player_position: &mut Option<(f64, f64, f64, f32, f32)>,
    trace: &mut PlayConnectionTrace,
    compression_threshold: Option<usize>,
    context: &'static str,
) -> Result<(), ConnectionError> {
    let Some(position) = *player_position else {
        return Err(ConnectionError::Codec(context));
    };
    let moved = (
        position.0 + 4.0,
        position.1,
        position.2,
        position.3,
        position.4,
    );
    send_packet(
        stream,
        ConnectionProtocol::Play,
        PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
        |output| encode_move_player_pos_rot(output, moved, 1),
        compression_threshold,
    )?;
    trace.movement_packets_sent += 1;
    *player_position = Some(moved);
    send_packet(
        stream,
        ConnectionProtocol::Play,
        PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID,
        |_output| Ok(()),
        compression_threshold,
    )?;
    Ok(())
}

/// Connect to a Java-protocol server, move across the configured view radius,
/// and observe chunk columns dropped from the client world copy.
pub fn connect_and_world_chunk_unload_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldChunkUnload),
    )
}

/// Connect to a Java-protocol server and observe a controlled dimension change
/// followed by the client-world replacement from a vanilla respawn packet.
pub fn connect_and_world_dimension_respawn_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldDimensionRespawn),
    )
}

/// Connect to a Java-protocol server, wait for an externally triggered death,
/// and observe the same-dimension respawn with an empty keep mask.
pub fn connect_and_world_player_death_respawn_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldPlayerDeathRespawn),
    )
}

/// Connect without selecting local known packs and observe registry-provided
/// DimensionType bounds during a controlled respawn.
pub fn connect_and_world_dimension_type_bounds_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::without_known_packs(ProbeAction::WorldDimensionTypeBounds),
    )
}

/// Connect to a Java-protocol server, keep the loaded world open, and observe standalone
/// clientbound light updates without adding movement to the scenario.
pub fn connect_and_world_light_update_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldLightUpdate),
    )
}

/// Connect to a Java-protocol server, keep the loaded world open, and observe standalone
/// clientbound block-entity data without adding movement to the scenario.
pub fn connect_and_world_block_entity_update_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldBlockEntityUpdate),
    )
}
