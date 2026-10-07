use super::*;

impl MobModel {
    /// `AbstractEquineModel.setupAnim` (and the baby and donkey subclasses'
    /// overrides): the head eases between its grazing, standing and ordinary
    /// poses, the legs stride or paw the air, the tail lifts with the walk and
    /// swishes; a donkey shows its chests.
    #[expect(
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    pub fn setup_equine(&mut self, state: &EquineState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let lerp = |alpha: f32, from: f32, to: f32| from + alpha * (to - from);
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let kind = state.kind;
        let nested = kind == EquineKind::BabyDonkey;
        // The overrides of each kind.
        let leg_stand_angle = if nested { pi / 3.0 } else { pi / 12.0 };
        let leg_standing_y = match kind {
            EquineKind::BabyHorse => 4.0,
            EquineKind::BabyDonkey => 1.0,
            _ => 12.0,
        };
        let leg_standing_z = match kind {
            EquineKind::BabyHorse => 0.0,
            EquineKind::BabyDonkey => 0.5,
            _ => 4.0,
        };
        let leg_standing_x_rot = if nested { 0.0 } else { -pi / 3.0 };
        let tail_x_rot_offset = match kind {
            EquineKind::BabyHorse => -pi / 2.0,
            EquineKind::BabyDonkey => -pi / 4.0,
            _ => 0.0,
        };
        let look = &state.quadruped;
        let clamped_y_rot = look.head_y_rot.clamp(-20.0, 20.0);
        let mut head_rot_x = look.x_rot * degrees;
        let (speed, position) = (look.walk_speed, look.walk_position);
        if speed > 0.2 {
            head_rot_x += cos(position * 0.8) * 0.15 * speed;
        }
        let eating = state.eat_animation;
        let standing = state.stand_animation;
        let not_standing = 1.0 - standing;
        let feeding = state.feeding_animation;
        let age = state.age_in_ticks;
        let root = &mut self.root;
        let water = if state.in_water { 0.2 } else { 1.0 };
        let leg_anim = cos(water * position * 0.6662 + pi);
        let leg_x_rot_anim = leg_anim * 0.8 * speed;
        let base_head_angle =
            (1.0 - standing.max(eating)) * (pi / 6.0 + head_rot_x + feeding * sin(age) * 0.05);
        // The head parts: first turned, then eased between the poses.
        let head = equine_part(root, nested, "head_parts");
        head.x_rot = pi / 6.0 + head_rot_x;
        head.y_rot = clamped_y_rot * degrees;
        head.x_rot = standing * (pi / 12.0 + head_rot_x)
            + eating * (2.181_661_6 + sin(age) * 0.05)
            + base_head_angle;
        head.y_rot = standing * clamped_y_rot * degrees + (1.0 - standing.max(eating)) * head.y_rot;
        // `animateHeadPartsPlacement`.
        match kind {
            EquineKind::BabyHorse => {
                head.y += lerp(eating, lerp(standing, 0.0, -2.0), 2.0);
                head.z = lerp(standing, head.z, -4.0);
            }
            EquineKind::BabyDonkey => {
                head.y = lerp(eating, head.y, -1.2);
                head.z = lerp(standing, head.z, -3.6);
            }
            _ => {
                head.y += lerp(eating, lerp(standing, 0.0, -8.0), 7.0);
                head.z = lerp(standing, head.z, -4.0);
            }
        }
        let body = root.child_mut("body").expect("an equine has a body");
        body.x_rot = standing * (-pi / 4.0) + not_standing * body.x_rot;
        let left_front = equine_part(root, nested, "left_front_leg");
        left_front.y -= leg_standing_y * standing;
        left_front.z += leg_standing_z * standing;
        let (front_y, front_z) = (left_front.y, left_front.z);
        let right_front = equine_part(root, nested, "right_front_leg");
        right_front.y = front_y;
        right_front.z = front_z;
        let stand_angle = leg_stand_angle * standing;
        let bob = cos(age * 0.6 + pi);
        let right_leg_rot = (leg_standing_x_rot + bob) * standing + leg_x_rot_anim * not_standing;
        let left_leg_rot = (leg_standing_x_rot - bob) * standing - leg_x_rot_anim * not_standing;
        equine_part(root, nested, "left_hind_leg").x_rot =
            stand_angle - leg_anim * 0.5 * speed * not_standing;
        equine_part(root, nested, "right_hind_leg").x_rot =
            stand_angle + leg_anim * 0.5 * speed * not_standing;
        equine_part(root, nested, "left_front_leg").x_rot = right_leg_rot;
        equine_part(root, nested, "right_front_leg").x_rot = left_leg_rot;
        // `offsetLegPositionWhenStanding`: the baby donkey's hind legs drop a little.
        if nested {
            let left = equine_part(root, nested, "left_hind_leg");
            left.y = lerp(standing, left.y, -0.3);
            let left_y = left.y;
            equine_part(root, nested, "right_hind_leg").y = lerp(standing, left_y, -0.3);
        }
        let tail = root
            .child_mut("body")
            .and_then(|body| body.child_mut("tail"))
            .expect("an equine has a tail");
        tail.x_rot = tail_x_rot_offset + pi / 6.0 + speed * 0.75;
        tail.y += speed * state.age_scale;
        tail.z += speed * 2.0 * state.age_scale;
        tail.y_rot = if state.animate_tail {
            cos(age * 0.7)
        } else {
            0.0
        };
        if nested {
            // `BabyDonkeyModel.setupAnim`: the head tipped down at a fixed pitch.
            let head_rot_x = -30.0 * degrees;
            let base_head_angle =
                (1.0 - standing.max(eating)) * (pi / 6.0 + head_rot_x + feeding * sin(age) * 0.05);
            equine_part(root, nested, "head_parts").x_rot = standing * (pi / 12.0 + head_rot_x)
                + eating * (pi / 2.0 + sin(age) * 0.05)
                + base_head_angle;
        }
        // `DonkeyModel.setupAnim`: the chests show when the animal carries them.
        if matches!(kind, EquineKind::Donkey | EquineKind::BabyDonkey) {
            for name in ["left_chest", "right_chest"] {
                if let Some(chest) = root.child_mut("body").and_then(|body| body.child_mut(name)) {
                    chest.visible = state.has_chest;
                }
            }
        }
    }

    /// `CamelModel.setupAnim`: the head follows the look within limits (and
    /// dips while the dash cools down), then the walk and each animation state
    /// that has started.
    pub fn setup_camel(&mut self, state: &CamelState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let y_rot = state.quadruped.head_y_rot.clamp(-30.0, 30.0);
        let mut x_rot = state.quadruped.x_rot.clamp(-25.0, 45.0);
        if state.jump_cooldown > 0.0 {
            let head_rotation = 45.0 * state.jump_cooldown / 55.0;
            x_rot = (x_rot + head_rotation).clamp(-25.0, 70.0);
        }
        if let Some(head) = self
            .root
            .child_mut("body")
            .and_then(|body| body.child_mut("head"))
        {
            head.y_rot = y_rot * degrees;
            head.x_rot = x_rot * degrees;
        }
        let prefix = if state.baby {
            "CamelBaby.CAMEL_BABY"
        } else {
            "Camel.CAMEL"
        };
        let walk = voxel_data::animation::definition(&format!("{prefix}_WALK"));
        if let Some(definition) = walk {
            // `applyWalk(pos, speed, 2.0, 2.5)`.
            let millis = (state.quadruped.walk_position * 50.0 * 2.0) as i64;
            let scale = (state.quadruped.walk_speed * 2.5).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            ("SIT", state.sit),
            ("SIT_POSE", state.sit_pose),
            ("STANDUP", state.sit_up),
            ("IDLE", state.idle),
            ("DASH", state.dash),
        ] {
            if !running.is_started() {
                continue;
            }
            if let Some(definition) = voxel_data::animation::definition(&format!("{prefix}_{name}"))
            {
                let millis = (running.time_in_millis(state.age_in_ticks) as f32 * 1.0) as i64;
                self.animate(definition, millis, 1.0);
            }
        }
    }

    /// `LlamaModel.setupAnim` (and the baby's): the head follows the look, the
    /// legs swing in diagonal pairs, and the chests show on a loaded llama.
    pub fn setup_llama(&mut self, state: &QuadrupedState, has_chest: bool) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = state.head_y_rot * degrees;
        }
        let (position, speed) = (state.walk_position, state.walk_speed);
        for (name, x_rot) in [
            ("right_hind_leg", cos(position * 0.6662) * 1.4 * speed),
            ("left_hind_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("right_front_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("left_front_leg", cos(position * 0.6662) * 1.4 * speed),
        ] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
        for name in ["right_chest", "left_chest"] {
            if let Some(chest) = self.root.child_mut(name) {
                chest.visible = has_chest;
            }
        }
    }

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
}
