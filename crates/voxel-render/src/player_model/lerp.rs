use super::*;

/// `HumanoidModel.setupAnim`'s swimming arms and legs.
pub(super) fn swim(root: &mut Part, state: &PlayerRenderState, pi: f32) {
    let swim_amount = state.swim_amount;
    let position = state.walk_position;
    let swim_position = position % 26.0;
    let swinging = |arm: Arm| state.attack_arm == arm && state.attack_time > 0.0;
    let right_amount = if state.right_arm_pose == ArmPose::Spear || swinging(Arm::Right) {
        0.0
    } else {
        swim_amount
    };
    let left_amount = if state.left_arm_pose == ArmPose::Spear || swinging(Arm::Left) {
        0.0
    } else {
        swim_amount
    };
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
