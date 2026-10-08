use super::*;

/// `EndCrystalRenderer.getY`: how high the crystal's glass floats, in blocks, at an age.
pub fn end_crystal_y(time_in_ticks: f32) -> f32 {
    let mut height = mth_sin_f64(f64::from(time_in_ticks * 0.2)) / 2.0 + 0.5;
    height = (height * height + height) * 0.4;
    height - 1.4
}

/// What a warden's models (the body and its four glowing layers) are posed from
/// (`WardenRenderState`): the look, the walk, the pulses of its tendrils, and
/// the animation states its pose and events run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WardenRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub tendril_animation: f32,
    pub roar: photon_world::animation_state::AnimationState,
    pub sniff: photon_world::animation_state::AnimationState,
    pub emerge: photon_world::animation_state::AnimationState,
    pub digging: photon_world::animation_state::AnimationState,
    pub attack: photon_world::animation_state::AnimationState,
    pub sonic_boom: photon_world::animation_state::AnimationState,
}

/// What a wither's models (the body and its armor) are posed from
/// (`WitherRenderState`): the middle head's look, the body's turn, and the side
/// heads' turns, all in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitherRenderState {
    pub age_in_ticks: f32,
    pub y_rot: f32,
    pub x_rot: f32,
    pub body_rot: f32,
    pub x_head_rots: [f32; 2],
    pub y_head_rots: [f32; 2],
}

/// What an ender dragon's model is posed from (`EnderDragonRenderState`): the
/// wing beat, the flight history its parts follow (and the partial tick it is read at),
/// and what its neck's pose follows while it lands, takes off or sits.
#[derive(Debug, Clone, PartialEq)]
pub struct EnderDragonRenderState {
    pub flap_time: f32,
    pub history: photon_world::mobs::ender_dragon::FlightHistory,
    pub partial_ticks: f32,
    pub is_landing_or_taking_off: bool,
    pub is_sitting: bool,
    pub distance_to_egg: f64,
}

/// What an end crystal's model is posed from (`EndCrystalRenderState`): its age
/// (which turns and bobs its glass) and whether it shows its base.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndCrystalRenderState {
    pub age_in_ticks: f32,
    pub shows_bottom: bool,
}

impl EnderDragonRenderState {
    /// `getHeadPartYOffset`.
    pub(super) fn head_part_y_offset(
        &self,
        part: i32,
        body: photon_world::mobs::ender_dragon::Sample,
        at: photon_world::mobs::ender_dragon::Sample,
    ) -> f32 {
        let result = if self.is_landing_or_taking_off {
            f64::from(part) / (self.distance_to_egg / 4.0).max(1.0)
        } else if self.is_sitting {
            f64::from(part)
        } else if part == 6 {
            0.0
        } else {
            at.y - body.y
        };
        result as f32
    }
}

impl MobModel {
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
        let wrap = photon_core::math::wrap_degrees_f32;
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
        let wrap = photon_core::math::wrap_degrees_f32;
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

    /// `WardenModel.setupAnim`: the head takes the look, then the walk, the
    /// idle sway and the tendrils' pulse move the parts, then each animation
    /// that has started runs over them. The glowing layers share the parts'
    /// names (the cut parts are empty), so the same call poses all five.
    pub fn setup_warden(&mut self, state: &WardenRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let half_pi = (std::f64::consts::PI / 2.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        // `animateWalk`.
        let speed = (3.0 * state.quadruped.walk_speed).min(0.5);
        let position = state.quadruped.walk_position * 0.8662;
        let (position_cos, position_sin) = (cos(position), sin(position));
        let speed_with_min = speed.min(0.35);
        // `animateIdlePose`.
        let scaled_age = state.age_in_ticks * 0.1;
        let (wobble_cos, wobble_sin) = (cos(scaled_age), sin(scaled_age));
        // `animateTendrils`: the cosine is a double there.
        let tendril_x_rot = state.tendril_animation
            * ((f64::from(state.age_in_ticks) * 2.25).cos()
                * std::f64::consts::PI
                * f64::from(0.1_f32)) as f32;
        let Some(bone) = self.root.child_mut("bone") else {
            return;
        };
        if let Some(body) = bone.child_mut("body") {
            body.z_rot = 0.1 * position_sin * speed;
            body.x_rot = 1.0 * position_cos * speed_with_min;
            body.z_rot += 0.025 * wobble_sin;
            body.x_rot += 0.025 * wobble_cos;
            if let Some(head) = body.child_mut("head") {
                head.x_rot = state.quadruped.x_rot * degrees;
                head.y_rot = state.quadruped.head_y_rot * degrees;
                head.z_rot += 0.3 * position_sin * speed;
                head.x_rot += 1.2 * cos(position + half_pi) * speed_with_min;
                head.z_rot += 0.06 * wobble_cos;
                head.x_rot += 0.06 * wobble_sin;
                if let Some(left) = head.child_mut("left_tendril") {
                    left.x_rot = tendril_x_rot;
                }
                if let Some(right) = head.child_mut("right_tendril") {
                    right.x_rot = -tendril_x_rot;
                }
            }
            // `resetArmPoses`: the arms stand where the model puts them, as the walk
            // does not move them there.
            for (name, swing, side) in [
                ("left_arm", -(0.8 * position_cos * speed), 13.0),
                ("right_arm", -(0.8 * position_sin * speed), -13.0),
            ] {
                if let Some(arm) = body.child_mut(name) {
                    arm.x_rot = swing;
                    arm.z_rot = 0.0;
                    arm.y_rot = 0.0;
                    arm.z = 1.0;
                    arm.x = side;
                    arm.y = -13.0;
                }
            }
        }
        if let Some(leg) = bone.child_mut("left_leg") {
            leg.x_rot = 1.0 * position_cos * speed;
        }
        if let Some(leg) = bone.child_mut("right_leg") {
            leg.x_rot = 1.0 * cos(position + pi) * speed;
        }
        for (name, running) in [
            ("Warden.WARDEN_ATTACK", state.attack),
            ("Warden.WARDEN_SONIC_BOOM", state.sonic_boom),
            ("Warden.WARDEN_DIG", state.digging),
            ("Warden.WARDEN_EMERGE", state.emerge),
            ("Warden.WARDEN_ROAR", state.roar),
            ("Warden.WARDEN_SNIFF", state.sniff),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }
}
