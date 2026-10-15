use super::*;

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

/// Connect to a Java-protocol server and observe one world tick where a known
/// projectile is attached as a passenger.
pub fn connect_and_world_entity_passenger_tick_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityPassengerTick)
            .with_drain_timeout(Duration::from_secs(60)),
    )
}

/// Connect to a Java-protocol server and observe a remote Player passenger
/// after the three client ticks used by its rotation interpolation handler.
pub fn connect_and_world_entity_passenger_rotation_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityPassengerRotation)
            .with_drain_timeout(Duration::from_secs(60)),
    )
}

/// Connect to a Java-protocol server and observe a full two-passenger boat
/// tick where the first passenger is a Cow.
pub fn connect_and_world_entity_passenger_animal_rotation_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldEntityPassengerAnimalRotation)
            .with_drain_timeout(Duration::from_secs(60)),
    )
}
