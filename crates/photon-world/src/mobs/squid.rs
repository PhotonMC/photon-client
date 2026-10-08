//! What a squid does of its own on the client (`Squid.aiStep`): its
//! tentacles' beat and the turns of its body, which follow how it moves in
//! the water.

use photon_core::math::{mth_atan2, mth_lerp_f32, mth_sin_f64};

/// `(float) Math.PI`.
const PI: f32 = std::f32::consts::PI;
/// `180.0F / (float) Math.PI`.
const DEGREES: f32 = 180.0 / PI;

#[derive(Debug, Clone, PartialEq)]
pub struct SquidState {
    pub x_body_rot: f32,
    x_body_rot_old: f32,
    pub z_body_rot: f32,
    z_body_rot_old: f32,
    tentacle_movement: f32,
    pub tentacle_angle: f32,
    tentacle_angle_old: f32,
    tentacle_speed: f32,
    rotate_speed: f32,
}

/// The turns are finite, so the comparison is a total one.
impl Eq for SquidState {}

/// What a squid's pose is drawn from, between two ticks (`SquidRenderer`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquidPose {
    pub tentacle_angle: f32,
    pub x_body_rot: f32,
    pub z_body_rot: f32,
}

impl SquidState {
    /// A squid that has just appeared: its tentacle speed drawn from a source
    /// seeded by the clock (the reference draws it, unseeded, in the
    /// constructor).
    pub(crate) fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        let mut random = photon_core::random::RandomSource::create(seed);
        Self::with_tentacle_speed(1.0 / (random.next_float() + 1.0) * 0.2)
    }

    pub(crate) fn with_tentacle_speed(tentacle_speed: f32) -> Self {
        Self {
            x_body_rot: 0.0,
            x_body_rot_old: 0.0,
            z_body_rot: 0.0,
            z_body_rot_old: 0.0,
            tentacle_movement: 0.0,
            tentacle_angle: 0.0,
            tentacle_angle_old: 0.0,
            tentacle_speed,
            rotate_speed: 0.0,
        }
    }

    /// Fixes the tentacle speed, which the reference draws from a source it
    /// does not seed, so a comparison can be made.
    pub fn fix_tentacle_speed(&mut self, tentacle_speed: f32) {
        self.tentacle_speed = tentacle_speed;
    }

    /// `Squid.handleEntityEvent(19)`: the tentacles start a new beat.
    pub(crate) fn restart_beat(&mut self) {
        self.tentacle_movement = 0.0;
    }

    /// `Squid.aiStep` on the client, after `LivingEntity.aiStep`. Returns the
    /// body's new turn when the squid is in water (it also becomes the
    /// entity's turn).
    pub(crate) fn ai_step(
        &mut self,
        in_water: bool,
        motion: (f64, f64, f64),
        y_body_rot: f32,
    ) -> Option<f32> {
        self.x_body_rot_old = self.x_body_rot;
        self.z_body_rot_old = self.z_body_rot;
        self.tentacle_angle_old = self.tentacle_angle;
        self.tentacle_movement += self.tentacle_speed;
        if f64::from(self.tentacle_movement) > std::f64::consts::PI * 2.0 {
            self.tentacle_movement = (std::f64::consts::PI * 2.0) as f32;
        }
        if !in_water {
            self.tentacle_angle = mth_sin_f64(f64::from(self.tentacle_movement)).abs() * PI * 0.25;
            self.x_body_rot += (-90.0 - self.x_body_rot) * 0.02;
            return None;
        }
        if self.tentacle_movement < PI {
            let scale = self.tentacle_movement / PI;
            self.tentacle_angle = mth_sin_f64(f64::from(scale * scale * PI)) * PI * 0.25;
            if f64::from(scale) > 0.75 {
                self.rotate_speed = 1.0;
            } else {
                self.rotate_speed *= 0.8;
            }
        } else {
            self.tentacle_angle = 0.0;
            self.rotate_speed *= 0.99;
        }
        let (x, y, z) = motion;
        let horizontal = (x * x + z * z).sqrt();
        let y_body_rot = y_body_rot + (-(mth_atan2(x, z) as f32) * DEGREES - y_body_rot) * 0.1;
        self.z_body_rot += PI * self.rotate_speed * 1.5;
        self.x_body_rot += (-(mth_atan2(horizontal, y) as f32) * DEGREES - self.x_body_rot) * 0.1;
        Some(y_body_rot)
    }

    /// What the renderer reads between two ticks.
    pub fn pose(&self, partial: f32) -> SquidPose {
        SquidPose {
            tentacle_angle: mth_lerp_f32(partial, self.tentacle_angle_old, self.tentacle_angle),
            x_body_rot: mth_lerp_f32(partial, self.x_body_rot_old, self.x_body_rot),
            z_body_rot: mth_lerp_f32(partial, self.z_body_rot_old, self.z_body_rot),
        }
    }
}
