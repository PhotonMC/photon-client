use super::*;

impl LocalPlayer {
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

    pub(super) fn apply_effects_from_blocks(
        &mut self,
        level: &Level<'_>,
    ) -> Result<(), Unsupported> {
        let mut movements = std::mem::take(&mut self.movements_this_tick);
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
        if self.no_physics {
            return Ok(());
        }
        if self.on_ground {
            let effect_pos = self.get_on_pos(level, 0.2);
            if level.info(effect_pos).name == "minecraft:slime_block" {
                let abs_delta_y = self.delta.y.abs();
                if abs_delta_y < 0.1 && !self.is_shift_key_down() {
                    let scale = 0.4 + abs_delta_y * 0.2;
                    self.delta = self.delta.multiply_components(scale, 1.0, scale);
                }
            }
        }
        self.check_inside_blocks(level, &movements)
    }

    /// `Entity.checkInsideBlocks` for the player: the shared walk, with the blocks that act on a
    /// player acting as they are visited.
    pub(super) fn check_inside_blocks(
        &mut self,
        level: &Level<'_>,
        movements: &[Movement],
    ) -> Result<(), Unsupported> {
        walk_inside_blocks(self, level, movements)
    }

    /// What a block the player is inside does to it (see [`living_inside_effect`]).
    pub(super) fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        let block_pos = self.block_position();
        living_inside_effect(
            level,
            visit,
            LivingEffects {
                delta: &mut self.delta,
                fall_distance: &mut self.fall_distance,
                stuck: &mut self.stuck_speed_multiplier,
                position: self.position,
                on_ground: self.on_ground,
                width: self.dimensions.width(),
                block_pos,
            },
        );
    }

    pub(super) fn update_fluid_interaction(
        &mut self,
        level: &Level<'_>,
    ) -> Result<(), Unsupported> {
        let found = crate::fluid::scan_in(
            level,
            self.fluid_interaction_box(),
            (
                self.bounding_box.min_y,
                self.position,
                self.dimensions.eye_height(),
            ),
            self.abilities.flying,
        );
        let fluids = found.inspect_err(|_| {
            self.water = FluidTracker::default();
            self.lava = FluidTracker::default();
        })?;
        self.water = fluids.water;
        self.lava = fluids.lava;
        let in_water = self.water.height > 0.0;
        let in_lava = self.lava.height > 0.0;
        if in_water {
            self.fall_distance = 0.0;
        }
        if in_water && !self.was_touching_water && !self.first_tick {
            self.water_splash_sound();
        }
        self.was_touching_water = in_water;
        if !self.abilities.flying {
            if in_water {
                self.apply_current(self.water, WATER_FLOW_SCALE);
            }
            if in_lava {
                self.apply_current(self.lava, LAVA_FLOW_SCALE);
            }
        }
        Ok(())
    }

    /// `Entity.getFluidInteractionBox`: the box a thousandth smaller, which the vehicle the player
    /// sits in may change (`AbstractBoat.modifyPassengerFluidInteractionBox`: only what is above
    /// the boat's top, and none of it while the boat's top is above the player's own).
    pub(super) fn fluid_interaction_box(&self) -> Option<Aabb> {
        let area = self.bounding_box.deflate(0.001);
        let Some(riding) = self.riding else {
            return Some(area);
        };
        if !riding.modifies_fluid_box || riding.vehicle_under_water {
            return Some(area);
        }
        if riding.vehicle_max_y >= area.max_y {
            return None;
        }
        let min_y = java_math::max(area.min_y, riding.vehicle_max_y);
        Some(Aabb::new(
            area.min_x, min_y, area.min_z, area.max_x, area.max_y, area.max_z,
        ))
    }

    pub(super) fn apply_current(&mut self, tracker: FluidTracker, scale: f64) {
        self.delta =
            crate::fluid::apply_current(tracker, self.delta, scale, crate::fluid::Pushed::Player);
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

    pub(super) fn update_player_pose(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if !self.can_fit(level, ClientEntityPose::Swimming)? {
            return Ok(());
        }
        let desired = if self.is_swimming() {
            ClientEntityPose::Swimming
        } else if self.is_shift_key_down() && !self.abilities.flying {
            ClientEntityPose::Crouching
        } else {
            ClientEntityPose::Standing
        };
        let actual =
            if self.is_spectator() || self.is_passenger() || self.can_fit(level, desired)? {
                desired
            } else if self.can_fit(level, ClientEntityPose::Crouching)? {
                ClientEntityPose::Crouching
            } else {
                ClientEntityPose::Swimming
            };
        self.set_pose(actual);
        Ok(())
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

    pub(super) fn set_pose(&mut self, pose: ClientEntityPose) {
        if pose != self.pose {
            self.pose = pose;
            self.dimensions = pose.player_dimensions();
            self.bounding_box = self.dimensions.make_bounding_box(self.position);
        }
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

    pub(super) fn send_position(&mut self) {
        if self.sprinting != self.was_sprinting {
            let action = if self.sprinting {
                PlayerCommandAction::StartSprinting
            } else {
                PlayerCommandAction::StopSprinting
            };
            self.sent.push(ServerboundMovement::PlayerCommand {
                entity_id: self.entity_id,
                action,
                data: 0,
            });
            self.was_sprinting = self.sprinting;
        }

        let delta_x = self.position.x - self.x_last;
        let delta_y = self.position.y - self.y_last;
        let delta_z = self.position.z - self.z_last;
        let delta_y_rot = f64::from(self.y_rot - self.y_rot_last);
        let delta_x_rot = f64::from(self.x_rot - self.x_rot_last);
        self.position_reminder += 1;
        let moved = delta_x * delta_x + delta_y * delta_y + delta_z * delta_z > 2.0e-4 * 2.0e-4
            || self.position_reminder >= 20;
        let rotated = delta_y_rot != 0.0 || delta_x_rot != 0.0;
        let on_ground = self.on_ground;
        let horizontal_collision = self.horizontal_collision;
        if moved && rotated {
            self.sent.push(ServerboundMovement::MovePlayerPosRot {
                position: self.position,
                y_rot: self.y_rot,
                x_rot: self.x_rot,
                on_ground,
                horizontal_collision,
            });
        } else if moved {
            self.sent.push(ServerboundMovement::MovePlayerPos {
                position: self.position,
                on_ground,
                horizontal_collision,
            });
        } else if rotated {
            self.sent.push(ServerboundMovement::MovePlayerRot {
                y_rot: self.y_rot,
                x_rot: self.x_rot,
                on_ground,
                horizontal_collision,
            });
        } else if self.last_on_ground != on_ground
            || self.last_horizontal_collision != horizontal_collision
        {
            self.sent.push(ServerboundMovement::MovePlayerStatusOnly {
                on_ground,
                horizontal_collision,
            });
        }
        if moved {
            self.x_last = self.position.x;
            self.y_last = self.position.y;
            self.z_last = self.position.z;
            self.position_reminder = 0;
        }
        if rotated {
            self.y_rot_last = self.y_rot;
            self.x_rot_last = self.x_rot;
        }
        self.last_on_ground = on_ground;
        self.last_horizontal_collision = horizontal_collision;
        self.auto_jump_enabled = self.auto_jump_option;
    }

    pub(super) fn on_update_abilities(&mut self) {
        self.sent.push(ServerboundMovement::PlayerAbilities {
            flying: self.abilities.flying,
        });
    }

    pub(super) fn set_pos(&mut self, position: Vec3) {
        self.position = position;
        self.bounding_box = self.dimensions.make_bounding_box(position);
    }

    pub(super) fn set_sprinting(&mut self, sprinting: bool) {
        self.sprinting = sprinting;
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

    pub(super) fn look_angle(&self) -> Vec3 {
        let real_x_rot = self.x_rot * MTH_DEG_TO_RAD;
        let real_y_rot = -self.y_rot * MTH_DEG_TO_RAD;
        let y_cos = mth_cos_f64(f64::from(real_y_rot));
        let y_sin = mth_sin_f64(f64::from(real_y_rot));
        let x_cos = mth_cos_f64(f64::from(real_x_rot));
        let x_sin = mth_sin_f64(f64::from(real_x_rot));
        Vec3::new(
            f64::from(y_sin * x_cos),
            f64::from(-x_sin),
            f64::from(y_cos * x_cos),
        )
    }

    pub(super) fn is_spectator(&self) -> bool {
        self.game_mode.is_spectator()
    }

    pub(super) fn is_in_water(&self) -> bool {
        self.was_touching_water
    }

    pub(super) fn is_in_lava(&self) -> bool {
        !self.first_tick && self.lava.height > 0.0
    }

    pub(super) fn is_under_water(&self) -> bool {
        self.was_eye_in_water && self.is_in_water()
    }

    pub(super) fn is_affected_by_fluids(&self) -> bool {
        !self.abilities.flying
    }

    /// `Player.isSwimming`: the flag, unless flying or spectating.
    pub(super) fn is_swimming(&self) -> bool {
        !self.abilities.flying && !self.is_spectator() && self.swimming
    }

    pub(super) fn is_moving_slowly(&self) -> bool {
        self.crouching || self.pose == ClientEntityPose::Swimming && !self.is_in_water()
    }

    pub(super) fn fluid_jump_threshold(&self) -> f64 {
        if self.dimensions.eye_height() < 0.4 {
            0.0
        } else {
            0.4
        }
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
