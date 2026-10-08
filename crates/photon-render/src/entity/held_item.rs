//! `ItemInHandLayer`: where a stack held in a hand is drawn, from the arm of
//! the posed model.

use crate::entity::player_model::{Arm, ArmPose, Part, SwingAnimation};
use crate::entity::spear_animation::{third_person_attack_item, third_person_use_item};
use crate::joml::{Matrix4, Pose, Quaternion};
use photon_data::spear::{FORWARD_MOVEMENT, Spear};

/// What a hand's item does besides being held (`ItemInHandLayer.submitArmWithItem` after the
/// item is in the palm): a spear thrust by the stab of the swing, and a spear held up in use.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HandMotion {
    /// The swing of this arm is a stab: how far through it is, and how far the main hand's spear
    /// thrusts (`kineticWeapon.forwardMovement`, none for an item that is not one).
    pub stab: Option<(f32, f32)>,
    /// The spear this hand holds in use.
    pub spear_use: Option<SpearUse>,
}

/// A spear held up by an arm posed `SPEAR` that is in use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpearUse {
    pub spear: &'static Spear,
    /// `ticksUsingItem(arm)`, never zero here.
    pub ticks_using_item: f32,
    pub attack_time: f32,
    /// `ticksSinceKineticHitFeedback`.
    pub hit_feedback: f32,
}

/// What spears change in a humanoid's model state and in what its hands hold.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpearView {
    pub swing_animation: SwingAnimation,
    pub right: Option<&'static Spear>,
    pub left: Option<&'static Spear>,
    /// What each hand's item does besides being held, the right hand's first.
    pub motions: [HandMotion; 2],
}

/// What a humanoid's spear fields are read from.
pub struct SpearInput {
    /// The spear each arm holds.
    pub spears: (Option<&'static Spear>, Option<&'static Spear>),
    pub main_arm: Arm,
    /// `attackTime` and `attackArm`, and the swing's type, which the item in the attacking hand
    /// decides.
    pub attack: (f32, Arm),
    pub swing_animation: SwingAnimation,
    pub arm_poses: (ArmPose, ArmPose),
    /// Whether an item is in use, whether the main hand uses it, and for how long.
    pub using: (bool, bool, f32),
    /// `ticksSinceKineticHitFeedback`.
    pub hit_feedback: f32,
}

/// The spear fields of a humanoid's render state (`ArmedEntityRenderState.swingAnimationType`,
/// the stacks its arms hold) and what each hand's item does with them
/// (`ItemInHandLayer.submitArmWithItem`): the stab of a swing thrusts the item, and a spear
/// held up in use follows its raising.
pub fn spear_view(input: &SpearInput) -> SpearView {
    let (right, left) = input.spears;
    let (attack_time, attack_arm) = input.attack;
    let spear_of_arm = |arm: Arm| if arm == Arm::Left { left } else { right };
    let swing_animation = input.swing_animation;
    let (using_item, use_main_hand, used_ticks) = input.using;
    // `HumanoidRenderState.ticksUsingItem(arm)`.
    let ticks_using = |arm: Arm| {
        if using_item && use_main_hand == (arm == input.main_arm) {
            used_ticks
        } else {
            0.0
        }
    };
    let motion = |arm: Arm, pose: ArmPose| {
        let stab =
            (attack_time > 0.0 && attack_arm == arm && swing_animation == SwingAnimation::Stab)
                .then(|| {
                    // The thrust is the main hand's spear's.
                    let forward = if spear_of_arm(input.main_arm).is_some() {
                        FORWARD_MOVEMENT
                    } else {
                        0.0
                    };
                    (attack_time, forward)
                });
        let ticks = ticks_using(arm);
        let spear_use = spear_of_arm(arm)
            .filter(|_| pose == ArmPose::Spear && ticks != 0.0)
            .map(|spear| SpearUse {
                spear,
                ticks_using_item: ticks,
                attack_time,
                hit_feedback: input.hit_feedback,
            });
        HandMotion { stab, spear_use }
    };
    SpearView {
        swing_animation,
        right,
        left,
        motions: [
            motion(Arm::Right, input.arm_poses.0),
            motion(Arm::Left, input.arm_poses.1),
        ],
    }
}

/// `ItemInHandLayer.submitArmWithItem` up to the item's own display
/// transform: from the pose the model is drawn from, to the arm's hand,
/// turned so the item lies along it and moved into the palm. A slim arm's
/// hand is half a unit further out (`PlayerModel.translateToHand`), a
/// skeleton's a whole unit (`hand_shift`); a baby holds its item closer in.
///
/// A spear's stab and its use then move the item on (`SpearAnimations`).
pub fn hand_pose(
    base: &Pose,
    root: &Part,
    arm: &Part,
    side: Arm,
    (hand_shift, baby): (f32, bool),
    motion: &HandMotion,
) -> Pose {
    let mut pose = *base;
    let left = side == Arm::Left;
    root.translate_and_rotate(&mut pose);
    arm.translate_and_rotate_shifted(&mut pose, if left { -hand_shift } else { hand_shift });
    in_hand(&mut pose, left, baby);
    if let Some((attack_time, forward_movement)) = motion.stab {
        third_person_attack_item(&mut pose, attack_time, forward_movement);
    }
    if let Some(used) = motion.spear_use {
        third_person_use_item(
            &mut pose,
            used.attack_time,
            used.hit_feedback,
            used.ticks_using_item,
            side,
            used.spear,
        );
    }
    pose
}

/// `PlayerItemInHandLayer.submitArmWithItem`'s test: which arm's item a player holds to the eye
/// (a spyglass in use, not swinging), if either.
pub fn eye_held_arm(
    (using_item, use_main_hand): (bool, bool),
    (main_arm, attack_time): (Arm, f32),
    held_on_head: bool,
    arm: Arm,
) -> bool {
    // The hand an arm holds in: the main hand for the main arm.
    let main_hand = arm == main_arm;
    held_on_head && using_item && use_main_hand == main_hand && attack_time < 1.0e-5
}

/// `PlayerItemInHandLayer.renderItemHeldToEye`: from the model's root to the head, whose pitch is
/// held between up a little and straight down, then as a head item sits, out to the side of the
/// eye (before the layer's own `translate(-0.5)`).
pub fn held_to_eye_pose(base: &Pose, root: &Part, head: &Part, side: Arm) -> Pose {
    let mut pose = *base;
    root.translate_and_rotate(&mut pose);
    let mut tilted = head.clone();
    let pi = std::f64::consts::PI;
    tilted.x_rot = head.x_rot.clamp((-pi / 6.0) as f32, (pi / 2.0) as f32);
    tilted.translate_and_rotate(&mut pose);
    pose.translate(0.0, -0.25, 0.0);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        180.0,
    )));
    pose.scale(0.625, -0.625, -0.625);
    let out = if side == Arm::Left { -2.5 } else { 2.5 };
    pose.translate(out / 16.0, -0.0625, 0.0);
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

/// `VexModel.translateToHand` and the item layer after it: the root, the body
/// and the arm, the item shrunk to a little over half and moved in toward the
/// middle and forward, a side to each hand.
pub fn vex_hand_pose(base: &Pose, (root, body, arm): (&Part, &Part, &Part), side: Arm) -> Pose {
    let mut pose = *base;
    let left = side == Arm::Left;
    root.translate_and_rotate(&mut pose);
    body.translate_and_rotate(&mut pose);
    arm.translate_and_rotate(&mut pose);
    pose.scale(0.55, 0.55, 0.55);
    pose.translate(if left { -0.046875 } else { 0.046875 }, -0.15625, 0.078125);
    in_hand(&mut pose, left, false);
    pose
}

/// `AllayModel.translateToHand` and the item layer after it: the root and the
/// body, the item moved up and forward, tilted as the right arm is (for either
/// hand), shrunk to seven tenths and moved a little out.
pub fn allay_hand_pose(
    base: &Pose,
    (root, body, right_arm): (&Part, &Part, &Part),
    side: Arm,
) -> Pose {
    let mut pose = *base;
    root.translate_and_rotate(&mut pose);
    body.translate_and_rotate(&mut pose);
    pose.translate(0.0, 0.0625, 0.1875);
    pose.rotate(&Quaternion::rotation_x(right_arm.x_rot));
    pose.scale(0.7, 0.7, 0.7);
    pose.translate(0.0625, 0.0, 0.0);
    in_hand(&mut pose, side == Arm::Left, false);
    pose
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
