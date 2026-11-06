use super::*;

mod stepped;
use stepped::SteppedData;

impl ClientEntity {}

/// An entity's handler of the server's positions: the stepped one of the living and the linear
/// one of the rest (`InterpolationHandler`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntityInterpolationState {
    /// Where the easing ends (`interpolationData`).
    pub(crate) target: DVec3,
    pub(crate) target_y_rot: JavaF32,
    pub(crate) target_x_rot: JavaF32,
    /// Where the entity was after the last step (`lastPositionAndRotation`).
    pub(crate) previous: DVec3,
    pub(crate) previous_y_rot: JavaF32,
    pub(crate) previous_x_rot: JavaF32,
    /// The linear handler's remaining steps.
    pub(crate) steps: u8,
    stepped: Option<SteppedData>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedEntityPosition {
    pub(crate) current: (f64, f64, f64),
    pub(crate) target: (f64, f64, f64),
    pub(crate) ticking: bool,
}

impl EntityInterpolationState {
    /// Whether this is still easing to exactly this target.
    fn is_heading_to(&self, end: DVec3, (y_rot, x_rot): (JavaF32, JavaF32)) -> bool {
        self.active()
            && self.target == end
            && self.target_y_rot == y_rot
            && self.target_x_rot == x_rot
    }

    pub(crate) fn target_position(&self) -> (f64, f64, f64) {
        (
            self.target.x.get(),
            self.target.y.get(),
            self.target.z.get(),
        )
    }

    pub(crate) fn target_rotation(&self) -> (f32, f32) {
        (self.target_y_rot.get(), self.target_x_rot.get())
    }

    /// `hasActiveInterpolation`.
    pub(crate) fn active(&self) -> bool {
        match &self.stepped {
            Some(stepped) => !stepped.remaining_steps.is_empty(),
            None => self.steps > 0,
        }
    }

    /// The ticks still to go, rounded up: the linear steps, or the stepped handler's remaining
    /// ticks.
    fn remaining(&self) -> u8 {
        match &self.stepped {
            Some(stepped) => stepped.remaining_ticks.get().ceil().clamp(0.0, 255.0) as u8,
            None => self.steps,
        }
    }

    /// `addDelta` on the end point and, for the stepped handler, on every step.
    fn add_delta(&mut self, delta: (f64, f64, f64)) {
        let add = |position: &mut DVec3| {
            position.x = JavaF64::new(position.x.get() + delta.0);
            position.y = JavaF64::new(position.y.get() + delta.1);
            position.z = JavaF64::new(position.z.get() + delta.2);
        };
        add(&mut self.target);
        if let Some(stepped) = self.stepped.as_mut() {
            for step in &mut stepped.remaining_steps {
                add(&mut step.position);
            }
            add(&mut stepped.last_position);
        }
    }

    /// `addRotation` on the end point and, for the stepped handler, on every step.
    fn add_rotation(&mut self, y_rot: f32, x_rot: f32) {
        self.target_y_rot = JavaF32::new(self.target_y_rot.get() + y_rot);
        self.target_x_rot = JavaF32::new(self.target_x_rot.get() + x_rot);
        if let Some(stepped) = self.stepped.as_mut() {
            for step in &mut stepped.remaining_steps {
                step.y_rot = JavaF32::new(step.y_rot.get() + y_rot);
                step.x_rot = JavaF32::new(step.x_rot.get() + x_rot);
            }
            stepped.last_y_rot = JavaF32::new(stepped.last_y_rot.get() + y_rot);
            stepped.last_x_rot = JavaF32::new(stepped.last_x_rot.get() + x_rot);
        }
    }
}

impl EntityView<'_> {
    pub(crate) fn interpolation_rotation(&self) -> (f32, f32) {
        self.lerp
            .interpolation
            .as_ref()
            .filter(|interpolation| interpolation.active())
            .map_or_else(
                || self.rotation.body_rotation(),
                EntityInterpolationState::target_rotation,
            )
    }

    pub fn interpolated_rotation(&self, partial_tick: f32) -> (f32, f32, f32) {
        let current = self.rotation.rotation();
        if partial_tick == 1.0 {
            return current;
        }
        let previous = self.rotation.previous_rotation();
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
            self.rotation.previous_body_y_rotation(),
            self.rotation.body_y_rotation(),
        )
    }
}

impl EntityViewMut<'_> {
    /// `InterpolationHandler.interpolateTo` with a `PositionPath`: `position` is its end and
    /// `path` its steps (empty for a linear one).
    pub(crate) fn interpolate_along(
        &mut self,
        position: (f64, f64, f64),
        path: &[EntityPathStep],
        (y_rot, x_rot): (f32, f32),
    ) {
        let steps = self.interpolation_length();
        if steps == 0 {
            // `InterpolationHandler.interpolateTo` with no steps: put there at once.
            self.snap_to(position, y_rot, x_rot);
            self.lerp.interpolation = None;
            return;
        }
        let end = DVec3::new(position.0, position.1, position.2);
        let (y_rot_value, x_rot_value) = (JavaF32::new(y_rot), JavaF32::new(x_rot));
        if self.is_interpolating_to(end, (y_rot_value, x_rot_value)) {
            return;
        }
        let current_position = self.position();
        let current_rotation = self.rotation.body_rotation();
        let current = DVec3::new(current_position.0, current_position.1, current_position.2);
        let stepped = self.uses_stepped_interpolation();
        let before = self.lerp.interpolation.as_ref().map(|state| {
            (
                state.target,
                state.target_y_rot.get(),
                state.target_x_rot.get(),
            )
        });
        let state = self
            .lerp
            .interpolation
            .get_or_insert_with(|| EntityInterpolationState {
                target: end,
                target_y_rot: y_rot_value,
                target_x_rot: x_rot_value,
                previous: current,
                previous_y_rot: JavaF32::new(current_rotation.0),
                previous_x_rot: JavaF32::new(current_rotation.1),
                steps: 0,
                stepped: None,
            });
        if stepped {
            state.queue_stepped(
                (current, current_rotation),
                (end, y_rot, x_rot, steps),
                (before, path),
            );
        } else {
            state.steps = steps.clamp(0, 255) as u8;
        }
        state.target = end;
        state.target_y_rot = y_rot_value;
        state.target_x_rot = x_rot_value;
        state.previous = current;
        state.previous_y_rot = JavaF32::new(current_rotation.0);
        state.previous_x_rot = JavaF32::new(current_rotation.1);
    }

    /// Whether the handler already aims at this place and turn.
    fn is_interpolating_to(&self, end: DVec3, rotation: (JavaF32, JavaF32)) -> bool {
        (self.lerp.interpolation.as_ref()).is_some_and(|state| state.is_heading_to(end, rotation))
    }

    /// `Entity.absSnapTo(x, y, z, yRot, xRot)`: the place held inside the world's limits and the
    /// turn within a full circle, both with no movement from the last tick.
    pub(crate) fn abs_snap_to(&mut self, position: (f64, f64, f64), y_rot: f32, x_rot: f32) {
        let (x, y, z) = (
            position.0.clamp(-3.0e7, 3.0e7),
            position.1,
            position.2.clamp(-3.0e7, 3.0e7),
        );
        let (y_rot, x_rot) = (y_rot % 360.0, x_rot.clamp(-90.0, 90.0) % 360.0);
        self.pos.position = DVec3::new(x, y, z);
        self.pos.old_position = DVec3::new(x, y, z);
        self.rotation.y_rot = JavaF32::new(y_rot);
        self.rotation.x_rot = JavaF32::new(x_rot);
        self.rotation.old_y_rot = JavaF32::new(y_rot);
        self.rotation.old_x_rot = JavaF32::new(x_rot);
    }

    /// `InterpolationHandler.cancel`: no more steps towards the last target.
    pub(crate) fn cancel_interpolation(&mut self) {
        if let Some(interpolation) = self.lerp.interpolation.as_mut() {
            interpolation.steps = 0;
            if let Some(stepped) = interpolation.stepped.as_mut() {
                stepped.remaining_steps.clear();
                stepped.remaining_ticks = JavaF32::new(0.0);
                stepped.speed = JavaF32::new(1.0);
            }
        }
    }

    pub(crate) fn snap_to(&mut self, position: (f64, f64, f64), y_rot: f32, x_rot: f32) {
        self.pos.position.x = JavaF64::new(position.0);
        self.pos.position.y = JavaF64::new(position.1);
        self.pos.position.z = JavaF64::new(position.2);
        self.rotation.y_rot = JavaF32::new(y_rot);
        self.rotation.x_rot = JavaF32::new(x_rot);
        self.pos.old_position.x = JavaF64::new(position.0);
        self.pos.old_position.y = JavaF64::new(position.1);
        self.pos.old_position.z = JavaF64::new(position.2);
        self.rotation.old_y_rot = JavaF32::new(y_rot);
        self.rotation.old_x_rot = JavaF32::new(x_rot);
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
        self.move_or_interpolate_along(position, &[], (y_rot, x_rot));
    }

    /// `Entity.moveOrInterpolateTo` with a `PositionPath` (see `interpolate_along`).
    pub(crate) fn move_or_interpolate_along(
        &mut self,
        position: (f64, f64, f64),
        path: &[EntityPathStep],
        (y_rot, x_rot): (f32, f32),
    ) {
        if self.is_shulker() || self.minecart_track().is_some() {
            self.set_position(position);
            self.rotation.y_rot = JavaF32::new(y_rot % 360.0);
            self.rotation.x_rot = JavaF32::new((x_rot % 360.0).clamp(-90.0, 90.0));
        } else {
            self.interpolate_along(position, path, (y_rot, x_rot));
        }
    }

    /// `InterpolationHandler.interpolate`: one tick of the easing.
    pub(crate) fn tick_position_interpolation(&mut self, tick_speed: f32, follow_position: bool) {
        let current_position = self.position();
        let current_rotation = self.rotation.body_rotation();
        let steps = self.interpolation_length();
        let Some(interpolation) = self
            .lerp
            .interpolation
            .as_mut()
            .filter(|interpolation| interpolation.active())
        else {
            return;
        };
        interpolation.follow_movement(current_position, current_rotation, follow_position);
        let (position, y_rot, x_rot) =
            interpolation.ease(current_position, current_rotation, (steps, tick_speed));
        interpolation.previous = DVec3::new(position.0, position.1, position.2);
        let (y_rot, x_rot) = (y_rot % 360.0, (x_rot % 360.0).clamp(-90.0, 90.0));
        // `setLastPositionAndRotation` reads the entity back, after the turn was wrapped.
        interpolation.previous_y_rot = JavaF32::new(y_rot);
        interpolation.previous_x_rot = JavaF32::new(x_rot);
        self.pos.position.x = JavaF64::new(position.0);
        self.pos.position.y = JavaF64::new(position.1);
        self.pos.position.z = JavaF64::new(position.2);
        self.rotation.y_rot = JavaF32::new(y_rot);
        self.rotation.x_rot = JavaF32::new(x_rot);
    }
}

impl EntityInterpolationState {
    /// `InterpolationHandler.interpolate`: the target moves by what the entity moved on its own
    /// since the last step.
    fn follow_movement(
        &mut self,
        position: (f64, f64, f64),
        rotation: (f32, f32),
        follow_position: bool,
    ) {
        let delta_position = (
            position.0 - self.previous.x.get(),
            position.1 - self.previous.y.get(),
            position.2 - self.previous.z.get(),
        );
        let length_squared = delta_position.0 * delta_position.0
            + delta_position.1 * delta_position.1
            + delta_position.2 * delta_position.2;
        if follow_position && length_squared > f64::from(1.0e-5_f32) {
            self.add_delta(delta_position);
        }
        self.add_rotation(
            rotation.0 - self.previous_y_rot.get(),
            rotation.1 - self.previous_x_rot.get(),
        );
    }

    /// `doInterpolate`: the place the handler puts the entity at, and one step used.
    fn ease(
        &mut self,
        position: (f64, f64, f64),
        rotation: (f32, f32),
        (interpolation_steps, tick_speed): (i32, f32),
    ) -> ((f64, f64, f64), f32, f32) {
        if let Some(stepped) = self.stepped.as_mut() {
            let end = (
                self.target,
                self.target_y_rot.get(),
                self.target_x_rot.get(),
            );
            let (target, y_rot, x_rot) = stepped.new_position(end);
            stepped.advance(tick_speed, interpolation_steps);
            return (
                (target.x.get(), target.y.get(), target.z.get()),
                y_rot,
                x_rot,
            );
        }
        let alpha = 1.0 / f64::from(self.steps);
        let target = self.target_position();
        let target_rotation = self.target_rotation();
        self.steps -= 1;
        (
            (
                mth_lerp_f64(alpha, position.0, target.0),
                mth_lerp_f64(alpha, position.1, target.1),
                mth_lerp_f64(alpha, position.2, target.2),
            ),
            mth_rot_lerp_f64(alpha, f64::from(rotation.0), f64::from(target_rotation.0)) as f32,
            mth_lerp_f64(alpha, f64::from(rotation.1), f64::from(target_rotation.1)) as f32,
        )
    }
}

impl EntityView<'_> {
    pub fn motion_interpolation_target(&self) -> Option<(f64, f64, f64)> {
        (self.vel.motion_lerp_steps > 0).then(|| {
            (
                self.vel.motion_lerp_target.x.get(),
                self.vel.motion_lerp_target.y.get(),
                self.vel.motion_lerp_target.z.get(),
            )
        })
    }

    pub fn motion_interpolation_steps(&self) -> u8 {
        self.vel.motion_lerp_steps
    }
}

impl EntityViewMut<'_> {
    pub(crate) fn tick_motion_interpolation(&mut self) {
        if self.vel.motion_lerp_steps == 0 {
            return;
        }

        let steps = f64::from(self.vel.motion_lerp_steps);
        let current = self.motion();
        let target = (
            self.vel.motion_lerp_target.x.get(),
            self.vel.motion_lerp_target.y.get(),
            self.vel.motion_lerp_target.z.get(),
        );
        self.vel.motion.x = JavaF64::new(current.0 + (target.0 - current.0) / steps);
        self.vel.motion.y = JavaF64::new(current.1 + (target.1 - current.1) / steps);
        self.vel.motion.z = JavaF64::new(current.2 + (target.2 - current.2) / steps);
        self.vel.motion_lerp_steps -= 1;
    }
}

impl EntityView<'_> {
    /// Where `adjustInterpolationPosition` would put the target if the entity moved on its own
    /// since the last easing: the place to test for a collision before the move is followed.
    pub(crate) fn interpolation_follow_probe(&self) -> Option<photon_core::math::Vec3> {
        let interpolation = self.lerp.interpolation.as_ref()?;
        let (x, y, z) = self.position();
        let delta = (
            x - interpolation.previous.x.get(),
            y - interpolation.previous.y.get(),
            z - interpolation.previous.z.get(),
        );
        if delta.0 * delta.0 + delta.1 * delta.1 + delta.2 * delta.2 <= f64::from(1.0e-5_f32) {
            return None;
        }
        let target = interpolation.target_position();
        Some(photon_core::math::Vec3::new(
            target.0 + delta.0,
            target.1 + delta.1,
            target.2 + delta.2,
        ))
    }

    pub(crate) fn interpolation_position(&self) -> (f64, f64, f64) {
        self.lerp
            .interpolation
            .as_ref()
            .filter(|interpolation| interpolation.active())
            .map_or_else(
                || self.position(),
                EntityInterpolationState::target_position,
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
}

impl EntityView<'_> {
    pub fn interpolation_target_position(&self) -> Option<(f64, f64, f64)> {
        self.lerp
            .interpolation
            .as_ref()
            .filter(|interpolation| interpolation.active())
            .map(EntityInterpolationState::target_position)
    }

    pub fn interpolation_target_rotation(&self) -> Option<(f32, f32)> {
        self.lerp
            .interpolation
            .as_ref()
            .filter(|interpolation| interpolation.active())
            .map(EntityInterpolationState::target_rotation)
    }

    pub fn interpolation_steps(&self) -> u8 {
        self.lerp
            .interpolation
            .as_ref()
            .map_or(0, EntityInterpolationState::remaining)
    }

    /// `hasActiveInterpolation`.
    pub fn interpolation_active(&self) -> bool {
        self.is_interpolating()
    }

    /// Whether a position interpolation is still under way
    /// (`LivingEntity.isInterpolating`).
    pub(crate) fn is_interpolating(&self) -> bool {
        self.lerp
            .interpolation
            .as_ref()
            .is_some_and(EntityInterpolationState::active)
    }
}

impl EntityView<'_> {
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

    /// Whether the handler is the stepped one: every living entity but the shulker
    /// (`LivingEntity.makeInterpolationHandler`).
    pub(crate) fn uses_stepped_interpolation(&self) -> bool {
        self.lerp.stepped_easing && self.living_kind().is_some() && !self.is_shulker()
    }

    /// The length of the handler's easing (`AbstractInterpolationHandler.interpolationSteps`): a
    /// display's own, the type's update interval for the stepped handler, three for the rest.
    pub(super) fn interpolation_length(&self) -> i32 {
        use photon_data::entity_type::ids;
        if let Some(display) = self.kind.display() {
            return display.pos_rot_duration();
        }
        if self.uses_stepped_interpolation() {
            return match self.type_id {
                ids::PLAYER | ids::ALLAY | ids::MANNEQUIN => 2,
                _ => i32::from(LIVING_ENTITY_POSITION_LERP_STEPS),
            };
        }
        i32::from(LIVING_ENTITY_POSITION_LERP_STEPS)
    }
}
