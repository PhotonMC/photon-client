//! The items on a shelf (`ShelfRenderer`): three slots along its front, each item scaled to a
//! quarter and set on the shelf's floor or hung from its middle.

use crate::joml::{Pose, Quaternion};
use crate::objects::item::ModelExtent;
use photon_core::direction::Direction;

/// How many items a shelf holds (`ShelfBlockEntity.MAX_ITEMS`).
pub const SLOTS: usize = 3;

/// `ShelfRenderer.submit`: the turn of the shelf's items, in degrees.
pub fn item_y_rot(facing: Direction) -> f32 {
    if facing.axis().is_horizontal() {
        -facing.to_y_rot()
    } else {
        180.0
    }
}

/// `ShelfRenderer.submitItem`: the pose of the item in a slot, with `base` the pose of the
/// shelf's corner and `extent` the item model's bounding box.
pub fn item_pose(
    base: &Pose,
    slot: usize,
    (y_rot, align_to_bottom): (f32, bool),
    extent: ModelExtent,
) -> Pose {
    let mut pose = *base;
    pose.translate(0.5, 0.5, 0.5);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        y_rot,
    )));
    let across = (slot as f32 - 1.0) * 0.3125;
    let up = if align_to_bottom { -0.25 } else { 0.0 };
    pose.translate(across, up as f32, -0.25);
    pose.scale(0.25, 0.25, 0.25);
    let mut offset_y = -extent.min_y;
    if !align_to_bottom {
        offset_y += -(extent.max_y - extent.min_y) / 2.0;
    }
    pose.translate(0.0, offset_y as f32, 0.0);
    pose
}
