use super::*;

/// Connect to a Java-protocol server, throw one item from a supplied survival
/// inventory stack with a stale container state id, and observe the full resync.
pub fn connect_and_survival_inventory_throw_one_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryThrowOneStateCorrection),
    )
}

/// Connect to a Java-protocol server, throw an entire supplied survival
/// inventory stack with a stale container state id, and observe the full resync.
pub fn connect_and_survival_inventory_throw_all_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryThrowAllStateCorrection),
    )
}

/// Connect to a Java-protocol server, start an empty-carried survival drag with
/// a stale container state id, and observe the no-op full resync.
pub fn connect_and_survival_inventory_drag_empty_cancel_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryDragEmptyCancelStateCorrection),
    )
}

/// Connect to a Java-protocol server, distribute a supplied survival stack over
/// two inventory slots with stale container state ids, and observe the resyncs.
pub fn connect_and_survival_inventory_drag_distribution_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryDragDistributionStateCorrection),
    )
}

/// Connect to a Java-protocol server, collect a second survival inventory stack
/// with a stale container state id, and observe the authoritative full resync.
pub fn connect_and_survival_inventory_double_click_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryDoubleClickStateCorrection),
    )
}

/// Connect to a Java-protocol server, collect a second survival inventory stack
/// up to the carried stack limit with a stale container state id, and observe
/// the authoritative full resync.
pub fn connect_and_survival_inventory_double_click_stack_limit_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryDoubleClickStackLimitStateCorrection),
    )
}

/// Connect to a Java-protocol server, pick up a supplied survival inventory
/// stack with an ahead-of-menu container state id, and observe the full resync.
pub fn connect_and_survival_inventory_ahead_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryAheadStateCorrection),
    )
}

/// Connect to a Java-protocol server, send a current inventory state id and
/// matching post-click hashes for a survival pickup, and observe no correction.
pub fn connect_and_survival_inventory_correct_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryCorrectPrediction),
    )
}

/// Connect to a Java-protocol server, send a current inventory state id with a
/// wrong post-click slot prediction and a correct carried prediction, and
/// observe the authoritative slot correction.
pub fn connect_and_survival_inventory_wrong_slot_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryWrongSlotPrediction),
    )
}

/// Connect to a Java-protocol server, send a current inventory state id with a
/// correct post-click slot prediction and a wrong carried prediction, and
/// observe the authoritative cursor correction.
pub fn connect_and_survival_inventory_wrong_carried_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryWrongCarriedPrediction),
    )
}

/// Connect to a Java-protocol server, send a current inventory state id with
/// both post-click predictions wrong, and observe the independent slot and
/// cursor corrections.
pub fn connect_and_survival_inventory_wrong_both_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryWrongBothPrediction),
    )
}

/// Connect to a Java-protocol server, pick up a component-bearing survival
/// stack with a wrong carried component hash, and observe the cursor correction.
pub fn connect_and_survival_inventory_component_hash_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryComponentHashPrediction),
    )
}

/// Connect to a Java-protocol server, pick up one table-selected component
/// stack with its Java-derived carried hash, and observe the server response.
pub fn connect_and_survival_inventory_item_component_conformance_probe(
    address: SocketAddr,
    username: &str,
    component_path: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    let Some(spec) = inventory_component_spec(component_path) else {
        return Err(ConnectionError::InvalidArgument("component_path"));
    };
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryItemComponentConformance(spec)),
    )
}

/// Connect to a Java-protocol server, pick up a survival stack without sending
/// its changed-slot hash, and observe the authoritative slot correction.
pub fn connect_and_survival_inventory_partial_changed_slot_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryPartialChangedSlotPrediction),
    )
}

/// Connect to a Java-protocol server, send correct pickup predictions with a
/// stale container state id, and observe the authoritative full resync.
pub fn connect_and_survival_inventory_stale_state_correct_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryStaleStateCorrectPrediction),
    )
}

/// Connect to a Java-protocol server, send correct pickup predictions with an
/// ahead container state id, and observe the authoritative full resync.
pub fn connect_and_survival_inventory_ahead_state_correct_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryAheadStateCorrectPrediction),
    )
}

/// Connect to a Java-protocol server, send a component-bearing pickup with an
/// ahead container state id and the exact component hash, and observe the full
/// resync.
pub fn connect_and_survival_inventory_ahead_component_correct_prediction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryAheadComponentCorrectPrediction),
    )
}

/// Connect to a Java-protocol server, shift-click a supplied survival inventory
/// stack with a stale container state id, and observe the authoritative full resync.
pub fn connect_and_survival_inventory_quick_move_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventoryQuickMoveStateCorrection),
    )
}

/// Connect to a Java-protocol server, swap a supplied survival hotbar stack into
/// the main inventory with a stale container state id, and observe the full resync.
pub fn connect_and_survival_inventory_swap_state_correction_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeConfig::new(ProbeAction::SurvivalInventorySwapStateCorrection),
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
