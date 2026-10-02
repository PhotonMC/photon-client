//! `ItemInHandLayer`: where a stack held in a hand is drawn, from the arm of
//! the posed model.

use crate::joml::{Pose, Quaternion};
use crate::player_model::{Arm, Part};

/// `ItemInHandLayer.submitArmWithItem` up to the item's own display
/// transform: from the pose the model is drawn from, to the arm's hand,
/// turned so the item lies along it and moved into the palm. A slim arm's
/// hand is half a unit further out (`PlayerModel.translateToHand`), a
/// skeleton's a whole unit (`hand_shift`); a baby holds its item closer in.
///
/// The swing of a spear and the pose of a stack in use (`SpearAnimations`,
/// `ArmPose.animateUseItem`) are not applied: the use is not read yet.
pub fn hand_pose(
    base: &Pose,
    root: &Part,
    arm: &Part,
    side: Arm,
    hand_shift: f32,
    baby: bool,
) -> Pose {
    let mut pose = *base;
    let left = side == Arm::Left;
    root.translate_and_rotate(&mut pose);
    arm.translate_and_rotate_shifted(&mut pose, if left { -hand_shift } else { hand_shift });
    pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
        -90.0,
    )));
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        180.0,
    )));
    let (x, y, z) = if baby {
        (0.0, 1.0, -4.5)
    } else {
        (1.0, 2.0, -10.0)
    };
    pose.translate(
        (if left { -1.0 } else { 1.0 }) * x / 16.0,
        y / 16.0,
        z / 16.0,
    );
    pose
}
