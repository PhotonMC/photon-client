use super::*;

impl MobModel {
    /// `EvokerFangsModel.setupAnim`: the jaws close as the bite goes on, the fangs sink and
    /// then shrink away at its end.
    pub fn setup_evoker_fangs(&mut self, bite_progress: f32) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let mut bite_amount = (bite_progress * 2.0).min(1.0);
        bite_amount = 1.0 - bite_amount * bite_amount * bite_amount;
        if let Some(base) = self.root.child_mut("base") {
            if let Some(jaw) = base.child_mut("upper_jaw") {
                jaw.z_rot = pi - bite_amount * 0.35 * pi;
            }
            if let Some(jaw) = base.child_mut("lower_jaw") {
                jaw.z_rot = pi + bite_amount * 0.35 * pi;
            }
            base.y -= (bite_progress + mth_sin_f64(f64::from(bite_progress * 2.7))) * 7.2;
        }
        let mut pre_scale = 1.0_f32;
        if bite_progress > 0.9 {
            pre_scale *= (1.0 - bite_progress) / 0.1;
        }
        self.root.y = 24.0 - 20.0 * pre_scale;
        self.root.scale = [pre_scale; 3];
    }

    /// `TridentModel` has no animation: its parts stay where the mesh puts them.
    pub fn setup_trident(&mut self) {
        self.root.reset();
    }

    /// `ArrowModel.setupAnim`: a stuck arrow shakes about its length for a few ticks.
    pub fn setup_arrow(&mut self, state: &ArrowRenderState) {
        self.root.reset();
        if state.shake > 0.0 {
            let pow = -mth_sin_f64(f64::from(state.shake * 3.0)) * state.shake;
            self.root.z_rot += pow * (std::f64::consts::PI / 180.0) as f32;
        }
    }

    /// `ParrotModel.prepare`: the pose's own resting turns.
    pub(super) fn parrot_prepare(&mut self, pose: ParrotPose) {
        let mut adjust = |name: &str, apply: &dyn Fn(&mut Part)| {
            if let Some(part) = self.root.child_mut(name) {
                apply(part);
            }
        };
        match pose {
            ParrotPose::Flying => {
                let tuck = (std::f64::consts::PI * 2.0 / 9.0) as f32;
                adjust("left_leg", &|leg| leg.x_rot += tuck);
                adjust("right_leg", &|leg| leg.x_rot += tuck);
            }
            ParrotPose::Sitting => {
                const LIFT: f32 = 1.9;
                let half = (std::f64::consts::PI / 2.0) as f32;
                adjust("head", &|part| part.y += LIFT);
                adjust("tail", &|part| {
                    part.x_rot += (std::f64::consts::PI / 6.0) as f32;
                    part.y += LIFT;
                });
                adjust("body", &|part| part.y += LIFT);
                adjust("left_wing", &|part| {
                    part.z_rot = -0.0873;
                    part.y += LIFT;
                });
                adjust("right_wing", &|part| {
                    part.z_rot = 0.0873;
                    part.y += LIFT;
                });
                adjust("left_leg", &|leg| {
                    leg.y += LIFT;
                    leg.x_rot += half;
                });
                adjust("right_leg", &|leg| {
                    leg.y += LIFT;
                    leg.x_rot += half;
                });
            }
            ParrotPose::Party => {
                adjust("left_leg", &|leg| {
                    leg.z_rot = (-std::f64::consts::PI / 9.0) as f32
                });
                adjust("right_leg", &|leg| {
                    leg.z_rot = (std::f64::consts::PI / 9.0) as f32
                });
            }
            ParrotPose::Standing | ParrotPose::OnShoulder => {}
        }
    }

    /// `BatModel.setupAnim`: a hanging bat's head takes the turn; then the
    /// flying and the hanging animations, each if started.
    pub fn setup_bat(&mut self, state: &BatRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if state.is_resting
            && let Some(head) = self.root.child_mut("head")
        {
            head.y_rot = state.head_y_rot * degrees;
        }
        self.animate_state("Bat.BAT_FLYING", state.fly, state.age_in_ticks);
        self.animate_state("Bat.BAT_RESTING", state.rest, state.age_in_ticks);
    }

    /// `VexModel.setupAnim`: the head takes the look, the arms bob, the body
    /// leans (unless charging, when the arms reach out, higher with an item),
    /// and the wings beat.
    pub fn setup_vex(&mut self, state: &VexRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI;
        let bob = mth_cos_f64(f64::from(state.age_in_ticks * 5.5 * degrees)) * 0.1;
        let wing_beat =
            mth_cos_f64(f64::from(state.age_in_ticks * 45.836_624 * degrees)) * degrees * 16.2;
        let Some(root) = self.root.child_mut("root") else {
            return;
        };
        if let Some(head) = root.child_mut("head") {
            head.y_rot = state.head_y_rot * degrees;
            head.x_rot = state.x_rot * degrees;
        }
        let Some(body) = root.child_mut("body") else {
            return;
        };
        let fifth = (pi / 5.0) as f32;
        let twelfth = (pi / 12.0) as f32;
        if let Some(arm) = body.child_mut("right_arm") {
            arm.z_rot = fifth + bob;
        }
        if let Some(arm) = body.child_mut("left_arm") {
            arm.z_rot = -(fifth + bob);
        }
        if state.is_charging {
            body.x_rot = 0.0;
            let reach = (pi * 7.0 / 6.0) as f32;
            let (right, left) = (state.right_hand_item, state.left_hand_item);
            // `setArmsCharging`: with nothing in either hand both arms reach
            // forward; otherwise only the arm that holds something is raised.
            let empty_handed = !right && !left;
            if (right || empty_handed)
                && let Some(arm) = body.child_mut("right_arm")
            {
                arm.x_rot = if empty_handed { -1.221_730_5 } else { reach };
                arm.y_rot = twelfth;
                arm.z_rot = -0.471_238_88 - bob;
            }
            if (left || empty_handed)
                && let Some(arm) = body.child_mut("left_arm")
            {
                arm.x_rot = if empty_handed { -1.221_730_5 } else { reach };
                arm.y_rot = -twelfth;
                arm.z_rot = 0.471_238_88 + bob;
            }
        } else {
            body.x_rot = (pi / 20.0) as f32;
        }
        let left_wing_y_rot = 1.099_557_4 + wing_beat;
        for (side, sign) in [("left_wing", 1.0_f32), ("right_wing", -1.0)] {
            if let Some(wing) = body.child_mut(side) {
                wing.y_rot = sign * left_wing_y_rot;
                wing.x_rot = 0.471_238_88;
                wing.z_rot = sign * -0.471_238_88;
            }
        }
    }

    /// `AllayModel.setupAnim`: the wings flap with the walk and the age, the
    /// body tips as the allay flies, the arms rise to what it holds, and it
    /// bobs while hovering or dances (spinning, then swaying).
    pub fn setup_allay(&mut self, state: &AllayRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let (animation_speed, animation_pos) =
            (state.quadruped.walk_speed, state.quadruped.walk_position);
        let age = state.age_in_ticks;
        let flap_speed = age * 20.0 * degrees + animation_pos;
        let flap_amount = cos(flap_speed) * pi * 0.15 + animation_speed;
        let idle_bob_speed = age * 9.0 * degrees;
        let flying_factor = (animation_speed / 0.3).min(1.0);
        let idle_bob_factor = 1.0 - flying_factor;
        let holding = state.holding_animation_progress;
        let Some(root) = self.root.child_mut("root") else {
            return;
        };
        if state.is_dancing {
            let dance_speed = age * 8.0 * degrees + animation_speed;
            let dance_frequency = cos(dance_speed) * 16.0 * degrees;
            let spinning = state.spinning_progress;
            let head_tilt_z = cos(dance_speed) * 14.0 * degrees;
            let head_tilt_y = cos(dance_speed) * 30.0 * degrees;
            if state.is_spinning {
                root.y_rot = (std::f64::consts::PI * 4.0) as f32 * spinning;
            }
            root.z_rot = dance_frequency * (1.0 - spinning);
            if let Some(head) = root.child_mut("head") {
                head.y_rot = head_tilt_y * (1.0 - spinning);
                head.z_rot = head_tilt_z * (1.0 - spinning);
            }
        } else if let Some(head) = root.child_mut("head") {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        }
        root.y += (f64::from(idle_bob_speed).cos() as f32) * 0.25 * idle_bob_factor;
        let Some(body) = root.child_mut("body") else {
            return;
        };
        let wing_tilt = 0.436_332_32 * (1.0 - flying_factor);
        if let Some(wing) = body.child_mut("right_wing") {
            wing.x_rot = wing_tilt;
            wing.y_rot = (-std::f64::consts::PI / 4.0) as f32 + flap_amount;
        }
        if let Some(wing) = body.child_mut("left_wing") {
            wing.x_rot = wing_tilt;
            wing.y_rot = (std::f64::consts::PI / 4.0) as f32 - flap_amount;
        }
        body.x_rot = flying_factor * (std::f64::consts::PI / 4.0) as f32;
        let arm_flying_rot_x = holding
            * lerp_f32(
                flying_factor,
                (-std::f64::consts::PI / 3.0) as f32,
                -1.134_464,
            );
        let arm_idle_bob_factor = idle_bob_factor * (1.0 - holding);
        let arm_idle_bob_amount = 0.436_332_32
            - cos(idle_bob_speed + (std::f64::consts::PI * 3.0 / 2.0) as f32)
                * pi
                * 0.075
                * arm_idle_bob_factor;
        if let Some(arm) = body.child_mut("right_arm") {
            arm.x_rot = arm_flying_rot_x;
            arm.z_rot = arm_idle_bob_amount;
            arm.y_rot = 0.279_252_68 * holding;
        }
        if let Some(arm) = body.child_mut("left_arm") {
            arm.x_rot = arm_flying_rot_x;
            arm.z_rot = -arm_idle_bob_amount;
            arm.y_rot = -0.279_252_68 * holding;
        }
    }

    /// `BlazeModel.setupAnim`: three rings of rods turn about the head, each
    /// bobbing, and the head takes the look.
    pub fn setup_blaze(&mut self, age_in_ticks: f32, head_y_rot: f32, x_rot: f32) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let quarter_turn = (std::f64::consts::PI / 2.0) as f32;
        // Each ring: its first rod's angle, and per rod the bob's phase and
        // the rod's height and radius.
        let rings: [(f32, std::ops::Range<usize>, f32, f32); 3] = [
            (age_in_ticks * pi * -0.1, 0..4, -2.0, 9.0),
            (
                (std::f64::consts::PI / 4.0) as f32 + age_in_ticks * pi * 0.03,
                4..8,
                2.0,
                7.0,
            ),
            (0.471_238_94 + age_in_ticks * pi * -0.05, 8..12, 11.0, 5.0),
        ];
        for (start, indices, height, radius) in rings {
            let mut angle = start;
            for i in indices {
                let bob = if i < 8 {
                    cos((i as f32 * 2.0 + age_in_ticks) * 0.25)
                } else {
                    cos((i as f32 * 1.5 + age_in_ticks) * 0.5)
                };
                if let Some(rod) = self.root.child_mut(&format!("part{i}")) {
                    rod.y = height + bob;
                    rod.x = cos(angle) * radius;
                    rod.z = sin(angle) * radius;
                }
                angle += quarter_turn;
            }
        }
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = head_y_rot * degrees;
            head.x_rot = x_rot * degrees;
        }
    }

    /// `ChestModel.setupAnim`: the lid, and the lock on it, tip back by how open the (eased) lid is.
    pub fn setup_chest(&mut self, open: f32) {
        self.root.reset();
        let angle = -(open * (std::f64::consts::PI / 2.0) as f32);
        for name in ["lid", "lock"] {
            if let Some(part) = self.root.child_mut(name) {
                part.x_rot = angle;
            }
        }
    }

    /// `BannerFlagModel.setupAnim`: the flag tips by the phase of its wave.
    pub fn setup_banner_flag(&mut self, phase: f32) {
        self.root.reset();
        if let Some(flag) = self.root.child_mut("flag") {
            let pi = std::f64::consts::PI as f32;
            let wave = mth_cos_f64(f64::from(2.0 * std::f64::consts::PI as f32 * phase));
            flag.x_rot = (-0.0125 + 0.01 * wave) * pi;
        }
    }

    /// `BellModel.setupAnim`: the bell swings away from the side it was struck from, less each
    /// tick.
    pub fn setup_bell(&mut self, ticks: f32, shake: Option<voxel_core::direction::Direction>) {
        use voxel_core::direction::Direction;
        self.root.reset();
        let (mut x_rot, mut z_rot) = (0.0_f32, 0.0_f32);
        if let Some(direction) = shake {
            let base = voxel_core::math::mth_sin_f64(f64::from(ticks / std::f32::consts::PI))
                / (4.0 + ticks / 3.0);
            match direction {
                Direction::North => x_rot = -base,
                Direction::South => x_rot = base,
                Direction::East => z_rot = -base,
                Direction::West => z_rot = base,
                Direction::Down | Direction::Up => {}
            }
        }
        if let Some(body) = self.root.child_mut("bell_body") {
            body.x_rot = x_rot;
            body.z_rot = z_rot;
        }
    }

    /// `ShulkerBoxModel.setupAnim`: the lid rises half a block and turns three quarters of a
    /// circle as the box opens.
    pub fn setup_shulker_box(&mut self, progress: f32) {
        self.root.reset();
        if let Some(lid) = self.root.child_mut("lid") {
            lid.x = 0.0;
            lid.y = 24.0 - progress * 0.5 * 16.0;
            lid.z = 0.0;
            lid.y_rot = 270.0 * progress * (std::f64::consts::PI / 180.0) as f32;
        }
    }

    /// `AbstractBoatModel.setupAnim`: each paddle swings by the time it has
    /// rowed (the right one mirrored).
    pub fn setup_boat(&mut self, state: &crate::boat::BoatRenderState) {
        self.root.reset();
        use std::f64::consts::PI;
        for (name, time, side) in [
            ("left_paddle", state.rowing_time_left, 0),
            ("right_paddle", state.rowing_time_right, 1),
        ] {
            let Some(paddle) = self.root.child_mut(name) else {
                continue;
            };
            paddle.x_rot = voxel_core::math::clamped_lerp_f32(
                (mth_sin_f64(f64::from(-time)) + 1.0) / 2.0,
                (-PI / 3.0) as f32,
                (-PI / 12.0) as f32,
            );
            paddle.y_rot = voxel_core::math::clamped_lerp_f32(
                (mth_sin_f64(f64::from(-time + 1.0)) + 1.0) / 2.0,
                (-PI / 4.0) as f32,
                (PI / 4.0) as f32,
            );
            if side == 1 {
                paddle.y_rot = PI as f32 - paddle.y_rot;
            }
        }
    }

    /// `HappyGhastModel.setupAnim`: a body that wears something is squeezed to
    /// fifteen sixteenths, and the nine tentacles (children of the body) sway
    /// as a ghast's.
    pub fn setup_happy_ghast(&mut self, age_in_ticks: f32, wears_body_item: bool) {
        self.root.reset();
        let Some(body) = self.root.child_mut("body") else {
            return;
        };
        if wears_body_item {
            body.scale = [0.9375; 3];
        }
        for i in 0..9 {
            if let Some(tentacle) = body.child_mut(&format!("tentacle{i}")) {
                tentacle.x_rot = 0.2 * mth_sin_f64(f64::from(age_in_ticks * 0.3 + i as f32)) + 0.4;
            }
        }
    }

    /// `HappyGhastHarnessModel.setupAnim`: the goggles sit on the brow, and are
    /// pulled down over the eyes while the ghast is ridden.
    // The reference writes the tilt as the literal `-0.7854F`, not as a quarter turn.
    #[allow(clippy::approx_constant)]
    pub fn setup_happy_ghast_harness(&mut self, is_ridden: bool) {
        self.root.reset();
        if let Some(goggles) = self.root.child_mut("goggles") {
            if is_ridden {
                goggles.x_rot = 0.0;
                goggles.y = 14.0;
            } else {
                goggles.x_rot = -0.7854;
                goggles.y = 9.0;
            }
        }
    }

    /// `GhastModel.setupAnim`: the nine tentacles sway out of step.
    pub fn setup_ghast(&mut self, age_in_ticks: f32) {
        self.root.reset();
        for i in 0..9 {
            if let Some(tentacle) = self.root.child_mut(&format!("tentacle{i}")) {
                tentacle.x_rot = 0.2 * mth_sin_f64(f64::from(age_in_ticks * 0.3 + i as f32)) + 0.4;
            }
        }
    }

    /// `PhantomModel.setupAnim`: the wings beat together (the right ones
    /// against the left) and the tail follows at twice the pace.
    pub fn setup_phantom(&mut self, flap_time: f32) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let anim = flap_time * 7.448_451 * degrees;
        let wing = mth_cos_f64(f64::from(anim)) * 16.0 * degrees;
        let tail = -(5.0 + mth_cos_f64(f64::from(anim * 2.0)) * 5.0) * degrees;
        let Some(body) = self.root.child_mut("body") else {
            return;
        };
        for (side, sign) in [("left", 1.0_f32), ("right", -1.0)] {
            if let Some(base) = body.child_mut(&format!("{side}_wing_base")) {
                base.z_rot = sign * wing;
                if let Some(tip) = base.child_mut(&format!("{side}_wing_tip")) {
                    tip.z_rot = sign * wing;
                }
            }
        }
        if let Some(base) = body.child_mut("tail_base") {
            base.x_rot = tail;
            if let Some(tip) = base.child_mut("tail_tip") {
                tip.x_rot = tail;
            }
        }
    }
}
