use super::*;

impl LocalPlayer {
    #[expect(
        clippy::cognitive_complexity,
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    pub(in super::super) fn local_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if self.sprint_trigger_time > 0 {
            self.sprint_trigger_time -= 1;
        }

        let was_jumping = self.input.key_presses.jump;
        let was_shift_key_down = self.input.key_presses.shift;
        let had_forward_impulse = self.input.move_vector().y > 1.0e-5;
        self.crouching = !self.abilities.flying
            && !self.is_swimming()
            && !self.is_passenger()
            && self.can_fit(level, ClientEntityPose::Crouching)?
            && (self.is_shift_key_down() || !self.can_fit(level, ClientEntityPose::Standing)?);
        self.input.tick();
        let mut was_auto_jump = false;
        if self.auto_jump_time > 0 {
            self.auto_jump_time -= 1;
            was_auto_jump = true;
            self.input.key_presses.jump = true;
        }

        if !self.no_physics {
            let half = f64::from(self.dimensions.width()) * 0.35;
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x - half, z + half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x - half, z - half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x + half, z - half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x + half, z + half);
        }

        if was_shift_key_down || self.input.key_presses.backward {
            self.sprint_trigger_time = 0;
        }

        if self.can_start_sprinting() {
            if !had_forward_impulse {
                if self.sprint_trigger_time > 0 {
                    self.set_sprinting(true);
                } else {
                    self.sprint_trigger_time = SPRINT_WINDOW;
                }
            }
            if self.input.key_presses.sprint {
                self.set_sprinting(true);
            }
        }

        if self.sprinting {
            if self.is_swimming() {
                if self.should_stop_swim_sprinting() {
                    self.set_sprinting(false);
                }
            } else if self.should_stop_run_sprinting() {
                self.set_sprinting(false);
            }
        }

        let mut just_toggled_creative_flight = false;
        if self.abilities.may_fly {
            if self.is_spectator() {
                if !self.abilities.flying {
                    self.abilities.flying = true;
                    just_toggled_creative_flight = true;
                    self.on_update_abilities();
                }
            } else if !was_jumping && self.input.key_presses.jump && !was_auto_jump {
                if self.jump_trigger_time == 0 {
                    self.jump_trigger_time = 7;
                } else if !self.is_swimming() {
                    self.abilities.flying = !self.abilities.flying;
                    if self.abilities.flying && self.on_ground {
                        self.jump_from_ground(level);
                    }
                    just_toggled_creative_flight = true;
                    self.on_update_abilities();
                    self.jump_trigger_time = 0;
                }
            }
        }
        // Gliding needs an elytra, which this port's player never wears.
        let _ = just_toggled_creative_flight;

        if self.is_in_water() && self.input.key_presses.shift && self.is_affected_by_fluids() {
            self.delta = self.delta.add_components(0.0, f64::from(-0.04_f32), 0.0);
        }

        if self.abilities.flying {
            let mut input_ya = 0;
            if self.input.key_presses.shift {
                input_ya -= 1;
            }
            if self.input.key_presses.jump {
                input_ya += 1;
            }
            if input_ya != 0 {
                let rise = input_ya as f32 * self.abilities.flying_speed() * 3.0_f32;
                self.delta = self.delta.add_components(0.0, f64::from(rise), 0.0);
            }
        }

        // `jumpableVehicle`: holding the jump key charges a jump of the mount (a saddled horse),
        // which letting go of it makes, and tells the server of.
        if self.riding.is_some_and(|riding| riding.can_jump) {
            if self.jump_riding_ticks < 0 {
                self.jump_riding_ticks += 1;
                if self.jump_riding_ticks == 0 {
                    self.jump_riding_scale = 0.0;
                }
            }
            if was_jumping && !self.input.key_presses.jump {
                self.jump_riding_ticks = -10;
                let charge = floor_f64(f64::from(self.jump_riding_scale * 100.0));
                self.ride_jump = Some(charge);
                self.sent.push(ServerboundMovement::PlayerCommand {
                    entity_id: self.entity_id,
                    action: PlayerCommandAction::StartRidingJump,
                    data: charge,
                });
            } else if !was_jumping && self.input.key_presses.jump {
                self.jump_riding_ticks = 0;
                self.jump_riding_scale = 0.0;
            } else if was_jumping {
                self.jump_riding_ticks += 1;
                if self.jump_riding_ticks < 10 {
                    self.jump_riding_scale = self.jump_riding_ticks as f32 * 0.1;
                } else {
                    self.jump_riding_scale = 0.8 + 2.0 / (self.jump_riding_ticks - 9) as f32 * 0.1;
                }
            }
        } else {
            self.jump_riding_scale = 0.0;
        }

        // AbstractClientPlayer.aiStep: the bob eases towards the ground speed.
        let target_bob = if self.on_ground && !self.is_swimming() {
            (self.delta.horizontal_distance() as f32).min(0.1)
        } else {
            0.0
        };
        self.bob_old = self.bob;
        self.bob += (target_bob - self.bob) * 0.4;
        self.player_ai_step(level)?;
        if self.on_ground && self.abilities.flying && !self.is_spectator() {
            self.abilities.flying = false;
            self.on_update_abilities();
        }
        Ok(())
    }

    pub(in super::super) fn player_ai_step(
        &mut self,
        level: &Level<'_>,
    ) -> Result<(), Unsupported> {
        if self.jump_trigger_time > 0 {
            self.jump_trigger_time -= 1;
        }
        if self.abilities.flying {
            self.fall_distance = 0.0;
        }
        self.living_ai_step(level)?;
        // `updateSwingTime`: the swing lasts as long as the swinging hand's item says.
        let swing_duration = self.current_swing_duration();
        if self.swinging {
            self.swing_time += 1;
            if self.swing_time >= swing_duration {
                self.swing_time = 0;
                self.swinging = false;
            }
        } else {
            self.swing_time = 0;
        }
        self.attack_anim = self.swing_time as f32 / swing_duration as f32;
        // `getTicksUsingItem`: counted while the item is in use.
        self.using_item_ticks = if self.hands.using_item {
            self.using_item_ticks.saturating_add(1)
        } else {
            0
        };
        self.y_head_rot = self.y_rot;
        Ok(())
    }

    pub(in super::super) fn living_ai_step(
        &mut self,
        level: &Level<'_>,
    ) -> Result<(), Unsupported> {
        if self.no_jump_delay > 0 {
            self.no_jump_delay -= 1;
        }

        let mut dx = self.delta.x;
        let mut dy = self.delta.y;
        let mut dz = self.delta.z;
        if self.delta.horizontal_distance_squared() < 9.0e-6 {
            dx = 0.0;
            dz = 0.0;
        }
        if self.delta.y.abs() < 0.003 {
            dy = 0.0;
        }
        self.delta = Vec3::new(dx, dy, dz);

        // LocalPlayer.applyInput.
        let modified = modify_local_player_input(
            self.input.move_vector(),
            LocalPlayerInputModifiers {
                using_item: false,
                passenger: self.is_passenger(),
                item_use_speed_multiplier: 1.0,
                moving_slowly: self.is_moving_slowly(),
                sneaking_movement_factor: SNEAKING_SPEED as f32,
            },
        );
        self.xxa = modified.x;
        self.zza = modified.y;
        self.jumping = self.input.key_presses.jump;
        self.y_bob_old = self.y_bob;
        self.x_bob_old = self.x_bob;
        self.x_bob += (self.x_rot - self.x_bob) * 0.5;
        self.y_bob += (self.y_rot - self.y_bob) * 0.5;

        if self.jumping && self.is_affected_by_fluids() {
            let fluid_height = if self.is_in_lava() {
                self.lava.height
            } else {
                self.water.height
            };
            let in_water_and_has_fluid_height = self.is_in_water() && fluid_height > 0.0;
            let threshold = self.fluid_jump_threshold();
            if !in_water_and_has_fluid_height || self.on_ground && !(fluid_height > threshold) {
                if !self.is_in_lava() || self.on_ground && self.lava.height <= threshold {
                    if (self.on_ground
                        || in_water_and_has_fluid_height && fluid_height <= threshold)
                        && self.no_jump_delay == 0
                    {
                        self.jump_from_ground(level);
                        self.no_jump_delay = 10;
                    }
                } else {
                    self.delta = self.delta.add_components(0.0, f64::from(0.04_f32), 0.0);
                }
            } else {
                self.delta = self.delta.add_components(0.0, f64::from(0.04_f32), 0.0);
            }
        } else {
            self.no_jump_delay = 0;
        }

        let input = Vec3::new(f64::from(self.xxa), 0.0, f64::from(self.zza));
        self.player_travel(level, input)?;
        self.apply_effects_from_blocks(level)?;
        // `calculateEntityAnimation`: the limbs swing with the distance
        // moved across the ground this tick.
        let x = self.position.x - self.old_position.x;
        let z = self.position.z - self.old_position.z;
        if self.is_passenger() {
            self.walk.stop();
        } else {
            self.walk.follow((x * x + z * z).sqrt(), false);
        }
        Ok(())
    }

    pub(in super::super) fn player_travel(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
    ) -> Result<(), Unsupported> {
        // `Player.travel`: a passenger travels as any living entity does.
        if self.is_passenger() {
            return self.living_travel(level, input);
        }
        if self.is_swimming() {
            let look_y = self.look_angle().y;
            let multiplier = if look_y < -0.2 { 0.085 } else { 0.06 };
            let above = BlockPos::new(
                floor_f64(self.position.x),
                floor_f64(self.position.y + 1.0 - 0.1),
                floor_f64(self.position.z),
            );
            if look_y <= 0.0 || self.jumping || level.info(above).fluid != Fluid::Empty {
                self.delta =
                    self.delta
                        .add_components(0.0, (look_y - self.delta.y) * multiplier, 0.0);
            }
        }

        if self.abilities.flying {
            let original_y = self.delta.y;
            self.living_travel(level, input)?;
            self.delta = self.delta.with(Axis::Y, original_y * 0.6);
            Ok(())
        } else {
            self.living_travel(level, input)
        }
    }

    pub(in super::super) fn living_travel(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
    ) -> Result<(), Unsupported> {
        if (self.is_in_water() || self.is_in_lava()) && self.is_affected_by_fluids() {
            self.travel_in_fluid(level, input)
        } else {
            self.travel_in_air(level, input)
        }
    }

    pub(in super::super) fn travel_in_air(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
    ) -> Result<(), Unsupported> {
        let pos_below = self.get_on_pos(level, 0.500_001_f32);
        let block_friction = if self.on_ground {
            compute_modified_friction(level.info(pos_below).friction, 1.0)
        } else {
            1.0
        };
        let movement =
            self.handle_relative_friction_and_calculate_movement(level, input, block_friction)?;
        let movement_y = if level.has_chunk_at(pos_below) {
            movement.y - GRAVITY
        } else if self.position.y > f64::from(level.world.dimension().min_y) {
            -0.1
        } else {
            0.0
        };
        let air_drag = compute_modified_friction(0.91, 1.0);
        let friction = block_friction * air_drag;
        let vertical_friction = compute_modified_friction(0.98, 1.0);
        self.delta = Vec3::new(
            movement.x * f64::from(friction),
            movement_y * f64::from(vertical_friction),
            movement.z * f64::from(friction),
        );
        Ok(())
    }

    pub(in super::super) fn travel_in_fluid(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
    ) -> Result<(), Unsupported> {
        let is_falling = self.delta.y <= 0.0;
        let old_y = self.position.y;
        let base_gravity = GRAVITY;
        if self.is_in_water() {
            let mut slow_down = if self.sprinting { 0.9_f32 } else { 0.8_f32 };
            let speed = 0.02_f32;
            // The player's water movement efficiency is zero: no blend.
            let _ = &mut slow_down;
            self.move_relative(speed, input);
            self.move_self(level, self.delta)?;
            let mut movement = self.delta;
            if self.horizontal_collision && self.on_climbable(level) {
                movement = Vec3::new(movement.x, 0.2, movement.z);
            }
            movement = movement.multiply_components(
                f64::from(slow_down),
                f64::from(0.8_f32),
                f64::from(slow_down),
            );
            self.delta = self.fluid_falling_adjusted_movement(base_gravity, is_falling, movement);
            self.jump_out_of_fluid(level, old_y)
        } else {
            self.move_relative(0.02, input);
            self.move_self(level, self.delta)?;
            if self.lava.height <= self.fluid_jump_threshold() {
                self.delta = self.delta.multiply_components(0.5, f64::from(0.8_f32), 0.5);
                self.delta =
                    self.fluid_falling_adjusted_movement(base_gravity, is_falling, self.delta);
            } else {
                self.delta = self.delta.scale(0.5);
            }
            self.delta = self.delta.add_components(0.0, -base_gravity / 4.0, 0.0);
            self.jump_out_of_fluid(level, old_y)
        }
    }

    pub(in super::super) fn fluid_falling_adjusted_movement(
        &self,
        base_gravity: f64,
        is_falling: bool,
        movement: Vec3,
    ) -> Vec3 {
        if base_gravity != 0.0 && !self.sprinting {
            let y = if is_falling
                && (movement.y - 0.005).abs() >= 0.003
                && (movement.y - base_gravity / 16.0).abs() < 0.003
            {
                -0.003
            } else {
                movement.y - base_gravity / 16.0
            };
            Vec3::new(movement.x, y, movement.z)
        } else {
            movement
        }
    }

    pub(in super::super) fn jump_out_of_fluid(
        &mut self,
        level: &Level<'_>,
        old_y: f64,
    ) -> Result<(), Unsupported> {
        let movement = self.delta;
        if self.horizontal_collision {
            let lift = movement.y + f64::from(0.6_f32) - self.position.y + old_y;
            let moved = self.bounding_box.move_by(movement.x, lift, movement.z);
            if level.no_collision(moved, self.fall_distance, self.position)?
                && !self.contains_any_liquid(level, moved)
            {
                self.delta = Vec3::new(movement.x, f64::from(0.3_f32), movement.z);
            }
        }
        Ok(())
    }

    pub(in super::super) fn contains_any_liquid(&self, level: &Level<'_>, area: Aabb) -> bool {
        let x0 = floor_f64(area.min_x);
        let x1 = voxel_core::math::ceil_f64(area.max_x);
        let y0 = floor_f64(area.min_y);
        let y1 = voxel_core::math::ceil_f64(area.max_y);
        let z0 = floor_f64(area.min_z);
        let z1 = voxel_core::math::ceil_f64(area.max_z);
        for x in x0..x1 {
            for y in y0..y1 {
                for z in z0..z1 {
                    if level.info(BlockPos::new(x, y, z)).fluid != Fluid::Empty {
                        return true;
                    }
                }
            }
        }
        false
    }
}
