use super::*;

impl MobModel {
    /// `GuardianModel.setupAnim`: the head takes the look, the spikes breathe
    /// out and in, the eye follows what the guardian looks at, and the tail
    /// sways, more toward its end.
    pub fn setup_guardian(&mut self, state: &GuardianState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let Some(head) = self.root.child_mut("head") else {
            return;
        };
        head.y_rot = state.head_y_rot * degrees;
        head.x_rot = state.x_rot * degrees;
        let withdrawal = (1.0 - state.spikes_animation) * 0.55;
        for (index, (x, y, z)) in SPIKE_X
            .iter()
            .zip(SPIKE_Y.iter())
            .zip(SPIKE_Z.iter())
            .map(|((x, y), z)| (*x, *y, *z))
            .enumerate()
        {
            // `getSpikeOffset`.
            let offset = 1.0
                + mth_cos_f64(f64::from(state.age_in_ticks * 1.5 + index as f32)) * 0.01
                - withdrawal;
            if let Some(spike) = head.child_mut(&format!("spike{index}")) {
                spike.x = x * offset;
                spike.y = 16.0 + y * offset;
                spike.z = z * offset;
            }
        }
        if let Some(eye) = head.child_mut("eye") {
            if let Some(look) = state.look {
                eye.y = if look.look_at_position[1] - look.eye_position[1] > 0.0 {
                    0.0
                } else {
                    1.0
                };
                // The way the eye is turned from, a quarter turn about the vertical.
                let (dx, dz) = (
                    look.eye_position[0] - look.look_at_position[0],
                    look.eye_position[2] - look.look_at_position[2],
                );
                let length = (dx * dx + dz * dz).sqrt();
                let (nx, nz) = if length < 1.0E-5 {
                    (0.0, 0.0)
                } else {
                    (dx / length, dz / length)
                };
                let angle = (std::f64::consts::PI / 2.0) as f32;
                let (cos, sin) = (
                    f64::from(mth_cos_f64(f64::from(angle))),
                    f64::from(mth_sin_f64(f64::from(angle))),
                );
                let (turned_x, turned_z) = (nx * cos + nz * sin, nz * cos - nx * sin);
                let dot = look.look_direction[0] * turned_x + look.look_direction[2] * turned_z;
                let signum = if dot > 0.0 {
                    1.0_f32
                } else if dot < 0.0 {
                    -1.0
                } else {
                    0.0
                };
                eye.x = (dot.abs() as f32).sqrt() * 2.0 * signum;
            }
            eye.visible = true;
        }
        let sway = mth_sin_f64(f64::from(state.tail_animation));
        if let Some(tail0) = head.child_mut("tail0") {
            tail0.y_rot = sway * pi * 0.05;
            if let Some(tail1) = tail0.child_mut("tail1") {
                tail1.y_rot = sway * pi * 0.1;
                if let Some(tail2) = tail1.child_mut("tail2") {
                    tail2.y_rot = sway * pi * 0.15;
                }
            }
        }
    }

    /// `NautilusModel.setupAnim` (and `ZombieNautilusCoralModel`'s corals):
    /// the body follows the look within ten degrees, and the swim animation
    /// runs by the stride and the age, so it never rests.
    pub fn setup_nautilus(&mut self, state: &NautilusState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(body) = self
            .root
            .child_mut("root")
            .and_then(|root| root.child_mut("body"))
        {
            body.y_rot = state.quadruped.head_y_rot.clamp(-10.0, 10.0) * degrees;
            body.x_rot = state.quadruped.x_rot.clamp(-10.0, 10.0) * degrees;
        }
        if let Some(definition) = voxel_data::animation::definition("Nautilus.SWIMMING") {
            // `applyWalk(pos + age / 5, speed + 0.2, 2.0, 3.0)`.
            let position = state.quadruped.walk_position + state.age_in_ticks / 5.0;
            let millis = (position * 50.0 * 2.0) as i64;
            let scale = ((state.quadruped.walk_speed + 0.2) * 3.0).min(1.0);
            self.animate(definition, millis, scale);
        }
        if let Some(corals) = self
            .root
            .child_mut("root")
            .and_then(|root| root.child_mut("shell"))
            .and_then(|shell| shell.child_mut("corals"))
        {
            corals.visible = !state.has_body_armor;
        }
    }

    /// `TadpoleModel.setupAnim`: the tail swings, wider out of water.
    pub fn setup_tadpole(&mut self, state: &TadpoleState) {
        self.root.reset();
        let amplitude = if state.is_in_water { 1.0_f32 } else { 1.5 };
        if let Some(tail) = self.root.child_mut("tail") {
            tail.y_rot = -amplitude * 0.25 * mth_sin_f64(f64::from(0.3 * state.age_in_ticks));
        }
    }

    /// `DolphinModel.setupAnim`: the body takes the turn and pitch; while it
    /// moves it undulates, the tail and its fin following.
    pub fn setup_dolphin(&mut self, state: &DolphinState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let wave = mth_cos_f64(f64::from(state.age_in_ticks * 0.3));
        let Some(body) = self.root.child_mut("body") else {
            return;
        };
        body.x_rot = state.x_rot * degrees;
        body.y_rot = state.y_rot * degrees;
        if state.is_moving {
            body.x_rot += -0.05 - 0.05 * wave;
            if let Some(tail) = body.child_mut("tail") {
                tail.x_rot = -0.1 * wave;
                if let Some(fin) = tail.child_mut("tail_fin") {
                    fin.x_rot = -0.2 * wave;
                }
            }
        }
    }

    /// `TurtleModel.setupAnim` (and `AdultTurtleModel`'s belly): the walk of
    /// the quadruped, then the flippers rowing on land (faster when laying)
    /// or paddling in water; the egg shows, and lowers the model a sixteenth.
    pub fn setup_turtle(&mut self, state: &TurtleState) {
        self.setup_quadruped(&state.quadruped);
        let (position, speed) = (state.quadruped.walk_position, state.quadruped.walk_speed);
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let mut set = |name: &str, apply: &dyn Fn(&mut Part)| {
            if let Some(part) = self.root.child_mut(name) {
                apply(part);
            }
        };
        if state.is_on_land {
            let lay_egg = if state.is_laying_egg { 4.0_f32 } else { 1.0 };
            let amplitude = if state.is_laying_egg { 2.0_f32 } else { 1.0 };
            let swing_pos = position * 5.0;
            let front = cos(lay_egg * swing_pos);
            let hind = cos(swing_pos);
            set("right_front_leg", &|leg| {
                leg.y_rot = -front * 8.0 * speed * amplitude;
            });
            set("left_front_leg", &|leg| {
                leg.y_rot = front * 8.0 * speed * amplitude;
            });
            set("right_hind_leg", &|leg| leg.y_rot = -hind * 3.0 * speed);
            set("left_hind_leg", &|leg| leg.y_rot = hind * 3.0 * speed);
        } else {
            let swing = cos(position * 0.6662 * 0.6) * (0.5 * speed);
            set("right_hind_leg", &|leg| leg.x_rot = swing);
            set("left_hind_leg", &|leg| leg.x_rot = -swing);
            set("right_front_leg", &|leg| leg.z_rot = -swing);
            set("left_front_leg", &|leg| leg.z_rot = swing);
        }
        if let Some(belly) = self.root.child_mut("egg_belly") {
            belly.visible = state.has_egg;
            if state.has_egg {
                self.root.y -= 1.0;
            }
        }
    }

    /// `CodModel.setupAnim` and `TropicalFish*Model.setupAnim`: the tail (named
    /// `tail_fin` on a cod, `tail` on a tropical fish) swings, wider out of water.
    pub fn setup_tail_fish(&mut self, tail: &str, age_in_ticks: f32, in_water: bool) {
        self.root.reset();
        let amplitude = if in_water { 1.0_f32 } else { 1.5_f32 };
        if let Some(tail) = self.root.child_mut(tail) {
            tail.y_rot = -amplitude * 0.45 * mth_sin_f64(f64::from(0.6 * age_in_ticks));
        }
    }

    /// `SalmonModel.setupAnim`: the back half of the body swings, harder and
    /// faster out of water.
    pub fn setup_salmon(&mut self, age_in_ticks: f32, in_water: bool) {
        self.root.reset();
        let (amplitude, angle) = if in_water {
            (1.0_f32, 1.0_f32)
        } else {
            (1.3, 1.7)
        };
        if let Some(back) = self.root.child_mut("body_back") {
            back.y_rot = -amplitude * 0.25 * mth_sin_f64(f64::from(angle * 0.6 * age_in_ticks));
        }
    }

    /// `Pufferfish*Model.setupAnim`: the two fins (named `right_fin` and
    /// `left_fin` on a small one, `right_blue_fin` and `left_blue_fin` on the
    /// others) beat against each other.
    pub fn setup_pufferfish(&mut self, fins: [&str; 2], age_in_ticks: f32) {
        self.root.reset();
        let beat = mth_sin_f64(f64::from(age_in_ticks * 0.2));
        if let Some(right) = self.root.child_mut(fins[0]) {
            right.z_rot = -0.2 + 0.4 * beat;
        }
        if let Some(left) = self.root.child_mut(fins[1]) {
            left.z_rot = 0.2 - 0.4 * beat;
        }
    }

    /// `SilverfishModel.setupAnim` and `EndermiteModel.setupAnim`: each
    /// segment sways about y and slides sideways in a wave that travels down
    /// the body (the silverfish's three layers follow the segments under them).
    pub fn setup_segmented(&mut self, age_in_ticks: f32, silverfish: bool) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let (segments, turn, slide) = if silverfish {
            (7_i32, 0.05_f32, 0.2_f32)
        } else {
            (4, 0.01, 0.1)
        };
        for index in 0..segments {
            let phase = age_in_ticks * 0.9 + index as f32 * 0.15 * pi;
            let distance = (index - 2).abs();
            let y_rot = mth_cos_f64(f64::from(phase)) * pi * turn * (1 + distance) as f32;
            let x = mth_sin_f64(f64::from(phase)) * pi * slide * distance as f32;
            if let Some(segment) = self.root.child_mut(&format!("segment{index}")) {
                segment.y_rot = y_rot;
                segment.x = x;
            }
        }
        if !silverfish {
            return;
        }
        let pose = |root: &mut Part, index: usize| {
            root.child_mut(&format!("segment{index}"))
                .map(|segment| (segment.y_rot, segment.x))
        };
        for (layer, segment, slides) in [(0, 2, false), (1, 4, true), (2, 1, true)] {
            let Some((y_rot, x)) = pose(&mut self.root, segment) else {
                continue;
            };
            if let Some(layer) = self.root.child_mut(&format!("layer{layer}")) {
                layer.y_rot = y_rot;
                if slides {
                    layer.x = x;
                }
            }
        }
    }

    /// `BeeModel.setupAnim` (and the adult's antennas): the stinger while it
    /// has one; in the air the wings beat and the legs hang; a bee that is not
    /// angry bobs; a rolling one turns over.
    pub fn setup_bee(&mut self, state: &BeeState) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let sin_cos = |value: f32| mth_cos_f64(f64::from(value));
        let age = state.age_in_ticks;
        let Some(bone) = self.root.child_mut("bone") else {
            return;
        };
        if let Some(stinger) = bone
            .child_mut("body")
            .and_then(|body| body.child_mut("stinger"))
        {
            stinger.visible = state.has_stinger;
        }
        if !state.on_ground {
            let speed = age * 120.321_13 * (std::f64::consts::PI / 180.0) as f32;
            let flap = sin_cos(speed) * pi * 0.15;
            let right_x = bone.child_mut("right_wing").map_or(0.0, |wing| {
                wing.y_rot = 0.0;
                wing.z_rot = flap;
                wing.x_rot
            });
            if let Some(wing) = bone.child_mut("left_wing") {
                wing.x_rot = right_x;
                wing.y_rot = 0.0;
                wing.z_rot = -flap;
            }
            for name in ["front_legs", "middle_legs", "back_legs"] {
                if let Some(leg) = bone.child_mut(name) {
                    leg.x_rot = (std::f64::consts::PI / 4.0) as f32;
                }
            }
        }
        if !state.angry && !state.on_ground {
            let speed = sin_cos(age * 0.18);
            bone.x_rot = 0.1 + speed * pi * 0.025;
            bone.y -= sin_cos(age * 0.18) * 0.9;
            if let Some(leg) = bone.child_mut("front_legs") {
                leg.x_rot = -speed * pi * 0.1 + (std::f64::consts::PI / 8.0) as f32;
            }
            if let Some(leg) = bone.child_mut("back_legs") {
                leg.x_rot = -speed * pi * 0.05 + (std::f64::consts::PI / 4.0) as f32;
            }
            if !state.baby
                && let Some(body) = bone.child_mut("body")
            {
                for name in ["left_antenna", "right_antenna"] {
                    if let Some(antenna) = body.child_mut(name) {
                        antenna.x_rot = speed * pi * 0.03;
                    }
                }
            }
        }
        if state.roll_amount > 0.0 {
            bone.x_rot = rot_lerp_rad(state.roll_amount, bone.x_rot, 3.091_592_8);
        }
    }
}
