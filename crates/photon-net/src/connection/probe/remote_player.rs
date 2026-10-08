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
