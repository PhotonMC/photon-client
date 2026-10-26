use super::*;

impl LocalPlayer {
    pub fn is_sprinting(&self) -> bool {
        self.sprinting
    }

    /// `Entity.canSpawnSprintParticle`.
    fn can_spawn_sprint_particle(&self) -> bool {
        self.sprinting
            && !self.is_in_water()
            && !self.is_spectator()
            && self.pose != ClientEntityPose::Crouching
            && !self.is_in_lava()
            && self.health > 0.0
    }

    /// `Entity.spawnSprintParticle`: a sprinting player kicks up flakes of the block under its
    /// feet, thrown back against its motion.
    pub(super) fn throw_sprint_particle(&mut self, level: &Level<'_>) {
        if !self.can_spawn_sprint_particle() {
            return;
        }
        let pos = self.get_on_pos(level, 0.2);
        let Some(state) = level.state_id(pos) else {
            return;
        };
        if crate::shadow::is_invisible_render_shape(crate::block::state_or_air(state).block().name)
        {
            return;
        }
        let Some(type_id) = photon_data::particle_type::id("block") else {
            return;
        };
        let width = f64::from(self.dimensions.width());
        let here = BlockPos::new(
            self.position.x.floor() as i32,
            self.position.y.floor() as i32,
            self.position.z.floor() as i32,
        );
        let mut x = self.position.x + (self.random.next_double() - 0.5) * width;
        let mut z = self.position.z + (self.random.next_double() - 0.5) * width;
        // Standing on the edge of a block, the flakes stay over the block they come from.
        if here.x != pos.x {
            x = x.clamp(f64::from(pos.x), f64::from(pos.x) + 1.0);
        }
        if here.z != pos.z {
            z = z.clamp(f64::from(pos.z), f64::from(pos.z) + 1.0);
        }
        self.use_edge
            .particles
            .push(crate::particles::LevelParticles {
                type_id,
                data: crate::particles::ParticleData::Block(state),
                override_limiter: false,
                always_show: false,
                at: (x, self.position.y + 0.1, z),
                // Count 0 reads the spread as the velocity.
                spread: (
                    (self.delta.x * -4.0) as f32,
                    1.5,
                    (self.delta.z * -4.0) as f32,
                ),
                max_speed: 1.0,
                count: 0,
            });
    }

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

        // Jump boost lets the auto-jump climb three quarters of a block more a level.
        let jump_height = 1.2_f32 + self.hands.jump_boost.map_or(0, |n| n + 1) as f32 * 0.75;
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
        self.update_water_vision();
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

    /// `LivingEntity.setSprinting`: the sprinting modifier comes off the movement speed, and goes
    /// back on while sprinting.
    pub(super) fn set_sprinting(&mut self, sprinting: bool) {
        self.sprinting = sprinting;
        let speed_id = crate::VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID;
        if !self
            .attributes
            .iter()
            .any(|held| held.attribute_id == speed_id)
        {
            // The reference's player has the attribute at its default before any packet.
            self.attributes.push(crate::ClientAttributeState::new(
                speed_id,
                f64::from(crate::VANILLA_PLAYER_DEFAULT_MOVEMENT_SPEED),
                Vec::new(),
            ));
        }
        let speed = self
            .attributes
            .iter_mut()
            .find(|held| held.attribute_id == speed_id)
            .expect("the movement speed was just made if it was missing");
        let modifiers = &mut speed.modifiers;
        modifiers.retain(|modifier| modifier.id() != crate::VANILLA_PLAYER_SPRINTING_MODIFIER_ID);
        if sprinting {
            modifiers.push(crate::ClientAttributeModifierState::new(
                crate::VANILLA_PLAYER_SPRINTING_MODIFIER_ID.to_owned(),
                f64::from(crate::VANILLA_PLAYER_SPRINTING_MODIFIER_AMOUNT),
                2,
            ));
        }
    }

    /// `Player.isSwimming`: the flag, unless flying or spectating.
    pub(super) fn is_swimming(&self) -> bool {
        !self.abilities.flying && !self.is_spectator() && self.swimming
    }

    /// `LocalPlayer.isSprintingPossible`: no blindness, a vehicle that can sprint or enough food
    /// (more than three bars, or the ability to fly), and no shallow water unless allowed.
    pub(super) fn is_sprinting_possible(&self, allowed_in_shallow_water: bool) -> bool {
        let able = match self.riding {
            Some(riding) => riding.can_sprint,
            None => self.hands.food > 6 || self.abilities.may_fly,
        };
        !self.hands.blinded
            && able
            && (allowed_in_shallow_water || !(self.is_in_water() && !self.is_under_water()))
    }

    pub(super) fn has_forward_impulse(&self) -> bool {
        self.input.move_vector().y > 1.0e-5
    }

    /// The sprint trigger of `LocalPlayer.aiStep`: sneaking, a slowing use or walking back
    /// closes the window, and a second tap on forward (or the sprint key) inside it sprints.
    pub(super) fn open_sprint_window(
        &mut self,
        was_shift_key_down: bool,
        had_forward_impulse: bool,
    ) {
        if was_shift_key_down
            || self.is_slow_due_to_using_item() && !self.is_passenger()
            || self.input.key_presses.backward
        {
            self.sprint_trigger_time = 0;
        }
        if !self.can_start_sprinting() {
            return;
        }
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

    pub(super) fn can_start_sprinting(&self) -> bool {
        !self.sprinting
            && self.has_forward_impulse()
            && self.is_sprinting_possible(self.abilities.flying)
            && !self.is_slow_due_to_using_item()
            && (!self.elytra.fall_flying || self.is_under_water())
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

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{ClientPlayerGameMode, DimensionBounds, SectionInput};

    /// A player at the middle of a stone floor, sprinting east, with the floor's top at y = 64.
    fn runner() -> (ClientWorld, LocalPlayer) {
        let mut blocks = vec![0; 4_096];
        let stone = photon_data::block::registry::default_state_named("minecraft:stone").0;
        for z in 0..16 {
            for x in 0..16 {
                blocks[63 % 16 * 256 + z * 16 + x] = stone;
            }
        }
        let mut world = ClientWorld::new(DimensionBounds::new(0, 128));
        let mut sections = Vec::new();
        for section in 0..8 {
            let states = if section == 3 {
                blocks.clone()
            } else {
                vec![0; 4_096]
            };
            sections.push((section == 3, states));
        }
        world.replace_chunk(
            0,
            0,
            sections.iter().map(|(floor, states)| SectionInput {
                non_empty_block_count: if *floor { 256 } else { 0 },
                fluid_count: 0,
                block_states: states,
                biomes: &[0; 64],
            }),
        );
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            Vec3::new(8.5, 64.0, 8.5),
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        player.sprinting = true;
        player.delta = Vec3::new(0.2, 0.0, 0.0);
        (world, player)
    }

    /// `Entity.spawnSprintParticle`: one flake of the block underfoot a tick, against the motion.
    #[test]
    fn a_sprinting_player_kicks_up_flakes_of_the_block_under_its_feet() {
        let (world, mut player) = runner();
        player.throw_sprint_particle(&Level::new(&world));
        let flakes = player.take_particles();
        assert_eq!(flakes.len(), 1);
        let flake = flakes[0];
        assert_eq!(flake.count, 0);
        assert_eq!(
            flake.type_id,
            photon_data::particle_type::id("block").expect("the block particle")
        );
        assert_eq!(
            flake.data,
            crate::particles::ParticleData::Block(
                photon_data::block::registry::default_state_named("minecraft:stone")
            )
        );
        assert!((flake.at.1 - 64.1).abs() < 1e-9);
        // Within the body's width of the player, thrown back and up.
        assert!((flake.at.0 - 8.5).abs() <= 0.3 + 1e-9 && (flake.at.2 - 8.5).abs() <= 0.3 + 1e-9);
        assert_eq!(flake.spread, (-0.8, 1.5, 0.0));
    }

    /// `Entity.canSpawnSprintParticle`: walking, crouching, spectating and dying throw nothing.
    #[test]
    fn only_a_sprinting_living_player_on_its_feet_kicks_up_flakes() {
        let (world, mut walking) = runner();
        walking.sprinting = false;
        walking.throw_sprint_particle(&Level::new(&world));
        assert!(walking.take_particles().is_empty());

        let (world, mut crouching) = runner();
        crouching.pose = ClientEntityPose::Crouching;
        crouching.throw_sprint_particle(&Level::new(&world));
        assert!(crouching.take_particles().is_empty());

        let (world, mut dead) = runner();
        dead.health = 0.0;
        dead.throw_sprint_particle(&Level::new(&world));
        assert!(dead.take_particles().is_empty());

        // Over the air, with no block to kick up, nothing either.
        let (world, mut flying) = runner();
        flying.position = Vec3::new(8.5, 90.0, 8.5);
        flying.throw_sprint_particle(&Level::new(&world));
        assert!(flying.take_particles().is_empty());
    }
}
