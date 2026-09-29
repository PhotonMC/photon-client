//! `ClientWorld` player layer split out of the former single-file module.

use super::*;

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
    pub(crate) x_bits: u64,
    pub(crate) y_bits: u64,
    pub(crate) z_bits: u64,
    pub(crate) old_x_bits: u64,
    pub(crate) old_y_bits: u64,
    pub(crate) old_z_bits: u64,
    pub(crate) motion_x_bits: u64,
    pub(crate) motion_y_bits: u64,
    pub(crate) motion_z_bits: u64,
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
    pub(crate) y_rot_bits: u32,
    pub(crate) x_rot_bits: u32,
    pub(crate) old_y_rot_bits: u32,
    pub(crate) old_x_rot_bits: u32,
    pub(crate) y_head_rot_bits: u32,
    pub(crate) old_y_head_rot_bits: u32,
    pub(crate) y_body_rot_bits: u32,
    pub(crate) old_y_body_rot_bits: u32,
    pub(crate) head_rotation_target_bits: u32,
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

impl ClientPlayerState {
    pub(crate) fn apply_input(
        &mut self,
        move_vector: voxel_core::math::Vec2,
        jump_pressed: bool,
        controlled_camera: bool,
        modifiers: LocalPlayerInputModifiers,
    ) {
        let applied = apply_local_player_input_fields(
            LocalPlayerInputState {
                move_vector,
                jump_pressed,
                current_impulses: voxel_core::math::Vec2::new(
                    f32::from_bits(self.xxa_bits),
                    f32::from_bits(self.zza_bits),
                ),
                current_jumping: self.jumping,
                current_bob: voxel_core::math::Vec2::new(
                    f32::from_bits(self.x_bob_bits),
                    f32::from_bits(self.y_bob_bits),
                ),
                previous_bob: voxel_core::math::Vec2::new(
                    f32::from_bits(self.x_bob_o_bits),
                    f32::from_bits(self.y_bob_o_bits),
                ),
                rotation: voxel_core::math::Vec2::new(
                    f32::from_bits(self.x_rot_bits),
                    f32::from_bits(self.y_rot_bits),
                ),
                controlled_camera,
            },
            modifiers,
        );
        self.xxa_bits = applied.movement.x.to_bits();
        self.zza_bits = applied.movement.y.to_bits();
        self.jumping = applied.jumping;
        self.x_bob_bits = applied.x_bob.to_bits();
        self.y_bob_bits = applied.y_bob.to_bits();
        self.x_bob_o_bits = applied.x_bob_old.to_bits();
        self.y_bob_o_bits = applied.y_bob_old.to_bits();
    }

    pub(crate) fn apply_input_from_attributes(
        &mut self,
        move_vector: voxel_core::math::Vec2,
        jump_pressed: bool,
        controlled_camera: bool,
        mut modifiers: LocalPlayerInputModifiers,
    ) {
        if controlled_camera && modifiers.moving_slowly && move_vector.length_squared() != 0.0 {
            modifiers.sneaking_movement_factor = self.sneaking_movement_factor();
        }
        self.apply_input(move_vector, jump_pressed, controlled_camera, modifiers);
    }

    pub(crate) fn apply_fluid_falling_adjustment(&mut self, base_gravity: f64, is_falling: bool) {
        let adjusted = adjust_local_player_fluid_falling_movement(LocalPlayerFluidFallingInput {
            base_gravity,
            is_falling,
            sprinting: self.is_sprinting(),
            movement: Vec3::new(
                f64::from_bits(self.motion_x_bits),
                f64::from_bits(self.motion_y_bits),
                f64::from_bits(self.motion_z_bits),
            ),
        });
        self.motion_y_bits = adjusted.y.to_bits();
    }

    pub(crate) fn apply_climbable_movement(&mut self) {
        let adjusted = clamp_local_player_climbable_movement(LocalPlayerClimbableMovementInput {
            movement: Vec3::new(
                f64::from_bits(self.motion_x_bits),
                f64::from_bits(self.motion_y_bits),
                f64::from_bits(self.motion_z_bits),
            ),
            on_climbable: self.on_climbable,
            scaffolding: self.in_scaffolding,
            suppress_sliding_down_ladder: self.suppress_sliding_down_ladder,
        });
        self.motion_x_bits = adjusted.x.to_bits();
        self.motion_y_bits = adjusted.y.to_bits();
        self.motion_z_bits = adjusted.z.to_bits();
    }

    pub(crate) fn apply_friction_influenced_speed(
        &mut self,
        on_ground: bool,
        block_friction: f32,
        speed: f32,
    ) {
        let adjusted =
            local_player_friction_influenced_speed(LocalPlayerFrictionInfluencedSpeedInput {
                on_ground,
                block_friction,
                speed,
                flying: self.abilities.flying(),
                flying_speed: self.abilities.flying_speed(),
                sprinting: self.is_sprinting(),
                passenger: self.vehicle_id.is_some(),
            });
        self.friction_influenced_speed_bits = adjusted.to_bits();
    }

    pub(crate) fn apply_friction_influenced_speed_from_attributes(
        &mut self,
        on_ground: bool,
        block_friction: f32,
    ) {
        let speed = self.movement_speed();
        self.apply_friction_influenced_speed(on_ground, block_friction, speed);
    }

    pub(crate) fn apply_modified_friction(&mut self, friction: f32, modifier: f32) {
        self.modified_friction_bits =
            local_player_modified_friction(LocalPlayerModifiedFrictionInput { friction, modifier })
                .to_bits();
    }

    pub(crate) fn apply_travel_flying(&mut self, parameters: LocalPlayerTravelFlyingParameters) {
        let movement = local_player_travel_flying(LocalPlayerTravelFlyingInput {
            input: parameters.input,
            delta_movement: Vec3::new(
                f64::from_bits(self.motion_x_bits),
                f64::from_bits(self.motion_y_bits),
                f64::from_bits(self.motion_z_bits),
            ),
            y_rot: parameters.y_rot,
            in_water: parameters.in_water,
            in_lava: parameters.in_lava,
            water_speed: parameters.water_speed,
            lava_speed: parameters.lava_speed,
            air_speed: parameters.air_speed,
        });
        self.motion_x_bits = movement.x.to_bits();
        self.motion_y_bits = movement.y.to_bits();
        self.motion_z_bits = movement.z.to_bits();
    }

    pub(crate) fn apply_travel_in_water(&mut self, parameters: LocalPlayerTravelInWaterParameters) {
        let movement = local_player_travel_in_water(LocalPlayerTravelInWaterInput {
            input: parameters.input,
            delta_movement: Vec3::new(
                f64::from_bits(self.motion_x_bits),
                f64::from_bits(self.motion_y_bits),
                f64::from_bits(self.motion_z_bits),
            ),
            y_rot: parameters.y_rot,
            sprinting: parameters.sprinting,
            water_slow_down: parameters.water_slow_down,
            water_efficiency: parameters.water_efficiency,
            on_ground: parameters.on_ground,
            speed: parameters.speed,
            dolphins_grace: parameters.dolphins_grace,
            horizontal_collision: parameters.horizontal_collision,
            on_climbable: parameters.on_climbable,
            free_after_jump: parameters.free_after_jump,
            base_gravity: parameters.base_gravity,
            is_falling: parameters.is_falling,
        });
        self.motion_x_bits = movement.x.to_bits();
        self.motion_y_bits = movement.y.to_bits();
        self.motion_z_bits = movement.z.to_bits();
    }

    pub(crate) fn apply_travel_in_water_from_attributes(
        &mut self,
        mut parameters: LocalPlayerTravelInWaterParameters,
    ) {
        parameters.speed = self.movement_speed();
        self.apply_travel_in_water(parameters);
    }

    pub(crate) fn apply_sprinting(&mut self, sprinting: bool) {
        if sprinting {
            self.shared_flags |= 1_u8 << SHARED_FLAG_SPRINTING;
        } else {
            self.shared_flags &= !(1_u8 << SHARED_FLAG_SPRINTING);
        }

        let Some(attribute) = self
            .attributes
            .iter_mut()
            .find(|attribute| attribute.attribute_id == VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID)
        else {
            if sprinting {
                self.attributes.push(ClientAttributeState::new(
                    VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID,
                    f64::from(VANILLA_PLAYER_DEFAULT_MOVEMENT_SPEED),
                    vec![ClientAttributeModifierState::new(
                        VANILLA_PLAYER_SPRINTING_MODIFIER_ID.to_owned(),
                        f64::from(VANILLA_PLAYER_SPRINTING_MODIFIER_AMOUNT),
                        2,
                    )],
                ));
            }
            return;
        };

        attribute
            .modifiers
            .retain(|modifier| modifier.id() != VANILLA_PLAYER_SPRINTING_MODIFIER_ID);
        if sprinting {
            attribute.modifiers.push(ClientAttributeModifierState::new(
                VANILLA_PLAYER_SPRINTING_MODIFIER_ID.to_owned(),
                f64::from(VANILLA_PLAYER_SPRINTING_MODIFIER_AMOUNT),
                2,
            ));
        }
    }

    pub(crate) fn apply_travel_in_lava(&mut self, parameters: LocalPlayerTravelInLavaParameters) {
        let movement = local_player_travel_in_lava(LocalPlayerTravelInLavaInput {
            input: parameters.input,
            delta_movement: Vec3::new(
                f64::from_bits(self.motion_x_bits),
                f64::from_bits(self.motion_y_bits),
                f64::from_bits(self.motion_z_bits),
            ),
            y_rot: parameters.y_rot,
            sprinting: parameters.sprinting,
            shallow_lava: parameters.shallow_lava,
            horizontal_collision: parameters.horizontal_collision,
            free_after_jump: parameters.free_after_jump,
            base_gravity: parameters.base_gravity,
            is_falling: parameters.is_falling,
        });
        self.motion_x_bits = movement.x.to_bits();
        self.motion_y_bits = movement.y.to_bits();
        self.motion_z_bits = movement.z.to_bits();
    }

    pub(crate) fn apply_handle_relative_friction(
        &mut self,
        parameters: LocalPlayerHandleRelativeFrictionParameters,
    ) {
        let movement =
            local_player_handle_relative_friction(LocalPlayerHandleRelativeFrictionInput {
                input: parameters.input,
                delta_movement: Vec3::new(
                    f64::from_bits(self.motion_x_bits),
                    f64::from_bits(self.motion_y_bits),
                    f64::from_bits(self.motion_z_bits),
                ),
                y_rot: parameters.y_rot,
                on_ground: parameters.on_ground,
                block_friction: parameters.block_friction,
                speed: parameters.speed,
                flying_speed: parameters.flying_speed,
                flying: parameters.flying,
                sprinting: parameters.sprinting,
                passenger: parameters.passenger,
                on_climbable: parameters.on_climbable,
                scaffolding: parameters.scaffolding,
                suppress_sliding_down_ladder: parameters.suppress_sliding_down_ladder,
                horizontal_collision: parameters.horizontal_collision,
                jumping: parameters.jumping,
            });
        self.motion_x_bits = movement.x.to_bits();
        self.motion_y_bits = movement.y.to_bits();
        self.motion_z_bits = movement.z.to_bits();
    }

    pub(crate) fn apply_travel_in_air(&mut self, parameters: LocalPlayerTravelInAirParameters) {
        let movement = local_player_travel_in_air(LocalPlayerTravelInAirInput {
            input: parameters.input,
            delta_movement: Vec3::new(
                f64::from_bits(self.motion_x_bits),
                f64::from_bits(self.motion_y_bits),
                f64::from_bits(self.motion_z_bits),
            ),
            y_rot: parameters.y_rot,
            on_ground: parameters.on_ground,
            block_friction: parameters.block_friction,
            friction_modifier: parameters.friction_modifier,
            speed: parameters.speed,
            flying_speed: parameters.flying_speed,
            flying: parameters.flying,
            sprinting: parameters.sprinting,
            passenger: parameters.passenger,
            on_climbable: parameters.on_climbable,
            scaffolding: parameters.scaffolding,
            suppress_sliding_down_ladder: parameters.suppress_sliding_down_ladder,
            horizontal_collision: parameters.horizontal_collision,
            jumping: parameters.jumping,
            levitation_amplifier: parameters.levitation_amplifier,
            gravity: parameters.gravity,
            client_side: parameters.client_side,
            has_chunk_at: parameters.has_chunk_at,
            current_y: parameters.current_y,
            min_y: parameters.min_y,
            should_discard_friction: parameters.should_discard_friction,
            air_drag_modifier: parameters.air_drag_modifier,
            omnidirectional_air_mover: parameters.omnidirectional_air_mover,
        });
        self.motion_x_bits = movement.x.to_bits();
        self.motion_y_bits = movement.y.to_bits();
        self.motion_z_bits = movement.z.to_bits();
    }

    pub(crate) fn apply_update_fall_flying_movement(
        &mut self,
        parameters: LocalPlayerUpdateFallFlyingMovementParameters,
    ) {
        let movement =
            local_player_update_fall_flying_movement(LocalPlayerUpdateFallFlyingMovementInput {
                look_angle: parameters.look_angle,
                delta_movement: Vec3::new(
                    f64::from_bits(self.motion_x_bits),
                    f64::from_bits(self.motion_y_bits),
                    f64::from_bits(self.motion_z_bits),
                ),
                x_rot: parameters.x_rot,
                gravity: parameters.gravity,
            });
        self.motion_x_bits = movement.x.to_bits();
        self.motion_y_bits = movement.y.to_bits();
        self.motion_z_bits = movement.z.to_bits();
    }

    pub(crate) fn apply_travel_dispatch(
        &mut self,
        parameters: LocalPlayerTravelDispatchParameters,
    ) -> LocalPlayerTravelDispatchResult {
        let result = local_player_travel_dispatch(LocalPlayerTravelDispatchInput {
            parameters,
            delta_movement: {
                let (x, y, z) = self.motion();
                Vec3::new(x, y, z)
            },
        });
        self.motion_x_bits = result.movement.x.to_bits();
        self.motion_y_bits = result.movement.y.to_bits();
        self.motion_z_bits = result.movement.z.to_bits();
        result
    }

    pub(crate) fn capture_old_transform(&mut self) {
        self.old_x_bits = self.x_bits;
        self.old_y_bits = self.y_bits;
        self.old_z_bits = self.z_bits;
        self.old_y_rot_bits = self.y_rot_bits;
        self.old_x_rot_bits = self.x_rot_bits;
        self.old_y_head_rot_bits = self.y_head_rot_bits;
        self.old_y_body_rot_bits = self.y_body_rot_bits;
    }

    pub(crate) fn turn(&mut self, mouse_x: f64, mouse_y: f64) {
        let x_delta = mouse_y as f32 * 0.15_f32;
        let y_delta = mouse_x as f32 * 0.15_f32;
        self.x_rot_bits = (f32::from_bits(self.x_rot_bits) + x_delta).to_bits();
        self.y_rot_bits = (f32::from_bits(self.y_rot_bits) + y_delta).to_bits();
        self.x_rot_bits = clamp_f32(f32::from_bits(self.x_rot_bits), -90.0, 90.0).to_bits();
        self.old_x_rot_bits = (f32::from_bits(self.old_x_rot_bits) + x_delta).to_bits();
        self.old_y_rot_bits = (f32::from_bits(self.old_y_rot_bits) + y_delta).to_bits();
        self.old_x_rot_bits = clamp_f32(f32::from_bits(self.old_x_rot_bits), -90.0, 90.0).to_bits();
    }

    pub(crate) fn set_position(&mut self, position: (f64, f64, f64)) {
        self.x_bits = position.0.to_bits();
        self.y_bits = position.1.to_bits();
        self.z_bits = position.2.to_bits();
    }

    pub(crate) fn tick_head_rotation(&mut self) {
        advance_head_rotation(
            &mut self.y_head_rot_bits,
            self.head_rotation_target_bits,
            &mut self.head_lerp_steps,
        );
    }

    pub(crate) fn add_vanilla_boat_passenger_rotation(&mut self, delta_rotation: f32) {
        self.y_rot_bits = (f32::from_bits(self.y_rot_bits) + delta_rotation).to_bits();
        self.y_head_rot_bits = (f32::from_bits(self.y_head_rot_bits) + delta_rotation).to_bits();
    }

    pub(crate) fn new(id: i32) -> Self {
        Self {
            id,
            generation: 0,
            tick_count: 0,
            x_bits: 0.0_f64.to_bits(),
            y_bits: 0.0_f64.to_bits(),
            z_bits: 0.0_f64.to_bits(),
            old_x_bits: 0.0_f64.to_bits(),
            old_y_bits: 0.0_f64.to_bits(),
            old_z_bits: 0.0_f64.to_bits(),
            motion_x_bits: 0.0_f64.to_bits(),
            motion_y_bits: 0.0_f64.to_bits(),
            motion_z_bits: 0.0_f64.to_bits(),
            friction_influenced_speed_bits: 0.0_f32.to_bits(),
            modified_friction_bits: 0.0_f32.to_bits(),
            xxa_bits: 0.0_f32.to_bits(),
            zza_bits: 0.0_f32.to_bits(),
            input_key_presses: Input::EMPTY,
            input_move_vector_x_bits: 0.0_f32.to_bits(),
            input_move_vector_y_bits: 0.0_f32.to_bits(),
            sprint_trigger_time: 0,
            jumping: false,
            on_climbable: false,
            in_scaffolding: false,
            suppress_sliding_down_ladder: false,
            x_bob_bits: 0.0_f32.to_bits(),
            y_bob_bits: 0.0_f32.to_bits(),
            x_bob_o_bits: 0.0_f32.to_bits(),
            y_bob_o_bits: 0.0_f32.to_bits(),
            y_rot_bits: 0.0_f32.to_bits(),
            x_rot_bits: 0.0_f32.to_bits(),
            old_y_rot_bits: 0.0_f32.to_bits(),
            old_x_rot_bits: 0.0_f32.to_bits(),
            y_head_rot_bits: 0.0_f32.to_bits(),
            old_y_head_rot_bits: 0.0_f32.to_bits(),
            y_body_rot_bits: 0.0_f32.to_bits(),
            old_y_body_rot_bits: 0.0_f32.to_bits(),
            head_rotation_target_bits: 0.0_f32.to_bits(),
            head_lerp_steps: 0,
            entity_data: None,
            game_mode: ClientPlayerGameMode::default(),
            abilities: ClientPlayerAbilities::default(),
            shared_flags: 0,
            living_flags: 0,
            arrow_count: 0,
            stinger_count: 0,
            effect_ambience: false,
            effect_particle_type_ids: Vec::new(),
            sleeping_pos: None,
            air_supply: DEFAULT_AIR_SUPPLY,
            custom_name_present: false,
            custom_name_visible: false,
            no_gravity: false,
            silent: false,
            pose: ClientEntityPose::Standing,
            ticks_frozen: 0,
            absorption_bits: 0,
            model_customisation: 0,
            attributes: Vec::new(),
            effects: Vec::new(),
            health_bits: 20.0_f32.to_bits(),
            food: 20,
            saturation_bits: 5.0_f32.to_bits(),
            inventory_state_id: 0,
            inventory_slots: BTreeMap::new(),
            carried_item: None,
            open_container: None,
            vehicle_id: None,
            passenger_ids: Vec::new(),
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn vehicle_id(&self) -> Option<i32> {
        self.vehicle_id
    }

    pub fn passenger_ids(&self) -> &[i32] {
        &self.passenger_ids
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.x_bits),
            f64::from_bits(self.y_bits),
            f64::from_bits(self.z_bits),
        )
    }

    pub fn previous_position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.old_x_bits),
            f64::from_bits(self.old_y_bits),
            f64::from_bits(self.old_z_bits),
        )
    }

    pub fn interpolated_position(&self, partial_tick: f32) -> (f64, f64, f64) {
        let partial_tick = f64::from(partial_tick);
        let current = self.position();
        let previous = self.previous_position();
        (
            mth_lerp_f64(partial_tick, previous.0, current.0),
            mth_lerp_f64(partial_tick, previous.1, current.1),
            mth_lerp_f64(partial_tick, previous.2, current.2),
        )
    }

    pub fn motion(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.motion_x_bits),
            f64::from_bits(self.motion_y_bits),
            f64::from_bits(self.motion_z_bits),
        )
    }

    pub fn friction_influenced_speed(&self) -> f32 {
        f32::from_bits(self.friction_influenced_speed_bits)
    }

    /// Return Java Player.getSpeed from the local movement-speed attribute.
    pub fn movement_speed(&self) -> f32 {
        self.attributes
            .iter()
            .find(|attribute| attribute.attribute_id == VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID)
            .map(|attribute| attribute.effective_value(0.0, 1024.0) as f32)
            .unwrap_or(VANILLA_PLAYER_DEFAULT_MOVEMENT_SPEED)
    }

    /// Return Java's effective LocalPlayer sneaking-speed attribute value.
    pub fn sneaking_movement_factor_value(&self) -> f64 {
        self.attributes
            .iter()
            .find(|attribute| attribute.attribute_id == VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID)
            .map(|attribute| attribute.effective_value(0.0, 1.0))
            .unwrap_or(VANILLA_PLAYER_DEFAULT_SNEAKING_SPEED)
    }

    /// Return the f32 value used by LocalPlayer.modifyInput.
    pub fn sneaking_movement_factor(&self) -> f32 {
        self.sneaking_movement_factor_value() as f32
    }

    pub fn modified_friction(&self) -> f32 {
        f32::from_bits(self.modified_friction_bits)
    }

    pub fn input_impulses(&self) -> voxel_core::math::Vec2 {
        voxel_core::math::Vec2::new(f32::from_bits(self.xxa_bits), f32::from_bits(self.zza_bits))
    }

    pub fn input_key_presses(&self) -> Input {
        self.input_key_presses
    }

    pub fn input_move_vector(&self) -> Vec2 {
        Vec2::new(
            f32::from_bits(self.input_move_vector_x_bits),
            f32::from_bits(self.input_move_vector_y_bits),
        )
    }

    pub fn sprint_trigger_time(&self) -> i32 {
        self.sprint_trigger_time
    }

    pub fn is_jumping(&self) -> bool {
        self.jumping
    }

    pub fn is_on_climbable(&self) -> bool {
        self.on_climbable
    }

    pub fn bob(&self) -> (f32, f32) {
        (
            f32::from_bits(self.x_bob_bits),
            f32::from_bits(self.y_bob_bits),
        )
    }

    pub fn previous_bob(&self) -> (f32, f32) {
        (
            f32::from_bits(self.x_bob_o_bits),
            f32::from_bits(self.y_bob_o_bits),
        )
    }

    pub fn rotation(&self) -> (f32, f32) {
        (
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.x_rot_bits),
        )
    }

    pub fn previous_rotation(&self) -> (f32, f32) {
        (
            f32::from_bits(self.old_y_rot_bits),
            f32::from_bits(self.old_x_rot_bits),
        )
    }

    pub fn head_rotation(&self) -> f32 {
        f32::from_bits(self.y_head_rot_bits)
    }

    pub fn body_y_rotation(&self) -> f32 {
        f32::from_bits(self.y_body_rot_bits)
    }

    pub fn previous_body_y_rotation(&self) -> f32 {
        f32::from_bits(self.old_y_body_rot_bits)
    }

    pub(crate) fn clamp_vanilla_boat_passenger_rotation(&mut self, vehicle_y_rot: f32) {
        let (old_y_rot, y_rot) = vanilla_boat_passenger_rotation_clamped_values(
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.old_y_rot_bits),
            vehicle_y_rot,
        );
        self.old_y_rot_bits = old_y_rot.to_bits();
        self.y_rot_bits = y_rot.to_bits();
        self.y_head_rot_bits = self.y_rot_bits;
        self.y_body_rot_bits = vehicle_y_rot.to_bits();
    }

    pub fn previous_head_rotation(&self) -> f32 {
        f32::from_bits(self.old_y_head_rot_bits)
    }

    pub fn interpolated_head_rotation(&self, partial_tick: f32) -> f32 {
        if partial_tick == 1.0 {
            return self.head_rotation();
        }
        mth_rot_lerp_f32(
            partial_tick,
            self.previous_head_rotation(),
            self.head_rotation(),
        )
    }

    pub fn interpolated_rotation(&self, partial_tick: f32) -> (f32, f32) {
        let current = self.rotation();
        if partial_tick == 1.0 {
            return current;
        }
        let previous = self.previous_rotation();
        (
            mth_rot_lerp_f32(partial_tick, previous.0, current.0),
            mth_lerp_f32(partial_tick, previous.1, current.1),
        )
    }

    pub fn entity_data(&self) -> Option<&[u8]> {
        self.entity_data.as_deref()
    }

    pub fn abilities(&self) -> ClientPlayerAbilities {
        self.abilities
    }

    pub fn game_mode(&self) -> ClientPlayerGameMode {
        self.game_mode
    }

    pub fn shared_flags(&self) -> u8 {
        self.shared_flags
    }

    pub fn living_flags(&self) -> u8 {
        self.living_flags
    }

    pub fn is_using_item(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_USING_ITEM)
    }

    pub fn used_item_is_off_hand(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_OFF_HAND)
    }

    pub fn is_auto_spin_attack(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_SPIN_ATTACK)
    }

    pub fn air_supply(&self) -> i32 {
        self.air_supply
    }

    pub fn has_custom_name(&self) -> bool {
        self.custom_name_present
    }

    pub fn is_custom_name_visible(&self) -> bool {
        self.custom_name_visible
    }

    pub fn should_show_name(&self) -> bool {
        true
    }

    pub fn is_no_gravity(&self) -> bool {
        self.no_gravity
    }

    pub fn is_silent(&self) -> bool {
        self.silent
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    /// Return player/Avatar dimensions derived from the current pose.
    pub fn player_dimensions(&self) -> ClientEntityDimensions {
        self.pose.player_dimensions()
    }

    pub fn ticks_frozen(&self) -> i32 {
        self.ticks_frozen
    }

    pub fn absorption(&self) -> f32 {
        f32::from_bits(self.absorption_bits)
    }

    pub fn model_customisation(&self) -> u8 {
        self.model_customisation
    }

    pub fn percent_frozen(&self) -> f32 {
        (self.ticks_frozen.min(DEFAULT_TICKS_REQUIRED_TO_FREEZE) as f32)
            / DEFAULT_TICKS_REQUIRED_TO_FREEZE as f32
    }

    pub fn is_fully_frozen(&self) -> bool {
        self.ticks_frozen >= DEFAULT_TICKS_REQUIRED_TO_FREEZE
    }

    pub fn is_freezing(&self) -> bool {
        self.ticks_frozen > 0
    }

    pub fn is_on_fire(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_ON_FIRE)
    }

    pub fn is_shift_key_down(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SHIFT_KEY_DOWN)
    }

    pub fn is_sprinting(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SPRINTING)
    }

    pub fn is_swimming(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SWIMMING)
    }

    pub fn is_invisible(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_INVISIBLE)
    }

    pub fn is_glowing(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_GLOWING)
    }

    pub fn is_fall_flying(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_FALL_FLYING)
    }

    pub fn attributes(&self) -> &[ClientAttributeState] {
        &self.attributes
    }

    pub fn effects(&self) -> &[ClientMobEffectState] {
        &self.effects
    }

    pub fn health(&self) -> f32 {
        f32::from_bits(self.health_bits)
    }

    pub fn arrow_count(&self) -> i32 {
        self.arrow_count
    }

    pub fn stinger_count(&self) -> i32 {
        self.stinger_count
    }

    pub fn is_effect_ambient(&self) -> bool {
        self.effect_ambience
    }

    pub fn effect_particle_type_ids(&self) -> &[i32] {
        &self.effect_particle_type_ids
    }

    pub fn sleeping_pos(&self) -> Option<voxel_core::binary::BlockPos> {
        self.sleeping_pos
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleeping_pos.is_some()
    }

    pub fn food(&self) -> i32 {
        self.food
    }

    pub fn saturation(&self) -> f32 {
        f32::from_bits(self.saturation_bits)
    }

    pub fn inventory_state_id(&self) -> i32 {
        self.inventory_state_id
    }

    pub fn inventory_slot(&self, slot: i16) -> Option<&ClientItemStackState> {
        self.inventory_slots.get(&slot).and_then(Option::as_ref)
    }

    pub fn inventory_slot_count(&self) -> usize {
        self.inventory_slots.len()
    }

    pub fn carried_item(&self) -> Option<&ClientItemStackState> {
        self.carried_item.as_ref()
    }

    pub fn open_container(&self) -> Option<&ClientContainerState> {
        self.open_container.as_ref()
    }
}
