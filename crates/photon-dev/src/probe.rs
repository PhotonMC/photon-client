//! Scenario adapters used by the local real-server probes.
//!
//! The production connection pieces live in `protocol` and `session`; this module
//! keeps scenario-specific packet sequences and observations at the probe boundary.

use photon_net::connection::Transport;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use photon_core::wire;
use photon_protocol::phase::ConnectionProtocol;
use photon_world::{
    BlockEntityInput, ClientPlayerAbilities, ClientPlayerGameMode, DimensionBounds,
    DimensionEnvironment, DimensionInput, LightLayerInput, LightUpdateInput,
    PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS, PLAYER_RESPAWN_KEEP_ENTITY_DATA, SectionInput,
    VANILLA_ARROW_ENTITY_TYPE_ID, VANILLA_COW_ENTITY_TYPE_ID,
    VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID, VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID,
    VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID, VANILLA_PRIMED_TNT_ENTITY_TYPE_ID,
    VANILLA_SNOWBALL_ENTITY_TYPE_ID,
};

use crate::trace::*;

/// `handleSetEntityData` on the probe's world.
fn apply_metadata(
    world: &mut photon_world::ClientWorld,
    observation: &photon_protocol::play::EntityMetadataObservation,
) -> bool {
    entities::metadata(world, observation, &registries())
}

/// The numbering of the version the probe speaks.
fn registries() -> std::sync::Arc<photon_data::version_registries::VersionRegistries> {
    photon_data::version_registries::VersionRegistries::of(photon_net::connection::connect_version())
}

/// The id the probe's version puts on the wire for a canonical item.
fn wire_item(item: photon_core::ids::ItemId) -> photon_core::ids::ItemId {
    photon_core::ids::ItemId(registries().items.wire_or_unknown(item.0))
}

use photon_net::connection::entities;
use photon_net::connection::{read_packet, send_packet};
use photon_protocol::play::*;
use photon_protocol::{common, login};

/// The player's entity type, read from the registry while compiling.
pub(crate) const PLAYER_ENTITY_TYPE: photon_core::ids::EntityTypeId =
    photon_data::entity_type::named("minecraft:player");

/// The stone item, read from the registry while compiling.
pub(crate) const STONE: photon_core::ids::ItemId =
    photon_data::item::registry::named("minecraft:stone");

mod observe;
pub(crate) use observe::*;
mod mode;
use mode::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PlayProbeConfig {
    pub(super) action: ProbeAction,
    pub(super) select_known_packs: bool,
    pub(super) use_item_on_delay: Duration,
    pub(super) movement_yaws: Option<&'static [i16]>,
    pub(super) movement_pause: Duration,
    pub(super) movement_hold: Duration,
    pub(super) observe_player_interpolation: bool,
    pub(super) observe_player_position_snap: bool,
    pub(super) observe_player_motion: bool,
    pub(super) drain_timeout_override: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProbeAction {
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

impl PlayProbeConfig {
    pub(super) const fn new(action: ProbeAction) -> Self {
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

    pub(super) const fn without_known_packs(action: ProbeAction) -> Self {
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

    pub(super) const fn with_use_item_on_delay(action: ProbeAction, delay: Duration) -> Self {
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

    pub(super) const fn with_movement_yaws(mut self, yaws: &'static [i16]) -> Self {
        self.movement_yaws = Some(yaws);
        self
    }

    pub(super) const fn with_movement_pause(mut self, pause: Duration) -> Self {
        self.movement_pause = pause;
        self
    }

    pub(super) const fn with_movement_hold(mut self, hold: Duration) -> Self {
        self.movement_hold = hold;
        self
    }

    pub(super) const fn with_player_interpolation_observation(mut self) -> Self {
        self.observe_player_interpolation = true;
        self
    }

    pub(super) const fn with_player_position_snap_observation(mut self) -> Self {
        self.observe_player_position_snap = true;
        self
    }

    pub(super) const fn with_player_motion_observation(mut self) -> Self {
        self.observe_player_motion = true;
        self
    }

    pub(super) const fn with_drain_timeout(mut self, timeout: Duration) -> Self {
        self.drain_timeout_override = Some(timeout);
        self
    }
}

mod breaking;
pub use breaking::*;
mod containers;
pub use containers::*;
mod entity_updates;
pub use entity_updates::*;
mod inventory;
pub use inventory::*;
mod items;
pub use items::*;
mod projectiles;
pub use projectiles::*;
mod remote_player;
pub use remote_player::*;
mod world;
pub use world::*;
