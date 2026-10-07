use super::*;

impl LocalPlayer {
    pub(in super::super) fn handle_relative_friction_and_calculate_movement(
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

    pub(in super::super) fn friction_influenced_speed(&self, block_friction: f32) -> f32 {
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

    pub(in super::super) fn handle_on_climbable(
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

    pub(in super::super) fn on_climbable(&self, level: &Level<'_>) -> bool {
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

    pub(in super::super) fn move_relative(&mut self, speed: f32, input: Vec3) {
        let relative = crate::input::local_player_relative_movement(input, speed, self.y_rot);
        self.delta = self.delta.add_vec3(relative);
    }

    /// `LocalPlayer.move`: the entity move, then the auto-jump look-ahead.
    pub(in super::super) fn move_self(
        &mut self,
        level: &Level<'_>,
        delta: Vec3,
    ) -> Result<(), Unsupported> {
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

    pub(in super::super) fn entity_move(
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
    pub(in super::super) fn check_fall_reset_clip(
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

    pub(in super::super) fn check_fall_damage(
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

    pub(in super::super) fn restitute_movement_after_collisions(
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

    pub(in super::super) fn collide(
        &self,
        level: &Level<'_>,
        movement: Vec3,
    ) -> Result<Vec3, Unsupported> {
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

    pub(in super::super) fn maybe_back_off_from_edge(
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

    pub(in super::super) fn is_above_ground(
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

    pub(in super::super) fn can_fall_at_least(
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

    pub(in super::super) fn check_supporting_block(
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

    pub(in super::super) fn get_on_pos(&self, level: &Level<'_>, offset: f32) -> BlockPos {
        level.on_pos(self.main_supporting_block, self.position, offset)
    }

    pub(in super::super) fn block_position(&self) -> BlockPos {
        BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        )
    }

    pub(in super::super) fn block_speed_factor(&self, level: &Level<'_>) -> f32 {
        if self.abilities.flying {
            return 1.0;
        }
        level.block_speed_factor(self.block_position(), self.get_on_pos(level, 0.500_001))
    }

    pub(in super::super) fn block_jump_factor(&self, level: &Level<'_>) -> f32 {
        let here = level.info(self.block_position()).jump_factor;
        let below = level.info(self.get_on_pos(level, 0.500_001)).jump_factor;
        if here == 1.0 { below } else { here }
    }
}
