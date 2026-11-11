use super::*;

/// `HumanoidModel.setupAnim`'s swimming arms and legs.
pub(super) fn swim(root: &mut Part, state: &PlayerRenderState, pi: f32) {
    let swim_amount = state.swim_amount;
    if !photon_core::versions::rules()
        .render
        .swim_head_tips_before_arms
    {
        swim_head(root, state);
    }
    let position = state.walk_position;
    let swim_position = position % 26.0;
    let (right_amount, left_amount) = arm_swim_amounts(state);
    let quadratic = |x: f32| -65.0 * x + x * x;
    let half_pi = (std::f64::consts::PI / 2.0) as f32;
    // Targets for the left arm (turned the short way) and the right arm.
    let (x_target, z_left, z_right) = if swim_position < 14.0 {
        (
            (0.0, 0.0),
            pi + 1.870_796_4 * quadratic(swim_position) / quadratic(14.0),
            pi - 1.870_796_4 * quadratic(swim_position) / quadratic(14.0),
        )
    } else if (14.0..22.0).contains(&swim_position) {
        let inner = (swim_position - 14.0) / 8.0;
        (
            (half_pi * inner, half_pi * inner),
            5.012_389 - 1.870_796_4 * inner,
            1.270_796_3 + 1.870_796_4 * inner,
        )
    } else if (22.0..26.0).contains(&swim_position) {
        let inner = (swim_position - 22.0) / 4.0;
        (
            (half_pi - half_pi * inner, half_pi - half_pi * inner),
            pi,
            pi,
        )
    } else {
        ((f32::NAN, f32::NAN), f32::NAN, f32::NAN)
    };
    if !x_target.0.is_nan() && !state.is_using_item {
        let left = root.child("left_arm");
        left.x_rot = rot_lerp_rad(left_amount, left.x_rot, x_target.0);
        left.y_rot = rot_lerp_rad(left_amount, left.y_rot, pi);
        left.z_rot = rot_lerp_rad(left_amount, left.z_rot, z_left);
        let right = root.child("right_arm");
        right.x_rot = lerp(right_amount, right.x_rot, x_target.1);
        right.y_rot = lerp(right_amount, right.y_rot, pi);
        right.z_rot = lerp(right_amount, right.z_rot, z_right);
    }
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let left_leg = root.child("left_leg");
    left_leg.x_rot = lerp(
        swim_amount,
        left_leg.x_rot,
        0.3 * cos(position * 0.333_333_34 + pi),
    );
    let right_leg = root.child("right_leg");
    right_leg.x_rot = lerp(
        swim_amount,
        right_leg.x_rot,
        0.3 * cos(position * 0.333_333_34),
    );
}

pub(super) fn square(value: f32) -> f32 {
    value * value
}

/// `Mth.lerp(float, float, float)`.
pub(super) fn lerp(alpha: f32, from: f32, to: f32) -> f32 {
    from + alpha * (to - from)
}

/// `Mth.rotLerpRad`: the short way round between two angles in radians.
pub(crate) fn rot_lerp_rad(alpha: f32, from: f32, to: f32) -> f32 {
    let turn = (std::f64::consts::PI * 2.0) as f32;
    let mut difference = to - from;
    while difference < (-std::f64::consts::PI) as f32 {
        difference += turn;
    }
    while difference >= std::f64::consts::PI as f32 {
        difference -= turn;
    }
    from + alpha * difference
}

/// `DrownedModel.setupSwimAnimation`: the arms reach forward and sway, the legs kick, the head
/// stays level.
pub(super) fn drowned_swim(root: &mut Part, state: &PlayerRenderState) {
    let swim = state.swim_amount;
    let age = state.age_in_ticks;
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let sway = swim * 0.35 * sin(0.1 * age);
    let kick = swim * 0.55 * sin(0.1 * age);
    let reach = (-std::f64::consts::PI * 4.0 / 5.0) as f32;
    let right = root.child("right_arm");
    right.x_rot = rot_lerp_rad(swim, right.x_rot, reach) + sway;
    right.z_rot = rot_lerp_rad(swim, right.z_rot, -0.15);
    let left = root.child("left_arm");
    left.x_rot = rot_lerp_rad(swim, left.x_rot, reach) - sway;
    left.z_rot = rot_lerp_rad(swim, left.z_rot, 0.15);
    root.child("left_leg").x_rot -= kick;
    root.child("right_leg").x_rot += kick;
    root.child("head").x_rot = 0.0;
}

/// The head tips down while swimming, unless gliding.
pub(super) fn swim_head(root: &mut Part, state: &PlayerRenderState) {
    if !state.is_fall_flying && state.swim_amount > 0.0 {
        let head = root.child("head");
        head.x_rot = rot_lerp_rad(
            state.swim_amount,
            head.x_rot,
            (-std::f64::consts::PI / 4.0) as f32,
        );
    }
}

/// How far each arm (right, left) joins the stroke: not the arm of a swing, nor one holding a
/// spear.
fn arm_swim_amounts(state: &PlayerRenderState) -> (f32, f32) {
    let swing_runs = if photon_core::versions::rules()
        .render
        .swim_arm_skips_swing_with_progress
    {
        state.attack_time > 0.0
    } else {
        state.current_swing
    };
    let amount = |arm: Arm, pose: ArmPose| {
        let swinging = swing_runs && state.attack_arm == arm;
        if pose == ArmPose::Spear || swinging {
            0.0
        } else {
            state.swim_amount
        }
    };
    (
        amount(Arm::Right, state.right_arm_pose),
        amount(Arm::Left, state.left_arm_pose),
    )
}
