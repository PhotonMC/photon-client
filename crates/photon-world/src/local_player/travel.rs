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
        self.attribute("minecraft:movement_speed") as f32
    }

    /// `LivingEntity.getJumpPower`: the jump strength attribute on this block, then what jump
    /// boost adds.
    pub(super) fn jump_power(&self, level: &Level<'_>) -> f32 {
        let jump_boost = self
            .hands
            .jump_boost
            .map_or(0.0, |amplifier| 0.1_f32 * (amplifier as f32 + 1.0));
        self.attribute("minecraft:jump_strength") as f32 * 1.0_f32 * self.block_jump_factor(level)
            + jump_boost
    }

    /// `LivingEntity.getEffectiveGravity`: the gravity attribute, which slow falling caps while
    /// the player falls.
    pub(super) fn effective_gravity(&self) -> f64 {
        let gravity = self.attribute("minecraft:gravity");
        if self.delta.y <= 0.0 && self.hands.slow_falling {
            gravity.min(0.01)
        } else {
            gravity
        }
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
        self.move_as(level, MoverType::SelfMove, delta)
    }

    /// `LocalPlayer.move` for a kind of mover.
    pub(super) fn move_as(
        &mut self,
        level: &Level<'_>,
        mover: MoverType,
        delta: Vec3,
    ) -> Result<(), Unsupported> {
        let previous_x = self.position.x;
        let previous_z = self.position.z;
        self.entity_move(level, mover, delta)?;
        let delta_x = (self.position.x - previous_x) as f32;
        let delta_z = (self.position.z - previous_z) as f32;
        self.update_auto_jump(level, delta_x, delta_z)?;
        // `Mth.length(float, float)` squares in double: there is no float
        // `lengthSquared` of two values.
        let (x, z) = (f64::from(delta_x), f64::from(delta_z));
        self.avatar.walk_dist += (x * x + z * z).sqrt() as f32 * 0.6;
        Ok(())
    }

    pub(super) fn entity_move(
        &mut self,
        level: &Level<'_>,
        mover: MoverType,
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

        let Some(delta) = self.delta_to_collide(level, mover, delta)? else {
            return Ok(());
        };
        let movement = self.collide(level, delta)?;
        let movement_length = movement.length_squared();
        if movement_length > 1.0e-7 || delta.length_squared() - movement_length < 1.0e-7 {
            if self.fall_distance != 0.0 && movement_length >= 1.0 {
                self.check_fall_reset_clip(level, movement)?;
            }
            let from = self.position;
            let to = from.add_vec3(movement);
            self.trace.movements.push(Movement {
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

    /// What `Entity.move` makes of a move before it collides: a piston's is limited, the stuck
    /// multiplier slows what is not a piston's, and a sneaking player does not walk off an edge.
    /// `None` when nothing is left of it.
    fn delta_to_collide(
        &mut self,
        level: &Level<'_>,
        mover: MoverType,
        delta: Vec3,
    ) -> Result<Option<Vec3>, Unsupported> {
        let piston = mover == MoverType::Piston;
        let mut delta = delta;
        if piston {
            delta = self.limit_piston_movement(level, delta);
            if delta.x == 0.0 && delta.y == 0.0 && delta.z == 0.0 {
                return Ok(None);
            }
        }
        if self.stuck_speed_multiplier.length_squared() > 1.0e-7 {
            let stuck = self.stuck_speed_multiplier;
            if !piston {
                delta = delta.multiply_components(stuck.x, stuck.y, stuck.z);
            }
            self.stuck_speed_multiplier = Vec3::ZERO;
            self.delta = Vec3::ZERO;
        }
        if piston {
            return Ok(Some(delta));
        }
        self.maybe_back_off_from_edge(level, delta).map(Some)
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
        if self.elytra.fall_flying {
            self.check_fall_distance_accumulation();
        }
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
        } else if self.elytra.fall_flying {
            self.travel_fall_flying(level, input)
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
            compute_modified_friction(
                level.info(pos_below).friction,
                self.attribute("minecraft:friction_modifier") as f32,
            )
        } else {
            1.0
        };
        let movement =
            self.handle_relative_friction_and_calculate_movement(level, input, block_friction)?;
        let movement_y = if let Some(amplifier) = self.hands.levitation {
            movement.y + (0.05 * f64::from(amplifier + 1) - movement.y) * 0.2
        } else if level.has_chunk_at(pos_below) {
            movement.y - self.effective_gravity()
        } else if self.position.y > f64::from(level.world.dimension().min_y) {
            -0.1
        } else {
            0.0
        };
        let air_drag_modifier = self.attribute("minecraft:air_drag_modifier") as f32;
        let air_drag = compute_modified_friction(0.91, air_drag_modifier);
        let friction = block_friction * air_drag;
        let vertical_friction = compute_modified_friction(0.98, air_drag_modifier);
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
        let base_gravity = self.effective_gravity();
        if self.is_in_water() {
            let mut slow_down = if self.sprinting { 0.9_f32 } else { 0.8_f32 };
            let mut speed = 0.02_f32;
            // Depth strider's water movement efficiency, half of it off the ground.
            let mut water_walker = self.attribute("minecraft:water_movement_efficiency") as f32;
            if !self.on_ground {
                water_walker *= 0.5;
            }
            if water_walker > 0.0 {
                slow_down += (0.546_000_06_f32 - slow_down) * water_walker;
                speed += (self.speed() - speed) * water_walker;
            }
            if self.hands.dolphins_grace {
                slow_down = 0.96;
            }
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
            if base_gravity != 0.0 {
                self.delta = self.delta.add_components(0.0, -base_gravity / 4.0, 0.0);
            }
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
