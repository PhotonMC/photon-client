use super::*;

/// Connect to a Java-protocol server, place a snowball stack in the creative hotbar,
/// select it and use the main hand to spawn a projectile.
pub fn connect_and_creative_snowball_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::CreativeSnowballUse),
    )
}

/// Connect to a Java-protocol server, spawn a snowball and apply one free-flight
/// client-world tick to the freshly added projectile.
pub fn connect_and_world_snowball_tick_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldSnowballTick)
            .with_drain_timeout(Duration::from_secs(2)),
    )
}

/// Connect to a Java-protocol server and apply one free-flight client-world
/// tick to a freshly added Arrow entity.
pub fn connect_and_world_arrow_tick_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldArrowTick)
            .with_drain_timeout(Duration::from_secs(30)),
    )
}

/// Connect to a Java-protocol server and apply two client-world ticks to a
/// freshly added Experience Orb entity.
pub fn connect_and_world_experience_orb_tick_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldExperienceOrbTick)
            .with_drain_timeout(Duration::from_secs(30)),
    )
}

/// Connect to a Java-protocol server and apply one free-flight client-world
/// tick to a freshly added primed TNT entity.
pub fn connect_and_world_primed_tnt_tick_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldPrimedTntTick)
            .with_drain_timeout(Duration::from_secs(30)),
    )
}

/// Connect to a Java-protocol server and apply one free-flight client-world
/// tick to a freshly added FallingBlock entity.
pub fn connect_and_world_falling_block_tick_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldFallingBlockTick)
            .with_drain_timeout(Duration::from_secs(30)),
    )
}

/// Connect to a Java-protocol server and apply one free-flight client-world
/// tick to a freshly added Firework Rocket entity.
pub fn connect_and_world_firework_tick_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::WorldFireworkTick)
            .with_drain_timeout(Duration::from_secs(30)),
    )
}
