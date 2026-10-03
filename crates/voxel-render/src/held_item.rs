//! `ItemInHandLayer`: where a stack held in a hand is drawn, from the arm of
//! the posed model.

use crate::joml::{Matrix4, Pose, Quaternion};
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
    in_hand(&mut pose, left, baby);
    pose
}

/// What `ItemInHandLayer.submitArmWithItem` does after the model has moved the
/// pose to the hand: lay the item along the arm and into the palm.
fn in_hand(pose: &mut Pose, left: bool, baby: bool) {
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
}

/// `CopperGolemModel.translateToHand` and the item layer after it: the body
/// and the arm, then, while the golem idles, the hand turned to face along
/// the arm and moved forward; otherwise the item shrunk and moved up and in.
pub fn copper_golem_hand_pose(
    base: &Pose,
    (root, body, arm): (&Part, &Part, &Part),
    side: Arm,
    idle: bool,
) -> Pose {
    let mut pose = *base;
    let left = side == Arm::Left;
    root.translate_and_rotate(&mut pose);
    body.translate_and_rotate(&mut pose);
    arm.translate_and_rotate(&mut pose);
    if idle {
        pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
            if left { 90.0 } else { -90.0 },
        )));
        pose.translate(0.0, 0.0, 0.125);
    } else {
        pose.scale(0.55, 0.55, 0.55);
        pose.translate(-0.125, 0.3125, -0.1875);
    }
    in_hand(&mut pose, left, false);
    pose
}

/// `CopperGolemModel.translateToHead` and the head layer after it (the
/// default transforms): the head, a little up and larger, then the item set
/// on top of it.
pub fn copper_golem_head_pose(base: &Pose, (root, body, head): (&Part, &Part, &Part)) -> Pose {
    let mut pose = *base;
    root.translate_and_rotate(&mut pose);
    body.translate_and_rotate(&mut pose);
    head.translate_and_rotate(&mut pose);
    pose.translate(0.0, 0.125, 0.0);
    pose.scale(1.0625, 1.0625, 1.0625);
    pose.translate(0.0, -0.25, 0.0);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        180.0,
    )));
    pose.scale(0.625, -0.625, -0.625);
    pose
}

/// `BlockDecorationLayer` on the copper golem's antenna
/// (`applyBlockOnAntennaTransform`, then the unit cube's bottom centre moved
/// to the antenna's centre, turned over).
pub fn copper_golem_antenna_pose(base: &Pose, (root, body, head): (&Part, &Part, &Part)) -> Pose {
    let mut pose = *base;
    root.translate_and_rotate(&mut pose);
    body.translate_and_rotate(&mut pose);
    head.translate_and_rotate(&mut pose);
    pose.translate(0.0, -1.75, 0.0);
    let mut cube = Matrix4::translation(-0.5, 0.0, -0.5);
    cube.rotate_around(
        &Quaternion::rotation_z(Quaternion::degrees_to_radians(180.0)),
        (0.5, 0.5, 0.5),
    );
    pose.mul_matrix(&cube);
    pose
}

/// `CustomHeadLayer.submit` up to the item's own display transform: scaled
/// across by the renderer's horizontal scale, to the head, then set on top of
/// it, turned and shrunk (`translateToHead`). `y_offset` and
/// `horizontal_scale` are the renderer's `Transforms`.
pub fn head_item_pose(
    base: &Pose,
    root: &Part,
    head: &Part,
    (y_offset, horizontal_scale): (f32, f32),
) -> Pose {
    let mut pose = *base;
    pose.scale(horizontal_scale, 1.0, horizontal_scale);
    root.translate_and_rotate(&mut pose);
    head.translate_and_rotate(&mut pose);
    pose.translate(0.0, -0.25 + y_offset, 0.0);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        180.0,
    )));
    pose.scale(0.625, -0.625, -0.625);
    pose
}

/// `CrossedArmsItemLayer`: what a villager, a wandering trader or a witch
/// holds sits on the folded arms, tipped back, a little larger than the
/// model's scale and turned over.
pub fn crossed_arms_item_pose(base: &Pose, root: &Part, arms: &Part) -> Pose {
    let mut pose = *base;
    root.translate_and_rotate(&mut pose);
    arms.translate_and_rotate(&mut pose);
    pose.rotate(&Quaternion::rotation_x(0.75));
    pose.scale(1.07, 1.07, 1.07);
    pose.translate(0.0, 0.13, -0.34);
    pose.rotate(&Quaternion::rotation_x(std::f64::consts::PI as f32));
    pose
}

/// `FoxHeldItemLayer`: what a fox holds in its mouth sits where its head is,
/// rolled and turned with it (not tipped by the head's own pose), then in
/// the mouth, laid flat (and on its side while the fox sleeps).
pub fn fox_item_pose(
    base: &Pose,
    head: &Part,
    (baby, sleeping): (bool, bool),
    (head_roll_angle, y_rot, x_rot): (f32, f32, f32),
) -> Pose {
    let mut pose = *base;
    pose.translate(head.x / 16.0, head.y / 16.0, head.z / 16.0);
    if baby {
        pose.scale(0.75, 0.75, 0.75);
    }
    pose.rotate(&Quaternion::rotation_z(head_roll_angle));
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        y_rot,
    )));
    pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
        x_rot,
    )));
    let (x, y, z) = match (baby, sleeping) {
        (true, true) => (0.4, 0.26, 0.15),
        (true, false) => (0.06, 0.26, -0.5),
        (false, true) => (0.46, 0.26, 0.22),
        (false, false) => (0.06, 0.27, -0.5),
    };
    pose.translate(x, y, z);
    pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
        90.0,
    )));
    if sleeping {
        pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
            90.0,
        )));
    }
    pose
}

/// `WitchItemLayer` for a potion: held at the tip of the nose, which the
/// head carries.
pub fn witch_potion_pose(base: &Pose, root: &Part, head: &Part, nose: &Part) -> Pose {
    let mut pose = *base;
    root.translate_and_rotate(&mut pose);
    head.translate_and_rotate(&mut pose);
    nose.translate_and_rotate(&mut pose);
    pose.translate(0.0625, 0.25, 0.0);
    for (turn, degrees) in [(true, 180.0), (false, 140.0), (true, 10.0), (false, 180.0)] {
        let angle = Quaternion::degrees_to_radians(degrees);
        pose.rotate(&if turn {
            Quaternion::rotation_z(angle)
        } else {
            Quaternion::rotation_x(angle)
        });
    }
    pose
}
