//! The parts of a living entity's tick that its drawn pose reads, which the
//! local player and the players the server adds share: the limbs' walk
//! animation (`WalkAnimationState`), the swimming pose easing in and out, and
//! the body turning towards where the entity walks (`LivingEntity.tickHeadTurn`).

use voxel_core::math::{mth_atan2, wrap_degrees_f32};

/// `WalkAnimationState`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WalkAnimation {
    speed_old: f32,
    speed: f32,
    position: f32,
    position_scale: f32,
}

// The values are sums of finite steps: never NaN.
impl Eq for WalkAnimation {}

impl Default for WalkAnimation {
    fn default() -> Self {
        Self {
            speed_old: 0.0,
            speed: 0.0,
            position: 0.0,
            position_scale: 1.0,
        }
    }
}

impl WalkAnimation {
    /// `update`: the speed moves a fraction of the way to the target, and
    /// the position by the speed.
    pub fn update(&mut self, target_speed: f32, factor: f32, position_scale: f32) {
        self.speed_old = self.speed;
        self.speed += (target_speed - self.speed) * factor;
        self.position += self.speed;
        self.position_scale = position_scale;
    }

    /// `setSpeed`: what a damage event does, to show the limbs at once.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// `stop`.
    pub fn stop(&mut self) {
        *self = Self {
            position_scale: self.position_scale,
            ..Self::default()
        };
    }

    /// `calculateEntityAnimation` and `updateWalkAnimation` for a grown
    /// entity on the ground: the distance it moved across it this tick sets
    /// the target speed.
    pub fn follow(&mut self, moved: f64) {
        self.update((moved as f32 * 4.0).min(1.0), 0.4, 1.0);
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// `speed(partialTicks)`.
    pub fn speed_at(&self, partial_tick: f32) -> f32 {
        (self.speed_old + partial_tick * (self.speed - self.speed_old)).min(1.0)
    }

    pub fn position(&self) -> f32 {
        self.position * self.position_scale
    }

    /// `position(partialTicks)`.
    pub fn position_at(&self, partial_tick: f32) -> f32 {
        (self.position - self.speed * (1.0 - partial_tick)) * self.position_scale
    }
}

/// `LivingEntity.updateSwimAmount`: the swimming pose eases in over about
/// eleven ticks and out as quickly.
pub(crate) fn ease_swim_amount(swim_amount: f32, swimming: bool) -> f32 {
    if swimming {
        (swim_amount + 0.09).min(1.0)
    } else {
        (swim_amount - 0.09).max(0.0)
    }
}

/// `Mth.rotateIfNecessary`: the angle that puts `base` no more than
/// `max_difference` from `target`, on the side `target` is.
pub(crate) fn rotate_if_necessary(base: f32, target: f32, max_difference: f32) -> f32 {
    let delta = wrap_degrees_f32(target - base);
    let clamped = if delta < -max_difference {
        -max_difference
    } else {
        delta.min(max_difference)
    };
    target - clamped
}

/// A mob's head and body, which `BodyRotationControl` keeps in step on the
/// client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MobBodyControl {
    pub head_stable_time: i32,
    pub last_stable_y_head_rot: f32,
}

/// `Mob.getMaxHeadYRot`.
const MOB_MAX_HEAD_Y_ROT: f32 = 75.0;

/// `BodyRotationControl.clientTick`: a mob that moves turns its body to
/// where it looks and its head no further than 75 degrees from it; one that
/// stands lets its body follow a head that turned far, then after half a
/// second the body comes back to face the head squarely.
pub(crate) fn tick_mob_body(
    control: &mut MobBodyControl,
    (y_body_rot, y_head_rot): (&mut f32, &mut f32),
    y_rot: f32,
    (xd, zd): (f64, f64),
    carries_mob: bool,
) {
    if xd * xd + zd * zd > f64::from(2.500_000_3e-7_f32) {
        *y_body_rot = y_rot;
        *y_head_rot = rotate_if_necessary(*y_head_rot, *y_body_rot, MOB_MAX_HEAD_Y_ROT);
        control.last_stable_y_head_rot = *y_head_rot;
        control.head_stable_time = 0;
    } else if !carries_mob {
        if (*y_head_rot - control.last_stable_y_head_rot).abs() > 15.0 {
            control.head_stable_time = 0;
            control.last_stable_y_head_rot = *y_head_rot;
            *y_body_rot = rotate_if_necessary(*y_body_rot, *y_head_rot, MOB_MAX_HEAD_Y_ROT);
        } else {
            control.head_stable_time += 1;
            if control.head_stable_time > 10 {
                let since = control.head_stable_time - 10;
                let fraction = (since as f32 / 10.0).clamp(0.0, 1.0);
                let remaining = MOB_MAX_HEAD_Y_ROT * (1.0 - fraction);
                *y_body_rot = rotate_if_necessary(*y_body_rot, *y_head_rot, remaining);
            }
        }
    }
}

/// The body turn of `LivingEntity.tick`: the body goes towards where the
/// entity walks (or, when it stands, stays), then never more than 50 degrees
/// from the head; the old angles are kept within half a turn of the new.
pub(crate) fn turn_body(
    (y_body_rot, y_body_rot_old): (&mut f32, &mut f32),
    (y_rot, y_head_rot, y_head_rot_old): (f32, f32, &mut f32),
    (xd, zd): (f64, f64),
) {
    let side_dist = (xd * xd + zd * zd) as f32;
    let mut target = *y_body_rot;
    if side_dist > 0.002_500_000_2 {
        let walk_direction = mth_atan2(zd, xd) as f32 * (180.0 / std::f32::consts::PI) - 90.0;
        let difference = (wrap_degrees_f32(y_rot) - walk_direction).abs();
        target = if 95.0 < difference && difference < 265.0 {
            walk_direction - 180.0
        } else {
            walk_direction
        };
    }
    // `tickHeadTurn`.
    *y_body_rot += wrap_degrees_f32(target - *y_body_rot) * 0.3;
    let head_difference = wrap_degrees_f32(y_rot - *y_body_rot);
    let max_head_rotation = 50.0;
    if head_difference.abs() > max_head_rotation {
        let sign = if head_difference > 0.0 { 1.0 } else { -1.0 };
        *y_body_rot += head_difference - sign * max_head_rotation;
    }
    keep_old_angles_near((*y_body_rot, y_body_rot_old), (y_head_rot, y_head_rot_old));
}

/// The range checks that end `LivingEntity.tick`: the angles of the tick
/// before are moved by whole turns to within half a turn of the new ones, so
/// that they interpolate the short way round.
pub(crate) fn keep_old_angles_near(
    (y_body_rot, y_body_rot_old): (f32, &mut f32),
    (y_head_rot, y_head_rot_old): (f32, &mut f32),
) {
    while y_body_rot - *y_body_rot_old < -180.0 {
        *y_body_rot_old -= 360.0;
    }
    while y_body_rot - *y_body_rot_old >= 180.0 {
        *y_body_rot_old += 360.0;
    }
    while y_head_rot - *y_head_rot_old < -180.0 {
        *y_head_rot_old -= 360.0;
    }
    while y_head_rot - *y_head_rot_old >= 180.0 {
        *y_head_rot_old += 360.0;
    }
}
