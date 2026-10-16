use super::*;

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

/// What a panda's models are posed from (`PandaRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PandaState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub unhappy: bool,
    pub sneezing: bool,
    pub sneeze_time: i32,
    pub eating: bool,
    pub scared: bool,
    pub sit_amount: f32,
    pub lie_on_back_amount: f32,
    pub roll_amount: f32,
    pub baby: bool,
}

/// What `PandaRenderer.setupRotations` turns the whole panda by: the roll it
/// somersaults in (`rollTime`, zero when it does not), how far it sits and
/// lies on its back, and its fright.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PandaRotation {
    pub roll_time: f32,
    pub sit_amount: f32,
    pub lie_on_back_amount: f32,
    /// The look's pitch the sitting tips are measured from.
    pub x_rot: f32,
    pub age_in_ticks: f32,
    pub scared: bool,
    pub baby: bool,
}

/// What a polar bear's models are posed from: the quadruped's, how far it
/// has stood up (`PolarBearRenderState.standScale`, zero to one) and the age
/// scale its shifts are scaled by.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PolarBearState {
    pub quadruped: QuadrupedState,
    pub stand_scale: f32,
    pub age_scale: f32,
}

impl MobModel {
    /// `PolarBearModel.setupAnim` (grown and baby): the quadruped's pose, then
    /// standing up lifts the body and head and raises the front legs.
    pub fn setup_polar_bear(&mut self, state: &PolarBearState) {
        self.setup_quadruped(&state.quadruped);
        let pi = std::f64::consts::PI as f32;
        let stand = state.stand_scale * state.stand_scale;
        let age = state.age_scale;
        let (mut leg_y, mut leg_z) = (0.0, 0.0);
        if let Some(leg) = self.root.child_mut("right_front_leg") {
            leg.y -= stand * age * 20.0;
            leg.z += stand * age * 4.0;
            leg.x_rot -= stand * pi * 0.45;
            (leg_y, leg_z) = (leg.y, leg.z);
        }
        if let Some(leg) = self.root.child_mut("left_front_leg") {
            leg.y = leg_y;
            leg.z = leg_z;
            leg.x_rot -= stand * pi * 0.45;
        }
        if let Some(body) = self.root.child_mut("body") {
            body.x_rot -= stand * pi * 0.35;
            body.y += stand * age * 2.0;
        }
        if let Some(head) = self.root.child_mut("head") {
            head.y -= stand * 24.0;
            head.z += stand * 13.0;
            head.x_rot += stand * pi * 0.15;
        }
    }

    /// `PandaModel.setupAnim` (and `BabyPandaModel`'s sitting): the
    /// quadruped's pose, then the unhappy shake, the sneeze, sitting (and
    /// eating or fright), lying on the back and the roll.
    pub fn setup_panda(&mut self, state: &PandaState) {
        self.setup_quadruped(&state.quadruped);
        let pi = std::f64::consts::PI as f32;
        let age = state.age_in_ticks;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        if state.unhappy {
            set(self, "head", &|head| {
                head.y_rot = 0.35 * sin(0.6 * age);
                head.z_rot = 0.35 * sin(0.6 * age);
            });
            set(self, "right_front_leg", &|leg| {
                leg.x_rot = -0.75 * sin(0.3 * age);
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = 0.75 * sin(0.3 * age);
            });
        } else {
            set(self, "head", &|head| head.z_rot = 0.0);
        }
        if state.sneezing {
            let time = state.sneeze_time;
            if time < 15 {
                set(self, "head", &|head| {
                    head.x_rot = -pi / 4.0 * time as f32 / 14.0;
                });
            } else if time < 20 {
                // The reference divides these as ints.
                let position = ((time - 15) / 5) as f32;
                set(self, "head", &|head| {
                    head.x_rot = -pi / 4.0 + pi / 4.0 * position;
                });
            }
        }
        if state.sit_amount > 0.0 {
            self.panda_sit(state);
            if state.eating {
                set(self, "head", &|head| {
                    head.x_rot = pi / 2.0 + 0.2 * sin(age * 0.6);
                });
                for name in ["right_front_leg", "left_front_leg"] {
                    set(self, name, &|leg| leg.x_rot = -0.4 - 0.2 * sin(age * 0.6));
                }
            }
            if state.scared {
                set(self, "head", &|head| head.x_rot = 2.170_796_4);
                for name in ["right_front_leg", "left_front_leg"] {
                    set(self, name, &|leg| leg.x_rot = -0.9);
                }
            }
        } else {
            for name in LEGS {
                set(self, name, &|leg| leg.z_rot = 0.0);
            }
        }
        if state.lie_on_back_amount > 0.0 {
            let amount = state.lie_on_back_amount;
            set(self, "right_hind_leg", &|leg| {
                leg.x_rot = -0.6 * sin(age * 0.15);
            });
            set(self, "left_hind_leg", &|leg| {
                leg.x_rot = 0.6 * sin(age * 0.15);
            });
            set(self, "right_front_leg", &|leg| {
                leg.x_rot = 0.3 * sin(age * 0.25);
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = -0.3 * sin(age * 0.25);
            });
            set(self, "head", &|head| {
                head.x_rot = rot_lerp_rad(amount, head.x_rot, pi / 2.0);
            });
        }
        if state.roll_amount > 0.0 {
            let amount = state.roll_amount;
            set(self, "head", &|head| {
                head.x_rot = rot_lerp_rad(amount, head.x_rot, 2.056_194_5);
            });
            set(self, "right_hind_leg", &|leg| {
                leg.x_rot = -0.5 * sin(age * 0.5);
            });
            set(self, "left_hind_leg", &|leg| {
                leg.x_rot = 0.5 * sin(age * 0.5);
            });
            set(self, "right_front_leg", &|leg| {
                leg.x_rot = 0.5 * sin(age * 0.5);
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = -0.5 * sin(age * 0.5);
            });
        }
    }

    /// `PandaModel.animateSitting` and the baby's own.
    pub(super) fn panda_sit(&mut self, state: &PandaState) {
        let pi = std::f64::consts::PI as f32;
        let amount = state.sit_amount;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        if state.baby {
            set(self, "body", &|body| {
                body.x_rot = rot_lerp_rad(amount, body.x_rot, (std::f64::consts::PI / 18.0) as f32);
                body.z = lerp_f32(amount, body.z, -1.5);
            });
            set(self, "head", &|head| {
                head.z = lerp_f32(amount, head.z, -11.5);
                head.y = lerp_f32(amount, head.y, 17.5);
            });
            for (name, z) in [
                ("right_front_leg", -5.0),
                ("left_front_leg", -5.0),
                ("right_hind_leg", 3.0),
                ("left_hind_leg", 3.0),
            ] {
                set(self, name, &|leg| leg.z = lerp_f32(amount, leg.z, z));
            }
        } else {
            set(self, "body", &|body| {
                body.x_rot = rot_lerp_rad(amount, body.x_rot, 1.740_796_3);
            });
            set(self, "head", &|head| {
                head.x_rot = rot_lerp_rad(amount, head.x_rot, pi / 2.0);
            });
        }
        set(self, "right_front_leg", &|leg| leg.z_rot = -0.270_796_42);
        set(self, "left_front_leg", &|leg| leg.z_rot = 0.270_796_42);
        set(self, "right_hind_leg", &|leg| leg.z_rot = 0.570_796_4);
        set(self, "left_hind_leg", &|leg| leg.z_rot = -0.570_796_4);
    }
}
