use super::*;

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

/// Connect to a Java-protocol server, pick up a supplied survival inventory stack
/// with a stale container state id, and observe the authoritative full resync.
pub fn connect_and_survival_inventory_pickup_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryPickupStateCorrection),
    )
}

/// Connect to a Java-protocol server, pick up half of a supplied survival
/// inventory stack with a stale container state id, and observe the full resync.
pub fn connect_and_survival_inventory_secondary_pickup_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventorySecondaryPickupStateCorrection),
    )
}
