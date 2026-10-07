use super::*;

/// Connect to a Java-protocol server and send a harmless block-action packet after loading.
pub fn connect_and_action_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::BlockAction),
    )
}

/// Connect to a Java-protocol server and break the block supporting the spawned player.
pub fn connect_and_block_break_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::BlockBreak),
    )
}

/// Connect to a Java-protocol server and send a block-break action outside interaction range.
pub fn connect_and_block_break_rejection_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::BlockBreakOutOfRange),
    )
}

/// Connect to a Java-protocol server, place a stone stack in the creative hotbar,
/// select it and request a side placement against the supporting block.
pub fn connect_and_creative_place_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::CreativePlace),
    )
}

/// Connect to a Java-protocol server, start breaking the supporting block in survival
/// mode and immediately abort the same action.
pub fn connect_and_survival_break_abort_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalBreakAbort),
    )
}

/// Connect to a Java-protocol server, break the supporting dirt block in survival mode
/// with a timed stop action and observe the authoritative air update.
pub fn connect_and_survival_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalBreakStop),
    )
}

/// Connect to a Java-protocol server, start breaking the supporting stone block in survival
/// mode and stop before the bare-hand hardness threshold is reached.
pub fn connect_and_survival_stone_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalStoneBreakStop),
    )
}

/// Connect to a Java-protocol server, start breaking the supporting oak-planks block in
/// survival mode and stop before the bare-hand hardness threshold is reached.
pub fn connect_and_survival_oak_planks_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalOakPlanksBreakStop),
    )
}

/// Connect to a Java-protocol server, select a wooden pickaxe supplied in hotbar slot 0,
/// and break the supporting stone block in survival mode.
pub fn connect_and_survival_stone_wooden_pickaxe_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalStoneWoodenPickaxeBreakStop),
    )
}

/// Connect to a Java-protocol server, select a wooden shovel supplied in hotbar slot 0,
/// and break the supporting dirt block in survival mode.
pub fn connect_and_survival_dirt_wooden_shovel_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalDirtWoodenShovelBreakStop),
    )
}

/// Connect to a Java-protocol server, select a wooden axe supplied in hotbar slot 0,
/// and break the supporting oak planks block in survival mode.
pub fn connect_and_survival_oak_planks_wooden_axe_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalOakPlanksWoodenAxeBreakStop),
    )
}

/// Connect to a Java-protocol server, stop a short survival dirt break early and
/// observe the server's delayed destroy completion on later ticks.
pub fn connect_and_survival_delayed_destroy_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalDelayedDestroy),
    )
}

/// Connect to a Java-protocol server, request a protected spawn-area block break
/// in survival mode and observe the denial message and sequence acknowledgement.
pub fn connect_and_survival_spawn_protection_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalSpawnProtection),
    )
}
