//! Scenario adapters used by the local real-server probes.
//!
//! The production connection pieces live in `protocol` and `session`; this module
//! keeps scenario-specific packet sequences and observations at the probe boundary.

use std::io;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use crate::binary::Uuid;
use crate::handshake::ClientIntent;
use crate::login::{ClientboundLoginFinishedPacket, ServerboundHelloPacket};
use crate::protocol::ConnectionProtocol;
use crate::resources::KnownPack;
use crate::wire;
use crate::world::{
    BlockEntityInput, ClientPlayerAbilities, ClientPlayerGameMode, DimensionBounds,
    DimensionEnvironment, DimensionInput, EntityInput, EntityMotionInput, EntityMoveInput,
    EntityPositionInput, EntityPositionUpdateMode, EntityRotationInput, EntityTeleportInput,
    LightLayerInput, LightUpdateInput, PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS,
    PLAYER_RESPAWN_KEEP_ENTITY_DATA, SectionInput, VANILLA_ARROW_ENTITY_TYPE_ID,
    VANILLA_COW_ENTITY_TYPE_ID, VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID,
    VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID, VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID,
    VANILLA_PRIMED_TNT_ENTITY_TYPE_ID, VANILLA_SNOWBALL_ENTITY_TYPE_ID,
};

use super::protocol::*;
use super::session::*;

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

fn send_world_chunk_unload_step(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayProbeConfig {
    action: ProbeAction,
    select_known_packs: bool,
    use_item_on_delay: Duration,
    movement_yaws: Option<&'static [i16]>,
    movement_pause: Duration,
    movement_hold: Duration,
    observe_player_interpolation: bool,
    observe_player_position_snap: bool,
    observe_player_motion: bool,
    drain_timeout_override: Option<Duration>,
}

impl PlayProbeConfig {
    const fn new(action: ProbeAction) -> Self {
        Self {
            action,
            select_known_packs: true,
            use_item_on_delay: Duration::ZERO,
            movement_yaws: None,
            movement_pause: Duration::ZERO,
            movement_hold: Duration::ZERO,
            observe_player_interpolation: false,
            observe_player_position_snap: false,
            observe_player_motion: false,
            drain_timeout_override: None,
        }
    }

    const fn without_known_packs(action: ProbeAction) -> Self {
        Self {
            action,
            select_known_packs: false,
            use_item_on_delay: Duration::ZERO,
            movement_yaws: None,
            movement_pause: Duration::ZERO,
            movement_hold: Duration::ZERO,
            observe_player_interpolation: false,
            observe_player_position_snap: false,
            observe_player_motion: false,
            drain_timeout_override: None,
        }
    }

    const fn with_use_item_on_delay(action: ProbeAction, delay: Duration) -> Self {
        Self {
            action,
            select_known_packs: true,
            use_item_on_delay: delay,
            movement_yaws: None,
            movement_pause: Duration::ZERO,
            movement_hold: Duration::ZERO,
            observe_player_interpolation: false,
            observe_player_position_snap: false,
            observe_player_motion: false,
            drain_timeout_override: None,
        }
    }

    const fn with_movement_yaws(mut self, yaws: &'static [i16]) -> Self {
        self.movement_yaws = Some(yaws);
        self
    }

    const fn with_movement_pause(mut self, pause: Duration) -> Self {
        self.movement_pause = pause;
        self
    }

    const fn with_movement_hold(mut self, hold: Duration) -> Self {
        self.movement_hold = hold;
        self
    }

    const fn with_player_interpolation_observation(mut self) -> Self {
        self.observe_player_interpolation = true;
        self
    }

    const fn with_player_position_snap_observation(mut self) -> Self {
        self.observe_player_position_snap = true;
        self
    }

    const fn with_player_motion_observation(mut self) -> Self {
        self.observe_player_motion = true;
        self
    }

    const fn with_drain_timeout(mut self, timeout: Duration) -> Self {
        self.drain_timeout_override = Some(timeout);
        self
    }
}

fn observe_player_interpolation(
    trace: &mut PlayConnectionTrace,
    entity_id: i32,
    before_position: (f64, f64, f64),
    before_rotation: (f32, f32),
) {
    if !trace.entity_interpolation_observations.is_empty() {
        return;
    }
    let Some(entity) = trace.world.entity_at(entity_id) else {
        return;
    };
    if entity.type_id != 156 {
        return;
    }
    let Some(target_position) = entity.interpolation_target_position() else {
        return;
    };
    let Some(target_rotation) = entity.interpolation_target_rotation() else {
        return;
    };
    if target_position == before_position && target_rotation == before_rotation {
        return;
    }

    let mut positions_after_ticks = Vec::with_capacity(3);
    let mut rotations_after_ticks = Vec::with_capacity(3);
    for _ in 0..3 {
        trace.world.tick();
        let Some(entity) = trace.world.entity_at(entity_id) else {
            return;
        };
        positions_after_ticks.push(entity.position());
        rotations_after_ticks.push((entity.rotation().0, entity.rotation().1));
    }
    let Some(entity) = trace.world.entity_at(entity_id) else {
        return;
    };
    trace
        .entity_interpolation_observations
        .push(EntityInterpolationObservation {
            entity_id,
            entity_type_id: entity.type_id,
            before_position,
            target_position,
            before_rotation,
            target_rotation,
            positions_after_ticks,
            rotations_after_ticks,
            tick_count: entity.tick_count(),
        });
}

fn observe_player_motion_interpolation(
    trace: &mut PlayConnectionTrace,
    entity_id: i32,
    before_motion: (f64, f64, f64),
    target_motion: (f64, f64, f64),
) {
    if !trace.entity_motion_interpolation_observations.is_empty() {
        return;
    }
    let Some(entity) = trace.world.entity_at(entity_id) else {
        return;
    };
    if entity.type_id != 156
        || before_motion == target_motion
        || entity.motion_interpolation_target() != Some(target_motion)
    {
        return;
    }

    let applied_motion = entity.motion();
    let mut motions_after_ticks = Vec::with_capacity(3);
    for _ in 0..3 {
        trace.world.tick();
        let Some(entity) = trace.world.entity_at(entity_id) else {
            return;
        };
        motions_after_ticks.push(entity.motion());
    }
    let Some(entity) = trace.world.entity_at(entity_id) else {
        return;
    };
    trace
        .entity_motion_interpolation_observations
        .push(EntityMotionInterpolationObservation {
            entity_id,
            entity_type_id: entity.type_id,
            before_motion,
            target_motion,
            applied_motion,
            motions_after_ticks,
            tick_count: entity.tick_count(),
        });
}

fn observe_player_position_snap(
    trace: &mut PlayConnectionTrace,
    packet_kind: &'static str,
    entity_id: i32,
    before_position: (f64, f64, f64),
    before_rotation: (f32, f32),
) {
    if !trace.entity_position_snap_observations.is_empty() {
        return;
    }
    let Some(entity) = trace.world.entity_at(entity_id) else {
        return;
    };
    if entity.type_id != 156 || entity.interpolation_steps() != 0 {
        return;
    }
    let target_position = entity.position();
    let delta_x = target_position.0 - before_position.0;
    let delta_y = target_position.1 - before_position.1;
    let delta_z = target_position.2 - before_position.2;
    if delta_x * delta_x + delta_y * delta_y + delta_z * delta_z <= 4096.0 {
        return;
    }
    let target_rotation = (entity.rotation().0, entity.rotation().1);
    trace
        .entity_position_snap_observations
        .push(EntityPositionSnapObservation {
            packet_kind,
            entity_id,
            entity_type_id: entity.type_id,
            before_position,
            target_position,
            before_rotation,
            target_rotation,
            applied_position: entity.position(),
            applied_rotation: entity.rotation(),
            previous_position: entity.previous_position(),
            previous_rotation: entity.previous_rotation(),
            interpolation_steps: entity.interpolation_steps(),
        });
}

fn dimension_bounds_for_spawn_info(
    trace: &PlayConnectionTrace,
    spawn_info: &SpawnInfoObservation,
) -> DimensionBounds {
    usize::try_from(spawn_info.dimension_type_id)
        .ok()
        .and_then(|id| trace.dimension_type_registry_bounds.get(id))
        .and_then(|bounds| *bounds)
        .unwrap_or_else(|| DimensionBounds::for_builtin_dimension(&spawn_info.dimension))
}

fn dimension_environment_for_spawn_info(
    trace: &PlayConnectionTrace,
    spawn_info: &SpawnInfoObservation,
) -> DimensionEnvironment {
    usize::try_from(spawn_info.dimension_type_id)
        .ok()
        .and_then(|id| trace.dimension_type_registry_environments.get(id))
        .and_then(|environment| environment.clone())
        .unwrap_or_else(|| DimensionEnvironment::for_builtin_dimension(&spawn_info.dimension))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeAction {
    Basic,
    Movement,
    WorldChunkUnload,
    WorldEntityTicking,
    WorldDimensionRespawn,
    WorldPlayerDeathRespawn,
    WorldDimensionTypeBounds,
    WorldLightUpdate,
    WorldBlockEntityUpdate,
    WorldEntityAdd,
    WorldEntityRemove,
    WorldEntityMetadata,
    WorldEntityPosition,
    WorldEntityPassengers,
    WorldEntityPassengerTick,
    WorldEntityPassengerRotation,
    WorldEntityPassengerAnimalRotation,
    BlockAction,
    BlockBreak,
    BlockBreakOutOfRange,
    UseItemOn,
    UseItem,
    CreativeSnowballUse,
    WorldSnowballTick,
    WorldArrowTick,
    WorldExperienceOrbTick,
    WorldPrimedTntTick,
    WorldFallingBlockTick,
    WorldFireworkTick,
    SurvivalSnowballUse,
    SurvivalWaterBucketUseOn,
    SurvivalEmptyBucketPickup,
    SurvivalMilkBucketUse,
    SurvivalHoneyBottleUse,
    SurvivalHoneyBottlePoisonCleanup,
    SurvivalHoneyBottleEarlyRelease,
    SurvivalPufferfishUse,
    SurvivalPufferfishFullHungerRejection,
    SurvivalGoldenAppleUse,
    SurvivalEnchantedGoldenAppleUse,
    SurvivalInventoryPickupStateCorrection,
    SurvivalInventorySecondaryPickupStateCorrection,
    SurvivalInventoryThrowOneStateCorrection,
    SurvivalInventoryThrowAllStateCorrection,
    SurvivalInventoryDragEmptyCancelStateCorrection,
    SurvivalInventoryDragDistributionStateCorrection,
    SurvivalInventoryDoubleClickStateCorrection,
    SurvivalInventoryDoubleClickStackLimitStateCorrection,
    SurvivalInventoryAheadStateCorrection,
    SurvivalInventoryCorrectPrediction,
    SurvivalInventoryWrongSlotPrediction,
    SurvivalInventoryWrongCarriedPrediction,
    SurvivalInventoryWrongBothPrediction,
    SurvivalInventoryComponentHashPrediction,
    SurvivalInventoryPartialChangedSlotPrediction,
    SurvivalInventoryStaleStateCorrectPrediction,
    SurvivalInventoryAheadStateCorrectPrediction,
    SurvivalInventoryAheadComponentCorrectPrediction,
    SurvivalInventoryItemComponentConformance(&'static InventoryComponentProbeSpec),
    SurvivalInventoryQuickMoveStateCorrection,
    SurvivalInventorySwapStateCorrection,
    CreativePlace,
    SurvivalBreakAbort,
    SurvivalBreakStop,
    SurvivalStoneBreakStop,
    SurvivalOakPlanksBreakStop,
    SurvivalStoneWoodenPickaxeBreakStop,
    SurvivalDirtWoodenShovelBreakStop,
    SurvivalOakPlanksWoodenAxeBreakStop,
    SurvivalDelayedDestroy,
    SurvivalSpawnProtection,
}

fn observe_world_entity_tick(
    trace: &mut PlayConnectionTrace,
    observation: &AddEntityObservation,
) -> Result<(), ConnectionError> {
    let entity = trace
        .world
        .entity_at(observation.entity_id)
        .ok_or(ConnectionError::Codec("world entity tick entity"))?;
    let before_position = entity.position();
    let before_motion = entity.motion();
    let before_rotation = entity.rotation();
    trace.world.tick();
    let entity = trace
        .world
        .entity_at(observation.entity_id)
        .ok_or(ConnectionError::Codec("world entity tick result"))?;
    trace
        .world_entity_tick_observations
        .push(WorldEntityTickObservation {
            entity_id: observation.entity_id,
            entity_type_id: observation.entity_type_id,
            before_position,
            after_position: entity.position(),
            before_motion,
            after_motion: entity.motion(),
            before_rotation,
            after_rotation: entity.rotation(),
            tick_count: entity.tick_count(),
        });
    Ok(())
}

fn observe_experience_orb_tick(
    trace: &mut PlayConnectionTrace,
    observation: &AddEntityObservation,
) -> Result<(), ConnectionError> {
    let entity = trace
        .world
        .entity_at(observation.entity_id)
        .ok_or(ConnectionError::Codec("experience orb tick entity"))?;
    let before_position = entity.position();
    let before_motion = entity.motion();
    trace.world.tick();
    let entity = trace
        .world
        .entity_at(observation.entity_id)
        .ok_or(ConnectionError::Codec("experience orb first tick result"))?;
    let after_first_tick_position = entity.position();
    let after_first_tick_motion = entity.motion();
    trace.world.tick();
    let entity = trace
        .world
        .entity_at(observation.entity_id)
        .ok_or(ConnectionError::Codec("experience orb second tick result"))?;
    trace
        .experience_orb_tick_observations
        .push(ExperienceOrbTickObservation {
            entity_id: observation.entity_id,
            entity_type_id: observation.entity_type_id,
            before_position,
            after_first_tick_position,
            after_second_tick_position: entity.position(),
            before_motion,
            after_first_tick_motion,
            after_second_tick_motion: entity.motion(),
            tick_count: entity.tick_count(),
        });
    Ok(())
}

fn connect_and_probe_with_mode(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
    probe: PlayProbeConfig,
) -> Result<PlayConnectionTrace, ConnectionError> {
    if username.is_empty() || username.encode_utf16().count() > crate::login::MAX_NAME_LENGTH {
        return Err(ConnectionError::InvalidArgument("username"));
    }
    if timeout.is_zero() {
        return Err(ConnectionError::InvalidArgument("timeout"));
    }

    let mut stream = connect_stream(address, timeout)?;
    send_handshake_intention(&mut stream, address, ClientIntent::Login)?;

    let hello = ServerboundHelloPacket::new(username.encode_utf16().collect(), Uuid::new(0, 1));
    send_packet(
        &mut stream,
        ConnectionProtocol::Login,
        LOGIN_HELLO_PACKET_ID,
        |payload| {
            hello
                .encode(payload)
                .map_err(|_| ConnectionError::Codec("login hello"))
        },
        None,
    )?;

    let started = Instant::now();
    let mut compression_threshold = None;
    let mut trace = PlayConnectionTrace::default();
    if matches!(probe.action, ProbeAction::CreativePlace) {
        trace.stone_item_registry_id = Some(VANILLA_STONE_ITEM_REGISTRY_ID);
    }
    if matches!(
        probe.action,
        ProbeAction::CreativeSnowballUse | ProbeAction::WorldSnowballTick
    ) {
        trace.snowball_item_registry_id = Some(VANILLA_SNOWBALL_ITEM_REGISTRY_ID);
    }

    loop {
        let body = read_packet(&mut stream, compression_threshold)?;
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Login,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            LOGIN_COMPRESSION_PACKET_ID => {
                let mut input = payload;
                let packet = crate::login::ClientboundLoginCompressionPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login compression"))?;
                compression_threshold = if packet.threshold < 0 {
                    None
                } else {
                    Some(packet.threshold as usize)
                };
                trace.compression_threshold = compression_threshold;
            }
            LOGIN_FINISHED_PACKET_ID => {
                let mut input = payload;
                ClientboundLoginFinishedPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login finished"))?;
                trace.login_finished = true;

                send_packet(
                    &mut stream,
                    ConnectionProtocol::Login,
                    LOGIN_ACKNOWLEDGED_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                send_configuration_start(&mut stream, compression_threshold)?;
                break;
            }
            0 => return Err(ConnectionError::ServerDisconnected),
            1 => {
                return Err(ConnectionError::UnsupportedPacket {
                    phase: ConnectionProtocol::Login,
                    id: packet_id,
                });
            }
            _ => {}
        }

        if started.elapsed() > timeout {
            return Err(ConnectionError::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "login timeout",
            )));
        }
    }

    let configuration_started = Instant::now();
    loop {
        let body = read_packet(&mut stream, compression_threshold)?;
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Configuration,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            CONFIG_REGISTRY_DATA_PACKET_ID => {
                let mut input = payload;
                let packet =
                    crate::registry_data::ClientboundRegistryDataPacket::decode(&mut input)
                        .map_err(|_| ConnectionError::Codec("registry data"))?;
                trace.registry_packet_count += 1;
                trace.registry_entry_count += packet.entries.len();
                if packet.registry.as_string() == "minecraft:dimension_type" {
                    for entry in packet.entries {
                        let bounds = entry
                            .dimension_type_bounds()
                            .map_err(|_| ConnectionError::Codec("dimension type registry"))?;
                        let environment = entry
                            .dimension_type_environment()
                            .map_err(|_| ConnectionError::Codec("dimension type environment"))?;
                        trace.dimension_type_registry_bounds.push(bounds);
                        trace.dimension_type_registry_environments.push(environment);
                    }
                }
            }
            CONFIG_UPDATE_TAGS_PACKET_ID => {
                let mut input = payload;
                let packet = crate::tags::ClientboundUpdateTagsPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("update tags"))?;
                trace.world.apply_update_tags_packet(&packet);
                trace.update_tags_packet_count += 1;
            }
            CONFIG_ENABLED_FEATURES_PACKET_ID => {
                let mut input = payload;
                let packet = crate::configuration::ClientboundUpdateEnabledFeaturesPacket::decode(
                    &mut input,
                )
                .map_err(|_| ConnectionError::Codec("enabled features"))?;
                trace.enabled_feature_count = packet.features.len();
            }
            CONFIG_SELECT_KNOWN_PACKS_PACKET_ID => {
                let mut input = payload;
                let packs = KnownPack::decode_list(&mut input, usize::MAX)
                    .map_err(|_| ConnectionError::Codec("clientbound known packs"))?;
                trace.known_pack_count = packs.len();
                let selected_packs = if probe.select_known_packs {
                    packs
                } else {
                    Vec::new()
                };
                trace.selected_known_pack_count = selected_packs.len();

                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_SERVERBOUND_SELECT_KNOWN_PACKS_PACKET_ID,
                    |output| {
                        KnownPack::encode_list(&selected_packs, output, MAX_KNOWN_PACKS)
                            .map_err(|_| ConnectionError::Codec("serverbound known packs"))
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration keep alive"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_KEEP_ALIVE_PACKET_ID,
                    |output| {
                        crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_PONG_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundPingPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration ping"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_PONG_PACKET_ID,
                    |output| {
                        crate::common::ServerboundPongPacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_CODE_OF_CONDUCT_PACKET_ID => {
                let mut input = payload;
                crate::configuration::ClientboundCodeOfConductPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("code of conduct"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_ACCEPT_CODE_OF_CONDUCT_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
            }
            CONFIG_FINISH_PACKET_ID => {
                crate::configuration::ClientboundFinishConfigurationPacket::decode(
                    &mut &payload[..],
                )
                .map_err(|_| ConnectionError::Codec("finish configuration"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_FINISH_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                trace.configuration_finished = true;
                break;
            }
            2 => return Err(ConnectionError::ServerDisconnected),
            9 | 11 => {
                return Err(ConnectionError::UnsupportedPacket {
                    phase: ConnectionProtocol::Configuration,
                    id: packet_id,
                });
            }
            _ => {}
        }

        if configuration_started.elapsed() > timeout {
            return Err(ConnectionError::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "configuration timeout",
            )));
        }
    }

    let play_started = Instant::now();
    let mut chunk_batch_started = false;
    let mut player_loaded_sent = false;
    let mut player_position = None;
    let mut creative_place_target_position = None;
    let mut creative_place_neighbor_position = None;
    let mut survival_water_bucket_target_position = None;
    let mut survival_water_bucket_neighbor_position = None;
    let mut world_chunk_unload_last_sent = None;
    let mut world_ready_drain_started = None;
    let mut death_respawn_command_sent = false;
    while play_started.elapsed() <= timeout {
        let body = match read_packet(&mut stream, compression_threshold) {
            Ok(body) => body,
            Err(ConnectionError::Io(error))
                if matches!(probe.action, ProbeAction::WorldPlayerDeathRespawn)
                    && world_ready_drain_started.is_none()
                    && matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
            {
                continue;
            }
            Err(ConnectionError::Io(error))
                if world_ready_drain_started.is_some()
                    && matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
            {
                if matches!(
                    probe.action,
                    ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
                ) && trace.movement_packets_sent < 96
                {
                    send_world_chunk_unload_step(
                        &mut stream,
                        &mut player_position,
                        &mut trace,
                        compression_threshold,
                        "world chunk unload timeout step position",
                    )?;
                    world_chunk_unload_last_sent = Some(Instant::now());
                    if trace.movement_packets_sent >= 96 {
                        stream.set_read_timeout(Some(WORLD_LIFECYCLE_DRAIN_TIMEOUT))?;
                    }
                    continue;
                }
                if matches!(
                    probe.action,
                    ProbeAction::SurvivalInventoryItemComponentConformance(_)
                ) && trace.container_click_packets_sent == 0
                {
                    return Err(ConnectionError::Codec("item component preparation"));
                }
                return Ok(trace);
            }
            Err(error) => return Err(error),
        };
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Play,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID => {
                let mut observation = parse_add_entity_packet(payload)?;
                observation.applied_to_world = trace.world.apply_entity(EntityInput {
                    entity_id: observation.entity_id,
                    uuid: observation.uuid,
                    type_id: observation.entity_type_id,
                    x: observation.x,
                    y: observation.y,
                    z: observation.z,
                    movement: observation.movement,
                    rotation: (observation.y_rot, observation.x_rot, observation.y_head_rot),
                    data: observation.data,
                });
                observation.applied_rotation = trace
                    .world
                    .entity_at(observation.entity_id)
                    .map(|entity| entity.rotation());
                let should_tick_world_entity = match probe.action {
                    ProbeAction::WorldSnowballTick => {
                        trace.use_item_packets_sent > 0
                            && observation.entity_type_id == VANILLA_SNOWBALL_ENTITY_TYPE_ID
                    }
                    ProbeAction::WorldArrowTick => {
                        observation.entity_type_id == VANILLA_ARROW_ENTITY_TYPE_ID
                    }
                    ProbeAction::WorldPrimedTntTick => {
                        observation.entity_type_id == VANILLA_PRIMED_TNT_ENTITY_TYPE_ID
                    }
                    ProbeAction::WorldFallingBlockTick => {
                        observation.entity_type_id == VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID
                    }
                    ProbeAction::WorldFireworkTick => {
                        observation.entity_type_id == VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID
                    }
                    _ => false,
                } && trace.world_entity_tick_observations.is_empty();
                if should_tick_world_entity {
                    observe_world_entity_tick(&mut trace, &observation)?;
                }
                if matches!(probe.action, ProbeAction::WorldExperienceOrbTick)
                    && observation.entity_type_id == VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID
                    && trace.experience_orb_tick_observations.is_empty()
                {
                    observe_experience_orb_tick(&mut trace, &observation)?;
                }
                trace.add_entity_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID => {
                let mut observation = parse_set_entity_motion_packet(payload)?;
                let motion_before = if probe.observe_player_motion {
                    trace
                        .world
                        .entity_at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| entity.motion())
                } else {
                    None
                };
                observation.applied_to_world = trace.world.apply_entity_motion(EntityMotionInput {
                    entity_id: observation.entity_id,
                    x: observation.x,
                    y: observation.y,
                    z: observation.z,
                });
                if observation.applied_to_world {
                    observation.applied_motion = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.motion());
                    if let Some(before_motion) = motion_before {
                        observe_player_motion_interpolation(
                            &mut trace,
                            observation.entity_id,
                            before_motion,
                            (observation.x, observation.y, observation.z),
                        );
                    }
                }
                trace.entity_motion_observations.push(observation);
            }
            PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID => {
                let mut observation = parse_rotate_head_packet(payload)?;
                observation.applied_to_world =
                    trace.world.apply_entity_rotation(EntityRotationInput {
                        entity_id: observation.entity_id,
                        y_rot: None,
                        x_rot: None,
                        y_head_rot: Some(observation.y_head_rot),
                        relative_y_rot: false,
                        relative_x_rot: false,
                    });
                if observation.applied_to_world {
                    observation.applied_head_rotation = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.rotation().2);
                }
                trace.entity_head_rotation_observations.push(observation);
            }
            PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID => {
                let mut observation = parse_remove_entities_packet(payload)?;
                observation.applied_entity_count =
                    trace.world.remove_entities(&observation.entity_ids);
                trace.remove_entities_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID => {
                let mut observation = parse_set_passengers_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_entity_passengers(observation.vehicle_id, &observation.passenger_ids);
                if observation.applied_to_world {
                    observation.applied_vehicle_passengers = trace
                        .world
                        .entity_passengers(observation.vehicle_id)
                        .map(<[i32]>::to_vec);
                    observation.applied_passenger_vehicles = observation
                        .passenger_ids
                        .iter()
                        .filter_map(|&passenger_id| {
                            trace.world.entity_passengers(passenger_id).map(|_| {
                                (passenger_id, trace.world.entity_vehicle_id(passenger_id))
                            })
                        })
                        .collect();
                }
                if matches!(
                    probe.action,
                    ProbeAction::WorldEntityPassengerTick
                        | ProbeAction::WorldEntityPassengerRotation
                        | ProbeAction::WorldEntityPassengerAnimalRotation
                ) && trace.entity_passenger_tick_observations.is_empty()
                    && observation.applied_to_world
                    && (probe.action != ProbeAction::WorldEntityPassengerAnimalRotation
                        || observation.passenger_ids.len() == 2)
                {
                    let passenger_id =
                        if probe.action == ProbeAction::WorldEntityPassengerAnimalRotation {
                            observation.passenger_ids.iter().copied().find(|&id| {
                                trace.world.entity_at(id).is_some_and(|entity| {
                                    entity.type_id == VANILLA_COW_ENTITY_TYPE_ID
                                })
                            })
                        } else {
                            observation
                                .passenger_ids
                                .iter()
                                .copied()
                                .find(|&id| trace.world.entity_at(id).is_some())
                        };
                    if let Some(passenger_id) = passenger_id
                        && let (Some(vehicle), Some(passenger)) = (
                            trace.world.entity_at(observation.vehicle_id),
                            trace.world.entity_at(passenger_id),
                        )
                    {
                        let before_vehicle_tick_count = vehicle.tick_count();
                        let before_passenger_tick_count = passenger.tick_count();
                        let before_passenger_position = passenger.position();
                        let before_passenger_motion = passenger.motion();
                        let before_passenger_rotation = passenger.rotation();
                        let before_passenger_body_y_rotation = passenger.body_y_rotation();
                        let passenger_type_id = passenger.type_id;
                        let tick_count =
                            if matches!(probe.action, ProbeAction::WorldEntityPassengerRotation) {
                                3
                            } else {
                                1
                            };
                        for _ in 0..tick_count {
                            trace.world.tick();
                        }
                        let vehicle = trace
                            .world
                            .entity_at(observation.vehicle_id)
                            .ok_or(ConnectionError::Codec("passenger tick vehicle"))?;
                        let passenger = trace
                            .world
                            .entity_at(passenger_id)
                            .ok_or(ConnectionError::Codec("passenger tick entity"))?;
                        trace.entity_passenger_tick_observations.push(
                            EntityPassengerTickObservation {
                                vehicle_id: observation.vehicle_id,
                                passenger_id,
                                passenger_type_id,
                                before_vehicle_tick_count,
                                after_vehicle_tick_count: vehicle.tick_count(),
                                before_passenger_tick_count,
                                after_passenger_tick_count: passenger.tick_count(),
                                before_passenger_position,
                                after_passenger_position: passenger.position(),
                                before_passenger_motion,
                                after_passenger_motion: passenger.motion(),
                                before_passenger_rotation,
                                after_passenger_rotation: passenger.rotation(),
                                before_passenger_body_y_rotation,
                                after_passenger_body_y_rotation: passenger.body_y_rotation(),
                            },
                        );
                    }
                }
                trace.entity_passengers_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID => {
                let mut observation = parse_entity_metadata_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_entity_metadata(observation.entity_id, &observation.metadata);
                trace.entity_metadata_observations.push(observation);
            }
            PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID => {
                let mut observation = parse_entity_position_sync_packet(payload)?;
                let snap_before = if probe.observe_player_position_snap {
                    trace
                        .world
                        .entity_at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world = trace.world.apply_entity_position_and_rotation(
                    EntityPositionInput {
                        entity_id: observation.entity_id,
                        x: observation.x,
                        y: observation.y,
                        z: observation.z,
                        relative_flags: 0,
                    },
                    EntityRotationInput {
                        entity_id: observation.entity_id,
                        y_rot: Some(observation.y_rot),
                        x_rot: Some(observation.x_rot),
                        y_head_rot: None,
                        relative_y_rot: false,
                        relative_x_rot: false,
                    },
                    EntityPositionUpdateMode::PositionSync,
                );
                if observation.applied_to_world {
                    trace
                        .world
                        .apply_entity_ground_state(observation.entity_id, observation.on_ground);
                    observation.applied_position = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.position());
                    observation.applied_rotation = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_on_ground = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if let Some((before_position, before_rotation)) = snap_before {
                        observe_player_position_snap(
                            &mut trace,
                            "position_sync",
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_position_sync_observations.push(observation);
            }
            PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID => {
                let mut observation = parse_entity_teleport_packet(payload)?;
                let motion_before = trace
                    .world
                    .entity_at(observation.entity_id)
                    .map(|entity| entity.motion());
                let snap_before = if probe.observe_player_position_snap {
                    trace
                        .world
                        .entity_at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world =
                    trace.world.apply_entity_teleport(EntityTeleportInput {
                        entity_id: observation.entity_id,
                        x: observation.x,
                        y: observation.y,
                        z: observation.z,
                        delta_x: observation.delta_x,
                        delta_y: observation.delta_y,
                        delta_z: observation.delta_z,
                        y_rot: observation.y_rot,
                        x_rot: observation.x_rot,
                        relative_flags: observation.relative_flags,
                    });
                if observation.applied_to_world {
                    observation.motion_before = motion_before;
                    trace
                        .world
                        .apply_entity_ground_state(observation.entity_id, observation.on_ground);
                    observation.applied_position = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.position());
                    observation.applied_rotation = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_motion = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.motion());
                    observation.applied_on_ground = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if let Some((before_position, before_rotation)) = snap_before {
                        observe_player_position_snap(
                            &mut trace,
                            "teleport",
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_teleport_observations.push(observation);
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID => {
                let mut observation = parse_move_entity_pos_packet(payload)?;
                let interpolation_before = if probe.observe_player_interpolation {
                    trace
                        .world
                        .entity_at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world = trace.world.apply_entity_move(EntityMoveInput {
                    entity_id: observation.entity_id,
                    delta_x: observation.delta_x,
                    delta_y: observation.delta_y,
                    delta_z: observation.delta_z,
                    has_position: observation.has_position,
                    y_rot: observation.y_rot,
                    x_rot: observation.x_rot,
                });
                if observation.applied_to_world {
                    trace
                        .world
                        .apply_entity_ground_state(observation.entity_id, observation.on_ground);
                    observation.applied_position = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.position());
                    observation.applied_rotation = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_on_ground = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if observation.has_position
                        && let Some((before_position, before_rotation)) = interpolation_before
                    {
                        observe_player_interpolation(
                            &mut trace,
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_move_observations.push(observation);
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID => {
                let mut observation = parse_move_entity_pos_rot_packet(payload)?;
                let interpolation_before = if probe.observe_player_interpolation {
                    trace
                        .world
                        .entity_at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world = trace.world.apply_entity_move(EntityMoveInput {
                    entity_id: observation.entity_id,
                    delta_x: observation.delta_x,
                    delta_y: observation.delta_y,
                    delta_z: observation.delta_z,
                    has_position: observation.has_position,
                    y_rot: observation.y_rot,
                    x_rot: observation.x_rot,
                });
                if observation.applied_to_world {
                    trace
                        .world
                        .apply_entity_ground_state(observation.entity_id, observation.on_ground);
                    observation.applied_position = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.position());
                    observation.applied_rotation = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_on_ground = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if observation.has_position
                        && let Some((before_position, before_rotation)) = interpolation_before
                    {
                        observe_player_interpolation(
                            &mut trace,
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_move_observations.push(observation);
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID => {
                let mut observation = parse_move_entity_rot_packet(payload)?;
                let interpolation_before = if probe.observe_player_interpolation {
                    trace
                        .world
                        .entity_at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world = trace.world.apply_entity_move(EntityMoveInput {
                    entity_id: observation.entity_id,
                    delta_x: observation.delta_x,
                    delta_y: observation.delta_y,
                    delta_z: observation.delta_z,
                    has_position: observation.has_position,
                    y_rot: observation.y_rot,
                    x_rot: observation.x_rot,
                });
                if observation.applied_to_world {
                    trace
                        .world
                        .apply_entity_ground_state(observation.entity_id, observation.on_ground);
                    observation.applied_rotation = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_on_ground = trace
                        .world
                        .entity_at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if observation.has_position
                        && let Some((before_position, before_rotation)) = interpolation_before
                    {
                        observe_player_interpolation(
                            &mut trace,
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_move_observations.push(observation);
            }
            PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID => {
                let mut observation = parse_open_screen_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_open_menu(
                    observation.container_id,
                    observation.menu_type_id,
                    &observation.title_nbt,
                );
                trace.open_screen_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID => {
                let mut observation = parse_container_close_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_local_player_container_close(observation.container_id);
                trace.container_close_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID => {
                let mut observation = parse_container_set_content_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_container_content(
                    observation.container_id,
                    observation.state_id,
                    &observation.items,
                    observation.carried_item.as_ref(),
                );
                trace.container_content_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID => {
                let mut observation = parse_container_set_slot_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_container_slot(
                    observation.container_id,
                    observation.state_id,
                    observation.slot,
                    observation.item.as_ref(),
                );
                trace.container_slot_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID => {
                let mut observation = parse_cursor_item_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_local_player_carried_item(observation.item.as_ref());
                trace.cursor_item_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID => {
                let mut observation = parse_set_player_inventory_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_local_player_inventory_slot(observation.slot, observation.item.as_ref());
                trace.player_inventory_observations.push(observation);
            }
            PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID => {
                let mut observation = parse_remove_mob_effect_packet(payload)?;
                observation.applied_to_world = trace.world.remove_local_player_mob_effect(
                    observation.entity_id,
                    observation.effect_registry_id,
                );
                trace.mob_effect_remove_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID => {
                trace
                    .system_chat_observations
                    .push(parse_system_chat_packet(payload)?);
            }
            PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID => {
                let mut input = payload;
                let sequence = read_varint(&mut input, "block changed ack sequence")?;
                if !input.is_empty() {
                    return Err(ConnectionError::Codec("block changed ack trailing payload"));
                }
                trace.block_changed_ack_sequences.push(sequence);
            }
            PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID => {
                trace
                    .block_destruction_observations
                    .push(parse_block_destruction_packet(payload)?);
            }
            PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID => {
                let mut observation = parse_block_entity_data_packet(payload)?;
                observation.applied_to_world =
                    trace.world.apply_block_entity_update(BlockEntityInput {
                        x: observation.x,
                        y: observation.y,
                        z: observation.z,
                        type_id: observation.type_id,
                        tag: observation.tag.as_deref(),
                    });
                trace.block_entity_observations.push(observation);
            }
            PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID => {
                let observation = parse_block_update_packet(payload)?;
                trace.world.update_block_state(
                    observation.x,
                    observation.y,
                    observation.z,
                    observation.state_id,
                );
                if creative_place_target_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.creative_place_target_update = Some(observation.clone());
                }
                if creative_place_neighbor_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.creative_place_neighbor_update = Some(observation.clone());
                }
                if survival_water_bucket_target_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.survival_water_bucket_target_update = Some(observation.clone());
                }
                if survival_water_bucket_neighbor_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.survival_water_bucket_neighbor_update = Some(observation.clone());
                }
                trace.block_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID => {
                let (x, z) = parse_forget_level_chunk_packet(payload)?;
                let removed_from_world = trace.world.unload_chunk(x, z);
                let tracked_entity_count = trace.world.loaded_entity_count();
                let ticking_entity_count = trace
                    .world
                    .entities()
                    .filter(|entity| entity.is_ticking())
                    .count();
                trace
                    .chunk_unload_observations
                    .push(ChunkUnloadObservation {
                        x,
                        z,
                        removed_from_world,
                        tracked_entity_count,
                        ticking_entity_count,
                        stopped_entity_count: tracked_entity_count - ticking_entity_count,
                    });
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID => {
                let radius = parse_set_chunk_cache_radius_packet(payload)?;
                trace.world.update_chunk_cache_radius(radius);
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID => {
                let (x, z) = parse_set_chunk_cache_center_packet(payload)?;
                trace.world.update_chunk_cache_center(x, z);
            }
            PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID => {
                let mut observation = parse_light_update_packet(payload)?;
                let application = trace.world.apply_light_updates(
                    observation.x,
                    observation.z,
                    LightUpdateInput {
                        sky: LightLayerInput {
                            mask: &observation.light_data.sky_mask,
                            empty_mask: &observation.light_data.empty_sky_mask,
                            updates: &observation.light_data.sky_updates,
                        },
                        block: LightLayerInput {
                            mask: &observation.light_data.block_mask,
                            empty_mask: &observation.light_data.empty_block_mask,
                            updates: &observation.light_data.block_updates,
                        },
                    },
                );
                if application.invalid_sky_updates != 0 || application.invalid_block_updates != 0 {
                    return Err(ConnectionError::Codec("light update data layer"));
                }
                observation.applied_sky_sections = application.applied_sky_sections;
                observation.applied_block_sections = application.applied_block_sections;
                observation.queued_for_chunk = application.queued;
                trace.light_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID => {
                chunk_batch_started = true;
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID => {
                let mut input = payload;
                wire::read_varint(&mut input)
                    .map_err(|_| ConnectionError::Codec("chunk batch finished"))?;
                trace.chunk_batch_finished_count += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID,
                    |output| {
                        // This is the initial value used by Java's chunk batch calculator:
                        // 7,000,000 ns / 2,000,000 ns per chunk.
                        output.extend_from_slice(&3.5_f32.to_be_bytes());
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            PLAY_CLIENTBOUND_LOGIN_PACKET_ID => {
                let observation = parse_login_packet(payload)?;
                trace.world.replace_dimension_with_environment(
                    DimensionInput {
                        key: &observation.spawn_info.dimension,
                        dimension_type_id: observation.spawn_info.dimension_type_id,
                        seed: observation.spawn_info.seed,
                        is_debug: observation.spawn_info.is_debug,
                        is_flat: observation.spawn_info.is_flat,
                        sea_level: observation.spawn_info.sea_level,
                    },
                    dimension_bounds_for_spawn_info(&trace, &observation.spawn_info),
                    dimension_environment_for_spawn_info(&trace, &observation.spawn_info),
                );
                trace.world.configure_chunk_cache(observation.chunk_radius);
                trace.world.set_local_player_id(observation.player_id);
                trace
                    .world
                    .apply_local_player_game_mode(ClientPlayerGameMode::from_id(i32::from(
                        observation.spawn_info.game_type,
                    )));
                trace.login_observation = Some(observation);
                trace.play_login = true;
            }
            PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID => {
                let packet = parse_player_position_packet(payload)?;
                let absolute = resolve_player_position(packet, player_position);
                let applied_to_world = trace.world.apply_local_player_position(
                    (absolute.0, absolute.1, absolute.2),
                    (packet.delta_x, packet.delta_y, packet.delta_z),
                    (absolute.3, absolute.4),
                );
                trace
                    .player_position_observations
                    .push(PlayerPositionObservation {
                        teleport_id: packet.teleport_id,
                        relative_flags: packet.relative_flags,
                        applied_to_world,
                    });
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID,
                    |output| {
                        wire::write_varint(packet.teleport_id, output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
                    |output| encode_move_player_pos_rot(output, absolute, 0),
                    compression_threshold,
                )?;
                player_position = Some(absolute);
            }
            PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID => {
                let mut observation = parse_set_health_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_health(
                    observation.health,
                    observation.food,
                    observation.saturation,
                );
                let is_dead = observation.health <= 0.0;
                trace.health_observations.push(observation);
                if matches!(probe.action, ProbeAction::WorldPlayerDeathRespawn)
                    && is_dead
                    && !death_respawn_command_sent
                {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
                        encode_perform_respawn,
                        compression_threshold,
                    )?;
                    death_respawn_command_sent = true;
                    trace.respawn_command_packets_sent += 1;
                    world_ready_drain_started = Some(Instant::now());
                    stream.set_read_timeout(Some(DIMENSION_RESPAWN_DRAIN_TIMEOUT))?;
                }
            }
            PLAY_CLIENTBOUND_RESPAWN_PACKET_ID => {
                let (spawn_info, data_to_keep) = parse_respawn_packet(payload)?;
                let cleared_chunk_count = trace.world.loaded_chunk_count();
                let cleared_entity_count = trace.world.loaded_entity_count();
                let dimension_changed = trace.world.replace_dimension_with_environment(
                    DimensionInput {
                        key: &spawn_info.dimension,
                        dimension_type_id: spawn_info.dimension_type_id,
                        seed: spawn_info.seed,
                        is_debug: spawn_info.is_debug,
                        is_flat: spawn_info.is_flat,
                        sea_level: spawn_info.sea_level,
                    },
                    dimension_bounds_for_spawn_info(&trace, &spawn_info),
                    dimension_environment_for_spawn_info(&trace, &spawn_info),
                );
                let local_player_recreated = trace.world.recreate_local_player(data_to_keep);
                trace
                    .world
                    .apply_local_player_game_mode(ClientPlayerGameMode::from_id(i32::from(
                        spawn_info.game_type,
                    )));
                let local_player_id = trace.world.local_player().map(|player| player.id());
                let local_player_generation =
                    trace.world.local_player().map(|player| player.generation());
                let local_player_attribute_count = trace
                    .world
                    .local_player()
                    .map(|player| player.attributes().len())
                    .unwrap_or(0);
                let local_player_attribute_modifier_count = trace
                    .world
                    .local_player()
                    .map(|player| {
                        player
                            .attributes()
                            .iter()
                            .map(|attribute| attribute.modifiers().len())
                            .sum()
                    })
                    .unwrap_or(0);
                trace.respawn_observations.push(RespawnObservation {
                    spawn_info,
                    data_to_keep,
                    local_player_id,
                    local_player_recreated,
                    local_player_generation,
                    keep_entity_data: data_to_keep & PLAYER_RESPAWN_KEEP_ENTITY_DATA != 0,
                    keep_attribute_modifiers: data_to_keep
                        & PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS
                        != 0,
                    local_player_attribute_count,
                    local_player_attribute_modifier_count,
                    dimension_changed,
                    cleared_chunk_count,
                    cleared_entity_count,
                    applied_to_world: true,
                });
            }
            PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID => {
                let mut observation = parse_update_attributes_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_local_player_attributes(observation.entity_id, &observation.attributes);
                trace.attribute_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID => {
                let packet = parse_player_abilities_packet(payload)?;
                let abilities = ClientPlayerAbilities::new(
                    packet.invulnerable,
                    packet.flying,
                    packet.can_fly,
                    packet.can_instabuild,
                    packet.flying_speed,
                    packet.walking_speed,
                );
                let applied_to_world = trace.world.apply_local_player_abilities(abilities);
                trace
                    .player_abilities_observations
                    .push(PlayerAbilitiesObservation {
                        packet,
                        applied_to_world,
                    });
            }
            PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID => {
                let packet = parse_game_event_packet(payload)?;
                let game_mode = (packet.event_id == GAME_EVENT_CHANGE_GAME_MODE_ID)
                    .then(|| ClientPlayerGameMode::from_game_event_param(packet.param));
                let applied_to_world = game_mode
                    .map(|game_mode| trace.world.apply_local_player_game_mode(game_mode))
                    .unwrap_or(false);
                let local_game_mode = trace.world.local_player().map(|player| player.game_mode());
                trace.game_event_observations.push(GameEventObservation {
                    packet,
                    applied_to_world,
                    local_game_mode,
                });
            }
            PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID => {
                let mut observation = parse_update_mob_effect_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_mob_effect(
                    observation.entity_id,
                    observation.effect_registry_id,
                    observation.amplifier,
                    observation.duration_ticks,
                    observation.flags,
                );
                trace.mob_effect_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => {
                let mut chunk = parse_chunk_packet(payload)?;
                trace.world.replace_chunk(
                    chunk.x,
                    chunk.z,
                    chunk
                        .section_observations
                        .iter()
                        .map(|section| SectionInput {
                            non_empty_block_count: section.non_empty_block_count,
                            fluid_count: section.fluid_count,
                            block_states: &section.block_state_values,
                            biomes: &section.biome_values,
                        }),
                );
                let applied_block_entities = trace.world.apply_block_entities(
                    chunk.x,
                    chunk.z,
                    chunk.block_entities.iter().map(|entity| BlockEntityInput {
                        x: entity.x,
                        y: entity.y,
                        z: entity.z,
                        type_id: entity.type_id,
                        tag: entity.tag.as_deref(),
                    }),
                );
                if applied_block_entities != chunk.block_entities.len() {
                    return Err(ConnectionError::Codec("chunk block entity application"));
                }
                for entity in &mut chunk.block_entities {
                    entity.applied_to_world = true;
                }
                let light_application = trace.world.apply_light_updates(
                    chunk.x,
                    chunk.z,
                    LightUpdateInput {
                        sky: LightLayerInput {
                            mask: &chunk.light_data.sky_mask,
                            empty_mask: &chunk.light_data.empty_sky_mask,
                            updates: &chunk.light_data.sky_updates,
                        },
                        block: LightLayerInput {
                            mask: &chunk.light_data.block_mask,
                            empty_mask: &chunk.light_data.empty_block_mask,
                            updates: &chunk.light_data.block_updates,
                        },
                    },
                );
                if light_application.invalid_sky_updates != 0
                    || light_application.invalid_block_updates != 0
                {
                    return Err(ConnectionError::Codec("chunk light data layer"));
                }
                trace.chunk_observations.push(chunk);
                trace.chunk_packet_count += 1;
            }
            PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("play keep alive"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID,
                    |output| {
                        crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            PLAY_CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID => {}
            _ => {}
        }

        if trace.play_login
            && trace.chunk_packet_count > 0
            && (!chunk_batch_started || trace.chunk_batch_finished_count > 0)
            && (!matches!(
                probe.action,
                ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
            ) || player_position.is_some())
            && world_ready_drain_started.is_none()
        {
            if !player_loaded_sent {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID,
                    |_output| Ok(()),
                    compression_threshold,
                )?;
                player_loaded_sent = true;
                trace.player_loaded_sent = true;
            }
            if matches!(probe.action, ProbeAction::Movement) && trace.movement_packets_sent == 0 {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("movement probe player position"));
                };
                if let Some(yaws) = probe.movement_yaws {
                    for (index, yaw) in yaws.iter().copied().enumerate() {
                        if !probe.movement_pause.is_zero() {
                            std::thread::sleep(probe.movement_pause);
                        }
                        let moved = (
                            position.0 + 0.25 * (index as f64 + 1.0),
                            position.1,
                            position.2,
                            f32::from(yaw),
                            position.4,
                        );
                        send_packet(
                            &mut stream,
                            ConnectionProtocol::Play,
                            PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
                            |output| encode_move_player_pos_rot(output, moved, 1),
                            compression_threshold,
                        )?;
                        trace.movement_packets_sent += 1;
                        player_position = Some(moved);
                    }
                    if !probe.movement_hold.is_zero() {
                        std::thread::sleep(probe.movement_hold);
                    }
                } else {
                    for offset in [0.25_f64, 0.5, 0.75] {
                        let moved = (
                            position.0 + offset,
                            position.1,
                            position.2,
                            position.3,
                            position.4,
                        );
                        send_packet(
                            &mut stream,
                            ConnectionProtocol::Play,
                            PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
                            |output| encode_move_player_pos_rot(output, moved, 1),
                            compression_threshold,
                        )?;
                        trace.movement_packets_sent += 1;
                        player_position = Some(moved);
                    }
                }
            }
            if matches!(
                probe.action,
                ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
            ) && trace.movement_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(1));
                send_world_chunk_unload_step(
                    &mut stream,
                    &mut player_position,
                    &mut trace,
                    compression_threshold,
                    "world chunk unload initial step position",
                )?;
                world_chunk_unload_last_sent = Some(Instant::now());
            }
            if matches!(probe.action, ProbeAction::BlockAction)
                && trace.player_action_packets_sent == 0
            {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_abort_destroy_block(output, (0, 0, 0), 1),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::BlockBreak)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("block break probe player position"));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 2),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::BlockBreakOutOfRange)
                && trace.player_action_packets_sent == 0
            {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, (1_000_000, 64, 1_000_000), 3),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalBreakAbort)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival break probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 6),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_abort_destroy_block(output, target, 7),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalBreakStop
                    | ProbeAction::SurvivalStoneBreakStop
                    | ProbeAction::SurvivalOakPlanksBreakStop
            ) && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival break stop probe player position",
                    ));
                };
                let target = support_block_position(position);
                let (start_sequence, stop_sequence) =
                    if matches!(probe.action, ProbeAction::SurvivalStoneBreakStop) {
                        (12, 13)
                    } else if matches!(probe.action, ProbeAction::SurvivalOakPlanksBreakStop) {
                        (18, 19)
                    } else {
                        (8, 9)
                    };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, start_sequence),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(650));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, stop_sequence),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalStoneWoodenPickaxeBreakStop
            ) && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival wooden pickaxe probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 20),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(2_200));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 21),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalDirtWoodenShovelBreakStop)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival wooden shovel probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 30),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(650));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 31),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalOakPlanksWoodenAxeBreakStop
            ) && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival wooden axe probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 32),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(1_200));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 33),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalDelayedDestroy)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival delayed destroy probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 14),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(100));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 15),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalSpawnProtection)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival spawn protection probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 16),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::UseItemOn) && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("use item on probe player position"));
                };
                std::thread::sleep(probe.use_item_on_delay);
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on(output, target, 4),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::UseItem) && trace.use_item_packets_sent == 0 {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 10, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalSnowballUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 22, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalWaterBucketUseOn)
                && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival water bucket probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                survival_water_bucket_target_position = Some(target);
                survival_water_bucket_neighbor_position = Some((target.0, target.1 + 1, target.2));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 1, (0.5, 1.0, 0.5), 23),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 24, 0.0, 90.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalEmptyBucketPickup)
                && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival empty bucket pickup probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let support = support_block_position(position);
                let target = (support.0, support.1 + 1, support.2);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 1, (0.5, 1.0, 0.5), 25),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 26, 0.0, 90.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalMilkBucketUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 27, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalHoneyBottleUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 28, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalHoneyBottlePoisonCleanup)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 29, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalHoneyBottleEarlyRelease)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 34, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(250));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    encode_release_use_item,
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            let consumable_use_sequence = match probe.action {
                ProbeAction::SurvivalPufferfishUse => Some(35),
                ProbeAction::SurvivalPufferfishFullHungerRejection => Some(38),
                ProbeAction::SurvivalGoldenAppleUse => Some(36),
                ProbeAction::SurvivalEnchantedGoldenAppleUse => Some(37),
                _ => None,
            };
            if let Some(sequence) =
                consumable_use_sequence.filter(|_| trace.use_item_packets_sent == 0)
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    move |output| encode_use_item(output, sequence, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryPickupStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventorySecondaryPickupStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_secondary_pickup_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryThrowOneStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_throw_one_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryThrowAllStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(30));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_throw_all_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryDragEmptyCancelStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_drag_empty_cancel_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryDragDistributionStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                for (slot, button, input_id) in [
                    (36_i16, 0_u8, 0_i32),
                    (-999_i16, 0_u8, 5_i32),
                    (9_i16, 1_u8, 5_i32),
                    (36_i16, 1_u8, 5_i32),
                    (-999_i16, 2_u8, 5_i32),
                ] {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                        |output| {
                            encode_inventory_click_with_input(output, 0, slot, button, input_id)
                        },
                        compression_threshold,
                    )?;
                    trace.container_click_packets_sent += 1;
                }
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryDoubleClickStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                for (slot, button, input_id) in [(36_i16, 0_u8, 0_i32), (10_i16, 0_u8, 6_i32)] {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                        |output| {
                            encode_inventory_click_with_input(output, 0, slot, button, input_id)
                        },
                        compression_threshold,
                    )?;
                    trace.container_click_packets_sent += 1;
                }
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryDoubleClickStackLimitStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                for (slot, button, input_id) in [(36_i16, 0_u8, 0_i32), (10_i16, 0_u8, 6_i32)] {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                        |output| {
                            encode_inventory_click_with_input(output, 0, slot, button, input_id)
                        },
                        compression_threshold,
                    )?;
                    trace.container_click_packets_sent += 1;
                }
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryAheadStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_click_with_input(output, 99, 36, 0, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(99);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryCorrectPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_prediction(output, 2, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryWrongSlotPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_wrong_slot_prediction(output, 2, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryWrongCarriedPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_wrong_carried_prediction(output, 2),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryWrongBothPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_wrong_both_prediction(output, 2, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryComponentHashPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| {
                        encode_inventory_pickup_with_component_hash_prediction(
                            output,
                            2,
                            1,
                            VANILLA_MILK_BUCKET_ITEM_REGISTRY_ID,
                        )
                    },
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryPartialChangedSlotPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| {
                        encode_inventory_pickup_with_partial_changed_slot_prediction(
                            output,
                            2,
                            8,
                            VANILLA_STONE_ITEM_REGISTRY_ID,
                        )
                    },
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryStaleStateCorrectPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_prediction(output, 0, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryAheadStateCorrectPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_prediction(output, 99, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(99);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryAheadComponentCorrectPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| {
                        let spec = inventory_component_spec("custom_model_data")
                            .expect("custom model data component is in the conformance table");
                        encode_inventory_pickup_with_component_spec_prediction(output, 99, spec)
                    },
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(99);
            }
            if let ProbeAction::SurvivalInventoryItemComponentConformance(spec) = probe.action
                && trace.container_click_packets_sent == 0
            {
                let prepared_state_id = trace
                    .container_slot_observations
                    .iter()
                    .rev()
                    .find(|observation| {
                        observation.container_id == 0
                            && observation.slot == 36
                            && observation.item_count == Some(1)
                            && observation.item_registry_id == Some(spec.item_registry_id)
                            && observation.positive_component_type_ids.as_slice()
                                == spec.expected_positive_component_type_ids
                            && observation.negative_component_type_ids.is_empty()
                    })
                    .map(|observation| observation.state_id);
                if let Some(prepared_state_id) = prepared_state_id {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                        |output| {
                            encode_inventory_pickup_with_component_spec_prediction(
                                output,
                                prepared_state_id,
                                spec,
                            )
                        },
                        compression_threshold,
                    )?;
                    trace.container_click_packets_sent += 1;
                    trace.container_click_state_id_sent = Some(prepared_state_id);
                }
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryQuickMoveStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_quick_move_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventorySwapStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_swap_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::CreativeSnowballUse | ProbeAction::WorldSnowballTick
            ) && trace.creative_mode_slot_packets_sent == 0
            {
                let Some(snowball_item_registry_id) = trace.snowball_item_registry_id else {
                    return Err(ConnectionError::Codec(
                        "creative snowball use item registry id",
                    ));
                };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                    |output| encode_creative_item_slot(output, 36, 1, snowball_item_registry_id),
                    compression_threshold,
                )?;
                trace.creative_mode_slot_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 11, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::CreativePlace)
                && trace.creative_mode_slot_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "creative place probe player position",
                    ));
                };
                let Some(stone_item_registry_id) = trace.stone_item_registry_id else {
                    return Err(ConnectionError::Codec(
                        "creative place stone item registry id",
                    ));
                };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                    |output| encode_creative_stone_slot(output, 36, stone_item_registry_id),
                    compression_threshold,
                )?;
                trace.creative_mode_slot_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;

                let target = support_block_position(position);
                creative_place_target_position = Some(target);
                creative_place_neighbor_position = Some((target.0, target.1, target.2 - 1));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 2, (0.5, 0.5, 0.0), 5),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::WorldPlayerDeathRespawn) {
                stream.set_read_timeout(Some(Duration::from_millis(100)))?;
            } else {
                world_ready_drain_started = Some(Instant::now());
                let drain_timeout = probe.drain_timeout_override.unwrap_or(match probe.action {
                    ProbeAction::SurvivalDelayedDestroy => DELAYED_DESTROY_DRAIN_TIMEOUT,
                    ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking => {
                        WORLD_LIFECYCLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::WorldDimensionRespawn | ProbeAction::WorldDimensionTypeBounds => {
                        DIMENSION_RESPAWN_DRAIN_TIMEOUT
                    }
                    ProbeAction::WorldLightUpdate => LIGHT_UPDATE_DRAIN_TIMEOUT,
                    ProbeAction::WorldBlockEntityUpdate => BLOCK_ENTITY_UPDATE_DRAIN_TIMEOUT,
                    ProbeAction::WorldEntityAdd => ENTITY_ADD_DRAIN_TIMEOUT,
                    ProbeAction::WorldEntityRemove => ENTITY_REMOVE_DRAIN_TIMEOUT,
                    ProbeAction::WorldEntityMetadata => ENTITY_METADATA_DRAIN_TIMEOUT,
                    ProbeAction::WorldEntityPosition => ENTITY_POSITION_SYNC_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalWaterBucketUseOn
                    | ProbeAction::SurvivalEmptyBucketPickup => WATER_BUCKET_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalMilkBucketUse => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalHoneyBottleUse => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalHoneyBottlePoisonCleanup => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalPufferfishUse
                    | ProbeAction::SurvivalPufferfishFullHungerRejection
                    | ProbeAction::SurvivalGoldenAppleUse
                    | ProbeAction::SurvivalEnchantedGoldenAppleUse => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryPickupStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventorySecondaryPickupStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryThrowOneStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryThrowAllStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryDragEmptyCancelStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryDragDistributionStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryDoubleClickStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryDoubleClickStackLimitStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryAheadStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryCorrectPrediction => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryWrongSlotPrediction => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryWrongCarriedPrediction => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryWrongBothPrediction => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryComponentHashPrediction => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryItemComponentConformance(_) => {
                        COMPONENT_PREPARATION_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryQuickMoveStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventorySwapStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
                    _ => WORLD_READY_DRAIN_TIMEOUT,
                });
                let read_timeout = if matches!(
                    probe.action,
                    ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
                ) && trace.movement_packets_sent < 96
                {
                    Duration::from_millis(100)
                } else {
                    drain_timeout
                };
                stream.set_read_timeout(Some(read_timeout))?;
            }
        }

        if let ProbeAction::SurvivalInventoryItemComponentConformance(spec) = probe.action
            && trace.container_click_packets_sent == 0
        {
            let prepared_state_id = trace
                .container_slot_observations
                .iter()
                .rev()
                .find(|observation| {
                    observation.container_id == 0
                        && observation.slot == 36
                        && observation.item_count == Some(1)
                        && observation.item_registry_id == Some(spec.item_registry_id)
                        && observation.positive_component_type_ids.as_slice()
                            == spec.expected_positive_component_type_ids
                        && observation.negative_component_type_ids.is_empty()
                })
                .map(|observation| observation.state_id);
            if let Some(prepared_state_id) = prepared_state_id {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| {
                        encode_inventory_pickup_with_component_spec_prediction(
                            output,
                            prepared_state_id,
                            spec,
                        )
                    },
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(prepared_state_id);
            }
        }

        if matches!(
            probe.action,
            ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
        ) && world_ready_drain_started.is_some()
            && trace.movement_packets_sent < 96
            && world_chunk_unload_last_sent
                .is_none_or(|last_sent| last_sent.elapsed() >= Duration::from_millis(50))
        {
            send_world_chunk_unload_step(
                &mut stream,
                &mut player_position,
                &mut trace,
                compression_threshold,
                "world chunk unload packet step position",
            )?;
            world_chunk_unload_last_sent = Some(Instant::now());
            if trace.movement_packets_sent >= 96 {
                stream.set_read_timeout(Some(WORLD_LIFECYCLE_DRAIN_TIMEOUT))?;
            }
        }

        let drain_timeout = probe.drain_timeout_override.unwrap_or(match probe.action {
            ProbeAction::SurvivalDelayedDestroy => DELAYED_DESTROY_DRAIN_TIMEOUT,
            ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking => {
                WORLD_LIFECYCLE_DRAIN_TIMEOUT
            }
            ProbeAction::WorldDimensionRespawn
            | ProbeAction::WorldPlayerDeathRespawn
            | ProbeAction::WorldDimensionTypeBounds => DIMENSION_RESPAWN_DRAIN_TIMEOUT,
            ProbeAction::WorldLightUpdate => LIGHT_UPDATE_DRAIN_TIMEOUT,
            ProbeAction::WorldBlockEntityUpdate => BLOCK_ENTITY_UPDATE_DRAIN_TIMEOUT,
            ProbeAction::WorldEntityAdd => ENTITY_ADD_DRAIN_TIMEOUT,
            ProbeAction::WorldEntityRemove => ENTITY_REMOVE_DRAIN_TIMEOUT,
            ProbeAction::WorldEntityMetadata => ENTITY_METADATA_DRAIN_TIMEOUT,
            ProbeAction::WorldEntityPosition => ENTITY_POSITION_SYNC_DRAIN_TIMEOUT,
            ProbeAction::SurvivalWaterBucketUseOn | ProbeAction::SurvivalEmptyBucketPickup => {
                WATER_BUCKET_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalMilkBucketUse => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalHoneyBottleUse => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalHoneyBottlePoisonCleanup => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalPufferfishUse
            | ProbeAction::SurvivalPufferfishFullHungerRejection
            | ProbeAction::SurvivalGoldenAppleUse
            | ProbeAction::SurvivalEnchantedGoldenAppleUse => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryPickupStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventorySecondaryPickupStateCorrection => {
                CONSUMABLE_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryThrowOneStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryThrowAllStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryDragEmptyCancelStateCorrection => {
                CONSUMABLE_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryDragDistributionStateCorrection => {
                CONSUMABLE_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryDoubleClickStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryDoubleClickStackLimitStateCorrection => {
                CONSUMABLE_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryAheadStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryCorrectPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryWrongSlotPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryWrongCarriedPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryWrongBothPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryComponentHashPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryItemComponentConformance(_) => {
                COMPONENT_PREPARATION_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryQuickMoveStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventorySwapStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            _ => WORLD_READY_DRAIN_TIMEOUT,
        });
        if world_ready_drain_started.is_some_and(|started| started.elapsed() >= drain_timeout) {
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryItemComponentConformance(_)
            ) && trace.container_click_packets_sent == 0
            {
                return Err(ConnectionError::Codec("item component preparation"));
            }
            return Ok(trace);
        }
    }

    Err(ConnectionError::Io(io::Error::new(
        io::ErrorKind::TimedOut,
        "play world did not arrive before timeout",
    )))
}
