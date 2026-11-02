//! `ClientWorld` player layer split out of the former single-file module.

use super::*;
use photon_core::java_float::{DVec3, JavaF32};

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
use photon_core::math::{Vec2, Vec3, clamp_f32, mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32};

/// What the data of a player holds besides the shared fields.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PlayerData {
    /// Which skin layers show, one bit each.
    pub(crate) model_customisation: u8,
    /// `Player.getScore`.
    pub(crate) score: i32,
    /// The variant of the parrot on the left and right shoulder.
    pub(crate) shoulder_parrots: [Option<i32>; 2],
    /// `ItemStack.popTime` of the hotbar's stacks and the off hand's.
    pub(crate) hotbar_pop: [i32; 10],
}

impl PlayerData {
    /// The ticks a stack the inventory menu takes into a hotbar or off-hand slot pops for.
    const POP_TICKS: i32 = 5;

    /// `ClientPacketListener.handleContainerSetSlot` for the inventory menu: a stack that is not
    /// empty and is bigger than what the slot held pops; any other new stack starts without.
    pub(crate) fn set_slot_pop(
        &mut self,
        slot: i16,
        (before, after): (Option<&ClientItemStackState>, Option<&ClientItemStackState>),
    ) {
        let index = match slot {
            36..=44 => slot - 36,
            45 => 9,
            _ => return,
        };
        let count =
            |stack: Option<&ClientItemStackState>| stack.map_or(0, |stack| stack.item_count());
        let grew = count(after) > 0 && count(before) < count(after);
        self.hotbar_pop[index as usize] = if grew { Self::POP_TICKS } else { 0 };
    }

    /// `ItemStack.inventoryTick`: each stack's pop runs down a tick.
    pub(crate) fn tick_pop(&mut self) {
        for pop in &mut self.hotbar_pop {
            *pop = (*pop - 1).max(0);
        }
    }

    /// What the data a packet sent says: a field it leaves out keeps its value.
    pub(crate) fn apply(&mut self, metadata: &[u8]) {
        use crate::metadata::*;
        if let Some(customisation) = player_model_customisation_from_metadata(metadata) {
            self.model_customisation = customisation;
        }
        if let Some(score) = player_score_from_metadata(metadata) {
            self.score = score;
        }
        for (shoulder, said) in self
            .shoulder_parrots
            .iter_mut()
            .zip(player_shoulder_parrots_from_metadata(metadata))
        {
            if let Some(variant) = said {
                *shoulder = variant;
            }
        }
    }
}

/// What the local player's sleep is: where its bed is (the sleeping position of its entity data)
/// and `Player.sleepCounter`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Sleep {
    pub(crate) pos: Option<photon_core::binary::BlockPos>,
    pub(crate) timer: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPlayerState {
    pub(crate) id: EntityId,
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
    pub(crate) effect_particles: Vec<EffectParticle>,
    pub(crate) sleep: Sleep,
    pub(crate) air_supply: i32,
    pub(crate) custom_name_present: bool,
    pub(crate) custom_name_visible: bool,
    pub(crate) no_gravity: bool,
    pub(crate) silent: bool,
    pub(crate) pose: ClientEntityPose,
    pub(crate) ticks_frozen: i32,
    /// `Player.getAbsorptionAmount`, from the entity data.
    pub(crate) absorption: JavaF32,
    /// What the entity data says of the player besides the shared fields.
    pub(crate) data: PlayerData,
    pub(crate) attributes: Vec<ClientAttributeState>,
    pub(crate) effects: Vec<ClientMobEffectState>,
    pub(crate) health: JavaF32,
    pub(crate) food: i32,
    pub(crate) saturation: JavaF32,
    pub(crate) inventory_state_id: i32,
    pub(crate) inventory_slots: BTreeMap<i16, Option<ClientItemStackState>>,
    pub(crate) carried_item: Option<ClientItemStackState>,
    pub(crate) open_container: Option<ClientContainerState>,
    pub(crate) vehicle_id: Option<EntityId>,
    pub(crate) passenger_ids: Vec<EntityId>,
}

pub use photon_data::game_mode::{ClientPlayerAbilities, ClientPlayerGameMode};
mod inventory;
mod motion;
mod pose;
pub use pose::*;
mod status;
mod travel;
