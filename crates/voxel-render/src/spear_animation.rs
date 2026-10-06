//! `SpearAnimations`: how a spear is held up and thrust on a model seen from outside: the arm
//! that raises it, sways it and lowers it while it is used, the stab of its swing, and where the
//! item sits in the hand through both. The reference writes its easing curves in `Ease`; the ones
//! these animations use are here.

use crate::joml::{Pose, Quaternion};
use crate::player_model::{Arm, Part};
use voxel_core::math::{mth_cos_f64, mth_sin_f64};
use voxel_data::spear::{FORWARD_MOVEMENT, Spear};

/// `(float) (Math.PI / 180.0)`.
const DEG: f32 = (std::f64::consts::PI / 180.0) as f32;
/// `(float) Math.PI`.
const PI: f32 = std::f32::consts::PI;

fn sin(value: f32) -> f32 {
    mth_sin_f64(f64::from(value))
}

fn cos(value: f32) -> f32 {
    mth_cos_f64(f64::from(value))
}

fn square(value: f32) -> f32 {
    value * value
}

fn cube(value: f32) -> f32 {
    value * value * value
}

/// `Ease.inQuad`.
pub fn in_quad(x: f32) -> f32 {
    x * x
}

/// `Ease.inOutSine`.
pub fn in_out_sine(x: f32) -> f32 {
    -(cos(PI * x) - 1.0) / 2.0
}

/// `Ease.inOutExpo`.
pub fn in_out_expo(x: f32) -> f32 {
    let x64 = f64::from(x);
    if x < 0.5 {
        if x == 0.0 {
            0.0
        } else {
            (2.0_f64.powf(20.0 * x64 - 10.0) / 2.0) as f32
        }
    } else if x == 1.0 {
        1.0
    } else {
        ((2.0 - 2.0_f64.powf(-20.0 * x64 + 10.0)) / 2.0) as f32
    }
}

/// `Ease.outBack`.
pub fn out_back(x: f32) -> f32 {
    1.0 + 2.701_58 * cube(x - 1.0) + 1.701_58 * square(x - 1.0)
}

/// `Ease.outQuart`.
pub fn out_quart(x: f32) -> f32 {
    1.0 - square(square(1.0 - x))
}

/// `Ease.outCubic`.
pub fn out_cubic(x: f32) -> f32 {
    1.0 - cube(1.0 - x)
}

/// `Ease.outCirc`.
pub fn out_circ(x: f32) -> f32 {
    f64::from(1.0 - square(x - 1.0)).sqrt() as f32
}

/// `Ease.inCirc`.
pub fn in_circ(x: f32) -> f32 {
    (-f64::from(1.0 - x * x).sqrt()) as f32 + 1.0
}

/// `Ease.inOutElastic`.
pub fn in_out_elastic(x: f32) -> f32 {
    if x == 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    let x64 = f64::from(x);
    let sin = ((20.0 * x64 - 11.125) * f64::from(PI) * 4.0 / 9.0).sin();
    if x < 0.5 {
        (-(2.0_f64.powf(20.0 * x64 - 10.0) * sin) / 2.0) as f32
    } else {
        (2.0_f64.powf(-20.0 * x64 + 10.0) * sin / 2.0 + 1.0) as f32
    }
}

/// `SpearAnimations.progress`: how far `time` is from `start` to `end`, held between none and all.
fn progress(time: f32, start: f32, end: f32) -> f32 {
    let value = (time - start) / (end - start);
    if value < 0.0 {
        0.0
    } else if value.is_nan() {
        value
    } else {
        value.min(1.0)
    }
}

/// `SpearAnimations.UseParams`: where a spear in use is in its raising, swaying, lowering and
/// raising back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UseParams {
    pub raise_progress: f32,
    pub raise_progress_start: f32,
    pub raise_progress_middle: f32,
    pub raise_progress_end: f32,
    pub sway_progress: f32,
    pub lower_progress: f32,
    pub raise_back_progress: f32,
    pub sway_intensity: f32,
    pub sway_scale_slow: f32,
    pub sway_scale_fast: f32,
}

impl UseParams {
    /// `UseParams.fromKineticWeapon`.
    pub fn new(spear: &Spear, time: f32) -> Self {
        let finish_raising = spear.delay_ticks;
        let finish_swaying = spear.dismount_ticks + finish_raising;
        let start_swaying = finish_swaying - 20;
        let finish_lowering = spear.knockback_ticks + finish_raising;
        let start_lowering = finish_lowering - 40;
        let finish_raising_back = spear.damage_ticks + finish_raising;
        let raise_progress = progress(time, 0.0, finish_raising as f32);
        let sway_progress = progress(time, start_swaying as f32, finish_swaying as f32);
        let lower_progress = out_cubic(in_out_elastic(progress(
            time - 20.0,
            start_lowering as f32,
            finish_lowering as f32,
        )));
        let raise_back_progress = progress(
            time,
            (finish_raising_back - 5) as f32,
            finish_raising_back as f32,
        );
        let sway_intensity = 2.0 * out_circ(sway_progress) - 2.0 * in_circ(raise_back_progress);
        Self {
            raise_progress,
            raise_progress_start: progress(raise_progress, 0.0, 0.5),
            raise_progress_middle: progress(raise_progress, 0.5, 0.8),
            raise_progress_end: progress(raise_progress, 0.8, 1.0),
            sway_progress,
            lower_progress,
            raise_back_progress,
            sway_intensity,
            sway_scale_slow: sin(time * 19.0 * DEG) * sway_intensity,
            sway_scale_fast: sin(time * 30.0 * DEG) * sway_intensity,
        }
    }
}

/// What `thirdPersonHandUse` reads of the model's state besides the arm and the head.
#[derive(Debug, Clone, Copy)]
pub struct HandUse<'a> {
    /// The spear the arm holds, if it holds one.
    pub spear: Option<&'a Spear>,
    pub fall_flying_or_swimming: bool,
    pub ticks_using_item: f32,
    pub is_using_item: bool,
    /// Whether the hand using the item is the main hand.
    pub use_item_main_hand: bool,
}

/// `SpearAnimations.thirdPersonHandUse`: the arm that holds a spear is raised with it, turned
/// with the head, and swayed as the use goes on.
pub fn third_person_hand_use(
    arm: &mut Part,
    (head_x_rot, head_y_rot): (f32, f32),
    holding_in_right_arm: bool,
    state: &HandUse,
) {
    let invert: i32 = if holding_in_right_arm { 1 } else { -1 };
    let invert = invert as f32;
    arm.y_rot = -0.1 * invert + head_y_rot;
    arm.x_rot = (-std::f64::consts::PI / 2.0) as f32 + head_x_rot + 0.8;
    if state.fall_flying_or_swimming {
        arm.x_rot -= 0.959_931_1;
    }
    let to_degrees = 180.0 / PI;
    arm.y_rot = DEG * (to_degrees * arm.y_rot).clamp(-60.0, 60.0);
    arm.x_rot = DEG * (to_degrees * arm.x_rot).clamp(-120.0, 30.0);
    // The reference tests against the hand the main hand's side holds in, not the main arm.
    // `!(ticks <= 0)`: a time that is not a number counts as used.
    let used = state.ticks_using_item > 0.0 || state.ticks_using_item.is_nan();
    let in_use = used && (!state.is_using_item || state.use_item_main_hand == holding_in_right_arm);
    let Some(spear) = state.spear.filter(|_| in_use) else {
        return;
    };
    let params = UseParams::new(spear, state.ticks_using_item);
    arm.y_rot += -invert * params.sway_scale_fast * DEG * params.sway_intensity * 1.0;
    arm.z_rot += -invert * params.sway_scale_slow * DEG * params.sway_intensity * 0.5;
    arm.x_rot += DEG
        * (-40.0 * params.raise_progress_start
            + 30.0 * params.raise_progress_middle
            + -20.0 * params.raise_progress_end
            + 20.0 * params.lower_progress
            + 10.0 * params.raise_back_progress
            + 0.6 * params.sway_scale_slow * params.sway_intensity);
}

/// The three curves of a stab, from how far its swing is: the preparation, the thrust and the
/// retraction.
fn stab_curves(attack_time: f32) -> (f32, f32, f32) {
    (
        in_out_sine(progress(attack_time, 0.0, 0.05)),
        in_quad(progress(attack_time, 0.05, 0.2)),
        in_out_expo(progress(attack_time, 0.4, 1.0)),
    )
}

/// `SpearAnimations.thirdPersonAttackHand`: the stab takes back the turn the swing put into the
/// arms, then draws the attacking arm back, thrusts it and brings it in.
pub fn third_person_attack_hand(root: &mut Part, attack_time: f32, attack_arm: Arm) {
    let body_y_rot = root.child("body").y_rot;
    root.child("right_arm").y_rot -= body_y_rot;
    root.child("left_arm").y_rot -= body_y_rot;
    root.child("left_arm").x_rot -= body_y_rot;
    let (prepare, attack, retract) = stab_curves(attack_time);
    let arm = root.child(match attack_arm {
        Arm::Left => "left_arm",
        Arm::Right => "right_arm",
    });
    arm.x_rot += (90.0 * prepare - 120.0 * attack + 30.0 * retract) * DEG;
}

/// `HumanoidModel.ArmPose.SPEAR.animateUseItem`, `SpearAnimations.thirdPersonUseItem`: the item
/// follows the raising, the sway and the swing of the spear it is.
pub fn third_person_use_item(
    pose: &mut Pose,
    attack_time: f32,
    ticks_since_hit_feedback: f32,
    time_held: f32,
    arm: Arm,
    spear: &Spear,
) {
    let (_, attack, retract) = stab_curves(attack_time);
    let params = UseParams::new(spear, time_held);
    let invert = if arm == Arm::Right { 1.0 } else { -1.0 };
    let raise_progress_modified = 1.0 - out_back(1.0 - params.raise_progress);
    let hit_feedback = hit_feedback_amount(ticks_since_hit_feedback);
    pose.translate(
        0.0,
        (f64::from(-hit_feedback) * 0.4) as f32,
        -FORWARD_MOVEMENT * (raise_progress_modified - params.raise_back_progress) + hit_feedback,
    );
    pose.rotate_around(
        &Quaternion::rotation_x(-Quaternion::degrees_to_radians(
            70.0 * (params.raise_progress - params.raise_back_progress) - 40.0 * (attack - retract),
        )),
        (0.0, -0.031_25, 0.125),
    );
    pose.rotate_around(
        &Quaternion::rotation_y(Quaternion::degrees_to_radians(
            invert * 90.0 * (params.raise_progress - params.sway_progress + 3.0 * retract + attack),
        )),
        (0.0, 0.0, 0.125),
    );
}

/// `SpearAnimations.thirdPersonAttackItem`: the item swings with the stab, tipped back and thrust.
pub fn third_person_attack_item(pose: &mut Pose, attack_time: f32, forward_movement: f32) {
    if attack_time <= 0.0 {
        return;
    }
    let (_, attack, retract) = stab_curves(attack_time);
    pose.rotate_around(
        &Quaternion::rotation_x(-Quaternion::degrees_to_radians(70.0 * (attack - retract))),
        (0.0, -0.125, 0.125),
    );
    pose.translate(0.0, forward_movement * (attack - retract), 0.0);
}

/// `SpearAnimations.hitFeedbackAmount`: the kick of a hit, over the ticks since it landed.
fn hit_feedback_amount(ticks_since_feedback_start: f32) -> f32 {
    0.4 * (out_quart(progress(ticks_since_feedback_start, 1.0, 3.0))
        - in_out_sine(progress(ticks_since_feedback_start, 3.0, 10.0)))
}
