use super::*;

impl LocalPlayer {
    #[expect(
        clippy::cognitive_complexity,
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    pub(super) fn local_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
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

    pub(super) fn player_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
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

    pub(super) fn living_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
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

    pub(super) fn player_travel(
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

    pub(super) fn living_travel(
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

    pub(super) fn travel_in_air(
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

    pub(super) fn travel_in_fluid(
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

    pub(super) fn fluid_falling_adjusted_movement(
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

    pub(super) fn jump_out_of_fluid(
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

    pub(super) fn contains_any_liquid(&self, level: &Level<'_>, area: Aabb) -> bool {
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

    pub(super) fn handle_relative_friction_and_calculate_movement(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
        friction: f32,
    ) -> Result<Vec3, Unsupported> {
        let speed = self.friction_influenced_speed(friction);
        self.move_relative(speed, input);
        self.delta = self.handle_on_climbable(level, self.delta)?;
        self.move_self(level, self.delta)?;
        let mut movement = self.delta;
        if (self.horizontal_collision || self.jumping) && self.on_climbable(level) {
            movement = Vec3::new(movement.x, 0.2, movement.z);
        }
        Ok(movement)
    }

    pub(super) fn friction_influenced_speed(&self, block_friction: f32) -> f32 {
        if self.on_ground {
            // The reference compares the float against a double literal.
            if f64::from(block_friction) > 0.6 {
                self.speed()
                    * (0.216_000_02_f32 / (block_friction * block_friction * block_friction))
            } else {
                self.speed()
            }
        } else if self.abilities.flying {
            if self.sprinting {
                self.abilities.flying_speed() * 2.0
            } else {
                self.abilities.flying_speed()
            }
        } else if self.sprinting {
            0.025_999_999_f32
        } else {
            0.02_f32
        }
    }

    pub(super) fn handle_on_climbable(
        &mut self,
        level: &Level<'_>,
        delta: Vec3,
    ) -> Result<Vec3, Unsupported> {
        if !self.on_climbable(level) {
            return Ok(delta);
        }
        self.fall_distance = 0.0;
        let max = f64::from(0.15_f32);
        let x = java_math::clamp(delta.x, -max, max);
        let z = java_math::clamp(delta.z, -max, max);
        let mut y = java_math::max(delta.y, -max);
        if y < 0.0 && self.is_shift_key_down() && !self.abilities.flying {
            if level.info(self.block_position()).name == "minecraft:scaffolding" {
                return unsupported("climbing scaffolding");
            }
            y = 0.0;
        }
        Ok(Vec3::new(x, y, z))
    }

    pub(super) fn on_climbable(&self, level: &Level<'_>) -> bool {
        if self.is_spectator() {
            return false;
        }
        let pos = self.block_position();
        let info = level.info(pos);
        if info.flags & flag::CLIMBABLE != 0 {
            return true;
        }
        if info.flags & flag::TRAPDOOR != 0 {
            let state = level.state_id(pos);
            let open = property(state, "open") == Some("true");
            if open {
                let below_pos = BlockPos::new(pos.x, pos.y - 1, pos.z);
                let below = level.info(below_pos);
                if below.flags & flag::LADDER != 0 {
                    return property(level.state_id(below_pos), "facing")
                        == property(state, "facing");
                }
            }
        }
        false
    }

    pub(super) fn move_relative(&mut self, speed: f32, input: Vec3) {
        let relative = crate::input::local_player_relative_movement(input, speed, self.y_rot);
        self.delta = self.delta.add_vec3(relative);
    }

    /// `LocalPlayer.move`: the entity move, then the auto-jump look-ahead.
    pub(super) fn move_self(&mut self, level: &Level<'_>, delta: Vec3) -> Result<(), Unsupported> {
        let previous_x = self.position.x;
        let previous_z = self.position.z;
        self.entity_move(level, delta)?;
        let delta_x = (self.position.x - previous_x) as f32;
        let delta_z = (self.position.z - previous_z) as f32;
        self.update_auto_jump(level, delta_x, delta_z)?;
        // `Mth.length(float, float)` squares in double: there is no float
        // `lengthSquared` of two values.
        let (x, z) = (f64::from(delta_x), f64::from(delta_z));
        self.walk_dist += (x * x + z * z).sqrt() as f32 * 0.6;
        Ok(())
    }

    pub(super) fn entity_move(
        &mut self,
        level: &Level<'_>,
        delta: Vec3,
    ) -> Result<(), Unsupported> {
        if self.no_physics {
            self.set_pos(self.position.add_vec3(delta));
            self.horizontal_collision = false;
            self.vertical_collision = false;
            self.vertical_collision_below = false;
            self.minor_horizontal_collision = false;
            return Ok(());
        }

        let mut delta = delta;
        if self.stuck_speed_multiplier.length_squared() > 1.0e-7 {
            let stuck = self.stuck_speed_multiplier;
            delta = delta.multiply_components(stuck.x, stuck.y, stuck.z);
            self.stuck_speed_multiplier = Vec3::ZERO;
            self.delta = Vec3::ZERO;
        }
        let delta = self.maybe_back_off_from_edge(level, delta)?;
        let movement = self.collide(level, delta)?;
        let movement_length = movement.length_squared();
        if movement_length > 1.0e-7 || delta.length_squared() - movement_length < 1.0e-7 {
            if self.fall_distance != 0.0 && movement_length >= 1.0 {
                self.check_fall_reset_clip(level, movement)?;
            }
            let from = self.position;
            let to = from.add_vec3(movement);
            self.movements_this_tick.push(Movement {
                from,
                to,
                original: Some(delta),
            });
            self.set_pos(to);
        }

        let x_collision = !equal_f64(delta.x, movement.x);
        let z_collision = !equal_f64(delta.z, movement.z);
        self.horizontal_collision = x_collision || z_collision;
        let moved_vertically = delta.y.abs() > 0.0;
        // The local player is always its own authority.
        self.vertical_collision = delta.y != movement.y;
        self.vertical_collision_below = self.vertical_collision && delta.y < 0.0;
        self.on_ground = self.vertical_collision_below;
        self.check_supporting_block(level, self.on_ground, Some(movement))?;

        self.minor_horizontal_collision = self.horizontal_collision
            && is_minor_horizontal_collision(self.y_rot, self.xxa, self.zza, movement);

        let effect_pos = self.get_on_pos(level, 0.2);
        let effect_state = level.info(effect_pos);
        self.check_fall_damage(level, movement.y, effect_pos)?;

        if moved_vertically && self.vertical_collision || self.horizontal_collision {
            self.restitute_movement_after_collisions(
                &effect_state,
                x_collision,
                z_collision,
                movement,
            );
        }

        self.apply_movement_emission(level, movement, effect_pos);

        let speed_factor = f64::from(self.block_speed_factor(level));
        self.delta = self
            .delta
            .multiply_components(speed_factor, 1.0, speed_factor);
        Ok(())
    }

    /// `Level.clip` with the fall-damage-resetting block shapes and water:
    /// whether the fall so far crosses anything that forgives it.
    pub(super) fn check_fall_reset_clip(
        &mut self,
        level: &Level<'_>,
        movement: Vec3,
    ) -> Result<(), Unsupported> {
        let distance = movement.length().min(8.0);
        let from = self.position;
        let to = from.add_vec3(movement.normalized().scale(distance));
        if level.clip_hits_fall_reset(from, to) {
            self.fall_distance = 0.0;
        }
        Ok(())
    }

    pub(super) fn check_fall_damage(
        &mut self,
        level: &Level<'_>,
        ya: f64,
        pos: BlockPos,
    ) -> Result<(), Unsupported> {
        if !self.is_in_water() {
            self.update_fluid_interaction(level)?;
        }
        if !self.is_in_water() && ya < 0.0 {
            self.fall_distance -= f64::from(ya as f32);
        }
        if self.on_ground {
            // The fall itself only hurts on the server; the client lands with the sounds.
            if self.fall_distance > 0.0 {
                self.fall_on(level, pos, self.fall_distance);
            }
            self.fall_distance = 0.0;
        }
        Ok(())
    }

    pub(super) fn restitute_movement_after_collisions(
        &mut self,
        effect_state: &StateInfo,
        x_collision: bool,
        z_collision: bool,
        movement: Vec3,
    ) {
        let suppressing_bounce = self.is_shift_key_down();
        let mut restitution = 0.0_f64;
        let current = self.delta;
        let mut after = current;
        if x_collision {
            after = after.with(Axis::X, -current.x * restitution);
        }
        if z_collision {
            after = after.with(Axis::Z, -current.z * restitution);
        }
        if self.vertical_collision {
            if self.vertical_collision_below {
                restitution = if !(-current.y < GRAVITY)
                    && !suppressing_bounce
                    && effect_state.flags & flag::SUPPRESSES_BOUNCE == 0
                {
                    java_math::max(restitution, f64::from(effect_state.bounce))
                } else {
                    0.0
                };
            }
            let (gravity_compensation, effective_drag) = if restitution > 0.0 {
                let portion = movement.y / current.y;
                let air_drag = f64::from(compute_modified_friction(0.98, 1.0));
                (portion * GRAVITY, 1.0 + portion * (air_drag - 1.0))
            } else {
                (0.0, 1.0)
            };
            after = after.with(
                Axis::Y,
                (gravity_compensation - current.y) * effective_drag * restitution,
            );
        }
        self.delta = after;
    }

    pub(super) fn collide(&self, level: &Level<'_>, movement: Vec3) -> Result<Vec3, Unsupported> {
        let aabb = self.bounding_box;
        let movement_step = if movement.length_squared() == 0.0 {
            movement
        } else {
            let colliders = level.colliders(
                aabb.expand_towards_vec3(movement),
                self.fall_distance,
                self.position,
            )?;
            collide_entity_movement(movement, aabb, &colliders)
        };
        let x_collision = movement.x != movement_step.x;
        let y_collision = movement.y != movement_step.y;
        let z_collision = movement.z != movement_step.z;
        let on_ground_after_collision = y_collision && movement.y < 0.0;
        let max_up_step = STEP_HEIGHT as f32;
        if (on_ground_after_collision || self.on_ground) && (x_collision || z_collision) {
            let grounded = if on_ground_after_collision {
                aabb.move_by(0.0, movement_step.y, 0.0)
            } else {
                aabb
            };
            let mut step_up_area =
                grounded.expand_towards(movement.x, f64::from(max_up_step), movement.z);
            if !on_ground_after_collision {
                step_up_area = step_up_area.expand_towards(0.0, f64::from(-1.0e-5_f32), 0.0);
            }
            let colliders = level.colliders(step_up_area, self.fall_distance, self.position)?;
            return Ok(select_step_up_movement(
                movement,
                movement_step,
                aabb,
                grounded,
                &colliders,
                max_up_step,
                movement_step.y as f32,
            ));
        }
        Ok(movement_step)
    }

    pub(super) fn maybe_back_off_from_edge(
        &self,
        level: &Level<'_>,
        delta: Vec3,
    ) -> Result<Vec3, Unsupported> {
        let max_down_step = STEP_HEIGHT as f32;
        if self.abilities.flying
            || delta.y > 0.0
            || !self.is_shift_key_down()
            || !self.is_above_ground(level, max_down_step)?
        {
            return Ok(delta);
        }
        let mut delta_x = delta.x;
        let mut delta_z = delta.z;
        let step = 0.05;
        let step_x = java_math::signum(delta_x) * step;
        let step_z = java_math::signum(delta_z) * step;
        let max_down = f64::from(max_down_step);

        while delta_x != 0.0 && self.can_fall_at_least(level, delta_x, 0.0, max_down)? {
            if delta_x.abs() <= step {
                delta_x = 0.0;
                break;
            }
            delta_x -= step_x;
        }
        while delta_z != 0.0 && self.can_fall_at_least(level, 0.0, delta_z, max_down)? {
            if delta_z.abs() <= step {
                delta_z = 0.0;
                break;
            }
            delta_z -= step_z;
        }
        while delta_x != 0.0
            && delta_z != 0.0
            && self.can_fall_at_least(level, delta_x, delta_z, max_down)?
        {
            if delta_x.abs() <= step {
                delta_x = 0.0;
            } else {
                delta_x -= step_x;
            }
            if delta_z.abs() <= step {
                delta_z = 0.0;
            } else {
                delta_z -= step_z;
            }
        }
        Ok(Vec3::new(delta_x, delta.y, delta_z))
    }

    pub(super) fn is_above_ground(
        &self,
        level: &Level<'_>,
        max_down_step: f32,
    ) -> Result<bool, Unsupported> {
        Ok(self.on_ground
            || self.fall_distance < f64::from(max_down_step)
                && !self.can_fall_at_least(
                    level,
                    0.0,
                    0.0,
                    f64::from(max_down_step) - self.fall_distance,
                )?)
    }

    pub(super) fn can_fall_at_least(
        &self,
        level: &Level<'_>,
        delta_x: f64,
        delta_z: f64,
        min_height: f64,
    ) -> Result<bool, Unsupported> {
        let bb = self.bounding_box;
        level.no_collision(
            Aabb::new(
                bb.min_x + 1.0e-7 + delta_x,
                bb.min_y - min_height - 1.0e-7,
                bb.min_z + 1.0e-7 + delta_z,
                bb.max_x - 1.0e-7 + delta_x,
                bb.min_y,
                bb.max_z - 1.0e-7 + delta_z,
            ),
            self.fall_distance,
            self.position,
        )
    }

    pub(super) fn check_supporting_block(
        &mut self,
        level: &Level<'_>,
        on_ground: bool,
        movement: Option<Vec3>,
    ) -> Result<(), Unsupported> {
        level.check_supporting_block(
            (
                &mut self.main_supporting_block,
                &mut self.on_ground_no_blocks,
            ),
            (self.bounding_box, self.position, self.fall_distance),
            on_ground,
            movement,
        )
    }

    pub(super) fn get_on_pos(&self, level: &Level<'_>, offset: f32) -> BlockPos {
        level.on_pos(self.main_supporting_block, self.position, offset)
    }

    pub(super) fn block_position(&self) -> BlockPos {
        BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        )
    }

    pub(super) fn block_speed_factor(&self, level: &Level<'_>) -> f32 {
        if self.abilities.flying {
            return 1.0;
        }
        level.block_speed_factor(self.block_position(), self.get_on_pos(level, 0.500_001))
    }

    pub(super) fn block_jump_factor(&self, level: &Level<'_>) -> f32 {
        let here = level.info(self.block_position()).jump_factor;
        let below = level.info(self.get_on_pos(level, 0.500_001)).jump_factor;
        if here == 1.0 { below } else { here }
    }

    pub(super) fn jump_from_ground(&mut self, level: &Level<'_>) {
        let jump_power = JUMP_STRENGTH * 1.0_f32 * self.block_jump_factor(level) + 0.0_f32;
        if jump_power <= 1.0e-5 {
            return;
        }
        self.delta = Vec3::new(
            self.delta.x,
            java_math::max(f64::from(jump_power), self.delta.y),
            self.delta.z,
        );
        if self.sprinting {
            let angle = self.y_rot * MTH_DEG_TO_RAD;
            self.delta = self.delta.add_vec3(Vec3::new(
                f64::from(-mth_sin_f64(f64::from(angle))) * 0.2,
                0.0,
                f64::from(mth_cos_f64(f64::from(angle))) * 0.2,
            ));
        }
    }
}
