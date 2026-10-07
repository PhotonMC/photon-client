//! `ClientWorld` player layer split out of the former single-file module.

use super::*;
use voxel_core::java_float::{DVec3, JavaF32};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPlayerState {
    pub(crate) id: i32,
    pub(crate) generation: u64,
    pub(crate) tick_count: i32,
    pub(crate) position: DVec3,
    pub(crate) old_position: DVec3,
    pub(crate) motion: DVec3,
    pub(crate) friction_influenced_speed: JavaF32,
    pub(crate) modified_friction: JavaF32,
    pub(crate) xxa: JavaF32,
    pub(crate) zza: JavaF32,
    pub(crate) input_key_presses: Input,
    pub(crate) input_move_vector_x: JavaF32,
    pub(crate) input_move_vector_y: JavaF32,
    pub(crate) sprint_trigger_time: i32,
    pub(crate) jumping: bool,
    pub(crate) on_climbable: bool,
    pub(crate) in_scaffolding: bool,
    pub(crate) suppress_sliding_down_ladder: bool,
    pub(crate) x_bob: JavaF32,
    pub(crate) y_bob: JavaF32,
    pub(crate) x_bob_o: JavaF32,
    pub(crate) y_bob_o: JavaF32,
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
    pub(crate) absorption: JavaF32,
    /// Which skin layers show, one bit each, from the entity data.
    pub(crate) model_customisation: u8,
    /// `Player.getScore`.
    pub(crate) score: i32,
    pub(crate) attributes: Vec<ClientAttributeState>,
    pub(crate) effects: Vec<ClientMobEffectState>,
    pub(crate) health: JavaF32,
    pub(crate) food: i32,
    pub(crate) saturation: JavaF32,
    pub(crate) inventory_state_id: i32,
    pub(crate) inventory_slots: BTreeMap<i16, Option<ClientItemStackState>>,
    pub(crate) carried_item: Option<ClientItemStackState>,
    pub(crate) open_container: Option<ClientContainerState>,
    pub(crate) vehicle_id: Option<i32>,
    pub(crate) passenger_ids: Vec<i32>,
}

mod game_mode;
pub use game_mode::*;
mod inventory;
mod motion;
mod pose;
pub use pose::*;
mod status;
mod travel;
