use super::*;

impl ClientEntity {
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

    /// The body's facing between two ticks (`Mth.rotLerp` of `yBodyRotO`
    /// and `yBodyRot`).
    pub fn interpolated_body_y_rotation(&self, partial_tick: f32) -> f32 {
        mth_rot_lerp_f32(
            partial_tick,
            self.previous_body_y_rotation(),
            self.body_y_rotation(),
        )
    }

    /// Whether a position interpolation is still under way
    /// (`LivingEntity.isInterpolating`).
    pub(crate) fn is_interpolating(&self) -> bool {
        self.interpolation
            .as_ref()
            .is_some_and(|interpolation| interpolation.steps > 0)
    }

    /// Whether the entity has an `InterpolationHandler` (`getInterpolation` is not null): the
    /// living, the minecarts, the displays, the boats and the experience orbs ease to the
    /// positions the server sends.
    pub(crate) fn has_interpolation_handler(&self) -> bool {
        self.living_kind().is_some()
            || self.is_minecart()
            || self.kind.display().is_some()
            || self.is_boat()
            || self.is_fishing_bobber()
            || vanilla_projectile_kind(self.type_id) == Some(VanillaProjectileKind::ExperienceOrb)
    }

    /// The length of the handler's easing (`InterpolationHandler.interpolationSteps`): three, or
    /// a display's own.
    pub(super) fn interpolation_length(&self) -> i32 {
        self.kind
            .display()
            .map_or(i32::from(LIVING_ENTITY_POSITION_LERP_STEPS), |display| {
                display.pos_rot_duration()
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EntityInterpolationState {
    pub(crate) target: DVec3,
    pub(crate) target_y_rot: JavaF32,
    pub(crate) target_x_rot: JavaF32,
    pub(crate) previous: DVec3,
    pub(crate) previous_y_rot: JavaF32,
    pub(crate) previous_x_rot: JavaF32,
    pub(crate) steps: u8,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedEntityPosition {
    pub(crate) current: (f64, f64, f64),
    pub(crate) target: (f64, f64, f64),
    pub(crate) ticking: bool,
}

impl EntityInterpolationState {
    pub(crate) fn target_position(self) -> (f64, f64, f64) {
        (
            self.target.x.get(),
            self.target.y.get(),
            self.target.z.get(),
        )
    }

    pub(crate) fn target_rotation(self) -> (f32, f32) {
        (self.target_y_rot.get(), self.target_x_rot.get())
    }
}
