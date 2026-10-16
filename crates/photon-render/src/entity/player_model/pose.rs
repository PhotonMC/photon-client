use super::*;

/// `ModelPart.Polygon`: corner texture coordinates from the face's texel
/// rectangle, and a mirrored face reversed, with its x normal flipped.
pub(super) fn polygon(
    corners: [[f32; 3]; 4],
    uv: [f32; 4],
    mirror: bool,
    facing: Direction,
    texture: (f32, f32),
) -> Polygon {
    let [u0, v0, u1, v1] = uv;
    let facing = if mirror && matches!(facing, Direction::West | Direction::East) {
        facing.opposite()
    } else {
        facing
    };
    let (x, y, z) = facing.step();
    let mut corners = [
        (corners[0], [u1 / texture.0, v0 / texture.1]),
        (corners[1], [u0 / texture.0, v0 / texture.1]),
        (corners[2], [u0 / texture.0, v1 / texture.1]),
        (corners[3], [u1 / texture.0, v1 / texture.1]),
    ];
    if mirror {
        corners.reverse();
    }
    Polygon {
        corners,
        normal: [x as f32, y as f32, z as f32],
    }
}

/// `ChestModel.createSingleBodyLayer` with the lid shut, posed at the origin:
/// the bottom, the lid and the lock, each face's corners in blocks with the
/// texture coordinate in the chest's 64x64 texture.
pub fn chest_quads() -> Vec<[ModelVertex; 4]> {
    let cube =
        |tex: (i32, i32), min: [f32; 3], size: [f32; 3]| Cube::new(tex, min, size, 0.0, false);
    let root = Part::new(
        "root",
        Vec::new(),
        [0.0; 3],
        vec![
            Part::new(
                "bottom",
                vec![cube((0, 19), [1.0, 0.0, 1.0], [14.0, 10.0, 14.0])],
                [0.0; 3],
                Vec::new(),
            ),
            Part::new(
                "lid",
                vec![cube((0, 0), [1.0, 0.0, 0.0], [14.0, 5.0, 14.0])],
                [0.0, 9.0, 1.0],
                Vec::new(),
            ),
            Part::new(
                "lock",
                vec![cube((0, 0), [7.0, -2.0, 14.0], [2.0, 4.0, 1.0])],
                [0.0, 9.0, 1.0],
                Vec::new(),
            ),
        ],
    );
    let mut faces = Vec::new();
    root.render(&Pose::default(), "", &mut faces);
    faces.into_iter().map(|(_, face)| face).collect()
}

/// `HumanoidModel.setupAnim` for empty hands, on foot: the head follows the
/// look, the limbs swing with the walk, the arms breathe, with the attack
/// swing, the crouch and the swim. `root` holds the humanoid mesh's parts
/// (the player's, a zombie's).
pub fn humanoid_anim(root: &mut Part, state: &PlayerRenderState) {
    use std::f64::consts::PI;
    let pi = PI as f32;
    root.reset();
    let show_body = !state.spectator;
    for name in ["body", "right_arm", "left_arm", "right_leg", "left_leg"] {
        root.child(name).visible = show_body;
    }

    let degrees_to_radians = (PI / 180.0) as f32;
    let swim_amount = state.swim_amount;
    {
        let head = root.child("head");
        head.x_rot = state.x_rot * degrees_to_radians;
        head.y_rot = state.head_y_rot * degrees_to_radians;
        if state.is_fall_flying {
            head.x_rot = (-PI / 4.0) as f32;
        } else if swim_amount > 0.0 {
            head.x_rot = rot_lerp_rad(swim_amount, head.x_rot, (-PI / 4.0) as f32);
        }
    }

    let position = state.walk_position;
    let speed = state.walk_speed;
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let slowdown = state.speed_value;
    root.child("right_arm").x_rot = cos(position * 0.6662 + pi) * 2.0 * speed * 0.5 / slowdown;
    root.child("left_arm").x_rot = cos(position * 0.6662) * 2.0 * speed * 0.5 / slowdown;
    root.child("right_leg").x_rot = cos(position * 0.6662) * 1.4 * speed / slowdown;
    root.child("left_leg").x_rot = cos(position * 0.6662 + pi) * 1.4 * speed / slowdown;
    root.child("right_leg").y_rot = 0.005;
    root.child("left_leg").y_rot = -0.005;
    root.child("right_leg").z_rot = 0.005;
    root.child("left_leg").z_rot = -0.005;
    if state.is_passenger {
        sit(root);
    }
    pose_arms(root, state);

    setup_attack_animation(root, state);

    if state.crouching {
        root.child("body").x_rot = 0.5;
        root.child("right_arm").x_rot += 0.4;
        root.child("left_arm").x_rot += 0.4;
        root.child("right_leg").z += 4.0;
        root.child("left_leg").z += 4.0;
        root.child("head").y += 4.2;
        root.child("body").y += 3.2;
        root.child("left_arm").y += 3.2;
        root.child("right_arm").y += 3.2;
    }

    // `AnimationUtils.bobModelPart`: the arms breathe.
    for (name, scale, pose) in [
        ("right_arm", 1.0_f32, state.right_arm_pose),
        ("left_arm", -1.0, state.left_arm_pose),
    ] {
        // A spyglass held up does not breathe.
        if pose == ArmPose::Spyglass {
            continue;
        }
        let age = state.age_in_ticks;
        let arm = root.child(name);
        arm.z_rot += scale * (cos(age * 0.09) * 0.05 + 0.05);
        arm.x_rot += scale * (sin(age * 0.067) * 0.05);
    }

    if swim_amount > 0.0 {
        swim(root, state, pi);
    }
}

/// `HumanoidModel.setupAnim`'s riding: the arms lift and the legs sit.
pub(super) fn sit(root: &mut Part) {
    let pi = std::f64::consts::PI;
    let lift = (-pi / 5.0) as f32;
    root.child("right_arm").x_rot += lift;
    root.child("left_arm").x_rot += lift;
    for (name, side) in [("right_leg", 1.0_f32), ("left_leg", -1.0)] {
        let leg = root.child(name);
        leg.x_rot = -1.413_716_7;
        leg.y_rot = side * (pi / 10.0) as f32;
        leg.z_rot = side * 0.078_539_82;
    }
}

/// The order `HumanoidModel.setupAnim` poses the arms in: the arm of the hand in use first, else
/// the one a two-handed pose in the off hand leaves for last.
pub(super) fn pose_arms(root: &mut Part, state: &PlayerRenderState) {
    let right_handed = state.main_arm == Arm::Right;
    if state.is_using_item {
        // The arm of the hand in use is posed first.
        if state.use_item_main_hand == right_handed {
            pose_arm(root, state, Arm::Right);
            if !state.right_arm_pose.affects_offhand_pose() {
                pose_arm(root, state, Arm::Left);
            }
        } else {
            pose_arm(root, state, Arm::Left);
            if !state.left_arm_pose.affects_offhand_pose() {
                pose_arm(root, state, Arm::Right);
            }
        }
    } else {
        // Which arm is posed first: a two-handed pose in the off hand makes the
        // other arm the one that is posed last.
        let two_handed_offhand = if right_handed {
            state.left_arm_pose.is_two_handed()
        } else {
            state.right_arm_pose.is_two_handed()
        };
        if right_handed == two_handed_offhand {
            pose_arm(root, state, Arm::Right);
            if !state.right_arm_pose.affects_offhand_pose() {
                pose_arm(root, state, Arm::Left);
            }
        } else {
            pose_arm(root, state, Arm::Left);
            if !state.left_arm_pose.affects_offhand_pose() {
                pose_arm(root, state, Arm::Right);
            }
        }
    }
}

/// `HumanoidModel.setupAttackAnimation`: the body turns with the swing and the swinging arm
/// whacks, or stabs.
pub(super) fn setup_attack_animation(root: &mut Part, state: &PlayerRenderState) {
    use std::f64::consts::PI;
    let pi = PI as f32;
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let attack_time = state.attack_time;
    // The reference swings unless the time is at most zero, so a NaN
    // time swings too.
    if state.swing_weapon_down && attack_time > 0.0 {
        swing_weapon_down(root, state.main_arm, attack_time, state.age_in_ticks);
    } else if attack_time > 0.0 || attack_time.is_nan() {
        let mut body_y_rot = sin(f64::from(attack_time).sqrt() as f32 * (PI * 2.0) as f32) * 0.2;
        if state.attack_arm == Arm::Left {
            body_y_rot *= -1.0;
        }
        root.child("body").y_rot = body_y_rot;
        {
            let right = root.child("right_arm");
            right.z = sin(body_y_rot) * 5.0 * 1.0;
            right.x = -cos(body_y_rot) * 5.0 * 1.0;
        }
        {
            let left = root.child("left_arm");
            left.z = -sin(body_y_rot) * 5.0 * 1.0;
            left.x = cos(body_y_rot) * 5.0 * 1.0;
        }
        root.child("right_arm").y_rot += body_y_rot;
        root.child("left_arm").y_rot += body_y_rot;
        root.child("left_arm").x_rot += body_y_rot;
        let swing = 1.0 - square(square(1.0 - attack_time));
        let aa = sin(swing * pi);
        let head_x_rot = root.child("head").x_rot;
        let bb = sin(attack_time * pi) * -(head_x_rot - 0.7) * 0.75;
        match state.swing_animation {
            SwingAnimation::Whack => {
                let arm = root.child(match state.attack_arm {
                    Arm::Left => "left_arm",
                    Arm::Right => "right_arm",
                });
                arm.x_rot -= aa * 1.2 + bb;
                arm.y_rot += body_y_rot * 2.0;
                arm.z_rot += sin(attack_time * pi) * -0.4;
            }
            SwingAnimation::Stab => {
                third_person_attack_hand(root, attack_time, state.attack_arm);
            }
        }
    }
}

/// `AnimationUtils.animateCrossbowCharge`: the arm that holds the crossbow and the one that pulls.
pub(crate) fn animate_crossbow_charge(
    root: &mut Part,
    (max_duration, ticks_using_item): (f32, f32),
    holding_in_right_arm: bool,
) {
    let pi = std::f64::consts::PI;
    let (holding, pulling) = if holding_in_right_arm {
        ("right_arm", "left_arm")
    } else {
        ("left_arm", "right_arm")
    };
    root.child(holding).y_rot = if holding_in_right_arm { -0.8 } else { 0.8 };
    root.child(holding).x_rot = -0.970_796_35;
    let held_x = root.child(holding).x_rot;
    let use_ticks = ticks_using_item.clamp(0.0, max_duration);
    let alpha = use_ticks / max_duration;
    let sign = if holding_in_right_arm { 1.0 } else { -1.0 };
    let pull = root.child(pulling);
    pull.y_rot = (0.4 + alpha * (0.85 - 0.4)) * sign;
    pull.x_rot = held_x + alpha * ((-pi / 2.0) as f32 - held_x);
}

/// `AnimationUtils.animateCrossbowHold`.
pub(crate) fn animate_crossbow_hold(root: &mut Part, holding_in_right_arm: bool) {
    let pi = std::f64::consts::PI;
    let (head_x, head_y) = {
        let head = root.child("head");
        (head.x_rot, head.y_rot)
    };
    let (holding, shooting) = if holding_in_right_arm {
        ("right_arm", "left_arm")
    } else {
        ("left_arm", "right_arm")
    };
    root.child(holding).y_rot = (if holding_in_right_arm { -0.3 } else { 0.3 }) + head_y;
    root.child(shooting).y_rot = (if holding_in_right_arm { 0.6 } else { -0.6 }) + head_y;
    root.child(holding).x_rot = (-pi / 2.0) as f32 + head_x + 0.1;
    root.child(shooting).x_rot = -1.5 + head_x;
}

/// `AnimationUtils.swingWeaponDown`.
pub(super) fn swing_weapon_down(root: &mut Part, main_arm: Arm, attack_time: f32, age: f32) {
    let pi = std::f64::consts::PI as f32;
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let attack2 = sin(attack_time * pi);
    let attack = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
    root.child("right_arm").z_rot = 0.0;
    root.child("left_arm").z_rot = 0.0;
    root.child("right_arm").y_rot = (std::f64::consts::PI / 20.0) as f32;
    root.child("left_arm").y_rot = (-std::f64::consts::PI / 20.0) as f32;
    if main_arm == Arm::Right {
        root.child("right_arm").x_rot = -1.884_955_8 + cos(age * 0.09) * 0.15;
        root.child("left_arm").x_rot = -0.0 + cos(age * 0.19) * 0.5;
        root.child("right_arm").x_rot += attack2 * 2.2 - attack * 0.4;
        root.child("left_arm").x_rot += attack2 * 1.2 - attack * 0.4;
    } else {
        root.child("right_arm").x_rot = -0.0 + cos(age * 0.19) * 0.5;
        root.child("left_arm").x_rot = -1.884_955_8 + cos(age * 0.09) * 0.15;
        root.child("right_arm").x_rot += attack2 * 1.2 - attack * 0.4;
        root.child("left_arm").x_rot += attack2 * 2.2 - attack * 0.4;
    }
    // `bobArms`.
    for (name, scale) in [("right_arm", 1.0_f32), ("left_arm", -1.0)] {
        let arm = root.child(name);
        arm.z_rot += scale * (cos(age * 0.09) * 0.05 + 0.05);
        arm.x_rot += scale * (sin(age * 0.067) * 0.05);
    }
}

/// `HumanoidModel.poseRightArm` and `poseLeftArm`.
pub(super) fn pose_arm(root: &mut Part, state: &PlayerRenderState, arm: Arm) {
    let pi = std::f64::consts::PI;
    let (head_x, head_y) = {
        let head = root.child("head");
        (head.x_rot, head.y_rot)
    };
    let (name, pose) = match arm {
        Arm::Right => ("right_arm", state.right_arm_pose),
        Arm::Left => ("left_arm", state.left_arm_pose),
    };
    let right = arm == Arm::Right;
    match pose {
        ArmPose::Empty => root.child(name).y_rot = 0.0,
        ArmPose::Item => {
            let part = root.child(name);
            part.x_rot = part.x_rot * 0.5 - (pi / 10.0) as f32;
            part.y_rot = 0.0;
        }
        ArmPose::Block => {
            // `poseBlockingArm`.
            let part = root.child(name);
            part.x_rot = part.x_rot * 0.5 - 0.942_477_9
                + head_x.clamp((-pi * 4.0 / 9.0) as f32, 0.436_332_32);
            part.y_rot = (if right { -30.0_f32 } else { 30.0 }) * (pi / 180.0) as f32
                + head_y.clamp((-pi / 6.0) as f32, (pi / 6.0) as f32);
        }
        ArmPose::BowAndArrow => {
            let (right_turn, left_turn) = match arm {
                Arm::Right => (-0.1 + head_y, 0.1 + head_y + 0.4),
                Arm::Left => (-0.1 + head_y - 0.4, 0.1 + head_y),
            };
            let raised = (-pi / 2.0) as f32 + head_x;
            root.child("right_arm").y_rot = right_turn;
            root.child("left_arm").y_rot = left_turn;
            root.child("right_arm").x_rot = raised;
            root.child("left_arm").x_rot = raised;
        }
        ArmPose::ThrowTrident => {
            let part = root.child(name);
            part.x_rot = part.x_rot * 0.5 - pi as f32;
            part.y_rot = 0.0;
        }
        ArmPose::CrossbowCharge => animate_crossbow_charge(
            root,
            (state.max_crossbow_charge_duration, state.ticks_using_item),
            right,
        ),
        ArmPose::CrossbowHold => animate_crossbow_hold(root, right),
        ArmPose::Spyglass => {
            let part = root.child(name);
            let crouch = if state.crouching {
                (pi / 12.0) as f32
            } else {
                0.0
            };
            part.x_rot = (head_x - 1.919_862_2 - crouch).clamp(-2.4, 3.3);
            part.y_rot = head_y
                + if right {
                    -(pi / 12.0) as f32
                } else {
                    (pi / 12.0) as f32
                };
        }
        ArmPose::TootHorn => {
            let part = root.child(name);
            part.x_rot = head_x.clamp(-1.2, 1.2) - 1.483_529_8;
            part.y_rot = head_y
                + if right {
                    -(pi / 6.0) as f32
                } else {
                    (pi / 6.0) as f32
                };
        }
        ArmPose::Brush => {
            let part = root.child(name);
            part.x_rot = part.x_rot * 0.5 - (pi / 5.0) as f32;
            part.y_rot = 0.0;
        }
        ArmPose::Spear => {
            let use_state = HandUse {
                spear: match arm {
                    Arm::Right => state.right_spear,
                    Arm::Left => state.left_spear,
                },
                fall_flying_or_swimming: state.is_fall_flying || state.swim_amount > 0.0,
                ticks_using_item: state.ticks_using_item,
                is_using_item: state.is_using_item,
                use_item_main_hand: state.use_item_main_hand,
            };
            third_person_hand_use(root.child(name), (head_x, head_y), right, &use_state);
        }
    }
}
