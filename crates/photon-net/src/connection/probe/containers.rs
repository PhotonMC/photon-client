use super::*;

/// Connect to a Java-protocol server, wait for an externally prepared container at
/// the supporting block, and observe its active menu state.
pub fn connect_and_world_player_container_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_world_player_menu_probe(address, username, timeout)
}

/// Connect to a Java-protocol server, wait for an externally prepared container at
/// the supporting block, and observe its active menu state.
pub fn connect_and_world_player_menu_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::with_use_item_on_delay(ProbeAction::UseItemOn, Duration::from_secs(20)),
    )
}
