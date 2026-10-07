//! `ClientWorld` player layer split out of the former single-file module.

use super::*;
use voxel_core::java_float::{DVec3, JavaF32, JavaF64};

use std::collections::BTreeMap;

use crate::input::{
    Input, LocalPlayerClimbableMovementInput, LocalPlayerFluidFallingInput,
    LocalPlayerFrictionInfluencedSpeedInput, LocalPlayerHandleRelativeFrictionInput,
    LocalPlayerHandleRelativeFrictionParameters, LocalPlayerInputModifiers, LocalPlayerInputState,
    LocalPlayerModifiedFrictionInput, LocalPlayerTravelDispatchInput,
    LocalPlayerTravelDispatchParameters, LocalPlayerTravelDispatchResult,
    LocalPlayerTravelFlyingInput, LocalPlayerTravelFlyingParameters, LocalPlayerTravelInAirInput,
    LocalPlayerTravelInAirParameters, LocalPlayerTravelInLavaInput,
    LocalPlayerTravelInLavaParameters, LocalPlayerTravelInWaterInput,
    LocalPlayerTravelInWaterParameters, LocalPlayerUpdateFallFlyingMovementInput,
    LocalPlayerUpdateFallFlyingMovementParameters, adjust_local_player_fluid_falling_movement,
    apply_local_player_input as apply_local_player_input_fields,
    clamp_local_player_climbable_movement, local_player_friction_influenced_speed,
    local_player_handle_relative_friction, local_player_modified_friction,
    local_player_travel_dispatch, local_player_travel_flying, local_player_travel_in_air,
    local_player_travel_in_lava, local_player_travel_in_water,
    local_player_update_fall_flying_movement,
};
use voxel_core::math::{
    Vec2, Vec3, clamp_f32, floor_f32, mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32,
};

/// Inputs used by Java Player's desired-pose and fit fallback decision.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClientPlayerPoseInput {
    pub sleeping: bool,
    pub swimming: bool,
    pub fall_flying: bool,
    pub auto_spin_attack: bool,
    pub shift_key_down: bool,
    pub flying: bool,
    pub spectator: bool,
    pub passenger: bool,
    pub swimming_fits: bool,
    pub desired_fits: bool,
    pub crouching_fits: bool,
}

impl ClientPlayerPoseInput {
    /// Choose the desired pose using Java Player's branch priority.
    pub fn desired_pose(self) -> ClientEntityPose {
        if self.sleeping {
            ClientEntityPose::Sleeping
        } else if self.swimming {
            ClientEntityPose::Swimming
        } else if self.fall_flying {
            ClientEntityPose::FallFlying
        } else if self.auto_spin_attack {
            ClientEntityPose::SpinAttack
        } else if self.shift_key_down && !self.flying {
            ClientEntityPose::Crouching
        } else {
            ClientEntityPose::Standing
        }
    }

    /// Apply Java Player's swimming gate, fit fallback, and bypass branches.
    pub fn updated_pose(self) -> Option<ClientEntityPose> {
        if !self.swimming_fits {
            return None;
        }
        let desired_pose = self.desired_pose();
        Some(if self.spectator || self.passenger || self.desired_fits {
            desired_pose
        } else if self.crouching_fits {
            ClientEntityPose::Crouching
        } else {
            ClientEntityPose::Swimming
        })
    }
}

/// The four Java game-mode values applied to the local player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClientPlayerGameMode {
    #[default]
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl ClientPlayerGameMode {
    /// Resolve Java's continuous id mapping, including its survival fallback.
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            _ => Self::Survival,
        }
    }

    /// Resolve the parameter used by Java's change-game-mode event.
    pub fn from_game_event_param(param: f32) -> Self {
        Self::from_id(floor_f32(param + 0.5_f32))
    }

    pub fn id(self) -> i32 {
        match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Survival => "survival",
            Self::Creative => "creative",
            Self::Adventure => "adventure",
            Self::Spectator => "spectator",
        }
    }

    pub fn is_spectator(self) -> bool {
        matches!(self, Self::Spectator)
    }

    pub fn is_creative(self) -> bool {
        matches!(self, Self::Creative)
    }

    pub fn is_survival(self) -> bool {
        matches!(self, Self::Survival | Self::Adventure)
    }

    /// Apply Java `GameType.updatePlayerAbilities` to an existing ability state.
    ///
    /// Creative deliberately leaves `flying` unchanged, while the other branches
    /// set it exactly as the Java game-mode implementation does.
    pub fn update_player_abilities(
        self,
        mut abilities: ClientPlayerAbilities,
    ) -> ClientPlayerAbilities {
        match self {
            Self::Creative => {
                abilities.may_fly = true;
                abilities.instabuild = true;
                abilities.invulnerable = true;
            }
            Self::Spectator => {
                abilities.may_fly = true;
                abilities.instabuild = false;
                abilities.invulnerable = true;
                abilities.flying = true;
            }
            Self::Survival | Self::Adventure => {
                abilities.may_fly = false;
                abilities.instabuild = false;
                abilities.invulnerable = false;
                abilities.flying = false;
            }
        }
        abilities.may_build = !matches!(self, Self::Adventure | Self::Spectator);
        abilities
    }
}

/// The six ability values carried by Java's clientbound player-abilities packet,
/// plus the local game-mode-derived build permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientPlayerAbilities {
    pub(crate) invulnerable: bool,
    pub(crate) flying: bool,
    pub(crate) may_fly: bool,
    pub(crate) instabuild: bool,
    pub(crate) may_build: bool,
    pub(crate) flying_speed_bits: u32,
    pub(crate) walking_speed_bits: u32,
}

impl Default for ClientPlayerAbilities {
    fn default() -> Self {
        Self::new(false, false, false, false, 0.05, 0.1)
    }
}

impl ClientPlayerAbilities {
    pub fn new(
        invulnerable: bool,
        flying: bool,
        may_fly: bool,
        instabuild: bool,
        flying_speed: f32,
        walking_speed: f32,
    ) -> Self {
        Self {
            invulnerable,
            flying,
            may_fly,
            instabuild,
            may_build: true,
            flying_speed_bits: flying_speed.to_bits(),
            walking_speed_bits: walking_speed.to_bits(),
        }
    }

    pub fn invulnerable(self) -> bool {
        self.invulnerable
    }

    pub fn flying(self) -> bool {
        self.flying
    }

    pub fn may_fly(self) -> bool {
        self.may_fly
    }

    pub fn instabuild(self) -> bool {
        self.instabuild
    }

    pub fn may_build(self) -> bool {
        self.may_build
    }

    pub fn flying_speed(self) -> f32 {
        f32::from_bits(self.flying_speed_bits)
    }

    pub fn walking_speed(self) -> f32 {
        f32::from_bits(self.walking_speed_bits)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPlayerState {
    pub(crate) id: i32,
    pub(crate) generation: u64,
    pub(crate) tick_count: i32,
    pub(crate) position: DVec3,
    pub(crate) old_position: DVec3,
    pub(crate) motion: DVec3,
    pub(crate) friction_influenced_speed_bits: u32,
    pub(crate) modified_friction_bits: u32,
    pub(crate) xxa_bits: u32,
    pub(crate) zza_bits: u32,
    pub(crate) input_key_presses: Input,
    pub(crate) input_move_vector_x_bits: u32,
    pub(crate) input_move_vector_y_bits: u32,
    pub(crate) sprint_trigger_time: i32,
    pub(crate) jumping: bool,
    pub(crate) on_climbable: bool,
    pub(crate) in_scaffolding: bool,
    pub(crate) suppress_sliding_down_ladder: bool,
    pub(crate) x_bob_bits: u32,
    pub(crate) y_bob_bits: u32,
    pub(crate) x_bob_o_bits: u32,
    pub(crate) y_bob_o_bits: u32,
    pub(crate) y_rot: JavaF32,
    pub(crate) x_rot: JavaF32,
    pub(crate) old_y_rot: JavaF32,
    pub(crate) old_x_rot: JavaF32,
    pub(crate) y_head_rot: JavaF32,
    pub(crate) old_y_head_rot: JavaF32,
    pub(crate) y_body_rot: JavaF32,
    pub(crate) old_y_body_rot: JavaF32,
    pub(crate) head_rotation_target: JavaF32,
    pub(crate) head_lerp_steps: u8,
    pub(crate) entity_data: Option<Vec<u8>>,
    pub(crate) game_mode: ClientPlayerGameMode,
    pub(crate) abilities: ClientPlayerAbilities,
    pub(crate) shared_flags: u8,
    pub(crate) living_flags: u8,
    pub(crate) arrow_count: i32,
    pub(crate) stinger_count: i32,
    pub(crate) effect_ambience: bool,
    pub(crate) effect_particle_type_ids: Vec<i32>,
    pub(crate) sleeping_pos: Option<voxel_core::binary::BlockPos>,
    pub(crate) air_supply: i32,
    pub(crate) custom_name_present: bool,
    pub(crate) custom_name_visible: bool,
    pub(crate) no_gravity: bool,
    pub(crate) silent: bool,
    pub(crate) pose: ClientEntityPose,
    pub(crate) ticks_frozen: i32,
    /// `Player.getAbsorptionAmount`, from the entity data.
    pub(crate) absorption_bits: u32,
    /// Which skin layers show, one bit each, from the entity data.
    pub(crate) model_customisation: u8,
    /// `Player.getScore`.
    pub(crate) score: i32,
    pub(crate) attributes: Vec<ClientAttributeState>,
    pub(crate) effects: Vec<ClientMobEffectState>,
    pub(crate) health_bits: u32,
    pub(crate) food: i32,
    pub(crate) saturation_bits: u32,
    pub(crate) inventory_state_id: i32,
    pub(crate) inventory_slots: BTreeMap<i16, Option<ClientItemStackState>>,
    pub(crate) carried_item: Option<ClientItemStackState>,
    pub(crate) open_container: Option<ClientContainerState>,
    pub(crate) vehicle_id: Option<i32>,
    pub(crate) passenger_ids: Vec<i32>,
}

mod client_player_state_previous;
mod client_player_state_travel;
