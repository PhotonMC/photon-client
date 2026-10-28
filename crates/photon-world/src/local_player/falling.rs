use super::*;

impl LocalPlayer {
    /// `Entity.getBoundingBox`.
    pub fn bounding_box(&self) -> photon_core::aabb::Aabb {
        self.bounding_box
    }

    pub fn horizontal_collision(&self) -> bool {
        self.horizontal_collision
    }

    pub fn vertical_collision(&self) -> bool {
        self.vertical_collision
    }

    pub fn minor_horizontal_collision(&self) -> bool {
        self.minor_horizontal_collision
    }

    pub(super) fn can_fit(
        &self,
        level: &Level<'_>,
        pose: ClientEntityPose,
    ) -> Result<bool, Unsupported> {
        level.no_collision(
            pose.player_dimensions()
                .make_bounding_box(self.position)
                .deflate(1.0e-7),
            self.fall_distance,
            self.position,
        )
    }

    pub(super) fn move_towards_closest_space(&mut self, level: &Level<'_>, x: f64, z: f64) {
        let pos = BlockPos::new(floor_f64(x), floor_f64(self.position.y), floor_f64(z));
        if !self.suffocates_at(level, pos) {
            return;
        }
        let xd = x - f64::from(pos.x);
        let zd = z - f64::from(pos.z);
        let mut chosen: Option<(i32, i32)> = None;
        let mut closest = f64::MAX;
        // West, east, north, south.
        for (step_x, step_z) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let axis_distance = if step_x != 0 { xd } else { zd };
            let distance_to_edge = if step_x + step_z > 0 {
                1.0 - axis_distance
            } else {
                axis_distance
            };
            let neighbour = BlockPos::new(pos.x + step_x, pos.y, pos.z + step_z);
            if distance_to_edge < closest && !self.suffocates_at(level, neighbour) {
                closest = distance_to_edge;
                chosen = Some((step_x, step_z));
            }
        }
        if let Some((step_x, step_z)) = chosen {
            let old = self.delta;
            self.delta = if step_x != 0 {
                Vec3::new(0.1 * f64::from(step_x), old.y, old.z)
            } else {
                Vec3::new(old.x, old.y, 0.1 * f64::from(step_z))
            };
        }
    }

    pub(super) fn suffocates_at(&self, level: &Level<'_>, pos: BlockPos) -> bool {
        let bb = self.bounding_box;
        let area = Aabb::new(
            f64::from(pos.x),
            bb.min_y,
            f64::from(pos.z),
            f64::from(pos.x) + 1.0,
            bb.max_y,
            f64::from(pos.z) + 1.0,
        )
        .deflate(1.0e-7);
        level.collides_with_suffocating_block(area)
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
                restitution = if !(-current.y < self.effective_gravity())
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
                let air_drag = f64::from(compute_modified_friction(
                    0.98,
                    self.attribute("minecraft:air_drag_modifier") as f32,
                ));
                (
                    portion * self.effective_gravity(),
                    1.0 + portion * (air_drag - 1.0),
                )
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
            let area = aabb.expand_towards_vec3(movement);
            let mut colliders = level.colliders(area, self.fall_distance, self.position)?;
            colliders.extend(level.entity_colliders(self.entity_id, area, self.position));
            collide_entity_movement(movement, aabb, &colliders)
        };
        let x_collision = movement.x != movement_step.x;
        let y_collision = movement.y != movement_step.y;
        let z_collision = movement.z != movement_step.z;
        let on_ground_after_collision = y_collision && movement.y < 0.0;
        let max_up_step = self.attribute("minecraft:step_height") as f32;
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
            let mut colliders = level.colliders(step_up_area, self.fall_distance, self.position)?;
            colliders.extend(level.entity_colliders(self.entity_id, step_up_area, self.position));
            return Ok(select_step_up_movement(
                movement,
                movement_step,
                (aabb, grounded),
                &colliders,
                (max_up_step, movement_step.y as f32),
            ));
        }
        Ok(movement_step)
    }

    pub(super) fn maybe_back_off_from_edge(
        &self,
        level: &Level<'_>,
        delta: Vec3,
    ) -> Result<Vec3, Unsupported> {
        let max_down_step = self.attribute("minecraft:step_height") as f32;
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
}

pub(super) fn unsupported<T>(what: impl Into<String>) -> Result<T, Unsupported> {
    Err(Unsupported(what.into()))
}

/// `BlockGetter.getFurthestCorner`.
pub(super) fn furthest_corner(direction: Vec3) -> [i32; 3] {
    let x_dot = direction.x.abs();
    let y_dot = direction.y.abs();
    let z_dot = direction.z.abs();
    let x_sign = if direction.x >= 0.0 { 1 } else { -1 };
    let y_sign = if direction.y >= 0.0 { 1 } else { -1 };
    let z_sign = if direction.z >= 0.0 { 1 } else { -1 };
    if x_dot <= y_dot && x_dot <= z_dot {
        [-x_sign, -z_sign, y_sign]
    } else if y_dot <= z_dot {
        [z_sign, -y_sign, -x_sign]
    } else {
        [-y_sign, x_sign, -z_sign]
    }
}
