use super::*;

/// Connect to a Java-protocol server and use the empty main hand on the supporting block.
pub fn connect_and_use_item_on_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::UseItemOn),
    )
}

/// Connect to a Java-protocol server and use the empty main hand in open space.
pub fn connect_and_use_item_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::UseItem),
    )
}

/// Connect to a Java-protocol server, select a supplied survival snowball stack and use it.
pub fn connect_and_survival_snowball_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalSnowballUse),
    )
}

/// Connect to a Java-protocol server, select a supplied survival water bucket and use it
/// on the upper face of the supporting block.
pub fn connect_and_survival_water_bucket_use_on_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalWaterBucketUseOn),
    )
}

/// Connect to a Java-protocol server, select a supplied survival empty bucket and use it
/// to pick up a water source beside the supporting block.
pub fn connect_and_survival_empty_bucket_pickup_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalEmptyBucketPickup),
    )
}

/// Connect to a Java-protocol server, use a component-bearing survival milk bucket,
/// and observe its completed bucket remainder.
pub fn connect_and_survival_milk_bucket_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalMilkBucketUse),
    )
}

/// Connect to a Java-protocol server, use a component-bearing survival honey bottle,
/// and observe its completed glass-bottle remainder.
pub fn connect_and_survival_honey_bottle_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalHoneyBottleUse),
    )
}

/// Connect to a Java-protocol server, use a component-bearing survival honey bottle
/// while poison is active, and observe the completed glass-bottle remainder and the
/// authoritative mob-effect removal packet.
pub fn connect_and_survival_honey_bottle_poison_cleanup_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalHoneyBottlePoisonCleanup),
    )
}

/// Connect to a Java-protocol server, start using a component-bearing survival honey bottle,
/// and release it before the consumable duration completes.
pub fn connect_and_survival_honey_bottle_early_release_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalHoneyBottleEarlyRelease),
    )
}

/// Connect to a Java-protocol server, consume a survival pufferfish, and observe its effects.
pub fn connect_and_survival_pufferfish_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalPufferfishUse),
    )
}

/// Connect to a Java-protocol server, attempt a full-hunger survival pufferfish use,
/// and observe the rejection without consuming the stack.
pub fn connect_and_survival_pufferfish_full_hunger_rejection_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalPufferfishFullHungerRejection),
    )
}

/// Connect to a Java-protocol server, consume a survival golden apple, and observe its effects.
pub fn connect_and_survival_golden_apple_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalGoldenAppleUse),
    )
}

/// Connect to a Java-protocol server, consume a survival enchanted golden apple, and observe its effects.
pub fn connect_and_survival_enchanted_golden_apple_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalEnchantedGoldenAppleUse),
    )
}
