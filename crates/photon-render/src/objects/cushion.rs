//! A cushion (`CushionRenderer`): where its model is drawn, from the side its turn snaps to.

use photon_core::direction::Direction;

use crate::joml::{Axis, Pose, Quaternion};

/// How far the model is lowered after it is flipped over (the models are drawn upside down).
const LOWERING: f32 = 0.25;

/// `CushionRenderer.submit` up to the model: from the pose the entity's place leaves, turned
/// to the side it faces, flipped over and lowered.
pub fn cushion_pose(base: &Pose, direction: Direction) -> Pose {
    let mut pose = *base;
    let degrees = Quaternion::degrees_to_radians;
    pose.rotate_axis(Axis::YP, degrees(180.0 - direction.to_y_rot()));
    pose.rotate_axis(Axis::XP, degrees(180.0));
    pose.translate(0.0, -LOWERING, 0.0);
    pose
}
