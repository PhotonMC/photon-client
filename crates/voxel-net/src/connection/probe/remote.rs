use super::*;

/// Connect to a Java-protocol server and drive the session until the first play chunk.
///
/// The server must use offline authentication, as the project deliberately does not
/// implement Mojang's encryption/authentication exchange in this first slice.
pub fn connect_and_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::Basic),
    )
}

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

/// Connect to a Java-protocol server and send a short controlled position trace after loading.
pub fn connect_and_move_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::Movement),
    )
}

/// Connect to a Java-protocol server and send a short movement trace with
/// changing head rotations so another client can observe a remote player.
pub fn connect_and_remote_player_head_rotation_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    const YAW_SEQUENCE: &[i16] = &[90, 180, 270];

    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::Movement)
            .with_movement_yaws(YAW_SEQUENCE)
            .with_movement_pause(Duration::from_millis(500))
            .with_movement_hold(Duration::from_secs(5)),
    )
}

/// Connect to a Java-protocol server and observe a remote player during a
/// short two-client head-rotation scenario.
pub fn connect_and_remote_player_head_rotation_observer_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityAdd)
            .with_drain_timeout(Duration::from_secs(5)),
    )
}

/// Connect to a Java-protocol server and send a short controlled movement trace
/// whose remote-player position interpolation can be observed by another client.
pub fn connect_and_remote_player_interpolation_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    const YAW_SEQUENCE: &[i16] = &[90, 180, 270];

    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::Movement)
            .with_movement_yaws(YAW_SEQUENCE)
            .with_movement_pause(Duration::from_millis(500))
            .with_movement_hold(Duration::from_secs(5)),
    )
}

/// Connect to a Java-protocol server and observe a remote Player's three-step
/// LivingEntity position interpolation during the controlled movement trace.
pub fn connect_and_remote_player_interpolation_observer_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityAdd)
            .with_player_interpolation_observation()
            .with_drain_timeout(Duration::from_secs(5)),
    )
}

/// Connect to a Java-protocol server and observe a remote Player's three-step
/// delta-movement interpolation after a clientbound motion packet.
pub fn connect_and_remote_player_motion_observer_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityAdd)
            .with_player_motion_observation()
            .with_drain_timeout(Duration::from_secs(25)),
    )
}

/// Connect to a Java-protocol server and hold a remote Player connection open
/// while another client triggers a motion update.
pub fn connect_and_remote_player_motion_mover_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    const YAW_SEQUENCE: &[i16] = &[0];

    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::Movement)
            .with_movement_yaws(YAW_SEQUENCE)
            .with_movement_pause(Duration::from_millis(500))
            .with_movement_hold(Duration::from_secs(25)),
    )
}

/// Connect to a Java-protocol server and hold a remote Player connection open
/// while a boat passenger rotation scenario is assembled externally.
pub fn connect_and_remote_player_passenger_rotation_mover_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    const YAW_SEQUENCE: &[i16] = &[0];

    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::Movement)
            .with_movement_yaws(YAW_SEQUENCE)
            .with_movement_pause(Duration::from_millis(500))
            .with_movement_hold(Duration::from_secs(25)),
    )
}

/// Connect to a Java-protocol server and hold a remote Player connection open
/// while another client triggers a large position update.
pub fn connect_and_remote_player_position_snap_mover_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    const YAW_SEQUENCE: &[i16] = &[0];

    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::Movement)
            .with_movement_yaws(YAW_SEQUENCE)
            .with_movement_pause(Duration::from_millis(500))
            .with_movement_hold(Duration::from_secs(8)),
    )
}

/// Connect to a Java-protocol server and observe a remote Player's large
/// position update being applied as an immediate snap.
pub fn connect_and_remote_player_position_snap_observer_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityPosition)
            .with_player_position_snap_observation()
            .with_drain_timeout(Duration::from_secs(8)),
    )
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

/// Connect to a Java-protocol server, move across the configured view radius,
/// and observe entity ticking eligibility while client chunks unload.
pub fn connect_and_world_entity_ticking_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityTicking),
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

/// Connect to a Java-protocol server, keep the loaded world open, and observe a
/// clientbound entity-add packet without adding movement to the scenario.
pub fn connect_and_world_entity_add_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityAdd),
    )
}

/// Connect to a Java-protocol server, keep the loaded world open, and observe
/// clientbound entity removals without adding movement to the scenario.
pub fn connect_and_world_entity_remove_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityRemove),
    )
}

/// Connect to a Java-protocol server, keep the loaded world open, and observe
/// clientbound entity metadata without adding movement to the scenario.
pub fn connect_and_world_entity_metadata_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityMetadata),
    )
}

/// Connect to a Java-protocol server, keep the loaded world open, and observe
/// clientbound entity position synchronization without adding movement to the scenario.
pub fn connect_and_world_entity_position_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityPosition),
    )
}

/// Connect to a Java-protocol server and keep the loaded world open long enough
/// to observe an entity teleport packet sent to a riding player.
pub fn connect_and_world_entity_teleport_delta_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityPosition)
            .with_drain_timeout(Duration::from_secs(60)),
    )
}

/// Connect to a Java-protocol server and keep the loaded world open long enough
/// to observe clientbound passenger-list updates from an external ride command.
pub fn connect_and_world_entity_passengers_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityPassengers)
            .with_drain_timeout(Duration::from_secs(60)),
    )
}

/// Connect to a Java-protocol server, retain the received passenger relation,
/// and leave one local boat-control tick to the caller.
pub fn connect_and_world_entity_passenger_boat_control_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityPassengers)
            .with_drain_timeout(Duration::from_secs(30)),
    )
}
