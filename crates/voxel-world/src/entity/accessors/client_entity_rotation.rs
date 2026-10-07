use super::*;

impl ClientEntity {
    pub(crate) fn tick_arrow(&mut self) {
        let current_position = self.position();
        let current_motion = self.motion();
        self.position.x = JavaF64::new(current_position.0 + current_motion.0);
        self.position.y = JavaF64::new(current_position.1 + current_motion.1);
        self.position.z = JavaF64::new(current_position.2 + current_motion.2);

        let horizontal_distance =
            (current_motion.0 * current_motion.0 + current_motion.2 * current_motion.2).sqrt();
        let target_x_rot = (mth_atan2(current_motion.1, horizontal_distance)
            * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot =
            (mth_atan2(current_motion.0, current_motion.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot = JavaF32::new(projectile_lerp_rotation_f32(self.x_rot.get(), target_x_rot));
        self.y_rot = JavaF32::new(projectile_lerp_rotation_f32(self.y_rot.get(), target_y_rot));

        let air_friction = f64::from(VANILLA_ARROW_AIR_FRICTION);
        let motion = (
            current_motion.0 * air_friction,
            current_motion.1 * air_friction,
            current_motion.2 * air_friction,
        );
        self.motion.x = JavaF64::new(motion.0);
        self.motion.y = JavaF64::new(
            motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_ARROW_GRAVITY
                },
        );
        self.motion.z = JavaF64::new(motion.2);
    }

    pub(crate) fn tick_experience_orb(&mut self, path_is_open: bool) {
        if self.first_tick {
            self.first_tick = false;
            return;
        }
        if !path_is_open {
            return;
        }

        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_EXPERIENCE_ORB_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + motion.0);
        self.position.y = JavaF64::new(current_position.1 + motion.1);
        self.position.z = JavaF64::new(current_position.2 + motion.2);

        let air_friction = f64::from(VANILLA_EXPERIENCE_ORB_AIR_FRICTION);
        self.motion.x = JavaF64::new(motion.0 * air_friction);
        self.motion.y = JavaF64::new(motion.1 * air_friction);
        self.motion.z = JavaF64::new(motion.2 * air_friction);
    }

    pub(crate) fn tick_primed_tnt(&mut self) {
        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_PRIMED_TNT_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + motion.0);
        self.position.y = JavaF64::new(current_position.1 + motion.1);
        self.position.z = JavaF64::new(current_position.2 + motion.2);

        let air_friction = f64::from(VANILLA_PRIMED_TNT_AIR_FRICTION);
        self.motion.x = JavaF64::new(motion.0 * air_friction);
        self.motion.y = JavaF64::new(motion.1 * air_friction);
        self.motion.z = JavaF64::new(motion.2 * air_friction);
    }

    pub(crate) fn tick_falling_block(&mut self) {
        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_FALLING_BLOCK_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + motion.0);
        self.position.y = JavaF64::new(current_position.1 + motion.1);
        self.position.z = JavaF64::new(current_position.2 + motion.2);

        let air_friction = f64::from(VANILLA_FALLING_BLOCK_AIR_FRICTION);
        self.motion.x = JavaF64::new(motion.0 * air_friction);
        self.motion.y = JavaF64::new(motion.1 * air_friction);
        self.motion.z = JavaF64::new(motion.2 * air_friction);
    }

    pub(crate) fn tick_firework(&mut self) {
        let current_motion = self.motion();
        let movement = (
            current_motion.0 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
            current_motion.1 + VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION,
            current_motion.2 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
        );
        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + movement.0);
        self.position.y = JavaF64::new(current_position.1 + movement.1);
        self.position.z = JavaF64::new(current_position.2 + movement.2);

        let horizontal_distance = (movement.0 * movement.0 + movement.2 * movement.2).sqrt();
        let target_x_rot =
            (mth_atan2(movement.1, horizontal_distance) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot =
            (mth_atan2(movement.0, movement.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot = JavaF32::new(projectile_lerp_rotation_f32(
            self.old_x_rot.get(),
            target_x_rot,
        ));
        self.y_rot = JavaF32::new(projectile_lerp_rotation_f32(
            self.old_y_rot.get(),
            target_y_rot,
        ));
        self.motion.x = JavaF64::new(movement.0);
        self.motion.y = JavaF64::new(movement.1);
        self.motion.z = JavaF64::new(movement.2);
    }

    pub(crate) fn set_position(&mut self, position: (f64, f64, f64)) {
        if self.is_shulker() {
            self.shulker_set_pos(position, block_of(self.position()));
            return;
        }
        self.position.x = JavaF64::new(position.0);
        self.position.y = JavaF64::new(position.1);
        self.position.z = JavaF64::new(position.2);
    }

    /// `Entity.getPositionCodec().getBase()`.
    pub(crate) fn packet_base(&self) -> (f64, f64, f64) {
        let [x, y, z] = self.tracking.packet_base_bits.map(f64::from_bits);
        (x, y, z)
    }

    pub(crate) fn set_packet_base(&mut self, base: (f64, f64, f64)) {
        self.tracking.packet_base_bits = [base.0.to_bits(), base.1.to_bits(), base.2.to_bits()];
    }

    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (
            self.position.x.get(),
            self.position.y.get(),
            self.position.z.get(),
        )
    }

    pub fn previous_position(&self) -> (f64, f64, f64) {
        (
            self.old_position.x.get(),
            self.old_position.y.get(),
            self.old_position.z.get(),
        )
    }

    pub fn interpolated_position(&self, partial_tick: f32) -> (f64, f64, f64) {
        let partial_tick = f64::from(partial_tick);
        let current = self.position();
        let previous = self.previous_position();
        (
            mth_lerp_f64(partial_tick, previous.0, current.0),
            mth_lerp_f64(partial_tick, previous.1, current.1),
            mth_lerp_f64(partial_tick, previous.2, current.2),
        )
    }

    pub fn motion(&self) -> (f64, f64, f64) {
        (
            self.motion.x.get(),
            self.motion.y.get(),
            self.motion.z.get(),
        )
    }

    pub fn boat_delta_rotation(&self) -> f32 {
        self.delta_rotation.get()
    }

    pub fn paddle_state(&self) -> (bool, bool) {
        (self.paddle_left, self.paddle_right)
    }

    pub(crate) fn set_motion_if_finite(&mut self, motion: (f64, f64, f64)) {
        if motion.0.is_finite() && motion.1.is_finite() && motion.2.is_finite() {
            self.motion.x = JavaF64::new(motion.0);
            self.motion.y = JavaF64::new(motion.1);
            self.motion.z = JavaF64::new(motion.2);
        }
    }

    pub fn motion_interpolation_target(&self) -> Option<(f64, f64, f64)> {
        (self.motion_lerp_steps > 0).then(|| {
            (
                self.motion_lerp_target.x.get(),
                self.motion_lerp_target.y.get(),
                self.motion_lerp_target.z.get(),
            )
        })
    }

    pub fn motion_interpolation_steps(&self) -> u8 {
        self.motion_lerp_steps
    }

    pub fn rotation(&self) -> (f32, f32, f32) {
        (self.y_rot.get(), self.x_rot.get(), self.y_head_rot.get())
    }

    /// `xRotO`.
    pub fn previous_x_rotation(&self) -> f32 {
        self.old_x_rot.get()
    }

    /// `yBodyRot`.
    pub fn y_body_rot(&self) -> f32 {
        self.y_body_rot.get()
    }

    /// `getYHeadRot`.
    pub fn y_head_rot(&self) -> f32 {
        self.y_head_rot.get()
    }

    pub fn head_rotation_target(&self) -> f32 {
        self.head_rotation_target.get()
    }

    pub fn head_rotation_lerp_steps(&self) -> u8 {
        self.head_lerp_steps
    }

    pub fn body_y_rotation(&self) -> f32 {
        self.y_body_rot.get()
    }

    pub fn previous_body_y_rotation(&self) -> f32 {
        self.old_y_body_rot.get()
    }

    pub(crate) fn clamp_vanilla_boat_passenger_rotation(
        &mut self,
        vehicle_y_rot: f32,
        animal_rotation_offset: Option<f32>,
    ) {
        let (old_y_rot, y_rot) = vanilla_boat_passenger_rotation_clamped_values(
            self.y_rot.get(),
            self.old_y_rot.get(),
            vehicle_y_rot,
        );
        self.old_y_rot = JavaF32::new(old_y_rot);
        self.y_rot = JavaF32::new(y_rot);
        if vanilla_passenger_is_living(self.type_id) {
            self.y_head_rot = self.y_rot;
            self.y_body_rot = JavaF32::new(vehicle_y_rot);
        }
        if let Some(rotation_offset) = animal_rotation_offset {
            self.y_body_rot = JavaF32::new(self.y_body_rot.get() + rotation_offset);
            self.y_head_rot = JavaF32::new(self.y_head_rot.get() + rotation_offset);
        }
    }

    pub(crate) fn add_vanilla_boat_passenger_rotation(&mut self, delta_rotation: f32) {
        self.y_rot = JavaF32::new(self.y_rot.get() + delta_rotation);
        if vanilla_passenger_is_living(self.type_id) {
            self.y_head_rot = JavaF32::new(self.y_head_rot.get() + delta_rotation);
        }
    }

    pub(crate) fn body_rotation(&self) -> (f32, f32) {
        (self.y_rot.get(), self.x_rot.get())
    }

    pub fn interpolation_target_position(&self) -> Option<(f64, f64, f64)> {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map(EntityInterpolationState::target_position)
    }

    pub fn interpolation_target_rotation(&self) -> Option<(f32, f32)> {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map(EntityInterpolationState::target_rotation)
    }

    pub fn interpolation_steps(&self) -> u8 {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or(0, |interpolation| interpolation.steps)
    }

    pub fn previous_rotation(&self) -> (f32, f32, f32) {
        (
            self.old_y_rot.get(),
            self.old_x_rot.get(),
            self.old_y_head_rot.get(),
        )
    }

    pub fn interpolated_rotation(&self, partial_tick: f32) -> (f32, f32, f32) {
        let current = self.rotation();
        if partial_tick == 1.0 {
            return current;
        }
        let previous = self.previous_rotation();
        (
            mth_rot_lerp_f32(partial_tick, previous.0, current.0),
            mth_lerp_f32(partial_tick, previous.1, current.1),
            mth_rot_lerp_f32(partial_tick, previous.2, current.2),
        )
    }

    /// `Entity.getYRot(partialTick)`: the turn between two ticks, a plain
    /// lerp (a boat's own, which does not wrap).
    pub fn y_rot_at(&self, partial_tick: f32) -> f32 {
        let current = self.rotation().0;
        if partial_tick == 1.0 {
            return current;
        }
        mth_lerp_f32(partial_tick, self.previous_rotation().0, current)
    }

    /// `Entity.getXRot(partialTick)`: the pitch between two ticks, a plain lerp.
    pub fn x_rot_at(&self, partial_tick: f32) -> f32 {
        let current = self.rotation().1;
        if partial_tick == 1.0 {
            return current;
        }
        mth_lerp_f32(partial_tick, self.previous_rotation().1, current)
    }

    /// The limbs' walk animation.
    pub fn walk_animation(&self) -> &crate::living::WalkAnimation {
        &self.walk
    }

    /// `getSwimAmount(partialTick)`.
    pub fn swim_amount(&self, partial_tick: f32) -> f32 {
        let old = self.old_swim_amount.get();
        old + partial_tick * (self.swim_amount.get() - old)
    }

    /// The body's facing between two ticks (`Mth.rotLerp` of `yBodyRotO`
    /// and `yBodyRot`).
    pub fn interpolated_body_y_rotation(&self, partial_tick: f32) -> f32 {
        mth_rot_lerp_f32(
            partial_tick,
            self.previous_body_y_rotation(),
            self.body_y_rotation(),
        )
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn is_ticking(&self) -> bool {
        self.ticking
    }

    pub fn is_always_ticking(&self) -> bool {
        self.always_ticking
    }

    pub fn metadata(&self) -> Option<&[u8]> {
        self.metadata.as_deref()
    }

    pub fn shared_flags(&self) -> u8 {
        self.shared_flags
    }

    pub fn living_flags(&self) -> u8 {
        self.living_flags
    }

    pub fn mob_flags(&self) -> u8 {
        self.mob_flags
    }

    pub fn is_no_ai(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_NO_AI)
    }

    pub fn is_left_handed(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_LEFT_HANDED)
    }

    /// Whether the type is a `Zombie` or one of its subclasses, which share
    /// `DATA_BABY_ID`.
    pub fn is_zombie_type(&self) -> bool {
        voxel_data::entity_type::name(self.type_id).is_some_and(|name| {
            matches!(
                name,
                "minecraft:zombie"
                    | "minecraft:husk"
                    | "minecraft:drowned"
                    | "minecraft:zombie_villager"
                    | "minecraft:zombified_piglin"
            )
        })
    }

    pub fn is_aggressive(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_AGGRESSIVE)
    }

    pub fn is_using_item(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_USING_ITEM)
    }

    /// `LivingEntity.getFallFlyingTicks`.
    pub fn fall_flying_ticks(&self) -> i32 {
        self.living.fall_fly_ticks
    }

    /// `ClientAvatarState.deltaMovementOnPreviousTick`.
    pub fn previous_motion(&self) -> (f64, f64, f64) {
        let [x, y, z] = self.living.previous_motion.map(f64::from_bits);
        (x, y, z)
    }
}
