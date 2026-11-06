//! The parts of a living entity's tick that its drawn pose reads, which the
//! local player and the players the server adds share: the limbs' walk
//! animation (`WalkAnimationState`), the swimming pose easing in and out, and
//! the body turning towards where the entity walks (`LivingEntity.tickHeadTurn`).

use photon_core::java_float::JavaF32;
use photon_core::math::{clamp_f32, mth_atan2, mth_degrees_difference_f32, wrap_degrees_f32};

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

    /// `calculateEntityAnimation` and `updateWalkAnimation` on the ground: the
    /// distance moved across it this tick sets the target speed, and a baby's
    /// steps count three times as far.
    pub fn follow(&mut self, moved: f64, baby: bool) {
        self.update(
            (moved as f32 * 4.0).min(1.0),
            0.4,
            if baby { 3.0 } else { 1.0 },
        );
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// `isMoving`.
    pub fn is_moving(&self) -> bool {
        self.speed > 1.0E-5
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

/// `Control.rotateTowards`: from an angle towards another, at most `max` turned.
fn rotate_towards(from_angle: f32, to_angle: f32, max: f32) -> f32 {
    from_angle + clamp_f32(mth_degrees_difference_f32(from_angle, to_angle), -max, max)
}

/// `LookControl.setLookAt(x, y, z, y_max, x_max)` then `tick()` for a mob
/// whose navigation is idle (the client never runs a path): the pitch resets,
/// then head and pitch turn towards the point, each by at most its limit.
/// `from` is the mob's feet in x and z and its eyes in y.
pub(crate) fn look_at_tick(
    (head, x_rot): (&mut f32, &mut f32),
    from: (f64, f64, f64),
    wanted: (f64, f64, f64),
    (y_max_rot_speed, x_max_rot_angle): (f32, f32),
) {
    // `1.0E-5F` widened to the doubles it is compared with, and the float
    // `180.0F / (float)Math.PI` the compiler folds into the angle's scale.
    let epsilon = f64::from(1.0e-5_f32);
    let degrees = f64::from(180.0_f32 / std::f32::consts::PI);
    *x_rot = 0.0;
    let xd = wanted.0 - from.0;
    let yd = wanted.1 - from.1;
    let zd = wanted.2 - from.2;
    let sd = (xd * xd + zd * zd).sqrt();
    if zd.abs() > epsilon || xd.abs() > epsilon {
        let y_rot_d = (mth_atan2(zd, xd) * degrees) as f32 - 90.0;
        *head = rotate_towards(*head, y_rot_d, y_max_rot_speed);
    }
    if yd.abs() > epsilon || sd.abs() > epsilon {
        let x_rot_d = (-(mth_atan2(yd, sd) * degrees)) as f32;
        *x_rot = rotate_towards(*x_rot, x_rot_d, x_max_rot_angle);
    }
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

/// `SwingAnimation.DEFAULT`'s duration: how long a swing lasts with nothing special in the hand.
pub const DEFAULT_SWING_DURATION: i32 = 6;

/// `SwingAnimationType`: `whack` is the default swing, `stab` a spear's.
pub const SWING_WHACK: u8 = 1;
pub const SWING_STAB: u8 = 2;

/// `LivingEntity.SwingDescription`: the swing under way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SwingDescription {
    off_hand: bool,
    kind: u8,
    animation_duration: i32,
    duration_ticks: i32,
}

/// A living entity's arm swing (`LivingEntity.SwingState`): what swings, how many ticks it has
/// run, and how far through it is, as the attack animation the model reads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SwingState {
    current: Option<SwingDescription>,
    ticks: i32,
    old_animation: JavaF32,
    animation: JavaF32,
}

impl SwingState {
    /// `SwingState.startIfAble`: a swing starts when none is under way or the one under way is
    /// not in its first half. `duration_ticks` is the animation's duration as the entity's
    /// haste or fatigue modify it.
    pub fn start_if_able(
        &mut self,
        off_hand: bool,
        kind: u8,
        animation_duration: i32,
        duration_ticks: i32,
    ) -> bool {
        if let Some(current) = self.current
            && self.ticks <= current.duration_ticks / 2
            && self.ticks > 0
        {
            return false;
        }
        let next = SwingDescription {
            off_hand,
            kind,
            animation_duration,
            duration_ticks,
        };
        if self.current != Some(next) {
            self.old_animation = JavaF32::new(0.0);
            self.animation = JavaF32::new(0.0);
        }
        self.current = Some(next);
        self.ticks = 0;
        true
    }

    /// `SwingState.tick`, run by `LivingEntity.baseTick`.
    pub fn tick(&mut self) {
        self.old_animation = self.animation;
        let Some(current) = self.current else {
            self.animation = JavaF32::new(0.0);
            return;
        };
        if current.duration_ticks > 0 {
            self.animation =
                JavaF32::new((self.ticks as f32 / current.duration_ticks as f32).min(1.0));
        }
        let past = self.ticks > current.duration_ticks;
        self.ticks += 1;
        if past {
            self.current = None;
            self.animation = JavaF32::new(0.0);
        }
    }

    pub fn is_swinging(&self) -> bool {
        self.current.is_some()
    }

    /// The ticks the swing has run.
    pub fn swing_time(&self) -> i32 {
        self.ticks
    }

    /// Whether the arm that swings is the off hand's.
    pub fn is_off_hand(&self) -> bool {
        self.current.is_some_and(|current| current.off_hand)
    }

    /// The swing's `SwingAnimationType` id, if one is under way.
    pub fn kind(&self) -> Option<u8> {
        self.current.map(|current| current.kind)
    }

    /// `SwingState.getAnimation`: between the last two ticks, wrapping when a new swing began.
    pub fn attack_anim(&self, partial: f32) -> f32 {
        let old = self.old_animation.get();
        let new = self.animation.get();
        let end = if new >= old { new } else { new + 1.0 };
        old + partial * (end - old)
    }
}
