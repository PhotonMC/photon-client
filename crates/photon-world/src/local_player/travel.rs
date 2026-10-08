use super::*;

impl LocalPlayer {
    /// `getAttributeValue(MOVEMENT_SPEED)` as the field of view reads it.
    pub fn movement_speed(&self) -> f32 {
        self.speed()
    }

    pub(super) fn apply_current(&mut self, tracker: FluidTracker, scale: f64) {
        self.delta =
            crate::fluid::apply_current(tracker, self.delta, scale, crate::fluid::Pushed::Player);
    }

    /// `getAttributeValue(MOVEMENT_SPEED)`, narrowed as `Player.getSpeed` does.
    pub(super) fn speed(&self) -> f32 {
        let base = f64::from(MOVEMENT_SPEED_BASE);
        let value = if self.sprinting {
            base * (1.0 + f64::from(SPRINTING_MODIFIER))
        } else {
            base
        };
        value.clamp(0.0, 1024.0) as f32
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

    pub(super) fn block_speed_factor(&self, level: &Level<'_>) -> f32 {
        if self.abilities.flying {
            return 1.0;
        }
        level.block_speed_factor(self.block_position(), self.get_on_pos(level, 0.500_001))
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
}

pub(crate) fn compute_modified_friction(friction: f32, modifier: f32) -> f32 {
    (1.0_f32 - (1.0_f32 - friction) * modifier).clamp(0.0, 1.0)
}
