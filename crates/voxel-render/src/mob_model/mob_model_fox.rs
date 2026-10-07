use super::*;

impl MobModel {
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
                leg.x_rot = -0.75 * sin(0.3 * age)
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = 0.75 * sin(0.3 * age)
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
                leg.x_rot = -0.6 * sin(age * 0.15)
            });
            set(self, "left_hind_leg", &|leg| {
                leg.x_rot = 0.6 * sin(age * 0.15)
            });
            set(self, "right_front_leg", &|leg| {
                leg.x_rot = 0.3 * sin(age * 0.25)
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = -0.3 * sin(age * 0.25)
            });
            set(self, "head", &|head| {
                head.x_rot = rot_lerp_rad(amount, head.x_rot, pi / 2.0)
            });
        }
        if state.roll_amount > 0.0 {
            let amount = state.roll_amount;
            set(self, "head", &|head| {
                head.x_rot = rot_lerp_rad(amount, head.x_rot, 2.056_194_5)
            });
            set(self, "right_hind_leg", &|leg| {
                leg.x_rot = -0.5 * sin(age * 0.5)
            });
            set(self, "left_hind_leg", &|leg| {
                leg.x_rot = 0.5 * sin(age * 0.5)
            });
            set(self, "right_front_leg", &|leg| {
                leg.x_rot = 0.5 * sin(age * 0.5)
            });
            set(self, "left_front_leg", &|leg| {
                leg.x_rot = -0.5 * sin(age * 0.5)
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
                head.x_rot = rot_lerp_rad(amount, head.x_rot, pi / 2.0)
            });
        }
        set(self, "right_front_leg", &|leg| leg.z_rot = -0.270_796_42);
        set(self, "left_front_leg", &|leg| leg.z_rot = 0.270_796_42);
        set(self, "right_hind_leg", &|leg| leg.z_rot = 0.570_796_4);
        set(self, "left_hind_leg", &|leg| leg.z_rot = -0.570_796_4);
    }

    /// `FoxModel.setupAnim` (and `AdultFoxModel`'s and `BabyFoxModel`'s): the
    /// walk, then the crouching, sleeping or sitting pose, the pounce, the
    /// head's look (not while it sleeps, crouches or has its face planted)
    /// and the legs of a faceplanted fox.
    pub fn setup_fox(&mut self, state: &FoxState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        // `setWalkingPose`.
        set(self, "head", &|head| head.z_rot = state.head_roll_angle);
        for name in LEGS {
            set(self, name, &|leg| leg.visible = true);
        }
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        if state.baby {
            if let Some(definition) = voxel_data::animation::definition("FoxBaby.FOX_BABY_WALK") {
                // `applyWalk(pos, speed, 1.0, 2.5)`.
                self.animate(definition, (position * 50.0) as i64, (speed * 2.5).min(1.0));
            }
        } else {
            for (name, offset) in [
                ("right_hind_leg", 0.0),
                ("left_hind_leg", pi),
                ("right_front_leg", pi),
                ("left_front_leg", 0.0),
            ] {
                set(self, name, &|leg| {
                    leg.x_rot = cos(position * 0.6662 + offset) * 1.4 * speed;
                });
            }
        }
        if state.crouching {
            self.fox_crouch(state);
        } else if state.sleeping {
            self.fox_sleep(state);
        } else if state.sitting {
            self.fox_sit(state);
        }
        if state.pouncing && !state.baby {
            let crouch = state.crouch_amount / 2.0;
            set(self, "body", &|body| body.y -= crouch);
            set(self, "head", &|head| head.y -= crouch);
        }
        if !state.sleeping && !state.faceplanted && !state.crouching {
            set(self, "head", &|head| {
                head.x_rot = state.quadruped.x_rot * degrees;
                head.y_rot = state.quadruped.head_y_rot * degrees;
            });
        }
        if state.sleeping {
            set(self, "head", &|head| {
                head.x_rot = 0.0;
                head.y_rot = (-std::f64::consts::PI * 2.0 / 3.0) as f32;
                head.z_rot = cos(state.age_in_ticks * 0.027) / 22.0;
            });
        }
        if state.faceplanted {
            self.leg_motion_pos += 0.67;
            let phase = self.leg_motion_pos * 0.4662;
            for (name, offset) in [
                ("right_hind_leg", 0.0),
                ("left_hind_leg", pi),
                ("right_front_leg", pi),
                ("left_front_leg", 0.0),
            ] {
                set(self, name, &|leg| leg.x_rot = cos(phase + offset) * 0.1);
            }
        }
    }

    /// `FoxModel.setCrouchingPose` and the model's own: the body sinks and
    /// the tail and legs wiggle.
    pub(super) fn fox_crouch(&mut self, state: &FoxState) {
        let wiggle = mth_cos_f64(f64::from(state.age_in_ticks)) * 0.05;
        let (crouch, scale) = (state.crouch_amount, state.age_scale);
        let sink = if state.baby { crouch / 6.0 } else { crouch };
        if let Some(body) = self.root.child_mut("body") {
            body.x_rot += 0.104_719_76;
            body.y_rot = wiggle;
            body.y += sink;
        }
        if let Some(head) = self.root.child_mut("head") {
            head.y += crouch * scale;
        }
        for (name, roll) in [
            ("right_hind_leg", wiggle),
            ("left_hind_leg", wiggle),
            ("right_front_leg", wiggle / 2.0),
            ("left_front_leg", wiggle / 2.0),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.z_rot = roll;
            }
        }
    }

    /// `FoxModel.setSleepingPose` and the model's own: the legs are hidden
    /// and the fox curls up on its side.
    pub(super) fn fox_sleep(&mut self, state: &FoxState) {
        let pi = std::f64::consts::PI;
        for name in LEGS {
            if let Some(leg) = self.root.child_mut(name) {
                leg.visible = false;
            }
        }
        let baby = state.baby;
        if let Some(body) = self.root.child_mut("body") {
            body.z_rot = (-pi / 2.0) as f32;
            if baby {
                body.x_rot = (-pi / 18.0) as f32;
                body.y += 1.5;
                body.z -= 1.5;
                body.x -= 1.5;
            } else {
                body.y += 5.0;
            }
            if let Some(tail) = body.child_mut("tail") {
                if baby {
                    tail.x_rot = -2.181_661_6;
                    tail.x -= 0.7;
                    tail.z += 0.6;
                    tail.y += 0.9;
                } else {
                    tail.x_rot = (-pi * 5.0 / 6.0) as f32;
                }
            }
        }
        if let Some(head) = self.root.child_mut("head") {
            if baby {
                head.x -= 2.0;
                head.y += 2.8;
                head.z -= 4.0;
            } else {
                head.x += 2.0;
                head.y += 2.99;
            }
            head.y_rot = (-pi * 2.0 / 3.0) as f32;
            head.z_rot = 0.0;
        }
    }
}
