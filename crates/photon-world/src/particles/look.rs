//! How a particle is drawn: the size of its quad, its light, its layer and its picture.

use photon_core::math::{clamp_f32, mth_sin_f64};

use super::super::ClientWorld;
use super::light::{add_smooth_block_emission, particle_light};
use super::particle::{Kind, Particle, ParticleAtlas};

impl Particle {
    /// The size of the quad at a partial tick: `getQuadSize`.
    pub fn quad_size_at(&self, partial: f32) -> f32 {
        let s = (self.age as f32 + partial) / self.lifetime as f32;
        match self.kind {
            Kind::AshSmoke { .. }
            | Kind::NoxiousGas { .. }
            | Kind::PlayerCloud
            | Kind::DragonBreath { .. }
            | Kind::TrialDetection
            | Kind::Dust { .. }
            | Kind::Crit
            | Kind::Bobbing
            | Kind::FallingDust { .. } => self.quad_size * clamp_f32(s * 32.0, 0.0, 1.0),
            Kind::Flame => self.quad_size * (1.0 - s * s * 0.5),
            Kind::Lava => self.quad_size * (1.0 - s * s),
            Kind::Attract(super::attract::Attract::Portal { reverse: false, .. }) => {
                let s = 1.0 - s;
                self.quad_size * (1.0 - s * s)
            }
            Kind::Attract(super::attract::Attract::Portal { reverse: true, .. }) => {
                self.quad_size * (1.0 - (self.age as f32 + partial) / (self.lifetime as f32 * 1.5))
            }
            // `OverlayParticle.getQuadSize`.
            Kind::Flash => {
                7.1 * mth_sin_f64(f64::from(
                    (self.age as f32 + partial - 1.0) * 0.25 * std::f32::consts::PI,
                ))
            }
            Kind::Shriek { .. } => {
                let s = (self.age as f32 + partial) / self.lifetime as f32 * 0.75;
                self.quad_size * clamp_f32(s, 0.0, 1.0)
            }
            // `BlockMarker.getQuadSize`.
            Kind::BlockMarker => 0.5,
            _ => self.quad_size,
        }
    }

    /// The light the quad is drawn with at a partial tick: `getLightCoords`.
    pub fn light_coords_at(&self, world: &ClientWorld, partial: f32) -> i32 {
        let light = particle_light(world, self.x, self.y, self.z);
        match self.kind {
            Kind::Firefly => self.firefly_light(partial),
            Kind::Flame | Kind::Glow => {
                add_smooth_block_emission(light, (self.age as f32 + partial) / self.lifetime as f32)
            }
            // `LightCoordsUtil.withBlock(light, 15)`.
            Kind::Lava
            | Kind::Soul { glowing: true }
            | Kind::SculkCharge
            | Kind::Drip(super::drip::Drip { glowing: true, .. })
            | Kind::Vibration(_)
            | Kind::Shriek { .. }
            | Kind::TrialDetection
            | Kind::Attract(
                super::attract::Attract::FlyTowards { glowing: true, .. }
                | super::attract::Attract::FlyStraight { .. },
            ) => (light & 0xFF_0000) | (15 << 4),
            Kind::Attract(
                super::attract::Attract::Portal { .. } | super::attract::Attract::FlyTowards { .. },
            ) => {
                let mut brightness = self.age as f32 / self.lifetime as f32;
                brightness *= brightness;
                brightness *= brightness;
                add_smooth_block_emission(light, brightness)
            }
            // `SimpleAnimatedParticle.getLightCoords`: full bright. `TrailParticle`'s is its own
            // fixed override.
            Kind::Water(super::water::Water::Ink)
            | Kind::Totem
            | Kind::Burst
            | Kind::EndRod
            | Kind::FireworkSpark(_)
            | Kind::Trail { .. } => 15_728_880,
            _ => light,
        }
    }

    /// `Particle.setPower`: the motion it started with, scaled (the rise less).
    pub fn set_power(&mut self, power: f32) {
        self.xd *= f64::from(power);
        self.yd = (self.yd - f64::from(0.1_f32)) * f64::from(power) + f64::from(0.1_f32);
        self.zd *= f64::from(power);
    }

    /// `SingleQuadParticle.Layer.translucent`: drawn blended rather than cut out.
    pub fn translucent(&self) -> bool {
        matches!(
            self.kind,
            Kind::Soul { .. }
                | Kind::CampfireSmoke
                | Kind::PlayerCloud
                | Kind::Water(super::water::Water::Ink)
                | Kind::Spell { .. }
                | Kind::Totem
                | Kind::EndRod
                | Kind::SculkCharge
                | Kind::FireworkSpark(_)
                | Kind::Flash
                | Kind::NoxiousGas { .. }
                | Kind::Firefly
        ) || matches!(self.kind, Kind::Attract(super::attract::Attract::FlyTowards { alpha, .. }) if alpha[0] < 1.0 || alpha[1] < 1.0)
    }

    /// `SingleQuadParticle.getFacingCameraMode`: whether the quad turns about the vertical only
    /// (`LOOKAT_Y`) rather than to face the camera.
    pub fn faces_camera_about_y_only(&self) -> bool {
        matches!(self.kind, Kind::TrialDetection)
    }

    /// The atlas the picture is in.
    pub fn atlas(&self) -> ParticleAtlas {
        match self.kind {
            Kind::Terrain { .. } | Kind::BlockMarker => ParticleAtlas::Blocks,
            _ => ParticleAtlas::Particles,
        }
    }

    /// The part of its picture a particle shows, as fractions (`u0`, `u1`, `v0`, `v1`), where it is
    /// not the whole: `getU0` and the rest ask the sprite for a place within it.
    pub fn sprite_window(&self) -> Option<[f32; 4]> {
        match self.kind {
            Kind::Terrain { uo, vo } | Kind::Goo { uo, vo } => {
                Some([(uo + 1.0) / 4.0, uo / 4.0, vo / 4.0, (vo + 1.0) / 4.0])
            }
            _ => None,
        }
    }
}
