use super::*;

impl MountBody {
    /// `LivingEntity.onClimbable`; `AbstractHorse` overrides it to never.
    pub(super) fn on_climbable(&self, level: &Level<'_>) -> bool {
        if matches!(self.kind, MountKind::Horse | MountKind::Camel) {
            return false;
        }
        let pos = self.block_position();
        let info = level.info(pos);
        if info.flags & flag::CLIMBABLE != 0 {
            return true;
        }
        if info.flags & flag::TRAPDOOR != 0 {
            let state = level.state_id(pos);
            if property(state, "open") == Some("true") {
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

    /// `Entity.moveRelative`.
    pub(super) fn move_relative(&mut self, speed: f32, input: Vec3) {
        let relative = crate::input::local_player_relative_movement(input, speed, self.y_rot);
        self.delta = self.delta.add_vec3(relative);
    }

    /// `Entity.move(SELF, ...)`: collision (it steps up what its step height allows), the flags,
    /// the fall, what the blocks do and the block's speed factor.
    pub(super) fn move_self(&mut self, level: &Level<'_>, delta: Vec3) -> Result<(), Unsupported> {
        let mut delta = delta;
        let (sx, sy, sz) = self.state.stuck_speed_multiplier;
        let stuck = Vec3::new(sx, sy, sz);
        if stuck.length_squared() > 1.0e-7 {
            delta = delta.multiply_components(stuck.x, stuck.y, stuck.z);
            self.state.stuck_speed_multiplier = (0.0, 0.0, 0.0);
            self.delta = Vec3::ZERO;
        }
        let movement = self.collide(level, delta)?;
        let movement_length = movement.length_squared();
        if movement_length > 1.0e-7 || delta.length_squared() - movement_length < 1.0e-7 {
            if self.state.fall_distance != 0.0 && movement_length >= 1.0 {
                let distance = movement.length().min(8.0);
                let to = self
                    .position
                    .add_vec3(movement.normalized().scale(distance));
                if level.clip_hits_fall_reset(self.position, to) {
                    self.state.fall_distance = 0.0;
                }
            }
            let from = self.position;
            let to = from.add_vec3(movement);
            self.movements.push(Movement {
                from,
                to,
                original: Some(delta),
            });
            self.set_pos(to);
        }

        let x_collision = !equal_f64(delta.x, movement.x);
        let z_collision = !equal_f64(delta.z, movement.z);
        self.state.horizontal_collision = x_collision || z_collision;
        let moved_vertically = delta.y.abs() > 0.0;
        self.state.vertical_collision = delta.y != movement.y;
        self.state.vertical_collision_below = self.state.vertical_collision && delta.y < 0.0;
        self.on_ground = self.state.vertical_collision_below;
        let moved_box = self.bounding_box();
        self.seen_by(level).check_supporting_block(
            (&mut self.supporting_block, &mut self.on_ground_no_blocks),
            (moved_box, self.position, self.state.fall_distance),
            self.on_ground,
            Some(movement),
        )?;

        let effect_pos = self.get_on_pos(level, 0.2);
        let effect_info = level.info(effect_pos);
        self.check_fall_damage(level, movement.y)?;

        if moved_vertically && self.state.vertical_collision || self.state.horizontal_collision {
            self.restitute_movement_after_collisions(
                effect_info.flags & flag::SUPPRESSES_BOUNCE != 0,
                effect_info.bounce,
                (x_collision, z_collision),
                movement,
            );
        }

        let below = self.get_on_pos(level, 0.500_001);
        let speed_factor = f64::from(level.block_speed_factor(self.block_position(), below));
        self.delta = self
            .delta
            .multiply_components(speed_factor, 1.0, speed_factor);
        Ok(())
    }

    /// `Entity.collide`: the blocks stop the move, and a mount on the ground steps up what its
    /// step height lets it.
    pub(super) fn collide(&self, level: &Level<'_>, movement: Vec3) -> Result<Vec3, Unsupported> {
        let level = &self.seen_by(level);
        let aabb = self.bounding_box();
        let movement_step = if movement.length_squared() == 0.0 {
            movement
        } else {
            let colliders = level.colliders(
                aabb.expand_towards_vec3(movement),
                self.state.fall_distance,
                self.position,
            )?;
            collide_entity_movement(movement, aabb, &colliders)
        };
        let x_collision = movement.x != movement_step.x;
        let y_collision = movement.y != movement_step.y;
        let z_collision = movement.z != movement_step.z;
        let on_ground_after_collision = y_collision && movement.y < 0.0;
        let max_up_step = self.attributes.step_height;
        if max_up_step > 0.0
            && (on_ground_after_collision || self.on_ground)
            && (x_collision || z_collision)
        {
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
            let colliders =
                level.colliders(step_up_area, self.state.fall_distance, self.position)?;
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

    /// `Entity.restituteMovementAfterCollisions` for a mob: a bump keeps none of its speed, a
    /// springy block throws it back at four fifths of its spring (a living entity at all of it).
    pub(super) fn restitute_movement_after_collisions(
        &mut self,
        suppresses_bounce: bool,
        block_bounce: f32,
        (x_collision, z_collision): (bool, bool),
        movement: Vec3,
    ) {
        let mut restitution = 0.0_f64;
        let current = self.delta;
        let mut after = current;
        if x_collision {
            after = Vec3::new(-current.x * restitution, after.y, after.z);
        }
        if z_collision {
            after = Vec3::new(after.x, after.y, -current.z * restitution);
        }
        if self.state.vertical_collision {
            if self.state.vertical_collision_below {
                let falls_slowly = -current.y < self.gravity();
                restitution = if !falls_slowly && !suppresses_bounce {
                    java_math::max(restitution, f64::from(block_bounce))
                } else {
                    0.0
                };
            }
            let (gravity_compensation, effective_drag) = if restitution > 0.0 {
                let portion = movement.y / current.y;
                let air_drag = f64::from(compute_modified_friction(
                    0.98,
                    self.attributes.air_drag_modifier,
                ));
                (portion * self.gravity(), 1.0 + portion * (air_drag - 1.0))
            } else {
                (0.0, 1.0)
            };
            after = Vec3::new(
                after.x,
                (gravity_compensation - current.y) * effective_drag * restitution,
                after.z,
            );
        }
        self.delta = after;
    }

    /// `Entity.getFluidJumpThreshold`.
    pub(super) fn fluid_jump_threshold(&self) -> f64 {
        if self.dimensions.eye_height() < 0.4 {
            0.0
        } else {
            0.4
        }
    }

    /// `Entity.getBlockSpeedFactor`.
    pub(super) fn block_speed_factor(&self, level: &Level<'_>) -> f32 {
        let below = self.get_on_pos(level, 0.500_001);
        level.block_speed_factor(self.block_position(), below)
    }

    /// `Entity.getBlockJumpFactor`.
    pub(super) fn block_jump_factor(&self, level: &Level<'_>) -> f32 {
        let here = level.info(self.block_position()).jump_factor;
        let below = level.info(self.get_on_pos(level, 0.500_001)).jump_factor;
        if here == 1.0 { below } else { here }
    }

    /// `Entity.updateFluidInteraction`: the water and lava the box is in, and what their
    /// currents do to the mount.
    pub(super) fn update_fluid_interaction(
        &mut self,
        level: &Level<'_>,
    ) -> Result<(), Unsupported> {
        let found = fluid::scan(
            level,
            (
                self.bounding_box(),
                self.position,
                self.dimensions.eye_height(),
            ),
            false,
        )?;
        self.water = found.water;
        self.lava = found.lava;
        if self.water.height > 0.0 {
            self.state.fall_distance = 0.0;
            self.delta =
                fluid::apply_current(self.water, self.delta, WATER_FLOW_SCALE, Pushed::Other);
        }
        if self.lava.height > 0.0 {
            self.delta =
                fluid::apply_current(self.lava, self.delta, LAVA_FLOW_SCALE, Pushed::Other);
        }
        Ok(())
    }

    /// `LivingEntity.travel`: in a fluid, or in the air.
    pub(super) fn travel(&mut self, level: &Level<'_>, input: Vec3) -> Result<(), Unsupported> {
        // `Camel.travel`: a camel that sits, or sits down or rises, stays where it is.
        let input = if self.kind == MountKind::Camel && self.camel.refuses_to_move && self.on_ground
        {
            self.delta = self.delta.multiply_components(0.0, 1.0, 0.0);
            input.multiply_components(0.0, 1.0, 0.0)
        } else {
            input
        };
        // `shouldTravelInFluid`: a strider does not travel in the lava it stands on.
        let stands_on_fluid = self.kind == MountKind::Strider
            && level.info(self.block_position()).fluid == Fluid::Lava;
        if (self.is_in_water() || self.is_in_lava()) && !stands_on_fluid {
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
            compute_modified_friction(
                level.info(pos_below).friction,
                self.attributes.friction_modifier,
            )
        } else {
            1.0
        };
        let movement =
            self.handle_relative_friction_and_calculate_movement(level, input, block_friction)?;
        // The client level has every chunk (`ClientLevel.hasChunk`).
        let movement_y = movement.y - self.gravity();
        let air_drag = compute_modified_friction(0.91, self.attributes.air_drag_modifier);
        let friction = block_friction * air_drag;
        let vertical_friction = compute_modified_friction(0.98, self.attributes.air_drag_modifier);
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
        let base_gravity = self.gravity();
        if self.is_in_water() {
            // `travelInWater`.
            let mut slow_down = 0.8_f32;
            let mut speed = 0.02_f32;
            let mut water_walker = self.attributes.water_movement_efficiency;
            if !self.on_ground {
                water_walker *= 0.5;
            }
            if water_walker > 0.0 {
                slow_down += (0.546_000_06 - slow_down) * water_walker;
                speed += (self.attributes.speed - speed) * water_walker;
            }
            self.move_relative(speed, input);
            self.move_self(level, self.delta)?;
            let mut movement = self.delta;
            if self.state.horizontal_collision && self.on_climbable(level) {
                movement = Vec3::new(movement.x, 0.2, movement.z);
            }
            let movement = movement.multiply_components(
                f64::from(slow_down),
                f64::from(0.8_f32),
                f64::from(slow_down),
            );
            self.delta = fluid_falling_adjusted_movement(base_gravity, is_falling, movement);
            self.jump_out_of_fluid(level, old_y)?;
            // `floatInWaterWhileRidden`.
            if self.floats_when_ridden && self.water.height > self.fluid_jump_threshold() {
                self.delta = self.delta.add_components(0.0, f64::from(0.04_f32), 0.0);
            }
            Ok(())
        } else {
            // `travelInLava`.
            self.move_relative(0.02, input);
            self.move_self(level, self.delta)?;
            if self.lava.height <= self.fluid_jump_threshold() {
                self.delta = self.delta.multiply_components(0.5, f64::from(0.8_f32), 0.5);
                self.delta = fluid_falling_adjusted_movement(base_gravity, is_falling, self.delta);
            } else {
                self.delta = self.delta.scale(0.5);
            }
            if base_gravity != 0.0 {
                self.delta = self.delta.add_components(0.0, -base_gravity / 4.0, 0.0);
            }
            self.jump_out_of_fluid(level, old_y)
        }
    }

    /// `LivingEntity.jumpOutOfFluid`.
    pub(super) fn jump_out_of_fluid(
        &mut self,
        level: &Level<'_>,
        old_y: f64,
    ) -> Result<(), Unsupported> {
        let movement = self.delta;
        if self.state.horizontal_collision {
            let lift = movement.y + f64::from(0.6_f32) - self.position.y + old_y;
            let moved = self.bounding_box().move_by(movement.x, lift, movement.z);
            if self
                .seen_by(level)
                .no_collision(moved, self.state.fall_distance, self.position)?
                && !contains_any_liquid(level, moved)
            {
                self.delta = Vec3::new(movement.x, f64::from(0.3_f32), movement.z);
            }
        }
        Ok(())
    }

    /// `LivingEntity.handleRelativeFrictionAndCalculateMovement`.
    pub(super) fn handle_relative_friction_and_calculate_movement(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
        friction: f32,
    ) -> Result<Vec3, Unsupported> {
        let speed = self.friction_influenced_speed(friction);
        self.move_relative(speed, input);
        self.delta = self.handle_on_climbable(level, self.delta);
        self.move_self(level, self.delta)?;
        let mut movement = self.delta;
        // `jumping` is the mount's own, which only its AI sets.
        if self.state.horizontal_collision && self.on_climbable(level) {
            movement = Vec3::new(movement.x, 0.2, movement.z);
        }
        Ok(movement)
    }

    /// `LivingEntity.getFrictionInfluencedSpeed`: in the air a mount a player rides goes at a
    /// tenth of its speed.
    pub(super) fn friction_influenced_speed(&self, block_friction: f32) -> f32 {
        let speed = self.attributes.speed;
        if self.on_ground {
            if f64::from(block_friction) > 0.6 {
                speed * (0.216_000_02_f32 / (block_friction * block_friction * block_friction))
            } else {
                speed
            }
        } else {
            speed * 0.1
        }
    }

    /// `LivingEntity.handleOnClimbable`.
    pub(super) fn handle_on_climbable(&mut self, level: &Level<'_>, delta: Vec3) -> Vec3 {
        if !self.on_climbable(level) {
            return delta;
        }
        self.state.fall_distance = 0.0;
        let max = f64::from(0.15_f32);
        Vec3::new(
            java_math::clamp(delta.x, -max, max),
            java_math::max(delta.y, -max),
            java_math::clamp(delta.z, -max, max),
        )
    }
}

/// `LivingEntity.getFluidFallingAdjustedMovement`: gravity's share in the fluid.
pub(super) fn fluid_falling_adjusted_movement(
    base_gravity: f64,
    is_falling: bool,
    movement: Vec3,
) -> Vec3 {
    if base_gravity != 0.0 {
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

/// `Level.containsAnyLiquid`.
pub(super) fn contains_any_liquid(level: &Level<'_>, area: Aabb) -> bool {
    for x in floor_f64(area.min_x)..ceil_f64(area.max_x) {
        for y in floor_f64(area.min_y)..ceil_f64(area.max_y) {
            for z in floor_f64(area.min_z)..ceil_f64(area.max_z) {
                if level.info(BlockPos::new(x, y, z)).fluid != Fluid::Empty {
                    return true;
                }
            }
        }
    }
    false
}
