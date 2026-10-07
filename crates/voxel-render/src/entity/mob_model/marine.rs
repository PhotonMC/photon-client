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

    /// `SquidModel.setupAnim`: every tentacle leans by the beat's angle.
    pub fn setup_squid(&mut self, tentacle_angle: f32) {
        self.root.reset();
        for index in 0..8 {
            if let Some(tentacle) = self.root.child_mut(&format!("tentacle{index}")) {
                tentacle.x_rot = tentacle_angle;
            }
        }
    }
}

/// `GlowSquidRenderer.getBlockLightLevel`: a glow squid lights itself, less
/// while it is dark (after a hurt), but never below the light around it.
pub fn glow_squid_block_light(dark_ticks: i32, block_light: i32) -> i32 {
    let glow = voxel_core::math::clamped_lerp_f32(1.0 - dark_ticks as f32 / 10.0, 0.0, 15.0) as i32;
    if glow == 15 {
        15
    } else {
        glow.max(block_light)
    }
}

/// `DolphinCarryingItemLayer`: what a dolphin carries is held under its chin,
/// lower and further forward the more its head is pitched.
pub fn dolphin_item_pose(base: &Pose, x_rot: f32) -> Pose {
    let angle_x_percent = x_rot.abs() / 60.0;
    let mut pose = *base;
    if x_rot < 0.0 {
        pose.translate(
            0.0,
            1.0 - angle_x_percent * 0.5,
            -1.0 + angle_x_percent * 0.5,
        );
    } else {
        pose.translate(
            0.0,
            1.0 + angle_x_percent * 0.8,
            -1.0 + angle_x_percent * 0.2,
        );
    }
    pose
}

/// What a dolphin's model is posed from (`DolphinRenderState`): the head's
/// turn and pitch, which turn the whole body, the age, and whether it moves.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DolphinState {
    pub y_rot: f32,
    pub x_rot: f32,
    pub age_in_ticks: f32,
    pub is_moving: bool,
}

/// What a guardian's eye follows (`GuardianRenderState`): its own eye and
/// look, and the position of what it looks at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianLook {
    pub eye_position: [f64; 3],
    pub look_direction: [f64; 3],
    pub look_at_position: [f64; 3],
}

/// What a guardian's model is posed from (`GuardianRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianState {
    pub head_y_rot: f32,
    pub x_rot: f32,
    pub age_in_ticks: f32,
    pub spikes_animation: f32,
    pub tail_animation: f32,
    pub look: Option<GuardianLook>,
}

/// `GuardianModel`'s tables: where each spike sits, before it breathes.
pub(super) const SPIKE_X: [f32; 12] = [
    0.0, 0.0, 8.0, -8.0, -8.0, 8.0, 8.0, -8.0, 0.0, 0.0, 8.0, -8.0,
];

pub(super) const SPIKE_Y: [f32; 12] = [
    -8.0, -8.0, -8.0, -8.0, 0.0, 0.0, 0.0, 0.0, 8.0, 8.0, 8.0, 8.0,
];

pub(super) const SPIKE_Z: [f32; 12] = [
    8.0, -8.0, 0.0, 0.0, -8.0, -8.0, 8.0, 8.0, 8.0, -8.0, 0.0, 0.0,
];

/// What a nautilus's model is posed from (`NautilusRenderState`): the look
/// and stride, and whether it wears body armor (which hides a zombie
/// nautilus's corals).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NautilusState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub has_body_armor: bool,
}

/// What a squid's renderer turns the pose by: the body's two turns, and
/// whether it is a baby (it hangs lower).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SquidTurns {
    pub x_body_rot: f32,
    pub z_body_rot: f32,
    pub baby: bool,
}
