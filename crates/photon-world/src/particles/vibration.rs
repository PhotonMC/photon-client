//! Particles that chase a target, each with its own tick: `VibrationSignalParticle`
//! (`VibrationParticleOption`: a position source and an arrival time, turning to face it) and
//! `TrailParticle` (`TrailParticleOption`: a fixed target, colour and duration).

use std::sync::Arc;

use photon_core::math::mth_atan2;
use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::engine::{PositionSource, PositionSourceExt};
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// What `VibrationSignalParticle` keeps besides the particle: where it is headed, and the two
/// angles (turned toward the destination) its drawing turns its quads by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Vibration {
    pub(super) destination: PositionSource,
    pub(super) rot: f32,
    pub(super) rot_o: f32,
    pub(super) pitch: f32,
    pub(super) pitch_o: f32,
}

/// The two angles a particle at `at` faces `destination` with, `Mth.atan2` of the difference.
fn facing(at: (f64, f64, f64), destination: (f64, f64, f64)) -> (f32, f32) {
    let dx = at.0 - destination.0;
    let dy = at.1 - destination.1;
    let dz = at.2 - destination.2;
    (
        mth_atan2(dx, dz) as f32,
        mth_atan2(dy, (dx * dx + dz * dz).sqrt()) as f32,
    )
}

impl Particle {
    /// `VibrationSignalParticle(..)` and its `Provider`.
    pub fn vibration(
        random: RandomSource,
        at: (f64, f64, f64),
        destination: PositionSource,
        arrival_in_ticks: i32,
        sprite: (&Arc<SpriteSet>, usize),
        world: &ClientWorld,
    ) -> Self {
        let (rot, pitch) = destination
            .position(world)
            .map_or((0.0, 0.0), |position| facing(at, position));
        let mut particle = Particle::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            sprite,
            Kind::Vibration(Vibration {
                destination,
                rot,
                rot_o: rot,
                pitch,
                pitch_o: pitch,
            }),
        )
        .quad();
        particle.quad_size = 0.3;
        particle.lifetime = arrival_in_ticks;
        particle
    }

    /// `VibrationSignalParticle.tick`: chases its destination, closing a `1/ticksRemaining`
    /// share of the distance, turning to face it; gone once its destination cannot be found.
    pub(super) fn vibration_tick(&mut self, world: &ClientWorld, vibration: Vibration) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let age = self.age;
        self.age += 1;
        if age >= self.lifetime {
            self.removed = true;
            return;
        }
        let Some(destination) = vibration.destination.position(world) else {
            self.removed = true;
            return;
        };
        let alpha = 1.0 / f64::from(self.lifetime - self.age);
        self.x += (destination.0 - self.x) * alpha;
        self.y += (destination.1 - self.y) * alpha;
        self.z += (destination.2 - self.z) * alpha;
        let (rot, pitch) = facing((self.x, self.y, self.z), destination);
        self.kind = Kind::Vibration(Vibration {
            destination: vibration.destination,
            rot,
            rot_o: vibration.rot,
            pitch,
            pitch_o: vibration.pitch,
        });
    }

    /// `TrailParticle(..)` and its `Provider`.
    pub fn trail(
        random: RandomSource,
        motion: ((f64, f64, f64), (f64, f64, f64)),
        target: (f64, f64, f64),
        color: u32,
        duration: i32,
        sprite: (&Arc<SpriteSet>, usize),
    ) -> Self {
        let (at, push) = motion;
        let mut particle = Particle::thrown(
            random,
            at,
            push,
            sprite,
            Kind::Trail {
                target: [target.0, target.1, target.2],
            },
        )
        .quad();
        // `ARGB.scaleRGB`: each 8-bit channel by its own draw, truncated and clamped.
        let original = |shift: u32| ((color >> shift) & 0xFF) as i32;
        let scale_r = 0.875 + particle.random.next_float() * 0.25;
        let scale_g = 0.875 + particle.random.next_float() * 0.25;
        let scale_b = 0.875 + particle.random.next_float() * 0.25;
        let scaled = |channel: i32, scale: f32| (channel as f32 * scale) as i32;
        particle.r_col = scaled(original(16), scale_r).clamp(0, 255) as f32 / 255.0;
        particle.g_col = scaled(original(8), scale_g).clamp(0, 255) as f32 / 255.0;
        particle.b_col = scaled(original(0), scale_b).clamp(0, 255) as f32 / 255.0;
        particle.quad_size = 0.26;
        particle.lifetime = duration;
        particle
    }

    /// `TrailParticle.tick`: chases its target, closing a `1/ticksRemaining` share of the
    /// distance.
    pub(super) fn trail_tick(&mut self, target: [f64; 3]) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let age = self.age;
        self.age += 1;
        if age >= self.lifetime {
            self.removed = true;
            return;
        }
        let alpha = 1.0 / f64::from(self.lifetime - self.age);
        self.x += (target[0] - self.x) * alpha;
        self.y += (target[1] - self.y) * alpha;
        self.z += (target[2] - self.z) * alpha;
    }
}
