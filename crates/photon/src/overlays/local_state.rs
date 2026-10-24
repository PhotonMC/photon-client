use super::light::Glide;
use super::*;
use crate::equipment::arm_poses;

/// What the player's own model is posed from.
pub(super) fn local_render_state(
    (player, (pose, x_rot, side)): (
        &LocalPlayer,
        &(photon_world::local_player::ModelPose, f32, f32),
    ),
    (arms, (attack_time, attack_arm), glide): (&arm_poses::ArmState, (f32, Arm), Option<Glide>),
    (spears, partial): (&photon_render::entity::held_item::SpearView, f32),
) -> PlayerRenderState {
    PlayerRenderState {
        is_fall_flying: glide.is_some(),
        speed_value: glide.map_or(1.0, |glide| glide.speed_value),
        head_y_rot: side * photon_core::math::wrap_degrees_f32(pose.head_rot - pose.body_rot),
        x_rot: *x_rot,
        walk_position: pose.walk_position,
        walk_speed: pose.walk_speed,
        age_in_ticks: player.tick_count() as f32 + partial,
        swim_amount: pose.swim_amount,
        crouching: pose.crouching,
        spectator: player.spectating(),
        attack_time,
        attack_arm,
        right_arm_pose: arms.right,
        left_arm_pose: arms.left,
        is_using_item: arms.using_item,
        use_item_main_hand: arms.use_main_hand,
        ticks_using_item: arms.ticks_using_item,
        max_crossbow_charge_duration: arms.max_crossbow_charge,
        swing_animation: spears.swing_animation,
        right_spear: spears.right,
        left_spear: spears.left,
        ..Default::default()
    }
}
