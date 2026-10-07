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

    /// `LivingEntity.checkFallDamage` on a client: the fluids are looked at again, and a fall adds
    /// up until the ground ends it.
    pub(super) fn check_fall_damage(
        &mut self,
        level: &Level<'_>,
        ya: f64,
    ) -> Result<(), Unsupported> {
        // `Strider.checkFallDamage`: in lava a fall is forgiven.
        if self.kind == MountKind::Strider && self.is_in_lava() {
            self.state.fall_distance = 0.0;
            return Ok(());
        }
        if !self.is_in_water() {
            self.update_fluid_interaction(level)?;
        }
        if !self.is_in_water() && ya < 0.0 {
            self.state.fall_distance -= f64::from(ya as f32);
        }
        if self.on_ground {
            self.state.fall_distance = 0.0;
        }
        Ok(())
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

    /// `Entity.applyEffectsFromBlocks`.
    pub(super) fn apply_effects_from_blocks(&mut self, level: &Level<'_>) {
        let mut movements = std::mem::take(&mut self.movements);
        match movements.last() {
            None => movements.push(Movement {
                from: self.old_position,
                to: self.position,
                original: None,
            }),
            Some(last)
                if last.to.distance_to_sqr(self.position) > f64::from(9.999_999_4e-11_f32) =>
            {
                let from = last.to;
                movements.push(Movement {
                    from,
                    to: self.position,
                    original: None,
                });
            }
            Some(_) => {}
        }
        if self.on_ground {
            let effect_pos = self.get_on_pos(level, 0.2);
            if level.info(effect_pos).name == "minecraft:slime_block" {
                let abs_delta_y = self.delta.y.abs();
                if abs_delta_y < 0.1 {
                    let scale = 0.4 + abs_delta_y * 0.2;
                    self.delta = self.delta.multiply_components(scale, 1.0, scale);
                }
            }
        }
        let _ = walk_inside_blocks(self, level, &movements);
    }
}
