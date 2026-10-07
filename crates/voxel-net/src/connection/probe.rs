//! Scenario adapters used by the local real-server probes.
//!
//! The production connection pieces live in `protocol` and `session`; this module
//! keeps scenario-specific packet sequences and observations at the probe boundary.

use std::io;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use crate::handshake::ClientIntent;
use crate::login::{ClientboundLoginFinishedPacket, ServerboundHelloPacket};
use crate::protocol::ConnectionProtocol;
use crate::resources::KnownPack;
use voxel_core::binary::Uuid;
use voxel_core::wire;
use voxel_world::{
    BlockEntityInput, ClientPlayerAbilities, ClientPlayerGameMode, DimensionBounds,
    DimensionEnvironment, DimensionInput, LightLayerInput, LightUpdateInput,
    PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS, PLAYER_RESPAWN_KEEP_ENTITY_DATA, SectionInput,
    VANILLA_ARROW_ENTITY_TYPE_ID, VANILLA_COW_ENTITY_TYPE_ID,
    VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID, VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID,
    VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID, VANILLA_PRIMED_TNT_ENTITY_TYPE_ID,
    VANILLA_SNOWBALL_ENTITY_TYPE_ID,
};

use super::entities;
use super::protocol::*;
use super::session::*;

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

mod remote;
pub use remote::*;
mod use_ops;
pub use use_ops::*;
mod survival;
pub use survival::*;
mod observe;
pub(crate) use observe::*;
mod play;
pub(crate) use play::*;
mod mode;
use mode::*;
