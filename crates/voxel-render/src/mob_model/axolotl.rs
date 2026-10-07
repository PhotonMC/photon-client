use super::*;

/// `Crackiness.GOLEM.byFraction`: how cracked an iron golem with this
/// fraction of its health left looks.
pub fn golem_crackiness(fraction: f32) -> Crackiness {
    if fraction < 0.25 {
        Crackiness::High
    } else if fraction < 0.5 {
        Crackiness::Medium
    } else if fraction < 0.75 {
        Crackiness::Low
    } else {
        Crackiness::None
    }
}

/// `Mth.triangleWave`.
pub(super) fn triangle_wave(index: f32, period: f32) -> f32 {
    ((index % period - period * 0.5).abs() - period * 0.25) / (period * 0.25)
}

/// What `IronGolemRenderer.setupRotations` rolls the pose by while the golem
/// walks, in degrees, or none while it stands.
pub fn iron_golem_roll(walk_position: f32, walk_speed: f32) -> Option<f32> {
    (walk_speed >= 0.01).then(|| 6.5 * triangle_wave(walk_position + 6.0, 13.0))
}

/// `EndCrystalRenderer.getY`: how high the crystal's glass floats, in blocks, at an age.
pub fn end_crystal_y(time_in_ticks: f32) -> f32 {
    let mut height = mth_sin_f64(f64::from(time_in_ticks * 0.2)) / 2.0 + 0.5;
    height = (height * height + height) * 0.4;
    height - 1.4
}

/// `GlowSquidRenderer.getBlockLightLevel`: a glow squid lights itself, less
/// while it is dark (after a hurt), but never below the light around it.
pub fn glow_squid_block_light(dark_ticks: i32, block_light: i32) -> i32 {
    let glow = voxel_core::math::clamped_lerp_f32(1.0 - dark_ticks as f32 / 10.0, 0.0, 15.0) as i32;
    if glow == 15 {
        15
    } else {
        glow.max(block_light)
    }
}

/// `PandaRenderer.setupRotations`, after what `LivingEntityRenderer` does:
/// the somersault in four quarter turns (eased between whole ticks), the
/// sitting tip (and the frightened shake) and lying on the back.
pub(super) fn panda_rotate(pose: &mut Pose, panda: &PandaRotation) {
    let rotate_x = |pose: &mut Pose, degrees: f32| {
        pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
            degrees,
        )));
    };
    if panda.roll_time > 0.0 {
        let transition = panda.roll_time - panda.roll_time.floor();
        let roll_pos = panda.roll_time.floor() as i32;
        let next = roll_pos + 1;
        let y = if panda.baby { 0.3 } else { 0.8 };
        let angle_of = |this: f32, next_angle: f32, threshold: f32| {
            if (next as f32) < threshold {
                lerp_f32(transition, this, next_angle)
            } else {
                this
            }
        };
        let (position, degrees) = if roll_pos < 8 {
            let angle = angle_of(90.0 * roll_pos as f32 / 7.0, 90.0 * next as f32 / 7.0, 8.0);
            ((y + 0.2) * (angle / 90.0), angle)
        } else if roll_pos < 16 {
            let counter = (roll_pos as f32 - 8.0) / 7.0;
            let angle = angle_of(
                90.0 + 90.0 * counter,
                90.0 + 90.0 * (next as f32 - 8.0) / 7.0,
                16.0,
            );
            (y + 0.2 + (y - 0.2) * (angle - 90.0) / 90.0, angle)
        } else if roll_pos < 24 {
            let counter = (roll_pos as f32 - 16.0) / 7.0;
            let angle = angle_of(
                180.0 + 90.0 * counter,
                180.0 + 90.0 * (next as f32 - 16.0) / 7.0,
                24.0,
            );
            (y + y * (270.0 - angle) / 90.0, angle)
        } else if roll_pos < 32 {
            let counter = (roll_pos as f32 - 24.0) / 7.0;
            let angle = angle_of(
                270.0 + 90.0 * counter,
                270.0 + 90.0 * (next as f32 - 24.0) / 7.0,
                32.0,
            );
            (y * ((360.0 - angle) / 90.0), angle)
        } else {
            (f32::NAN, 0.0)
        };
        if !position.is_nan() {
            pose.translate(0.0, position, 0.0);
            rotate_x(pose, -degrees);
        }
    }
    let sit = panda.sit_amount;
    if sit > 0.0 {
        pose.translate(0.0, 0.8 * sit, 0.0);
        rotate_x(pose, lerp_f32(sit, panda.x_rot, panda.x_rot + 90.0));
        pose.translate(0.0, -sit, 0.0);
        if panda.scared {
            // `Math.cos` in doubles, not `Mth.cos`.
            let shake = (f64::from(panda.age_in_ticks * 1.25).cos()
                * std::f64::consts::PI
                * f64::from(0.05_f32)) as f32;
            pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
                shake,
            )));
            if panda.baby {
                pose.translate(0.0, 0.8, 0.55);
            }
        }
    }
    let lie = panda.lie_on_back_amount;
    if lie > 0.0 {
        let y = if panda.baby { 0.5 } else { 1.3 };
        pose.translate(0.0, y * lie, 0.0);
        rotate_x(pose, lerp_f32(lie, panda.x_rot, panda.x_rot + 180.0));
    }
}

/// `PandaHoldsItemLayer`: what a sitting panda that is not frightened holds
/// is carried in front of it, bobbing while it eats.
pub fn panda_item_pose(base: &Pose, age_in_ticks: f32, eating: bool) -> Pose {
    let (mut z, mut y) = (-0.6_f32, 1.4_f32);
    if eating {
        let bob = mth_sin_f64(f64::from(age_in_ticks * 0.6));
        z -= 0.2 * bob + 0.2;
        y -= 0.09 * bob;
    }
    let mut pose = *base;
    pose.translate(0.1, y, z);
    pose
}

/// A part of an adult axolotl by its path from the root.
pub(super) fn at<'a>(root: &'a mut Part, path: &str) -> &'a mut Part {
    path.split('/').fold(root, |part, name| {
        part.child_mut(name).expect("the axolotl has this part")
    })
}

/// `AdultAxolotlModel.setupSwimmingAnimation`.
pub(super) fn axolotl_swimming(root: &mut Part, age_in_ticks: f32, x_rot: f32, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let degrees = (std::f64::consts::PI / 180.0) as f32;
    let half_pi = (std::f64::consts::PI / 2.0) as f32;
    let anim_move_speed = age_in_ticks * 0.33;
    let sine_sway = mth_sin_f64(f64::from(anim_move_speed));
    let cosine_sway = mth_cos_f64(f64::from(anim_move_speed));
    let body_sway = 0.13 * sine_sway;
    at(root, "body").x_rot += (x_rot * degrees + body_sway) * factor;
    at(root, "body/head").x_rot -= body_sway * 1.8 * factor;
    at(root, "body").y -= 0.45 * cosine_sway * factor;
    at(root, "body/head/top_gills").x_rot += (-0.5 * sine_sway - 0.8) * factor;
    let gill_y_rot = (0.3 * sine_sway + 0.9) * factor;
    at(root, "body/head/left_gills").y_rot += gill_y_rot;
    at(root, "body/head/right_gills").y_rot -= gill_y_rot;
    let tail = at(root, "body/tail");
    tail.y_rot += 0.3 * mth_cos_f64(f64::from(anim_move_speed * 0.9)) * factor;
    let hind = at(root, "body/left_hind_leg");
    hind.x_rot += 1.884_955_8 * factor;
    hind.y_rot += -0.4 * sine_sway * factor;
    hind.z_rot += half_pi * factor;
    let front = at(root, "body/left_front_leg");
    front.x_rot += 1.884_955_8 * factor;
    front.y_rot += (-0.2 * cosine_sway - 0.1) * factor;
    front.z_rot += half_pi * factor;
}

/// `AdultAxolotlModel.setupWaterHoveringAnimation`.
pub(super) fn axolotl_hovering(root: &mut Part, age_in_ticks: f32, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let anim_move_speed = age_in_ticks * 0.075;
    let cosine_sway = mth_cos_f64(f64::from(anim_move_speed));
    let sine_sway = mth_sin_f64(f64::from(anim_move_speed)) * 0.15;
    let body_x_rot = (-0.15 + 0.075 * cosine_sway) * factor;
    let body = at(root, "body");
    body.x_rot += body_x_rot;
    body.y -= sine_sway * factor;
    at(root, "body/head").x_rot -= body_x_rot;
    at(root, "body/head/top_gills").x_rot += 0.2 * cosine_sway * factor;
    let gill_y_rot = (-0.3 * cosine_sway - 0.19) * factor;
    at(root, "body/head/left_gills").y_rot += gill_y_rot;
    at(root, "body/head/right_gills").y_rot -= gill_y_rot;
    let hind = at(root, "body/left_hind_leg");
    hind.x_rot += ((std::f64::consts::PI * 3.0 / 4.0) as f32 - cosine_sway * 0.11) * factor;
    hind.y_rot += 0.471_238_94 * factor;
    hind.z_rot += 1.727_876_1 * factor;
    let front = at(root, "body/left_front_leg");
    front.x_rot += ((std::f64::consts::PI / 4.0) as f32 - cosine_sway * 0.2) * factor;
    front.y_rot += 2.042_035 * factor;
    at(root, "body/tail").y_rot += 0.5 * cosine_sway * factor;
}

/// `AdultAxolotlModel.setupGroundCrawlingAnimation`.
pub(super) fn axolotl_crawling(root: &mut Part, age_in_ticks: f32, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let half_pi = (std::f64::consts::PI / 2.0) as f32;
    let anim_move_speed = age_in_ticks * 0.11;
    let cosine_sway = mth_cos_f64(f64::from(anim_move_speed));
    let hind_leg_y_rot_sway = (cosine_sway * cosine_sway - 2.0 * cosine_sway) / 5.0;
    let front_leg_y_rot_sway = 0.7 * cosine_sway;
    let head_and_tail_y_rot = 0.09 * cosine_sway * factor;
    at(root, "body/head").y_rot += head_and_tail_y_rot;
    at(root, "body/tail").y_rot += head_and_tail_y_rot;
    let sine = mth_sin_f64(f64::from(anim_move_speed));
    let gill_angle = (0.6 - 0.08 * (cosine_sway * cosine_sway + 2.0 * sine)) * factor;
    at(root, "body/head/top_gills").x_rot += gill_angle;
    at(root, "body/head/left_gills").y_rot -= gill_angle;
    at(root, "body/head/right_gills").y_rot += gill_angle;
    let hind_leg_x_rot = 0.942_477_9 * factor;
    let front_leg_x_rot = 1.099_557_4 * factor;
    let left_hind = at(root, "body/left_hind_leg");
    left_hind.x_rot += hind_leg_x_rot;
    left_hind.y_rot += (1.5 - hind_leg_y_rot_sway) * factor;
    left_hind.z_rot += -0.1 * factor;
    let left_front = at(root, "body/left_front_leg");
    left_front.x_rot += front_leg_x_rot;
    left_front.y_rot += (half_pi - front_leg_y_rot_sway) * factor;
    let right_hind = at(root, "body/right_hind_leg");
    right_hind.x_rot += hind_leg_x_rot;
    right_hind.y_rot += (-1.0 - hind_leg_y_rot_sway) * factor;
    let right_front = at(root, "body/right_front_leg");
    right_front.x_rot += front_leg_x_rot;
    right_front.y_rot += ((-std::f64::consts::PI / 2.0) as f32 - front_leg_y_rot_sway) * factor;
}

/// `AdultAxolotlModel.setupLayStillOnGroundAnimation`.
pub(super) fn axolotl_lying_still(root: &mut Part, age_in_ticks: f32, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let anim_move_speed = age_in_ticks * 0.09;
    let sine_sway = mth_sin_f64(f64::from(anim_move_speed));
    let cosine_sway = mth_cos_f64(f64::from(anim_move_speed));
    let movement = sine_sway * sine_sway - 2.0 * sine_sway;
    let movement2 = cosine_sway * cosine_sway - 3.0 * sine_sway;
    let head = at(root, "body/head");
    head.x_rot += -0.09 * movement * factor;
    head.z_rot += -0.2 * factor;
    at(root, "body/tail").y_rot += (-0.1 + 0.1 * movement) * factor;
    let gill_angle = (0.6 + 0.05 * movement2) * factor;
    at(root, "body/head/top_gills").x_rot += gill_angle;
    at(root, "body/head/left_gills").y_rot -= gill_angle;
    at(root, "body/head/right_gills").y_rot += gill_angle;
    let hind = at(root, "body/left_hind_leg");
    hind.x_rot += 1.1 * factor;
    hind.y_rot += 1.0 * factor;
    let front = at(root, "body/left_front_leg");
    front.x_rot += 0.8 * factor;
    front.y_rot += 2.3 * factor;
    front.z_rot -= 0.5 * factor;
}

/// `AdultAxolotlModel.setupPlayDeadAnimation`.
pub(super) fn axolotl_playing_dead(root: &mut Part, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    let hind = at(root, "body/left_hind_leg");
    hind.x_rot += 1.413_716_7 * factor;
    hind.y_rot += 1.099_557_4 * factor;
    hind.z_rot += (std::f64::consts::PI / 4.0) as f32 * factor;
    let front = at(root, "body/left_front_leg");
    front.x_rot += (std::f64::consts::PI / 4.0) as f32 * factor;
    front.y_rot += 2.042_035 * factor;
    let body = at(root, "body");
    body.x_rot += -0.15 * factor;
    body.z_rot += 0.35 * factor;
}

/// `AdultAxolotlModel.applyMirrorLegRotations`: the right legs add the left
/// ones, mirrored.
pub(super) fn axolotl_mirror_legs(root: &mut Part, factor: f32) {
    if factor <= 1.0E-5 {
        return;
    }
    for (left, right) in [
        ("body/left_hind_leg", "body/right_hind_leg"),
        ("body/left_front_leg", "body/right_front_leg"),
    ] {
        let (x, y, z) = {
            let left = at(root, left);
            (left.x_rot, left.y_rot, left.z_rot)
        };
        let right = at(root, right);
        right.x_rot += x * factor;
        right.y_rot += -y * factor;
        right.z_rot += -z * factor;
    }
}

/// `DolphinCarryingItemLayer`: what a dolphin carries is held under its chin,
/// lower and further forward the more its head is pitched.
pub fn dolphin_item_pose(base: &Pose, x_rot: f32) -> Pose {
    let angle_x_percent = x_rot.abs() / 60.0;
    let mut pose = *base;
    if x_rot < 0.0 {
        pose.translate(
            0.0,
            1.0 - angle_x_percent * 0.5,
            -1.0 + angle_x_percent * 0.5,
        );
    } else {
        pose.translate(
            0.0,
            1.0 + angle_x_percent * 0.8,
            -1.0 + angle_x_percent * 0.2,
        );
    }
    pose
}

/// `Mth.rotLerp` (which wraps the radians it is given as if they were degrees).
pub(super) fn rot_lerp(amount: f32, from: f32, to: f32) -> f32 {
    from + amount * voxel_core::math::wrap_degrees_f32(to - from)
}

/// The arms of `IllagerModel.setupAnim`'s arm pose: each arm's rotations
/// about x, y and z, given the head's turn and pitch.
pub(super) fn illager_arms(
    state: &IllagerState,
    (head_y_rot, head_x_rot): (f32, f32),
    (right_arm, left_arm): (&mut [f32; 3], &mut [f32; 3]),
) {
    let pi = std::f64::consts::PI as f32;
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let age = state.age_in_ticks;
    match state.arm_pose {
        IllagerArmPose::Attacking => {
            let attack_time = state.attack_time;
            if state.has_item {
                // `AnimationUtils.swingWeaponDown`.
                let attack_2 = sin(attack_time * pi);
                let attack = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
                right_arm[2] = 0.0;
                left_arm[2] = 0.0;
                right_arm[1] = (std::f64::consts::PI / 20.0) as f32;
                left_arm[1] = (-std::f64::consts::PI / 20.0) as f32;
                if state.main_arm_left {
                    right_arm[0] = -0.0 + cos(age * 0.19) * 0.5;
                    left_arm[0] = -1.884_955_8 + cos(age * 0.09) * 0.15;
                    right_arm[0] += attack_2 * 1.2 - attack * 0.4;
                    left_arm[0] += attack_2 * 2.2 - attack * 0.4;
                } else {
                    right_arm[0] = -1.884_955_8 + cos(age * 0.09) * 0.15;
                    left_arm[0] = -0.0 + cos(age * 0.19) * 0.5;
                    right_arm[0] += attack_2 * 2.2 - attack * 0.4;
                    left_arm[0] += attack_2 * 1.2 - attack * 0.4;
                }
            } else {
                // `AnimationUtils.animateZombieArms`, aggressive.
                let arm_drop = -pi / 1.5;
                let y_modifier = sin(attack_time * pi);
                let x_modifier = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
                let x_rot = arm_drop + y_modifier * 1.2 - x_modifier * 0.4;
                let y_rot = 0.1 - y_modifier * 0.6;
                *right_arm = [x_rot, -y_rot, 0.0];
                *left_arm = [x_rot, y_rot, 0.0];
            }
            // `bobArms`.
            for (arm, scale) in [(&mut *right_arm, 1.0_f32), (&mut *left_arm, -1.0)] {
                arm[2] += scale * (cos(age * 0.09) * 0.05 + 0.05);
                arm[0] += scale * (sin(age * 0.067) * 0.05);
            }
        }
        IllagerArmPose::Spellcasting => {
            // The arms' offsets the reference sets are the ones they start from.
            right_arm[0] = cos(age * 0.6662) * 0.25;
            left_arm[0] = cos(age * 0.6662) * 0.25;
            right_arm[2] = (std::f64::consts::PI * 3.0 / 4.0) as f32;
            left_arm[2] = (-std::f64::consts::PI * 3.0 / 4.0) as f32;
            right_arm[1] = 0.0;
            left_arm[1] = 0.0;
        }
        IllagerArmPose::BowAndArrow => {
            right_arm[1] = -0.1 + head_y_rot;
            right_arm[0] = (-std::f64::consts::PI / 2.0) as f32 + head_x_rot;
            left_arm[0] = -0.942_477_9 + head_x_rot;
            left_arm[1] = head_y_rot - 0.4;
            left_arm[2] = (std::f64::consts::PI / 2.0) as f32;
        }
        IllagerArmPose::CrossbowHold => {
            // `animateCrossbowHold`, held in the right arm.
            right_arm[1] = -0.3 + head_y_rot;
            left_arm[1] = 0.6 + head_y_rot;
            right_arm[0] = (-std::f64::consts::PI / 2.0) as f32 + head_x_rot + 0.1;
            left_arm[0] = -1.5 + head_x_rot;
        }
        IllagerArmPose::CrossbowCharge => {
            // `animateCrossbowCharge`, held in the right arm.
            right_arm[1] = -0.8;
            right_arm[0] = -0.970_796_35;
            left_arm[0] = right_arm[0];
            let maximum = state.max_crossbow_charge as f32;
            let use_ticks = if state.ticks_using_item < 0.0 {
                0.0
            } else if state.ticks_using_item > maximum {
                maximum
            } else {
                state.ticks_using_item
            };
            let alpha = use_ticks / maximum;
            left_arm[1] = 0.4 + alpha * (0.85 - 0.4);
            let half_pi = (-std::f64::consts::PI / 2.0) as f32;
            left_arm[0] += alpha * (half_pi - left_arm[0]);
        }
        IllagerArmPose::Celebrating => {
            right_arm[0] = cos(age * 0.6662) * 0.05;
            right_arm[2] = 2.670_354;
            right_arm[1] = 0.0;
            left_arm[0] = cos(age * 0.6662) * 0.05;
            left_arm[2] = (-std::f64::consts::PI * 3.0 / 4.0) as f32;
            left_arm[1] = 0.0;
        }
        IllagerArmPose::Crossed | IllagerArmPose::Neutral => {}
    }
}

/// A part of an equine's model: a baby donkey's hang under its body.
pub(super) fn equine_part<'a>(root: &'a mut Part, nested: bool, name: &str) -> &'a mut Part {
    let holder = if nested && name != "body" {
        root.child_mut("body").expect("the baby donkey has a body")
    } else {
        root
    };
    holder.child_mut(name).expect("an equine has this part")
}
