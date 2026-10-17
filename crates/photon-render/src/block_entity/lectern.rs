//! The lectern's book (`LecternRenderer`): the pose it lies in, and how open it is held.

use super::enchant_table::{BookPose, book_pose};
use crate::joml::{Pose, Quaternion};

/// How the book on a lectern is held: `BookModel.State.forAnimation(0, 0.1, 0.9, 1.2)`.
pub fn lectern_book() -> BookPose {
    let held = book_pose(0.0, 0.0, 1.2);
    BookPose {
        pages: (0.1, 0.9),
        ..held
    }
}

/// The pose the book is drawn from, with `base` the pose of the block's corner, turned by `y_rot`
/// degrees and laid on the slope.
pub fn lectern_pose(base: &Pose, y_rot: f32) -> Pose {
    let mut pose = *base;
    pose.translate(0.5, 1.0625, 0.5);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        -y_rot,
    )));
    pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
        67.5,
    )));
    pose.translate(0.0, -0.125, 0.0);
    pose
}
