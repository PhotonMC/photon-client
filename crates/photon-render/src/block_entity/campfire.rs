//! The food on a campfire (`CampfireRenderer`): one item to a slot, lying flat on the grill, each
//! slot a quarter turn from the last.

use crate::joml::{Pose, Quaternion};
use photon_core::direction::Direction;

/// How many items a campfire cooks at once.
pub const SLOTS: usize = 4;

/// `CampfireRenderer.submit`: the pose of the item in a slot, with `base` the pose of the block's
/// corner and `facing` the way the campfire faces.
pub fn item_pose(base: &Pose, facing: Direction, slot: usize) -> Pose {
    let mut pose = *base;
    pose.translate(0.5, 0.449_218_75, 0.5);
    let turn = Direction::from_2d_data_value((slot as i32 + facing.data2d()) % 4);
    let degrees = |angle: f32| Quaternion::degrees_to_radians(angle);
    pose.rotate(&Quaternion::rotation_y(degrees(-turn.to_y_rot())));
    pose.rotate(&Quaternion::rotation_x(degrees(90.0)));
    pose.translate(-0.3125, -0.3125, 0.0);
    pose.scale(0.375, 0.375, 0.375);
    pose
}
