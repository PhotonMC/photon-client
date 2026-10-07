use super::*;

impl BoatBody {
    /// `Entity.restituteMovementAfterCollisions` for a boat: it keeps none of a sideways bump and
    /// bounces only off a springy block, at four fifths of its spring.
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
        if self.ride.vertical_collision {
            if self.ride.vertical_collision_below {
                let falls_slowly = -current.y < self.gravity();
                restitution = if !falls_slowly && !suppresses_bounce {
                    java_math::max(restitution, f64::from(block_bounce * 0.8_f32))
                } else {
                    0.0
                };
            }
            let (gravity_compensation, effective_drag) = if restitution > 0.0 {
                let portion = movement.y / current.y;
                (portion * self.gravity(), 1.0 + portion * (AIR_DRAG - 1.0))
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

    /// `Entity.applyEffectsFromBlocks`: a slime block slows what rides over it, then the blocks
    /// the boat moved through act on it.
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
            let effect_pos = level.on_pos(self.supporting_block, self.position, 0.2);
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

    pub(super) fn make_stuck_in_block(&mut self, speed_multiplier: Vec3) {
        self.ride.fall_distance = 0.0;
        self.ride.stuck_speed_multiplier =
            (speed_multiplier.x, speed_multiplier.y, speed_multiplier.z);
    }
}
