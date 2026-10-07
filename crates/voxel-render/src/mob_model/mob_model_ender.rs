use super::*;

impl MobModel {
    /// `CreakingModel.setupAnim`: the head takes the look, the walk runs if the
    /// creaking can move, then the attack, the sway and the death animations
    /// that have started. The eyes' layer shares the parts' names.
    pub fn setup_creaking(&mut self, state: &CreakingRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(head) = self
            .root
            .child_mut("root")
            .and_then(|root| root.child_mut("upper_body"))
            .and_then(|upper| upper.child_mut("head"))
        {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        }
        if state.can_move
            && let Some(definition) = voxel_data::animation::definition("Creaking.CREAKING_WALK")
        {
            // `applyWalk(pos, speed, 1, 1)`.
            let millis = (state.quadruped.walk_position * 50.0 * 1.0) as i64;
            let scale = (state.quadruped.walk_speed * 1.0).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            ("Creaking.CREAKING_ATTACK", state.attack),
            ("Creaking.CREAKING_INVULNERABLE", state.invulnerable),
            ("Creaking.CREAKING_DEATH", state.death),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }

    /// `ArmadilloModel.setupAnim` (the adult's and the baby's alike, with the
    /// baby's own animations): hiding in the shell draws the ball in place of
    /// the body's cubes, legs and tail, and the walk stops; otherwise the head
    /// takes the look within its limits and the walk runs; then the animations
    /// that have started.
    pub fn setup_armadillo(&mut self, state: &ArmadilloRenderState, baby: bool) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let hiding = state.is_hiding_in_shell;
        if let Some(cube) = self.root.child_mut("cube") {
            cube.visible = hiding;
        }
        for name in ["left_hind_leg", "right_hind_leg"] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.visible = !hiding;
            }
        }
        if let Some(body) = self.root.child_mut("body") {
            body.skip_draw = hiding;
            if let Some(tail) = body.child_mut("tail") {
                tail.visible = !hiding;
            }
            if !hiding && let Some(head) = body.child_mut("head") {
                head.x_rot = state.quadruped.x_rot.clamp(-22.5, 25.0) * degrees;
                head.y_rot = state.quadruped.head_y_rot.clamp(-32.5, 32.5) * degrees;
            }
        }
        let (walk, roll_out, roll_up, peek) = if baby {
            (
                "BabyArmadillo.ARMADILLO_BABY_WALK",
                "BabyArmadillo.ARMADILLO_BABY_ROLL_OUT",
                "BabyArmadillo.ARMADILLO_BABY_ROLL_UP",
                "BabyArmadillo.ARMADILLO_BABY_PEEK",
            )
        } else {
            (
                "Armadillo.ARMADILLO_WALK",
                "Armadillo.ARMADILLO_ROLL_OUT",
                "Armadillo.ARMADILLO_ROLL_UP",
                "Armadillo.ARMADILLO_PEEK",
            )
        };
        if !hiding && let Some(definition) = voxel_data::animation::definition(walk) {
            // `applyWalk(pos, speed, 16.5, 2.5)`.
            let millis = (state.quadruped.walk_position * 50.0 * 16.5) as i64;
            let scale = (state.quadruped.walk_speed * 2.5).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            (roll_out, state.roll_out),
            (roll_up, state.roll_up),
            (peek, state.peek),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }

    /// `SnifferModel.setupAnim` (the baby's layer has the same parts and uses
    /// the same model): the head takes the look, the walk (the searching one
    /// while it searches) runs, then the animations that have started.
    pub fn setup_sniffer(&mut self, state: &SnifferRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(head) = self
            .root
            .child_mut("bone")
            .and_then(|bone| bone.child_mut("body"))
            .and_then(|body| body.child_mut("head"))
        {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        }
        let walk = if state.is_searching {
            "Sniffer.SNIFFER_SNIFF_SEARCH"
        } else {
            "Sniffer.SNIFFER_WALK"
        };
        if let Some(definition) = voxel_data::animation::definition(walk) {
            // `applyWalk(pos, speed, 9, 100)`.
            let millis = (state.quadruped.walk_position * 50.0 * 9.0) as i64;
            let scale = (state.quadruped.walk_speed * 100.0).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            ("Sniffer.SNIFFER_DIG", state.digging),
            ("Sniffer.SNIFFER_LONGSNIFF", state.sniffing),
            ("Sniffer.SNIFFER_STAND_UP", state.rising),
            ("Sniffer.SNIFFER_HAPPY", state.feeling_happy),
            ("Sniffer.SNIFFER_SNIFFSNIFF", state.scenting),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }

    /// `WitherBossModel.setupAnim`: the side heads turn after their own targets
    /// (relative to the body), the ribcage and the tail sway, and the middle
    /// head takes the look. The armor's layer has the same parts.
    pub fn setup_wither(&mut self, state: &WitherRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        for (name, index) in [("right_head", 0), ("left_head", 1)] {
            if let Some(head) = self.root.child_mut(name) {
                head.y_rot = (state.y_head_rots[index] - state.body_rot) * degrees;
                head.x_rot = state.x_head_rots[index] * degrees;
            }
        }
        let anim = mth_cos_f64(f64::from(state.age_in_ticks * 0.1));
        let ribcage_x_rot = (0.065 + 0.05 * anim) * pi;
        if let Some(ribcage) = self.root.child_mut("ribcage") {
            ribcage.x_rot = ribcage_x_rot;
        }
        if let Some(tail) = self.root.child_mut("tail") {
            tail.x = -2.0;
            tail.y = 6.9 + mth_cos_f64(f64::from(ribcage_x_rot)) * 10.0;
            tail.z = -0.5 + mth_sin_f64(f64::from(ribcage_x_rot)) * 10.0;
            tail.x_rot = (0.265 + 0.1 * anim) * pi;
        }
        if let Some(head) = self.root.child_mut("center_head") {
            head.y_rot = state.y_rot * degrees;
            head.x_rot = state.x_rot * degrees;
        }
    }

    /// `EnderDragonModel.setupAnim`: the jaw and the body bounce with the wing beat, the
    /// neck's five parts and the head follow the flight history's turns and heights
    /// along a chain, the wings beat, the legs bounce, and the tail's twelve parts
    /// follow the history behind.
    pub fn setup_ender_dragon(&mut self, state: &EnderDragonRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let wrap = voxel_core::math::wrap_degrees_f32;
        let at = |delay: i32| state.history.get_at(delay, state.partial_ticks);
        let flap_time = state.flap_time * (std::f64::consts::PI * 2.0) as f32;
        let bounce = sin(flap_time - 1.0) + 1.0;
        let bounce = (bounce * bounce + bounce * 2.0) * 0.05;
        self.root.y = (bounce - 2.0) * 16.0;
        self.root.z = -48.0;
        self.root.x_rot = bounce * 2.0 * degrees;
        let neck0 = self
            .root
            .child_mut("neck0")
            .map(|neck| (neck.x, neck.y, neck.z));
        let (mut xx, mut yy, mut zz) = neck0.unwrap_or_default();
        let start = at(6);
        let rot2 = wrap(at(5).y_rot - at(10).y_rot);
        let rot = wrap(at(5).y_rot + rot2 / 2.0);
        for i in 0..5 {
            let point = at(5 - i);
            let neck_x_rot = cos(i as f32 * 0.45 + flap_time) * 0.15;
            if let Some(neck) = self.root.child_mut(&format!("neck{i}")) {
                neck.y_rot = wrap(point.y_rot - start.y_rot) * degrees * 1.5;
                neck.x_rot =
                    neck_x_rot + state.head_part_y_offset(i, start, point) * degrees * 1.5 * 5.0;
                neck.z_rot = -wrap(point.y_rot - rot) * degrees * 1.5;
                neck.y = yy;
                neck.z = zz;
                neck.x = xx;
                xx -= sin(neck.y_rot) * cos(neck.x_rot) * 10.0;
                yy += sin(neck.x_rot) * 10.0;
                zz -= cos(neck.y_rot) * cos(neck.x_rot) * 10.0;
            }
        }
        let current = at(0);
        if let Some(head) = self.root.child_mut("head") {
            head.y = yy;
            head.z = zz;
            head.x = xx;
            head.y_rot = wrap(current.y_rot - start.y_rot) * degrees;
            head.x_rot = wrap(state.head_part_y_offset(6, start, current)) * degrees * 1.5 * 5.0;
            head.z_rot = -wrap(current.y_rot - rot) * degrees;
            if let Some(jaw) = head.child_mut("jaw") {
                jaw.x_rot = (sin(flap_time) + 1.0) * 0.2;
            }
        }
        self.pose_ender_dragon_body(bounce, flap_time, rot2);
        self.pose_ender_dragon_tail(state, flap_time, rot);
    }

    /// The wings and legs of `EnderDragonModel.setupAnim`: the wings beat (the right
    /// ones the left ones' mirror), the legs bounce (`poseLimbs`), the body rolls.
    pub(super) fn pose_ender_dragon_body(&mut self, bounce: f32, flap_time: f32, rot2: f32) {
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let left_x_rot = 0.125 - cos(flap_time) * 0.2;
        let left_z_rot = -(sin(flap_time) + 0.125) * 0.8;
        let left_tip_z_rot = (sin(flap_time + 2.0) + 0.5) * 0.75;
        if let Some(body) = self.root.child_mut("body") {
            body.z_rot = -rot2 * 1.5 * degrees;
            for (side, sign) in [("left", 1.0_f32), ("right", -1.0)] {
                if let Some(wing) = body.child_mut(&format!("{side}_wing")) {
                    wing.x_rot = left_x_rot;
                    wing.y_rot = -0.25 * sign;
                    wing.z_rot = if sign > 0.0 { left_z_rot } else { -left_z_rot };
                    if let Some(tip) = wing.child_mut(&format!("{side}_wing_tip")) {
                        tip.z_rot = if sign > 0.0 {
                            left_tip_z_rot
                        } else {
                            -left_tip_z_rot
                        };
                    }
                }
                // `poseLimbs`.
                for (leg, tip, foot, leg_x, tip_x, foot_x, up) in [
                    (
                        format!("{side}_front_leg"),
                        format!("{side}_front_leg_tip"),
                        format!("{side}_front_foot"),
                        1.3_f32,
                        -0.5_f32,
                        0.75_f32,
                        false,
                    ),
                    (
                        format!("{side}_hind_leg"),
                        format!("{side}_hind_leg_tip"),
                        format!("{side}_hind_foot"),
                        1.0,
                        0.5,
                        0.75,
                        true,
                    ),
                ] {
                    if let Some(leg_part) = body.child_mut(&leg) {
                        leg_part.x_rot = leg_x + bounce * 0.1;
                        if let Some(tip_part) = leg_part.child_mut(&tip) {
                            tip_part.x_rot = if up {
                                tip_x + bounce * 0.1
                            } else {
                                tip_x - bounce * 0.1
                            };
                            if let Some(foot_part) = tip_part.child_mut(&foot) {
                                foot_part.x_rot = foot_x + bounce * 0.1;
                            }
                        }
                    }
                }
            }
        }
    }

    /// The tail of `EnderDragonModel.setupAnim`: twelve parts follow the history behind.
    pub(super) fn pose_ender_dragon_tail(
        &mut self,
        state: &EnderDragonRenderState,
        flap_time: f32,
        rot: f32,
    ) {
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let wrap = voxel_core::math::wrap_degrees_f32;
        let at = |delay: i32| state.history.get_at(delay, state.partial_ticks);
        let mut tail_x_rot = 0.0_f32;
        let tail0 = self
            .root
            .child_mut("tail0")
            .map(|tail| (tail.x, tail.y, tail.z));
        let (mut xx, mut yy, mut zz) = tail0.unwrap_or_default();
        let start = at(11);
        for i in 0..12 {
            let point = at(12 + i);
            tail_x_rot += sin(i as f32 * 0.45 + flap_time) * 0.05;
            if let Some(tail) = self.root.child_mut(&format!("tail{i}")) {
                tail.y_rot = (wrap(point.y_rot - start.y_rot) * 1.5 + 180.0) * degrees;
                tail.x_rot = tail_x_rot + (point.y - start.y) as f32 * degrees * 1.5 * 5.0;
                tail.z_rot = wrap(point.y_rot - rot) * degrees * 1.5;
                tail.y = yy;
                tail.z = zz;
                tail.x = xx;
                yy += sin(tail.x_rot) * 10.0;
                zz -= cos(tail.y_rot) * cos(tail.x_rot) * 10.0;
                xx -= sin(tail.y_rot) * cos(tail.x_rot) * 10.0;
            }
        }
    }

    /// `EndCrystalModel.setupAnim`: the base shows or not, the glass floats up and
    /// down, and each of the three nested cubes is turned about a tilted axis, the
    /// outer one, the inner one and the core by the same turn, within the other.
    pub fn setup_end_crystal(&mut self, state: &EndCrystalRenderState) {
        use crate::joml::Quaternion;
        self.root.reset();
        let sin_45 = (std::f64::consts::PI / 4.0).sin() as f32;
        let third_turn = (std::f64::consts::PI / 3.0) as f32;
        let speed = state.age_in_ticks * 3.0;
        let crystal_y = end_crystal_y(state.age_in_ticks) * 16.0;
        if let Some(base) = self.root.child_mut("base") {
            base.visible = state.shows_bottom;
        }
        let Some(outer) = self.root.child_mut("outer_glass") else {
            return;
        };
        outer.y += crystal_y / 2.0;
        outer.rotate_by(
            &Quaternion::rotation_y(Quaternion::degrees_to_radians(speed))
                .rotate_axis(third_turn, sin_45, 0.0, sin_45),
        );
        let tilted = Quaternion::set_angle_axis(third_turn, sin_45, 0.0, sin_45)
            .rotate_y(speed * (std::f64::consts::PI / 180.0) as f32);
        if let Some(inner) = outer.child_mut("inner_glass") {
            inner.rotate_by(&tilted);
            if let Some(core) = inner.child_mut("cube") {
                core.rotate_by(&tilted);
            }
        }
    }

    /// `SkullModel.setupAnim` and its piglin's and dragon's: the head faces straight ahead (the
    /// layer gives the skull no turn), a piglin's ears swing and a dragon's jaw works with the
    /// walk (`animation_pos`).
    pub fn setup_skull(&mut self, kind: crate::skull::SkullKind, animation_pos: f32) {
        use crate::skull::SkullKind;
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        match kind {
            SkullKind::Piglin => {
                if let Some(head) = self.root.child_mut("head") {
                    head.y_rot = 0.0;
                    head.x_rot = 0.0;
                    // `Math.cos` of a float's double, not the table `Mth.cos` reads.
                    let ear = |value: f32| f64::from(value).cos();
                    if let Some(left) = head.child_mut("left_ear") {
                        left.z_rot = (-(ear(animation_pos * pi * 0.2 * 1.2) + 2.5)) as f32 * 0.2;
                    }
                    if let Some(right) = head.child_mut("right_ear") {
                        right.z_rot = (ear(animation_pos * pi * 0.2) + 2.5) as f32 * 0.2;
                    }
                }
            }
            SkullKind::Dragon => {
                if let Some(head) = self.root.child_mut("head") {
                    if let Some(jaw) = head.child_mut("jaw") {
                        jaw.x_rot = (f64::from(animation_pos * pi * 0.2).sin() + 1.0) as f32 * 0.2;
                    }
                    head.y_rot = 0.0;
                    head.x_rot = 0.0;
                }
            }
            _ => {
                if let Some(head) = self.root.child_mut("head") {
                    head.y_rot = 0.0;
                    head.x_rot = 0.0;
                }
            }
        }
    }

    /// Move the whole model (`PoseStack.translate` before its own pose), in model units.
    pub fn shift_root(&mut self, [x, y, z]: [f32; 3]) {
        self.root.x += x;
        self.root.y += y;
        self.root.z += z;
    }

    /// `ElytraModel.setupAnim`: the wings spread by the animation state's rotations, lowered when
    /// the wearer sneaks.
    pub fn setup_elytra(&mut self, [rot_x, rot_y, rot_z]: [f32; 3], crouching: bool) {
        self.root.reset();
        let y = if crouching { 3.0 } else { 0.0 };
        if let Some(left) = self.root.child_mut("left_wing") {
            left.y = y;
            left.x_rot = rot_x;
            left.z_rot = rot_z;
            left.y_rot = rot_y;
        }
        if let Some(right) = self.root.child_mut("right_wing") {
            right.y = y;
            right.x_rot = rot_x;
            right.z_rot = -rot_z;
            right.y_rot = -rot_y;
        }
    }

    /// `WindChargeModel.setupAnim`: the charge and the wind about it spin opposite ways.
    pub fn setup_wind_charge(&mut self, age_in_ticks: f32) {
        self.root.reset();
        let spin = age_in_ticks * 16.0 * (std::f64::consts::PI / 180.0) as f32;
        let Some(bone) = self.root.child_mut("bone") else {
            return;
        };
        if let Some(charge) = bone.child_mut("wind_charge") {
            charge.y_rot = -spin;
        }
        if let Some(wind) = bone.child_mut("wind") {
            wind.y_rot = spin;
        }
    }

    /// `LlamaSpitModel` has no animation: its part stays where the mesh puts it.
    pub fn setup_llama_spit(&mut self) {
        self.root.reset();
    }

    /// `LeashKnotModel` has no animation: its part stays where the mesh puts it.
    pub fn setup_leash_knot(&mut self) {
        self.root.reset();
    }

    /// `ShulkerBulletModel.setupAnim`: the bullet is turned the way it flies.
    pub fn setup_shulker_bullet(&mut self, y_rot: f32, x_rot: f32) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(main) = self.root.child_mut("main") {
            main.y_rot = y_rot * degrees;
            main.x_rot = x_rot * degrees;
        }
    }
}
