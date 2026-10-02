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
    while *y_body_rot - *y_body_rot_old < -180.0 {
        *y_body_rot_old -= 360.0;
    }
    while *y_body_rot - *y_body_rot_old >= 180.0 {
        *y_body_rot_old += 360.0;
    }
    while y_head_rot - *y_head_rot_old < -180.0 {
        *y_head_rot_old -= 360.0;
    }
    while y_head_rot - *y_head_rot_old >= 180.0 {
        *y_head_rot_old += 360.0;
    }
}
