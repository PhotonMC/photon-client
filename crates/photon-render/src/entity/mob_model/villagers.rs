use super::*;

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

/// What a villager's (or a wandering trader's) model is posed from.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VillagerState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    /// Shaking its head: `AbstractVillager.getUnhappyCounter` is above zero.
    pub unhappy: bool,
}

/// `AbstractIllager.IllagerArmPose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IllagerArmPose {
    Crossed,
    Attacking,
    Spellcasting,
    BowAndArrow,
    CrossbowHold,
    CrossbowCharge,
    Celebrating,
    Neutral,
}

/// What an illager's model is posed from (`IllagerRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IllagerState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub riding: bool,
    pub arm_pose: IllagerArmPose,
    /// `mainArm` is the left.
    pub main_arm_left: bool,
    /// How far through the swing (`attackTime`, which `attackAnim` equals).
    pub attack_time: f32,
    /// Whether the main hand holds anything.
    pub has_item: bool,
    pub max_crossbow_charge: i32,
    pub ticks_using_item: f32,
    /// The hat layer of the head: only an illusioner shows it.
    pub hat_visible: bool,
}

/// What the witch's model is posed from (`WitchRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WitchState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    /// The entity's number: how fast the nose twitches.
    pub entity_id: i32,
    pub holding_item: bool,
}

impl IllagerArmPose {
    /// The pose of the reference's name (`CROSSBOW_HOLD`).
    pub fn named(name: &str) -> Option<Self> {
        Some(match name {
            "CROSSED" => Self::Crossed,
            "ATTACKING" => Self::Attacking,
            "SPELLCASTING" => Self::Spellcasting,
            "BOW_AND_ARROW" => Self::BowAndArrow,
            "CROSSBOW_HOLD" => Self::CrossbowHold,
            "CROSSBOW_CHARGE" => Self::CrossbowCharge,
            "CELEBRATING" => Self::Celebrating,
            "NEUTRAL" => Self::Neutral,
            _ => return None,
        })
    }
}

impl MobModel {
    /// `VillagerModel.setupAnim` (the baby's model inherits it): the head
    /// follows the look, or shakes while the villager is unhappy, and the legs
    /// swing as a pair.
    pub fn setup_villager(&mut self, state: &VillagerState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let look = &state.quadruped;
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = look.head_y_rot * degrees;
            head.x_rot = look.x_rot * degrees;
            if state.unhappy {
                head.z_rot = 0.3 * mth_sin_f64(f64::from(0.45 * state.age_in_ticks));
                head.x_rot = 0.4;
            } else {
                head.z_rot = 0.0;
            }
        }
        let (position, speed) = (look.walk_position, look.walk_speed);
        let swing = [
            ("right_leg", cos(position * 0.6662) * 1.4 * speed * 0.5),
            ("left_leg", cos(position * 0.6662 + pi) * 1.4 * speed * 0.5),
        ];
        for (name, x_rot) in swing {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
                leg.y_rot = 0.0;
            }
        }
    }

    /// `IllagerModel.setupAnim`: the head follows the look; riding or walking
    /// sets the limbs; then the arm pose takes the arms over, and only a
    /// crossed pose shows the folded arms in place of the two.
    pub fn setup_illager(&mut self, state: &IllagerState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (head_y_rot, head_x_rot) = (
            state.quadruped.head_y_rot * degrees,
            state.quadruped.x_rot * degrees,
        );
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = head_y_rot;
            head.x_rot = head_x_rot;
            if let Some(hat) = head.child_mut("hat") {
                hat.visible = state.hat_visible;
            }
        }
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        // Rotations of each limb: x, y, z.
        let (mut right_arm, mut left_arm) = ([0.0_f32; 3], [0.0_f32; 3]);
        let (right_leg, left_leg);
        if state.riding {
            let fifth = (-std::f64::consts::PI / 5.0) as f32;
            right_arm = [fifth, 0.0, 0.0];
            left_arm = [fifth, 0.0, 0.0];
            let tenth = (std::f64::consts::PI / 10.0) as f32;
            right_leg = [-1.413_716_7, tenth, 0.078_539_82];
            left_leg = [-1.413_716_7, -tenth, -0.078_539_82];
        } else {
            right_arm[0] = cos(position * 0.6662 + pi) * 2.0 * speed * 0.5;
            left_arm[0] = cos(position * 0.6662) * 2.0 * speed * 0.5;
            right_leg = [cos(position * 0.6662) * 1.4 * speed * 0.5, 0.0, 0.0];
            left_leg = [cos(position * 0.6662 + pi) * 1.4 * speed * 0.5, 0.0, 0.0];
        }
        illager_arms(
            state,
            (head_y_rot, head_x_rot),
            (&mut right_arm, &mut left_arm),
        );
        let set = |root: &mut Part, name: &str, [x_rot, y_rot, z_rot]: [f32; 3]| {
            if let Some(part) = root.child_mut(name) {
                part.x_rot = x_rot;
                part.y_rot = y_rot;
                part.z_rot = z_rot;
            }
        };
        set(&mut self.root, "right_arm", right_arm);
        set(&mut self.root, "left_arm", left_arm);
        set(&mut self.root, "right_leg", right_leg);
        set(&mut self.root, "left_leg", left_leg);
        let crossed = state.arm_pose == IllagerArmPose::Crossed;
        if let Some(arms) = self.root.child_mut("arms") {
            arms.visible = crossed;
        }
        for name in ["left_arm", "right_arm"] {
            if let Some(arm) = self.root.child_mut(name) {
                arm.visible = !crossed;
            }
        }
    }

    /// `WitchModel.setupAnim`: the head follows the look, the legs swing as a
    /// pair, and the nose twitches at a rate the entity's number sets; holding
    /// something draws it up and forward.
    pub fn setup_witch(&mut self, state: &WitchState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let look = &state.quadruped;
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = look.head_y_rot * degrees;
            head.x_rot = look.x_rot * degrees;
        }
        let (position, speed) = (look.walk_position, look.walk_speed);
        for (name, x_rot) in [
            ("right_leg", cos(position * 0.6662) * 1.4 * speed * 0.5),
            ("left_leg", cos(position * 0.6662 + pi) * 1.4 * speed * 0.5),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
        let twitch = 0.01 * (state.entity_id % 10) as f32;
        if let Some(nose) = self
            .root
            .child_mut("head")
            .and_then(|head| head.child_mut("nose"))
        {
            nose.x_rot = mth_sin_f64(f64::from(state.age_in_ticks * twitch)) * 4.5 * degrees;
            nose.z_rot = cos(state.age_in_ticks * twitch) * 2.5 * degrees;
            if state.holding_item {
                nose.x = 0.0;
                nose.y = 1.0;
                nose.z = -1.5;
                nose.x_rot = -0.9;
            }
        }
    }
}
