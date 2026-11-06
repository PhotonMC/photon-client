//! The stepped handler's queue (`SteppedInterpolationHandler.InterpolationData`).

use super::*;

/// One place the stepped handler passes through (`SteppedInterpolationHandler.Step`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SteppedStep {
    pub(super) position: DVec3,
    pub(super) y_rot: JavaF32,
    pub(super) x_rot: JavaF32,
    pub(super) tick_offset: i32,
}

/// What the stepped handler keeps besides the end point (`SteppedInterpolationHandler.InterpolationData`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SteppedData {
    pub(super) remaining_steps: std::collections::VecDeque<SteppedStep>,
    pub(super) last_position: DVec3,
    pub(super) last_y_rot: JavaF32,
    pub(super) last_x_rot: JavaF32,
    pub(super) current_step_ticks: JavaF32,
    pub(super) remaining_ticks: JavaF32,
    pub(super) speed: JavaF32,
}

impl SteppedData {
    /// `SteppedInterpolationHandler.interpolateTo`: a queue that ran dry starts over from where
    /// the entity is, then the step is added.
    pub(super) fn queue_step(
        &mut self,
        current: DVec3,
        current_rotation: (f32, f32),
        (end, y_rot, x_rot, steps): (DVec3, f32, f32, i32),
    ) {
        self.queue_step_start(current, current_rotation);
        self.add_step(end, y_rot, x_rot, steps);
    }

    /// `setStartingPoint`, when the queue ran dry.
    pub(super) fn queue_step_start(&mut self, current: DVec3, current_rotation: (f32, f32)) {
        if self.remaining_steps.is_empty() {
            self.last_position = current;
            self.last_y_rot = JavaF32::new(current_rotation.0);
            self.last_x_rot = JavaF32::new(current_rotation.1);
            self.current_step_ticks = JavaF32::new(1.0);
        }
    }

    /// `startInterpolating`: the starting point if the queue ran dry, then the steps of the path.
    pub(super) fn queue_path(
        &mut self,
        (current, current_rotation): (DVec3, (f32, f32)),
        (end, y_rot, x_rot, steps): (DVec3, f32, f32, i32),
        ((before, before_y_rot, before_x_rot), path): ((DVec3, f32, f32), Vec<(DVec3, i32)>),
    ) {
        if path.is_empty() || end == before {
            self.queue_step(current, current_rotation, (end, y_rot, x_rot, steps));
            return;
        }
        self.queue_step_start(current, current_rotation);
        if y_rot == before_y_rot && x_rot == before_x_rot {
            for (position, ticks) in path {
                self.add_step(position, y_rot, x_rot, ticks);
            }
            return;
        }
        let total: i32 = path.iter().map(|(_, ticks)| ticks).sum();
        let mut offset = 0;
        for (position, ticks) in path {
            offset += ticks;
            let alpha = offset as f32 / total as f32;
            self.add_step(
                position,
                mth_rot_lerp_f32(alpha, before_y_rot, y_rot),
                mth_lerp_f32(alpha, before_x_rot, x_rot),
                ticks,
            );
        }
    }

    pub(super) fn new(position: DVec3, y_rot: JavaF32, x_rot: JavaF32) -> Self {
        Self {
            remaining_steps: std::collections::VecDeque::new(),
            last_position: position,
            last_y_rot: y_rot,
            last_x_rot: x_rot,
            current_step_ticks: JavaF32::new(1.0),
            remaining_ticks: JavaF32::new(0.0),
            speed: JavaF32::new(1.0),
        }
    }

    pub(super) fn add_step(&mut self, position: DVec3, y_rot: f32, x_rot: f32, ticks: i32) {
        self.remaining_steps.push_back(SteppedStep {
            position,
            y_rot: JavaF32::new(y_rot),
            x_rot: JavaF32::new(x_rot),
            tick_offset: ticks,
        });
        self.remaining_ticks = JavaF32::new(self.remaining_ticks.get() + ticks as f32);
    }

    /// `InterpolationData.advance`.
    pub(super) fn advance(&mut self, ticks: f32, interpolation_steps: i32) {
        let steps = interpolation_steps as f32;
        let remaining = self.remaining_ticks.get();
        let target_speed = (remaining / steps).max(1.0);
        let mut speed = mth_lerp_f32(1.0 / steps, self.speed.get(), target_speed);
        let ticks = if ticks * speed < remaining {
            ticks * speed
        } else {
            speed = 1.0;
            remaining
        };
        self.speed = JavaF32::new(speed);
        self.current_step_ticks = JavaF32::new(self.current_step_ticks.get() + ticks);
        self.remaining_ticks = JavaF32::new(remaining - ticks);
    }

    /// `getNewPositionAndRotation`: where the entity is along the steps, dropping the ones
    /// passed; the end point once none is left.
    pub(super) fn new_position(&mut self, end: (DVec3, f32, f32)) -> (DVec3, f32, f32) {
        while let Some(step) = self.remaining_steps.front() {
            let offset = step.tick_offset;
            if self.current_step_ticks.get() < offset as f32 {
                let alpha = self.current_step_ticks.get() / offset as f32;
                let (a, b) = (self.last_position, step.position);
                let position = DVec3::new(
                    mth_lerp_f64(f64::from(alpha), a.x.get(), b.x.get()),
                    mth_lerp_f64(f64::from(alpha), a.y.get(), b.y.get()),
                    mth_lerp_f64(f64::from(alpha), a.z.get(), b.z.get()),
                );
                return (
                    position,
                    mth_rot_lerp_f32(alpha, self.last_y_rot.get(), step.y_rot.get()),
                    mth_lerp_f32(alpha, self.last_x_rot.get(), step.x_rot.get()),
                );
            }
            self.current_step_ticks = JavaF32::new(self.current_step_ticks.get() - offset as f32);
            self.last_position = step.position;
            self.last_y_rot = step.y_rot;
            self.last_x_rot = step.x_rot;
            self.remaining_steps.pop_front();
        }
        end
    }
}

impl EntityInterpolationState {
    /// The stepped handler's `startInterpolating` for a path; `before` is the end the handler
    /// aimed at until now (none before the first).
    pub(super) fn queue_stepped(
        &mut self,
        (current, current_rotation): (DVec3, (f32, f32)),
        (end, y_rot, x_rot, steps): (DVec3, f32, f32, i32),
        (before, path): (Option<(DVec3, f32, f32)>, &[EntityPathStep]),
    ) {
        let data = self.stepped.get_or_insert_with(|| {
            SteppedData::new(
                current,
                JavaF32::new(current_rotation.0),
                JavaF32::new(current_rotation.1),
            )
        });
        let path = path
            .iter()
            .map(|&((x, y, z), ticks)| (DVec3::new(x, y, z), ticks))
            .collect();
        let before = before.unwrap_or((DVec3::new(0.0, 0.0, 0.0), 0.0, 0.0));
        data.queue_path(
            (current, current_rotation),
            (end, y_rot, x_rot, steps),
            (before, path),
        );
    }
}
