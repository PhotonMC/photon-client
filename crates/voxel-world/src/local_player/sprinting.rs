use super::*;

impl LocalPlayer {
    pub fn is_sprinting(&self) -> bool {
        self.sprinting
    }

    #[expect(
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    pub(super) fn update_auto_jump(
        &mut self,
        level: &Level<'_>,
        xa: f32,
        za: f32,
    ) -> Result<(), Unsupported> {
        let can_auto_jump = self.auto_jump_enabled
            && self.auto_jump_time <= 0
            && self.on_ground
            && !self.is_shift_key_down()
            && self.input.move_vector().length_squared() > 0.0
            && self.block_jump_factor(level) >= 1.0;
        if !can_auto_jump {
            return Ok(());
        }
        let move_begin = self.position;
        let move_end = move_begin.add_components(f64::from(xa), 0.0, f64::from(za));
        let mut move_diff = Vec3::new(f64::from(xa), 0.0, f64::from(za));
        let current_speed = self.speed();
        let mut move_dist_sq = move_diff.length_squared() as f32;
        if move_dist_sq <= 0.001 {
            let move_vector = self.input.move_vector();
            let input_xa = current_speed * move_vector.x;
            let input_za = current_speed * move_vector.y;
            let sin = mth_sin_f64(f64::from(self.y_rot * MTH_DEG_TO_RAD));
            let cos = mth_cos_f64(f64::from(self.y_rot * MTH_DEG_TO_RAD));
            move_diff = Vec3::new(
                f64::from(input_xa * cos - input_za * sin),
                move_diff.y,
                f64::from(input_za * cos + input_xa * sin),
            );
            move_dist_sq = move_diff.length_squared() as f32;
            if move_dist_sq <= 0.001 {
                return Ok(());
            }
        }

        let move_dist_inverted = mth_inv_sqrt_f32(move_dist_sq);
        let move_dir = move_diff.scale(f64::from(move_dist_inverted));
        let facing = Vec3::direction_from_rotation_components(self.x_rot, self.y_rot);
        let facing_vs_moving = (facing.x * move_dir.x + facing.z * move_dir.z) as f32;
        if facing_vs_moving < -0.15 {
            return Ok(());
        }
        let mut ceiling_pos = BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.bounding_box.max_y),
            floor_f64(self.position.z),
        );
        if !level.collision_shape(ceiling_pos).is_empty() {
            return Ok(());
        }
        ceiling_pos = BlockPos::new(ceiling_pos.x, ceiling_pos.y + 1, ceiling_pos.z);
        if !level.collision_shape(ceiling_pos).is_empty() {
            return Ok(());
        }

        let jump_height = 1.2_f32;
        let look_ahead_dist = (current_speed * 7.0).max(1.0 / move_dist_inverted);
        let seg_begin = move_begin;
        let seg_end = move_end.add_vec3(move_dir.scale(f64::from(look_ahead_dist)));
        let player_width = self.dimensions.width();
        let player_height = self.dimensions.height();
        let test_box = Aabb::from_vec3(
            seg_begin,
            seg_end.add_components(0.0, f64::from(player_height), 0.0),
        )
        .inflate(f64::from(player_width), 0.0, f64::from(player_width));
        let seg_begin = seg_begin.add_components(0.0, f64::from(0.51_f32), 0.0);
        let seg_end = seg_end.add_components(0.0, f64::from(0.51_f32), 0.0);
        let right_dir = move_dir.cross(Vec3::new(0.0, 1.0, 0.0));
        let right_offset = right_dir.scale(f64::from(player_width * 0.5));
        let left_begin = seg_begin.subtract_vec3(right_offset);
        let left_end = seg_end.subtract_vec3(right_offset);
        let right_begin = seg_begin.add_vec3(right_offset);
        let right_end = seg_end.add_vec3(right_offset);

        let mut obstacle_height = f32::from_bits(1);
        'boxes: for (_, shape) in level.block_collisions(test_box, self.fall_distance)? {
            for bounds in shape.to_aabbs() {
                if bounds.intersects_vec3(left_begin, left_end)
                    || bounds.intersects_vec3(right_begin, right_end)
                {
                    obstacle_height = bounds.max_y as f32;
                    let center = bounds.center();
                    let obstacle_pos = BlockPos::new(
                        floor_f64(center.x),
                        floor_f64(center.y),
                        floor_f64(center.z),
                    );
                    let mut steps = 1;
                    while (steps as f32) < jump_height {
                        let above =
                            BlockPos::new(obstacle_pos.x, obstacle_pos.y + steps, obstacle_pos.z);
                        let shape = level.collision_shape(above);
                        if !shape.is_empty() {
                            obstacle_height = (shape.max(Axis::Y) + f64::from(above.y)) as f32;
                            if f64::from(obstacle_height) - self.position.y > f64::from(jump_height)
                            {
                                return Ok(());
                            }
                        }
                        if steps > 1 {
                            ceiling_pos =
                                BlockPos::new(ceiling_pos.x, ceiling_pos.y + 1, ceiling_pos.z);
                            if !level.collision_shape(ceiling_pos).is_empty() {
                                return Ok(());
                            }
                        }
                        steps += 1;
                    }
                    break 'boxes;
                }
            }
        }

        if obstacle_height != f32::from_bits(1) {
            let y_delta = (f64::from(obstacle_height) - self.position.y) as f32;
            if !(y_delta <= 0.5) && !(y_delta > jump_height) {
                self.auto_jump_time = 1;
            }
        }
        Ok(())
    }

    pub(super) fn update_swimming(&mut self, level: &Level<'_>) {
        if self.abilities.flying {
            self.swimming = false;
        } else if self.is_swimming() {
            self.swimming = self.sprinting && self.is_in_water() && !self.is_passenger();
        } else {
            self.swimming = self.sprinting
                && self.is_under_water()
                && !self.is_passenger()
                && level.info(self.block_position()).is_water();
        }
    }

    pub(super) fn set_sprinting(&mut self, sprinting: bool) {
        self.sprinting = sprinting;
    }

    /// `Player.isSwimming`: the flag, unless flying or spectating.
    pub(super) fn is_swimming(&self) -> bool {
        !self.abilities.flying && !self.is_spectator() && self.swimming
    }

    pub(super) fn is_sprinting_possible(&self, allowed_in_shallow_water: bool) -> bool {
        // A passenger sprints only in a vehicle that can be (and it is its authority).
        if self.riding.is_some_and(|riding| !riding.can_sprint) {
            return false;
        }
        // No blindness, and a full food bar or the ability to fly.
        allowed_in_shallow_water || !(self.is_in_water() && !self.is_under_water())
    }

    pub(super) fn has_forward_impulse(&self) -> bool {
        self.input.move_vector().y > 1.0e-5
    }

    pub(super) fn can_start_sprinting(&self) -> bool {
        !self.sprinting
            && self.has_forward_impulse()
            && self.is_sprinting_possible(self.abilities.flying)
            && (!self.is_moving_slowly() || self.is_under_water())
    }

    pub(super) fn should_stop_run_sprinting(&self) -> bool {
        !self.is_sprinting_possible(self.abilities.flying)
            || !self.has_forward_impulse()
            || self.horizontal_collision && !self.minor_horizontal_collision
    }

    pub(super) fn should_stop_swim_sprinting(&self) -> bool {
        !self.is_sprinting_possible(true)
            || !self.is_in_water()
            || !self.has_forward_impulse() && !self.on_ground && !self.input.key_presses.shift
    }
}
