use super::*;

impl MobModel {
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

    /// `StriderModel.setupAnim` (and `AdultStriderModel`'s and
    /// `BabyStriderModel`'s own part): the body turns with the look unless it
    /// is ridden and rocks with the walk, the legs swing, and the bristles
    /// flow and stir.
    pub fn setup_strider(&mut self, state: &StriderState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let position = state.quadruped.walk_position;
        let speed = state.quadruped.walk_speed.min(0.25);
        let age = state.age_in_ticks;
        let set = |model: &mut Self, name: &str, change: &dyn Fn(&mut Part)| {
            if let Some(part) = model.root.child_mut(name) {
                change(part);
            }
        };
        set(self, "body", &|body| {
            if state.ridden {
                body.x_rot = 0.0;
                body.y_rot = 0.0;
            } else {
                body.x_rot = state.quadruped.x_rot * degrees;
                body.y_rot = state.quadruped.head_y_rot * degrees;
            }
            body.z_rot = 0.1 * sin(position * 1.5) * 4.0 * speed;
        });
        let half = position * 1.5 * 0.5;
        set(self, "left_leg", &|leg| {
            leg.x_rot = sin(half) * 2.0 * speed;
            leg.z_rot = (std::f64::consts::PI / 18.0) as f32 * cos(half) * speed;
        });
        set(self, "right_leg", &|leg| {
            leg.x_rot = sin(half + pi) * 2.0 * speed;
            leg.z_rot = (std::f64::consts::PI / 18.0) as f32 * cos(half + pi) * speed;
        });
        let flow = cos(position * 1.5 + pi) * speed;
        let leg_swing = |sign: f32| 2.0 * sin(half + sign) * 2.0 * speed;
        if state.baby {
            set(self, "body", &|body| {
                body.y = 17.25;
                body.y -= 1.0 * cos(position * 1.5) * 2.0 * speed;
                for (name, scale, stir) in [
                    ("bristle2", 0.6, 0.1 * sin(age * 0.4)),
                    ("bristle1", 1.2, 0.1 * sin(age * 0.2)),
                    ("bristle0", 1.3, 0.05 * sin(age * -0.4)),
                ] {
                    if let Some(bristle) = body.child_mut(name) {
                        bristle.x_rot += flow * scale;
                        bristle.x_rot += stir;
                    }
                }
            });
            set(self, "left_leg", &|leg| leg.y = 20.0 + leg_swing(pi));
            set(self, "right_leg", &|leg| leg.y = 20.0 + leg_swing(0.0));
        } else {
            set(self, "body", &|body| {
                body.y = 2.0;
                body.y -= 2.0 * cos(position * 1.5) * 2.0 * speed;
                // The sides' bristles rest turned, then take the flow and the stirs.
                for (name, rest, scale, stir) in [
                    (
                        "right_bottom_bristle",
                        -1.221_730_5,
                        1.3,
                        0.05 * sin(age * -0.4),
                    ),
                    (
                        "right_middle_bristle",
                        -1.134_464,
                        1.2,
                        0.1 * sin(age * 0.2),
                    ),
                    (
                        "right_top_bristle",
                        -0.872_664_63,
                        0.6,
                        0.1 * sin(age * 0.4),
                    ),
                    ("left_top_bristle", 0.872_664_63, 0.6, 0.1 * sin(age * 0.4)),
                    ("left_middle_bristle", 1.134_464, 1.2, 0.1 * sin(age * 0.2)),
                    (
                        "left_bottom_bristle",
                        1.221_730_5,
                        1.3,
                        0.05 * sin(age * -0.4),
                    ),
                ] {
                    if let Some(bristle) = body.child_mut(name) {
                        bristle.z_rot = rest;
                        bristle.z_rot += flow * scale;
                        bristle.z_rot += stir;
                    }
                }
            });
            set(self, "left_leg", &|leg| leg.y = 8.0 + leg_swing(pi));
            set(self, "right_leg", &|leg| leg.y = 8.0 + leg_swing(0.0));
        }
    }

    /// `RavagerModel.setupAnim`: the neck thrusts and the mouth snaps in the
    /// attack; otherwise the neck rests, sways while stunned, and the mouth
    /// opens in the roar; the head follows the look and the legs swing.
    pub fn setup_ravager(&mut self, state: &RavagerState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (stunned, attack) = (state.stunned_ticks_remaining, state.attack_ticks_remaining);
        let Some(neck_x_rot) = self.root.child_mut("neck").map(|neck| neck.x_rot) else {
            return;
        };
        let mut neck: (f32, f32, f32);
        let mut neck_x = None;
        let mouth_x_rot;
        if attack > 0.0 {
            // `Mth.triangleWave(attackTick, 10)`.
            let wave = ((attack % 10.0 - 10.0 * 0.5).abs() - 10.0 * 0.25) / (10.0 * 0.25);
            let scaled = (1.0 + wave) * 0.5;
            let head_pos = scaled * scaled * scaled * 12.0;
            let y_offset = head_pos * sin(neck_x_rot);
            neck = (0.0, -7.0 - y_offset, -6.5 + head_pos);
            mouth_x_rot = if attack > 5.0 {
                sin((-4.0 + attack) / 4.0) * pi * 0.4
            } else {
                (std::f64::consts::PI / 20.0) as f32 * sin(pi * attack / 10.0)
            };
        } else {
            let y_offset = -sin(neck_x_rot);
            neck = (0.0, -7.0 - y_offset, 5.5);
            let is_stunned = stunned > 0.0;
            neck_x = Some(if is_stunned { 0.219_911_49 } else { 0.0 });
            let mut mouth = pi * if is_stunned { 0.05 } else { 0.01 };
            if is_stunned {
                let speed = f64::from(stunned) / 40.0;
                neck.0 = (speed * 10.0).sin() as f32 * 3.0;
            } else if state.roar_animation > 0.0 {
                let mouth_anim = sin(state.roar_animation * pi * 0.25);
                mouth = (std::f64::consts::PI / 2.0) as f32 * mouth_anim;
            }
            mouth_x_rot = mouth;
        }
        if let Some(part) = self.root.child_mut("neck") {
            (part.x, part.y, part.z) = neck;
            if let Some(x_rot) = neck_x {
                part.x_rot = x_rot;
            }
            if let Some(head) = part.child_mut("head") {
                head.x_rot = state.quadruped.x_rot * degrees;
                head.y_rot = state.quadruped.head_y_rot * degrees;
                if let Some(mouth) = head.child_mut("mouth") {
                    mouth.x_rot = mouth_x_rot;
                }
            }
        }
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let leg_rot = 0.4 * speed;
        for (name, offset) in [
            ("right_hind_leg", 0.0),
            ("left_hind_leg", pi),
            ("right_front_leg", pi),
            ("left_front_leg", 0.0),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = cos(position * 0.6662 + offset) * leg_rot;
            }
        }
    }

    /// `HoglinModel.setupAnim` (and `BabyHoglinModel`'s): the ears flap with
    /// the walk, the head turns and butts by the attack swing (a baby's
    /// lowers with it), and the legs swing in diagonal pairs.
    pub fn setup_hoglin(&mut self, state: &HoglinState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let (position, speed) = (state.walk_position, state.walk_speed);
        let ear_swing = speed * mth_sin_f64(f64::from(position));
        let ear_rest = (std::f64::consts::PI * 2.0 / 9.0) as f32;
        let butt = 1.0 - ((10 - 2 * state.attack_ticks).abs() as f32) / 10.0;
        if let Some(head) = self.root.child_mut("head") {
            if let Some(ear) = head.child_mut("right_ear") {
                ear.z_rot = -ear_rest - ear_swing;
            }
            if let Some(ear) = head.child_mut("left_ear") {
                ear.z_rot = ear_rest + ear_swing;
            }
            head.y_rot = state.head_y_rot * degrees;
            head.x_rot = lerp_f32(butt, 0.872_664_63, (-std::f64::consts::PI / 9.0) as f32);
            if state.baby {
                head.y += butt * 2.5;
            }
        }
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let right_front = cos(position) * 1.2 * speed;
        let left_front = cos(position + pi) * 1.2 * speed;
        for (name, x_rot) in [
            ("right_front_leg", right_front),
            ("left_front_leg", left_front),
            ("right_hind_leg", left_front),
            ("left_hind_leg", right_front),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
    }

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

    /// `GoatModel.setupAnim` (and `BabyGoatModel`'s): the quadruped's pose, the
    /// horns the goat still has, and the head lowered to ram; a baby's head
    /// tips forward when it is not ramming.
    pub fn setup_goat(&mut self, state: &GoatState) {
        self.setup_quadruped(&state.quadruped);
        if let Some(head) = self.root.child_mut("head") {
            for (name, visible) in [
                ("left_horn", state.has_left_horn),
                ("right_horn", state.has_right_horn),
            ] {
                if let Some(horn) = head.child_mut(name) {
                    horn.visible = visible;
                }
            }
            if state.ramming_x_head_rot != 0.0 {
                head.x_rot = state.ramming_x_head_rot;
            } else if state.baby {
                head.x_rot = (std::f64::consts::PI / 8.0) as f32;
            }
        }
    }

    /// `SheepModel.setupAnim` (and the wool's): the quadruped's pose, with
    /// the head lowered and tipped to eat.
    pub fn setup_sheep(&mut self, state: &SheepState) {
        self.setup_quadruped(&state.quadruped);
        if let Some(head) = self.root.child_mut("head") {
            head.y += state.head_eat_position_scale * 9.0 * state.age_scale;
            head.x_rot = state.head_eat_angle_scale;
        }
    }

    /// `ChickenModel.setupAnim`: the legs swing with the walk, the wings beat
    /// with the flap, and a grown chicken's head follows the look.
    pub fn setup_chicken(&mut self, state: &ChickenState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let flap_angle = (sin(state.flap) + 1.0) * state.flap_speed;
        let (position, speed) = (state.walk_position, state.walk_speed);
        let moves = [
            (
                "right_leg",
                Some(cos(position * 0.6662) * 1.4 * speed),
                None,
            ),
            (
                "left_leg",
                Some(cos(position * 0.6662 + pi) * 1.4 * speed),
                None,
            ),
            ("right_wing", None, Some(flap_angle)),
            ("left_wing", None, Some(-flap_angle)),
        ];
        for (name, x_rot, z_rot) in moves {
            if let Some(part) = self.root.child_mut(name) {
                if let Some(x_rot) = x_rot {
                    part.x_rot = x_rot;
                }
                if let Some(z_rot) = z_rot {
                    part.z_rot = z_rot;
                }
            }
        }
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = state.head_y_rot * degrees;
        }
    }

    /// `ZombieModel.setupAnim` (and `DrownedModel.setupAnim`): the humanoid
    /// pose, then the arms held out in front.
    pub fn setup_zombie(&mut self, state: &ZombieState) {
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        humanoid_anim(&mut self.root, &state.humanoid);
        self.animate_zombie_arms(&state.humanoid, state.aggressive);
        let humanoid = &state.humanoid;
        let swim = humanoid.swim_amount;
        if state.drowned && swim > 0.0 {
            let age = humanoid.age_in_ticks;
            let sway = swim * 0.35 * sin(0.1 * age);
            let kick = swim * 0.55 * sin(0.1 * age);
            let reach = (-std::f64::consts::PI * 4.0 / 5.0) as f32;
            if let Some(arm) = self.root.child_mut("right_arm") {
                arm.x_rot = rot_lerp_rad(swim, arm.x_rot, reach) + sway;
                arm.z_rot = rot_lerp_rad(swim, arm.z_rot, -0.15);
            }
            if let Some(arm) = self.root.child_mut("left_arm") {
                arm.x_rot = rot_lerp_rad(swim, arm.x_rot, reach) - sway;
                arm.z_rot = rot_lerp_rad(swim, arm.z_rot, 0.15);
            }
            if let Some(leg) = self.root.child_mut("left_leg") {
                leg.x_rot -= kick;
            }
            if let Some(leg) = self.root.child_mut("right_leg") {
                leg.x_rot += kick;
            }
            if let Some(head) = self.root.child_mut("head") {
                head.x_rot = 0.0;
            }
        }
    }

    /// `AnimationUtils.animateZombieArms`: unless the swing is a stab, `animateAttackArms` over a
    /// drop that depends on the flag (a baby holding something keeps its arms down), then the arms'
    /// breathing again.
    pub(super) fn animate_zombie_arms(&mut self, humanoid: &PlayerRenderState, aggressive: bool) {
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let pi = std::f64::consts::PI as f32;
        if humanoid.swing_animation != SwingAnimation::Stab {
            let raise_arms = !humanoid.is_baby || humanoid.main_hand_empty;
            let arm_drop = if raise_arms {
                -pi / if aggressive { 1.5 } else { 2.25 }
            } else {
                0.0
            };
            let attack_time = humanoid.attack_time;
            let y_modifier = (if raise_arms { 1.0 } else { -1.0 }) * sin(attack_time * pi);
            let x_modifier = sin((1.0 - (1.0 - attack_time) * (1.0 - attack_time)) * pi);
            let x_rot = arm_drop + y_modifier * 1.2 - x_modifier * 0.4;
            let y_rot = 0.1 - y_modifier * 0.6;
            for (name, y) in [
                ("right_arm", if raise_arms { -y_rot } else { y_rot }),
                ("left_arm", if raise_arms { y_rot } else { -y_rot }),
            ] {
                if let Some(arm) = self.root.child_mut(name) {
                    arm.x_rot = x_rot;
                    arm.y_rot = y;
                    arm.z_rot = 0.0;
                }
            }
        }
        for (name, bob) in [("right_arm", 1.0_f32), ("left_arm", -1.0)] {
            if let Some(arm) = self.root.child_mut(name) {
                // `bobArms`.
                arm.z_rot += bob * (cos(humanoid.age_in_ticks * 0.09) * 0.05 + 0.05);
                arm.x_rot += bob * (sin(humanoid.age_in_ticks * 0.067) * 0.05);
            }
        }
    }
}
