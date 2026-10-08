use super::*;

impl MobModel {
    /// `WolfModel.setupAnim` (grown and baby): the tail and legs by the walk
    /// (the legs folded while it sits), the body rolled by the shake, then
    /// the head follows the look and the tail rises.
    pub fn setup_wolf(&mut self, state: &WolfState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let scale = state.age_scale;
        let leg_swing = |offset: f32| cos(position * 0.6662 + offset) * 1.4 * speed;
        let tail_y_rot = if state.angry { 0.0 } else { leg_swing(0.0) };
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        set(self, "tail", &|tail| tail.y_rot = tail_y_rot);
        if state.sitting {
            set(self, "body", &|part| {
                part.y += 4.0 * scale;
                part.z -= 2.0 * scale;
                part.x_rot = (std::f64::consts::PI / 4.0) as f32;
                if state.baby {
                    part.x_rot -= (std::f64::consts::PI / 2.0) as f32;
                }
            });
            set(self, "tail", &|part| {
                part.y += 9.0 * scale;
                part.z -= 2.0 * scale;
            });
            for name in ["right_hind_leg", "left_hind_leg"] {
                set(self, name, &|part| {
                    part.y += 6.7 * scale;
                    part.z -= 5.0 * scale;
                    part.x_rot = (std::f64::consts::PI * 3.0 / 2.0) as f32;
                });
            }
            for (name, side) in [("right_front_leg", 1.0), ("left_front_leg", -1.0)] {
                set(self, name, &|part| {
                    part.x_rot = 5.811_947;
                    part.x += side * 0.01 * scale;
                    part.y += 1.0 * scale;
                });
            }
            if !state.baby {
                set(self, "upper_body", &|part| {
                    part.y += 2.0;
                    part.x_rot = (std::f64::consts::PI * 2.0 / 5.0) as f32;
                    part.y_rot = 0.0;
                });
            }
        } else {
            for (name, offset) in [
                ("right_hind_leg", 0.0),
                ("left_hind_leg", pi),
                ("right_front_leg", pi),
                ("left_front_leg", 0.0),
            ] {
                set(self, name, &|part| part.x_rot = leg_swing(offset));
            }
        }
        // `shakeOffWater`: the body rolls; the grown wolf's head, upper body
        // and tail roll in their own parts, the baby's whole head and tail.
        set(self, "body", &|part| {
            part.z_rot = state.body_roll_angle(-0.16)
        });
        if state.baby {
            set(self, "head", &|part| {
                part.z_rot = state.head_roll_angle + state.body_roll_angle(0.0);
            });
            set(self, "tail", &|part| {
                part.z_rot = state.body_roll_angle(-0.2)
            });
        } else {
            set(self, "head", &|head| {
                if let Some(real) = head.child_mut("real_head") {
                    real.z_rot = state.head_roll_angle + state.body_roll_angle(0.0);
                }
            });
            set(self, "upper_body", &|part| {
                part.z_rot = state.body_roll_angle(-0.08)
            });
            set(self, "tail", &|tail| {
                if let Some(real) = tail.child_mut("real_tail") {
                    real.z_rot = state.body_roll_angle(-0.2);
                }
            });
        }
        set(self, "head", &|head| {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        });
        set(self, "tail", &|tail| tail.x_rot = state.tail_angle);
    }

    /// `FoxModel.setSittingPose` and the model's own: the head faces front
    /// and the body tips back onto the haunches.
    pub(super) fn fox_sit(&mut self, state: &FoxState) {
        let pi = std::f64::consts::PI;
        let scale = state.age_scale;
        let front = (-pi / 12.0) as f32;
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = 0.0;
            head.y_rot = 0.0;
            if state.baby {
                head.y -= 0.75;
            } else {
                head.y -= 6.5;
                head.z += 2.75;
            }
        }
        if let Some(body) = self.root.child_mut("body") {
            if state.baby {
                body.x_rot = -0.959_931;
                body.z -= 4.5 * scale;
                body.y += 3.0 * scale;
            } else {
                body.x_rot = (pi / 6.0) as f32;
                body.y -= 7.0;
                body.z += 3.0;
            }
            if let Some(tail) = body.child_mut("tail") {
                if state.baby {
                    tail.y -= 0.6;
                    tail.z -= 2.0 * scale;
                    tail.x_rot = 0.959_931_14;
                } else {
                    tail.x_rot = (pi / 4.0) as f32;
                    tail.z -= 1.0;
                }
            }
        }
        for (name, side) in [("right_front_leg", 1.0), ("left_front_leg", -1.0)] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = front;
                if state.baby {
                    leg.z -= 1.5;
                    leg.x += side * 0.01;
                }
            }
        }
        for (name, side) in [("right_hind_leg", 1.0), ("left_hind_leg", -1.0)] {
            if let Some(leg) = self.root.child_mut(name) {
                if state.baby {
                    leg.z -= 3.75;
                    leg.x += side * 0.01;
                } else {
                    leg.x_rot = (-pi * 5.0 / 12.0) as f32;
                    leg.y += 4.0;
                    leg.z -= 0.25;
                }
            }
        }
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
            if let Some(definition) = photon_data::animation::definition("FoxBaby.FOX_BABY_WALK") {
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

/// What a wolf's models are posed from (`WolfRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WolfState {
    pub quadruped: QuadrupedState,
    pub angry: bool,
    pub sitting: bool,
    pub baby: bool,
    /// How far the tail is raised, and how far the head rolls to a side.
    pub tail_angle: f32,
    pub head_roll_angle: f32,
    /// The shake's progress, which rolls the body.
    pub shake_anim: f32,
    /// `LivingEntityRenderState.ageScale`: what the sitting pose is scaled by.
    pub age_scale: f32,
}

/// What a fox's models are posed from (`FoxRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FoxState {
    pub quadruped: QuadrupedState,
    /// How far its head rolls to a side while it is interested, and how far
    /// it has crouched (zero to five).
    pub head_roll_angle: f32,
    pub crouch_amount: f32,
    pub crouching: bool,
    pub sleeping: bool,
    pub sitting: bool,
    pub faceplanted: bool,
    pub pouncing: bool,
    pub age_in_ticks: f32,
    /// `LivingEntityRenderState.ageScale`: what a baby's sitting shifts are scaled by.
    pub age_scale: f32,
    pub baby: bool,
}

impl WolfState {
    /// `WolfRenderState.getBodyRollAngle`.
    pub(super) fn body_roll_angle(&self, offset: f32) -> f32 {
        let pi = std::f64::consts::PI as f32;
        let progress = ((self.shake_anim + offset) / 1.8).clamp(0.0, 1.0);
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        sin(progress * pi) * sin(progress * pi * 11.0) * 0.15 * pi
    }
}
