//! The dragon breath speck and the trial spawner's detection puff.

use std::sync::Arc;

use photon_core::random::RandomSource;

use super::engine::ParticleData;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

impl Particle {
    /// `DragonBreathParticle.Provider`: a purple speck thrown as given, then by its power.
    pub(super) fn dragon_breath(
        random: RandomSource,
        (at, (xa, ya, za)): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        data: ParticleData,
    ) -> Option<Self> {
        let ParticleData::Power(power) = data else {
            return None;
        };
        let mut particle = Self::at(
            random,
            at,
            (sprites, sprites.first()),
            Kind::DragonBreath {
                has_hit_ground: false,
            },
        )
        .quad();
        particle.friction = 0.96;
        (particle.xd, particle.yd, particle.zd) = (xa, ya, za);
        // `Mth.nextFloat(random, min, max)`: the lower end alone, with no draw, when they are equal.
        let between = |random: &mut RandomSource, min: f32, max: f32| {
            if min >= max {
                min
            } else {
                random.next_float() * (max - min) + min
            }
        };
        particle.r_col = between(&mut particle.random, 0.717_647_1, 0.874_509_8);
        particle.g_col = between(&mut particle.random, 0.0, 0.0);
        particle.b_col = between(&mut particle.random, 0.823_529_4, 0.976_470_6);
        particle.quad_size *= 0.75;
        particle.lifetime = (20.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        particle.has_physics = false;
        particle.set_sprite_from_age();
        particle.set_power(power);
        Some(particle)
    }

    /// `DragonBreathParticle.tick`.
    pub(super) fn dragon_breath_tick(
        &mut self,
        world: &super::super::ClientWorld,
        mut has_hit_ground: bool,
    ) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let aged = self.age >= self.lifetime;
        self.age += 1;
        if aged {
            self.removed = true;
            return;
        }
        self.set_sprite_from_age();
        if self.on_ground {
            self.yd = 0.0;
            has_hit_ground = true;
        }
        if has_hit_ground {
            self.yd += 0.002;
        }
        self.move_by(world, self.xd, self.yd, self.zd);
        if self.y == self.yo {
            self.xd *= 1.1;
            self.zd *= 1.1;
        }
        self.xd *= f64::from(self.friction);
        self.zd *= f64::from(self.friction);
        if has_hit_ground {
            self.yd *= f64::from(self.friction);
        }
        self.kind = Kind::DragonBreath { has_hit_ground };
    }
}

impl Particle {
    /// `TrialSpawnerDetectionParticle.Provider`: a puff, scaled by one and a half.
    pub(super) fn trial_detection(
        random: RandomSource,
        (at, (xa, ya, za)): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let scale = 1.5_f32;
        let mut particle = Self::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            (sprites, sprites.first()),
            Kind::TrialDetection,
        )
        .quad();
        particle.friction = 0.96;
        particle.gravity = -0.1;
        particle.speed_up_when_y_motion_is_blocked = true;
        particle.xd *= 0.0;
        particle.yd *= 0.9;
        particle.zd *= 0.0;
        particle.xd += xa;
        particle.yd += ya;
        particle.zd += za;
        particle.quad_size *= 0.75 * scale;
        // `Mth.randomBetween(random, 0.5F, 1.0F)`.
        let between = particle.random.next_float() * (1.0 - 0.5) + 0.5;
        particle.lifetime = ((8.0 / between * scale) as i32).max(1);
        particle.set_sprite_from_age();
        particle.has_physics = true;
        particle
    }
}
