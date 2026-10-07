use super::*;

impl MobModel {
    /// `AdultFelineModel.setupAnim` and `BabyFelineModel.setupAnim`: crouching
    /// or sprinting lifts the tail and lowers the body, the legs swing with
    /// the walk (not while it sits), and lying down and relaxing turn the
    /// head and fold the legs.
    pub fn setup_feline(&mut self, state: &FelineState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let half_pi = (std::f64::consts::PI / 2.0) as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let scale = state.age_scale;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        if state.crouching {
            set(self, "body", &|part| part.y += scale);
            set(self, "head", &|part| part.y += 2.0 * scale);
            set(self, "tail1", &|part| {
                part.y += scale;
                part.x_rot = half_pi;
            });
            set(self, "tail2", &|part| {
                part.y += -4.0 * scale;
                part.z += 2.0 * scale;
                part.x_rot = half_pi;
            });
        } else if state.sprinting {
            let tail1_y = self.root.child_mut("tail1").map_or(0.0, |part| part.y);
            set(self, "tail1", &|part| part.x_rot = half_pi);
            set(self, "tail2", &|part| {
                part.y = tail1_y;
                part.z += 2.0 * scale;
                part.x_rot = half_pi;
            });
        }
        set(self, "head", &|head| {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        });
        if !state.sitting {
            if !state.baby {
                set(self, "body", &|part| part.x_rot = half_pi);
            }
            let swing = |offset: f32| cos(position * 0.6662 + offset) * speed;
            if state.sprinting {
                set(self, "left_hind_leg", &|part| part.x_rot = swing(0.0));
                set(self, "right_hind_leg", &|part| part.x_rot = swing(0.3));
                set(self, "left_front_leg", &|part| {
                    part.x_rot = cos(position * 0.6662 + pi + 0.3) * speed;
                });
                set(self, "right_front_leg", &|part| part.x_rot = swing(pi));
                set(self, "tail2", &|part| {
                    part.x_rot =
                        1.727_876_1 + (std::f64::consts::PI / 10.0) as f32 * cos(position) * speed;
                });
            } else {
                set(self, "left_hind_leg", &|part| part.x_rot = swing(0.0));
                set(self, "right_hind_leg", &|part| part.x_rot = swing(pi));
                set(self, "left_front_leg", &|part| part.x_rot = swing(pi));
                set(self, "right_front_leg", &|part| part.x_rot = swing(0.0));
                let sway = if state.crouching {
                    0.471_238_94
                } else {
                    (std::f64::consts::PI / 4.0) as f32
                };
                set(self, "tail2", &|part| {
                    part.x_rot = 1.727_876_1 + sway * cos(position) * speed;
                });
            }
        } else {
            self.setup_feline_sitting(state);
        }
        self.setup_feline_lying(state);
        if state.relax_state_one_amount > 0.0 {
            set(self, "head", &|part| {
                part.x_rot = rot_lerp(state.relax_state_one_amount, part.x_rot, -0.581_776_44);
            });
        }
    }

    /// The sitting part of `setup_feline`: the baby's small shifts, or the
    /// grown cat's folded pose.
    pub(super) fn setup_feline_sitting(&mut self, state: &FelineState) {
        let scale = state.age_scale;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        if state.baby {
            set(self, "body", &|part| {
                part.x_rot += -0.436_332_32;
                part.y += 1.25;
            });
            set(self, "head", &|part| part.z += 0.75);
            set(self, "tail1", &|part| {
                part.x_rot += 0.545_415_4;
                part.y += 4.0;
                part.z -= 0.9;
            });
            for name in ["left_hind_leg", "right_hind_leg"] {
                set(self, name, &|part| part.z -= 0.9);
            }
        } else {
            set(self, "body", &|part| {
                part.x_rot = (std::f64::consts::PI / 4.0) as f32;
                part.y += -4.0 * scale;
                part.z += 5.0 * scale;
            });
            set(self, "head", &|part| {
                part.y += -3.3 * scale;
                part.z += scale;
            });
            set(self, "tail1", &|part| {
                part.y += 8.0 * scale;
                part.z += -2.0 * scale;
                part.x_rot = 1.727_876_1;
            });
            set(self, "tail2", &|part| {
                part.y += 2.0 * scale;
                part.z += -0.8 * scale;
                part.x_rot = 2.670_354;
            });
            for name in ["left_front_leg", "right_front_leg"] {
                set(self, name, &|part| {
                    part.x_rot = (-std::f64::consts::PI / 20.0) as f32;
                    part.y += 2.0 * scale;
                    part.z -= 2.0 * scale;
                });
            }
            for name in ["left_hind_leg", "right_hind_leg"] {
                set(self, name, &|part| {
                    part.x_rot = (-std::f64::consts::PI / 2.0) as f32;
                    part.y += 3.0 * scale;
                    part.z -= 4.0 * scale;
                });
            }
        }
    }

    /// The lying-down part of `setup_feline`: the baby's small shifts, or the
    /// grown cat's turned head and folded legs and tail.
    pub(super) fn setup_feline_lying(&mut self, state: &FelineState) {
        let half_pi = (std::f64::consts::PI / 2.0) as f32;
        let scale = state.age_scale;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        let (lie, tail) = (state.lie_down_amount, state.lie_down_amount_tail);
        if lie > 0.0 && state.baby {
            set(self, "body", &|part| part.x += 1.0);
            set(self, "head", &|part| {
                part.x_rot = rot_lerp(lie, part.x_rot, (std::f64::consts::PI / 18.0) as f32);
                part.z_rot = rot_lerp(lie, part.z_rot, (-std::f64::consts::PI * 5.0 / 12.0) as f32);
                part.x += 1.5;
                part.y += 0.75;
                part.z -= 0.5;
            });
            set(self, "right_front_leg", &|part| {
                part.x_rot = (-std::f64::consts::PI / 4.0) as f32;
                part.x += 3.5;
                part.y -= 0.5;
            });
            set(self, "left_front_leg", &|part| {
                part.x_rot = -half_pi;
                part.x += 1.5;
                part.y -= 1.0;
                part.z -= 2.0;
            });
            set(self, "right_hind_leg", &|part| {
                part.x_rot = (std::f64::consts::PI * 2.0 / 9.0) as f32;
                part.y_rot = (std::f64::consts::PI / 9.0) as f32;
                part.z_rot = (-std::f64::consts::PI / 9.0) as f32;
                part.x += 2.5;
                part.y -= 0.25;
                part.z += 0.5;
            });
            set(self, "left_hind_leg", &|part| {
                part.x += 1.5;
                part.z -= 1.0;
            });
            set(self, "tail1", &|part| {
                part.x_rot += rot_lerp(tail, part.x_rot, (-std::f64::consts::PI / 6.0) as f32);
                part.y_rot += rot_lerp(tail, part.y_rot, 0.0);
                part.z_rot += rot_lerp(tail, part.z_rot, (-std::f64::consts::PI / 18.0) as f32);
                part.x += 1.0;
                part.y += 0.5;
                part.z -= 0.25;
            });
        } else if lie > 0.0 {
            set(self, "head", &|part| {
                part.z_rot = rot_lerp(lie, part.z_rot, -1.270_796_3);
                part.y_rot = rot_lerp(lie, part.y_rot, 1.270_796_3);
            });
            set(self, "left_front_leg", &|part| part.x_rot = -1.270_796_3);
            set(self, "right_front_leg", &|part| {
                part.x_rot = -0.470_796_35;
                part.z_rot = -0.2;
                part.x += scale;
            });
            set(self, "left_hind_leg", &|part| part.x_rot = -0.4);
            set(self, "right_hind_leg", &|part| {
                part.x_rot = 0.5;
                part.z_rot = -0.5;
                part.x += 0.8 * scale;
                part.y += 2.0 * scale;
            });
            set(self, "tail1", &|part| {
                part.x_rot = rot_lerp(tail, part.x_rot, 0.8);
            });
            set(self, "tail2", &|part| {
                part.x_rot = rot_lerp(tail, part.x_rot, -0.4);
            });
        }
    }
}

/// What a cat's or an ocelot's models are posed from (`FelineRenderState`,
/// `CatRenderState`): an ocelot never sits, lies down or relaxes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FelineState {
    pub quadruped: QuadrupedState,
    pub crouching: bool,
    pub sprinting: bool,
    pub sitting: bool,
    pub baby: bool,
    /// How far it has lain down, its tail's own progress, and how far it has
    /// relaxed (its head lifted).
    pub lie_down_amount: f32,
    pub lie_down_amount_tail: f32,
    pub relax_state_one_amount: f32,
    /// `LivingEntityRenderState.ageScale`: what the pose's shifts are scaled by.
    pub age_scale: f32,
}

pub(super) const LEGS: [&str; 4] = [
    "right_hind_leg",
    "left_hind_leg",
    "right_front_leg",
    "left_front_leg",
];
