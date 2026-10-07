use super::*;

impl ClientEntity {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub(crate) fn collision_candidate(
        &self,
        source_entity_id: Option<i32>,
        source_type_id: Option<i32>,
        source_position: Option<Vec3>,
        same_vehicle: bool,
    ) -> Option<ClientEntityCollisionCandidate> {
        let (width, height) = if is_vanilla_boat_entity_type(self.type_id) {
            (
                (VANILLA_BOAT_HALF_WIDTH * 2.0) as f32,
                VANILLA_BOAT_HEIGHT as f32,
            )
        } else if is_vanilla_shulker_entity_type(self.type_id) {
            (1.0, 1.0)
        } else if is_vanilla_minecart_entity_type(self.type_id) {
            (0.98, 0.7)
        } else if is_vanilla_player_entity_type(self.type_id) {
            (0.6, 1.8)
        } else if is_vanilla_happy_ghast_entity_type(self.type_id) {
            (4.0, 4.0)
        } else {
            return None;
        };
        let (x, y, z) = self.position();
        let dimensions = if is_vanilla_happy_ghast_entity_type(self.type_id) && self.baby {
            ClientEntityDimensions::scalable(width, height).scale(0.2375, 0.2375)
        } else {
            ClientEntityDimensions::scalable(width, height)
        };
        let bounding_box = if is_vanilla_shulker_entity_type(self.type_id) {
            self.shulker_bounding_box()
        } else {
            dimensions.make_bounding_box(Vec3::new(x, y, z))
        };
        let can_be_collided_with = if is_vanilla_happy_ghast_entity_type(self.type_id) {
            !self.baby
                && self.alive
                && ((source_type_id == Some(VANILLA_PLAYER_ENTITY_TYPE_ID)
                    && source_position.is_some_and(|position| position.y >= bounding_box.max_y))
                    || (source_type_id == Some(VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID)
                        && self.happy_ghast_is_vehicle)
                    || self.happy_ghast_still_timeout)
        } else {
            is_vanilla_boat_entity_type(self.type_id)
                || is_vanilla_shulker_entity_type(self.type_id)
        };
        let is_pushable = if is_vanilla_player_entity_type(self.type_id) {
            self.alive
                && !self.spectator
                && (self.flying || !self.on_climbable)
                && self
                    .health_bits
                    .is_none_or(|health| f32::from_bits(health) > 0.0)
        } else {
            is_vanilla_pushable_entity_type(self.type_id)
                || (is_vanilla_happy_ghast_entity_type(self.type_id) && self.alive)
        };
        Some(ClientEntityCollisionCandidate {
            bounding_box,
            is_source: source_entity_id == Some(self.id),
            spectator: self.spectator,
            can_be_collided_with,
            is_pushable,
            same_vehicle,
        })
    }

    pub fn vehicle_id(&self) -> Option<i32> {
        self.vehicle_id
    }

    pub fn passenger_ids(&self) -> &[i32] {
        &self.passenger_ids
    }

    pub(crate) fn turn(&mut self, mouse_x: f64, mouse_y: f64) {
        let x_delta = mouse_y as f32 * 0.15_f32;
        let y_delta = mouse_x as f32 * 0.15_f32;
        self.x_rot = JavaF32::new(self.x_rot.get() + x_delta);
        self.y_rot = JavaF32::new(self.y_rot.get() + y_delta);
        self.x_rot = JavaF32::new(clamp_f32(self.x_rot.get(), -90.0, 90.0));
        self.old_x_rot = JavaF32::new(self.old_x_rot.get() + x_delta);
        self.old_y_rot = JavaF32::new(self.old_y_rot.get() + y_delta);
        self.old_x_rot = JavaF32::new(clamp_f32(self.old_x_rot.get(), -90.0, 90.0));
    }

    pub(crate) fn capture_old_transform(&mut self) {
        self.old_position.x = self.position.x;
        self.old_position.y = self.position.y;
        self.old_position.z = self.position.z;
        self.old_x_rot = self.x_rot;
        self.old_y_rot = self.y_rot;
        self.old_y_head_rot = self.y_head_rot;
        self.old_y_body_rot = self.y_body_rot;
    }

    pub(crate) fn tick_head_rotation(&mut self) {
        advance_head_rotation(
            &mut self.y_head_rot,
            self.head_rotation_target,
            &mut self.head_lerp_steps,
        );
    }

    /// Whether the type is a `LivingEntity`.
    pub fn is_living_entity(&self) -> bool {
        self.living_kind().is_some()
    }

    /// Which kind of living entity this is, as far as its drawn pose goes.
    pub(crate) fn living_kind(&self) -> Option<LivingKind> {
        if self.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            Some(LivingKind::Player)
        } else if !vanilla_passenger_is_living(self.type_id) {
            None
        } else if self.type_id == armor_stand_type_id() {
            Some(LivingKind::Other)
        } else {
            Some(LivingKind::Mob)
        }
    }

    /// What a living entity's drawn pose follows from its tick: the limbs'
    /// swing with the distance moved, and the body turning (a player's after
    /// where it walks, a mob's by its body control), and for a player the
    /// swimming pose easing in or out.
    pub(crate) fn tick_living_pose(&mut self, kind: LivingKind) {
        let moved = (
            self.position.x.get() - self.old_position.x.get(),
            self.position.z.get() - self.old_position.z.get(),
        );
        if kind == LivingKind::Player {
            self.old_swim_amount = self.swim_amount;
            self.swim_amount = JavaF32::new(crate::living::ease_swim_amount(
                self.swim_amount.get(),
                self.pose == ClientEntityPose::Swimming,
            ));
        }
        let mut body = self.y_body_rot.get();
        let mut body_old = self.old_y_body_rot.get();
        let mut head = self.y_head_rot.get();
        let mut head_old = self.old_y_head_rot.get();
        if kind == LivingKind::Mob && self.is_shulker() {
            // `Shulker.ShulkerBodyRotationControl`: the body does not follow.
            crate::living::keep_old_angles_near((body, &mut body_old), (head, &mut head_old));
        } else if kind == LivingKind::Mob {
            let mut control = crate::living::MobBodyControl {
                head_stable_time: self.head_stable_time,
                last_stable_y_head_rot: self.last_stable_y_head_rot.get(),
            };
            crate::living::tick_mob_body(
                &mut control,
                (&mut body, &mut head),
                self.y_rot.get(),
                moved,
                !self.passenger_ids.is_empty(),
            );
            self.head_stable_time = control.head_stable_time;
            self.last_stable_y_head_rot = JavaF32::new(control.last_stable_y_head_rot);
            crate::living::keep_old_angles_near((body, &mut body_old), (head, &mut head_old));
            self.y_head_rot = JavaF32::new(head);
        } else {
            crate::living::turn_body(
                (&mut body, &mut body_old),
                (self.y_rot.get(), head, &mut head_old),
                moved,
            );
        }
        self.y_body_rot = JavaF32::new(body);
        self.old_y_body_rot = JavaF32::new(body_old);
        self.old_y_head_rot = JavaF32::new(head_old);
        // `calculateEntityAnimation`: a rider or a dead entity stands still.
        // The dragon's own `aiStep` does not reach `calculateEntityAnimation`.
        if self.vehicle_id.is_none() && self.alive && !self.is_ender_dragon() {
            let camel = matches!(
                voxel_data::entity_type::name(self.type_id),
                Some("minecraft:camel" | "minecraft:camel_husk")
            );
            if camel {
                // `Camel.updateWalkAnimation`: only a standing camel that is
                // not dashing walks, and its stride is quicker to settle.
                let walking = self.pose == ClientEntityPose::Standing
                    && !self
                        .kind
                        .horse()
                        .is_some_and(|horse| horse.camel.is_dash_animation_started());
                let target = if walking {
                    ((moved.0.hypot(moved.1)) as f32 * 6.0).min(1.0)
                } else {
                    0.0
                };
                self.walk
                    .update(target, 0.2, if self.baby { 3.0 } else { 1.0 });
            } else if self.is_creaking() {
                // `Creaking.updateWalkAnimation`: a quicker stride, capped at three.
                let target = (moved.0.hypot(moved.1) as f32 * 25.0).min(3.0);
                self.walk.update(target, 0.4, 1.0);
            } else if self.is_frog() {
                let target = self.kind.frog().map_or_else(
                    || ((moved.0.hypot(moved.1)) as f32 * 25.0).min(1.0),
                    |frog| frog.walk_target(moved.0.hypot(moved.1)),
                );
                self.walk
                    .update(target, 0.4, if self.baby { 3.0 } else { 1.0 });
            } else {
                self.walk.follow(moved.0.hypot(moved.1), self.baby);
            }
        } else {
            self.walk.stop();
        }
    }

    pub(crate) fn interpolation_position(&self) -> (f64, f64, f64) {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or_else(
                || self.position(),
                EntityInterpolationState::target_position,
            )
    }

    pub(crate) fn interpolation_rotation(&self) -> (f32, f32) {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or_else(
                || self.body_rotation(),
                EntityInterpolationState::target_rotation,
            )
    }

    pub(crate) fn interpolate_to(&mut self, position: (f64, f64, f64), y_rot: f32, x_rot: f32) {
        let steps = self.interpolation_length();
        if steps == 0 {
            // `InterpolationHandler.interpolateTo` with no steps: put there at once.
            self.snap_to(position, y_rot, x_rot);
            self.interpolation = None;
            return;
        }
        let target_matches = self.interpolation.is_some_and(|interpolation| {
            interpolation.steps > 0
                && interpolation.target.x == JavaF64::new(position.0)
                && interpolation.target.y == JavaF64::new(position.1)
                && interpolation.target.z == JavaF64::new(position.2)
                && interpolation.target_y_rot == JavaF32::new(y_rot)
                && interpolation.target_x_rot == JavaF32::new(x_rot)
        });
        if target_matches {
            return;
        }

        let current_position = self.position();
        let current_rotation = self.body_rotation();
        self.interpolation = Some(EntityInterpolationState {
            target: DVec3::new(position.0, position.1, position.2),
            target_y_rot: JavaF32::new(y_rot),
            target_x_rot: JavaF32::new(x_rot),
            previous: DVec3::new(current_position.0, current_position.1, current_position.2),
            previous_y_rot: JavaF32::new(current_rotation.0),
            previous_x_rot: JavaF32::new(current_rotation.1),
            steps: steps.clamp(0, 255) as u8,
        });
    }

    pub(crate) fn snap_to(&mut self, position: (f64, f64, f64), y_rot: f32, x_rot: f32) {
        self.position.x = JavaF64::new(position.0);
        self.position.y = JavaF64::new(position.1);
        self.position.z = JavaF64::new(position.2);
        self.y_rot = JavaF32::new(y_rot);
        self.x_rot = JavaF32::new(x_rot);
        self.old_position.x = JavaF64::new(position.0);
        self.old_position.y = JavaF64::new(position.1);
        self.old_position.z = JavaF64::new(position.2);
        self.old_y_rot = JavaF32::new(y_rot);
        self.old_x_rot = JavaF32::new(x_rot);
        if self.is_shulker() {
            // `Entity.snapTo` sets the old place from the raw one, then
            // `reapplyPosition` runs `Shulker.setPos` from the raw block.
            self.shulker_set_pos(position, block_of(position));
        }
    }

    /// `Entity.moveOrInterpolateTo`: a shulker and a minecart of the new behaviour have no
    /// interpolation (`getInterpolation` is null), so they are put at the place, and turned,
    /// at once; the rest interpolate.
    pub(crate) fn move_or_interpolate_to(
        &mut self,
        position: (f64, f64, f64),
        y_rot: f32,
        x_rot: f32,
    ) {
        if self.is_shulker() || self.minecart_track().is_some() {
            self.set_position(position);
            self.y_rot = JavaF32::new(y_rot % 360.0);
            self.x_rot = JavaF32::new((x_rot % 360.0).clamp(-90.0, 90.0));
        } else {
            self.interpolate_to(position, y_rot, x_rot);
        }
    }

    pub(crate) fn tick_position_interpolation(&mut self) {
        let current_position = self.position();
        let current_rotation = self.body_rotation();
        let Some(interpolation) = self
            .interpolation
            .as_mut()
            .filter(|interpolation| interpolation.steps > 0)
        else {
            return;
        };

        let alpha = 1.0 / f64::from(interpolation.steps);
        let previous_position = (
            interpolation.previous.x.get(),
            interpolation.previous.y.get(),
            interpolation.previous.z.get(),
        );
        let delta_position = (
            current_position.0 - previous_position.0,
            current_position.1 - previous_position.1,
            current_position.2 - previous_position.2,
        );
        interpolation.target.x = JavaF64::new(interpolation.target.x.get() + delta_position.0);
        interpolation.target.y = JavaF64::new(interpolation.target.y.get() + delta_position.1);
        interpolation.target.z = JavaF64::new(interpolation.target.z.get() + delta_position.2);

        let previous_rotation = (
            interpolation.previous_y_rot.get(),
            interpolation.previous_x_rot.get(),
        );
        let delta_rotation = (
            current_rotation.0 - previous_rotation.0,
            current_rotation.1 - previous_rotation.1,
        );
        interpolation.target_y_rot =
            JavaF32::new(interpolation.target_y_rot.get() + delta_rotation.0);
        interpolation.target_x_rot =
            JavaF32::new(interpolation.target_x_rot.get() + delta_rotation.1);

        let target_position = interpolation.target_position();
        let target_rotation = interpolation.target_rotation();
        let new_position = (
            mth_lerp_f64(alpha, current_position.0, target_position.0),
            mth_lerp_f64(alpha, current_position.1, target_position.1),
            mth_lerp_f64(alpha, current_position.2, target_position.2),
        );
        let new_rotation = (
            mth_rot_lerp_f64(
                alpha,
                f64::from(current_rotation.0),
                f64::from(target_rotation.0),
            ) as f32,
            mth_lerp_f64(
                alpha,
                f64::from(current_rotation.1),
                f64::from(target_rotation.1),
            ) as f32,
        );
        interpolation.steps -= 1;
        interpolation.previous.x = JavaF64::new(new_position.0);
        interpolation.previous.y = JavaF64::new(new_position.1);
        interpolation.previous.z = JavaF64::new(new_position.2);
        interpolation.previous_y_rot = JavaF32::new(new_rotation.0);
        interpolation.previous_x_rot = JavaF32::new(new_rotation.1);
        self.position.x = JavaF64::new(new_position.0);
        self.position.y = JavaF64::new(new_position.1);
        self.position.z = JavaF64::new(new_position.2);
        self.y_rot = JavaF32::new(new_rotation.0 % 360.0);
        self.x_rot = JavaF32::new((new_rotation.1 % 360.0).clamp(-90.0, 90.0));
    }

    pub(crate) fn tick_motion_interpolation(&mut self) {
        if self.motion_lerp_steps == 0 {
            return;
        }

        let steps = f64::from(self.motion_lerp_steps);
        let current = self.motion();
        let target = (
            self.motion_lerp_target.x.get(),
            self.motion_lerp_target.y.get(),
            self.motion_lerp_target.z.get(),
        );
        self.motion.x = JavaF64::new(current.0 + (target.0 - current.0) / steps);
        self.motion.y = JavaF64::new(current.1 + (target.1 - current.1) / steps);
        self.motion.z = JavaF64::new(current.2 + (target.2 - current.2) / steps);
        self.motion_lerp_steps -= 1;
    }

    pub(crate) fn tick_snowball(&mut self) {
        let air_drag = f64::from(0.99_f32);
        let gravity = if self.no_gravity {
            0.0
        } else {
            SNOWBALL_GRAVITY
        };
        let current_motion = self.motion();
        let motion = (
            current_motion.0 * air_drag,
            (current_motion.1 - gravity) * air_drag,
            current_motion.2 * air_drag,
        );
        self.motion.x = JavaF64::new(motion.0);
        self.motion.y = JavaF64::new(motion.1);
        self.motion.z = JavaF64::new(motion.2);

        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + motion.0);
        self.position.y = JavaF64::new(current_position.1 + motion.1);
        self.position.z = JavaF64::new(current_position.2 + motion.2);

        let horizontal_distance = (motion.0 * motion.0 + motion.2 * motion.2).sqrt();
        let target_x_rot =
            (mth_atan2(motion.1, horizontal_distance) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot = (mth_atan2(motion.0, motion.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot = JavaF32::new(projectile_lerp_rotation_f32(
            self.old_x_rot.get(),
            target_x_rot,
        ));
        self.y_rot = JavaF32::new(projectile_lerp_rotation_f32(
            self.old_y_rot.get(),
            target_y_rot,
        ));
    }
}
