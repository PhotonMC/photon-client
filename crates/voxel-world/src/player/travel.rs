use super::*;
use voxel_core::java_float::{JavaF32, JavaF64};

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
                current_impulses: voxel_core::math::Vec2::new(self.xxa.get(), self.zza.get()),
                current_jumping: self.jumping,
                current_bob: voxel_core::math::Vec2::new(self.x_bob.get(), self.y_bob.get()),
                previous_bob: voxel_core::math::Vec2::new(self.x_bob_o.get(), self.y_bob_o.get()),
                rotation: voxel_core::math::Vec2::new(self.x_rot.get(), self.y_rot.get()),
                controlled_camera,
            },
            modifiers,
        );
        self.xxa = JavaF32::new(applied.movement.x);
        self.zza = JavaF32::new(applied.movement.y);
        self.jumping = applied.jumping;
        self.x_bob = JavaF32::new(applied.x_bob);
        self.y_bob = JavaF32::new(applied.y_bob);
        self.x_bob_o = JavaF32::new(applied.x_bob_old);
        self.y_bob_o = JavaF32::new(applied.y_bob_old);
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
                self.motion.x.get(),
                self.motion.y.get(),
                self.motion.z.get(),
            ),
        });
        self.motion.y = JavaF64::new(adjusted.y);
    }

    pub(crate) fn apply_climbable_movement(&mut self) {
        let adjusted = clamp_local_player_climbable_movement(LocalPlayerClimbableMovementInput {
            movement: Vec3::new(
                self.motion.x.get(),
                self.motion.y.get(),
                self.motion.z.get(),
            ),
            on_climbable: self.on_climbable,
            scaffolding: self.in_scaffolding,
            suppress_sliding_down_ladder: self.suppress_sliding_down_ladder,
        });
        self.motion.x = JavaF64::new(adjusted.x);
        self.motion.y = JavaF64::new(adjusted.y);
        self.motion.z = JavaF64::new(adjusted.z);
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
        self.friction_influenced_speed = JavaF32::new(adjusted);
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
        self.modified_friction = JavaF32::new(local_player_modified_friction(
            LocalPlayerModifiedFrictionInput { friction, modifier },
        ));
    }

    pub(crate) fn apply_travel_flying(&mut self, parameters: LocalPlayerTravelFlyingParameters) {
        let movement = local_player_travel_flying(LocalPlayerTravelFlyingInput {
            input: parameters.input,
            delta_movement: Vec3::new(
                self.motion.x.get(),
                self.motion.y.get(),
                self.motion.z.get(),
            ),
            y_rot: parameters.y_rot,
            in_water: parameters.in_water,
            in_lava: parameters.in_lava,
            water_speed: parameters.water_speed,
            lava_speed: parameters.lava_speed,
            air_speed: parameters.air_speed,
        });
        self.motion.x = JavaF64::new(movement.x);
        self.motion.y = JavaF64::new(movement.y);
        self.motion.z = JavaF64::new(movement.z);
    }

    pub(crate) fn apply_travel_in_water(&mut self, parameters: LocalPlayerTravelInWaterParameters) {
        let movement = local_player_travel_in_water(LocalPlayerTravelInWaterInput {
            input: parameters.input,
            delta_movement: Vec3::new(
                self.motion.x.get(),
                self.motion.y.get(),
                self.motion.z.get(),
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
        self.motion.x = JavaF64::new(movement.x);
        self.motion.y = JavaF64::new(movement.y);
        self.motion.z = JavaF64::new(movement.z);
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
                self.motion.x.get(),
                self.motion.y.get(),
                self.motion.z.get(),
            ),
            y_rot: parameters.y_rot,
            sprinting: parameters.sprinting,
            shallow_lava: parameters.shallow_lava,
            horizontal_collision: parameters.horizontal_collision,
            free_after_jump: parameters.free_after_jump,
            base_gravity: parameters.base_gravity,
            is_falling: parameters.is_falling,
        });
        self.motion.x = JavaF64::new(movement.x);
        self.motion.y = JavaF64::new(movement.y);
        self.motion.z = JavaF64::new(movement.z);
    }

    pub(crate) fn apply_handle_relative_friction(
        &mut self,
        parameters: LocalPlayerHandleRelativeFrictionParameters,
    ) {
        let movement =
            local_player_handle_relative_friction(LocalPlayerHandleRelativeFrictionInput {
                input: parameters.input,
                delta_movement: Vec3::new(
                    self.motion.x.get(),
                    self.motion.y.get(),
                    self.motion.z.get(),
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
        self.motion.x = JavaF64::new(movement.x);
        self.motion.y = JavaF64::new(movement.y);
        self.motion.z = JavaF64::new(movement.z);
    }

    pub(crate) fn apply_travel_in_air(&mut self, parameters: LocalPlayerTravelInAirParameters) {
        let movement = local_player_travel_in_air(LocalPlayerTravelInAirInput {
            input: parameters.input,
            delta_movement: Vec3::new(
                self.motion.x.get(),
                self.motion.y.get(),
                self.motion.z.get(),
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
        self.motion.x = JavaF64::new(movement.x);
        self.motion.y = JavaF64::new(movement.y);
        self.motion.z = JavaF64::new(movement.z);
    }

    pub(crate) fn apply_update_fall_flying_movement(
        &mut self,
        parameters: LocalPlayerUpdateFallFlyingMovementParameters,
    ) {
        let movement =
            local_player_update_fall_flying_movement(LocalPlayerUpdateFallFlyingMovementInput {
                look_angle: parameters.look_angle,
                delta_movement: Vec3::new(
                    self.motion.x.get(),
                    self.motion.y.get(),
                    self.motion.z.get(),
                ),
                x_rot: parameters.x_rot,
                gravity: parameters.gravity,
            });
        self.motion.x = JavaF64::new(movement.x);
        self.motion.y = JavaF64::new(movement.y);
        self.motion.z = JavaF64::new(movement.z);
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
        self.motion.x = JavaF64::new(result.movement.x);
        self.motion.y = JavaF64::new(result.movement.y);
        self.motion.z = JavaF64::new(result.movement.z);
        result
    }
}
