use super::*;

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
                rotation: voxel_core::math::Vec2::new(self.x_rot.get(), self.y_rot.get()),
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

    pub(crate) fn capture_old_transform(&mut self) {
        self.old_position.x = self.position.x;
        self.old_position.y = self.position.y;
        self.old_position.z = self.position.z;
        self.old_y_rot = self.y_rot;
        self.old_x_rot = self.x_rot;
        self.old_y_head_rot = self.y_head_rot;
        self.old_y_body_rot = self.y_body_rot;
    }

    pub(crate) fn turn(&mut self, mouse_x: f64, mouse_y: f64) {
        let x_delta = mouse_y as f32 * 0.15_f32;
        let y_delta = mouse_x as f32 * 0.15_f32;
        self.x_rot = JavaF32::new(self.x_rot.get() + x_delta);
        self.y_rot = JavaF32::new(self.y_rot.get() + y_delta);
        self.x_rot = JavaF32::new(clamp_f32(self.x_rot.get(), -90.0, 90.0));
        self.old_x_rot = JavaF32::new(self.old_x_rot.get() + x_delta);
        self.old_y_rot = JavaF32::new(self.old_y_rot.get() + y_delta);
        self.old_x_rot = JavaF32::new(clamp_f32(self.old_x_rot.get(), -90.0, 90.0));
    }

    pub(crate) fn set_position(&mut self, position: (f64, f64, f64)) {
        self.position.x = JavaF64::new(position.0);
        self.position.y = JavaF64::new(position.1);
        self.position.z = JavaF64::new(position.2);
    }

    pub(crate) fn tick_head_rotation(&mut self) {
        advance_head_rotation(
            &mut self.y_head_rot,
            self.head_rotation_target,
            &mut self.head_lerp_steps,
        );
    }

    pub(crate) fn add_vanilla_boat_passenger_rotation(&mut self, delta_rotation: f32) {
        self.y_rot = JavaF32::new(self.y_rot.get() + delta_rotation);
        self.y_head_rot = JavaF32::new(self.y_head_rot.get() + delta_rotation);
    }

    pub(crate) fn new(id: i32) -> Self {
        Self {
            id,
            generation: 0,
            tick_count: 0,
            position: DVec3::ZERO,
            old_position: DVec3::ZERO,
            motion: DVec3::ZERO,
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
            y_rot: JavaF32::new(0.0_f32),
            x_rot: JavaF32::new(0.0_f32),
            old_y_rot: JavaF32::new(0.0_f32),
            old_x_rot: JavaF32::new(0.0_f32),
            y_head_rot: JavaF32::new(0.0_f32),
            old_y_head_rot: JavaF32::new(0.0_f32),
            y_body_rot: JavaF32::new(0.0_f32),
            old_y_body_rot: JavaF32::new(0.0_f32),
            head_rotation_target: JavaF32::new(0.0_f32),
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
            score: 0,
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
}
