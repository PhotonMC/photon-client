//! `SmokeParticle` on `BaseAshSmokeParticle`.

use std::sync::Arc;

use photon_core::random::RandomSource;

use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

impl Particle {
    /// `BaseAshSmokeParticle(..)`: a particle that rises, darkens at random and follows
    /// its sprite set as it ages.
    pub(super) fn ash_smoke(
        random: RandomSource,
        at: (f64, f64, f64),
        direction: (f32, f32, f32),
        push: (f64, f64, f64),
        (scale, sprites): (f32, &Arc<SpriteSet>),
        (color_random, max_lifetime, gravity, has_physics): (f32, i32, f32, bool),
    ) -> Self {
        let mut particle = Self::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            (sprites, sprites.first()),
            Kind::AshSmoke { decays: false },
        )
        .quad();
        particle.friction = 0.96;
        particle.gravity = gravity;
        particle.speed_up_when_y_motion_is_blocked = true;
        particle.xd *= f64::from(direction.0);
        particle.yd *= f64::from(direction.1);
        particle.zd *= f64::from(direction.2);
        particle.xd += push.0;
        particle.yd += push.1;
        particle.zd += push.2;
        let colour = particle.random.next_float() * color_random;
        particle.r_col = colour;
        particle.g_col = colour;
        particle.b_col = colour;
        particle.quad_size *= 0.75 * scale;
        particle.lifetime = (f64::from(max_lifetime)
            / (f64::from(particle.random.next_float()) * 0.8 + 0.2)
            * f64::from(scale)) as i32;
        particle.lifetime = particle.lifetime.max(1);
        particle.set_sprite_from_age();
        particle.has_physics = has_physics;
        particle
    }

    /// `SmokeParticle.Provider` and `LargeSmokeParticle.Provider` (two and a half times the
    /// size): smoke, which goes where it is pushed and rises.
    pub fn smoke(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        scale: f32,
    ) -> Self {
        Self::ash_smoke(
            random,
            at,
            (0.1, 0.1, 0.1),
            push,
            (scale, sprites),
            (0.3, 8, -0.1, true),
        )
    }

    /// `WhiteSmokeParticle.Provider`: smoke tinted a pale grey.
    pub fn white_smoke(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let mut particle = Self::smoke(random, at, push, sprites, 1.0);
        (particle.r_col, particle.g_col, particle.b_col) = (0.729_411_8, 0.694_117_67, 0.760_784_3);
        particle
    }

    /// `AshParticle.Provider`: ash that drifts down, whatever it was asked with.
    pub fn ash(random: RandomSource, at: (f64, f64, f64), sprites: &Arc<SpriteSet>) -> Self {
        Self::ash_smoke(
            random,
            at,
            (0.1, -0.1, 0.1),
            (0.0, 0.0, 0.0),
            (1.0, sprites),
            (0.5, 20, 0.1, false),
        )
    }

    /// `WhiteAshParticle.Provider`: pale ash pushed down and back by the provider's random.
    pub fn white_ash(
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let mut draw = || f64::from(provider_random.next_float());
        let xa = draw() * -1.9 * draw() * 0.1;
        let ya = draw() * -0.5 * draw() * 0.1 * 5.0;
        let za = draw() * -1.9 * draw() * 0.1;
        let mut particle = Self::ash_smoke(
            random,
            at,
            (0.1, -0.1, 0.1),
            (xa, ya, za),
            (1.0, sprites),
            (0.0, 20, 0.0125, false),
        );
        let [r, g, b] = WHITE_ASH_RGB;
        (particle.r_col, particle.g_col, particle.b_col) = (r, g, b);
        particle
    }

    /// `DustPlumeParticle.Provider`: a puff that bursts up and slows each tick, pale and a
    /// little darker at random.
    pub fn dust_plume(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let mut particle = Self::ash_smoke(
            random,
            at,
            (0.7, 0.6, 0.7),
            (push.0, push.1 + f64::from(0.15_f32), push.2),
            (1.0, sprites),
            (0.5, 7, 0.5, false),
        );
        particle.kind = Kind::AshSmoke { decays: true };
        let shift = particle.random.next_float() * 0.2;
        let [r, g, b] = WHITE_ASH_RGB;
        (particle.r_col, particle.g_col, particle.b_col) = (r - shift, g - shift, b - shift);
        particle
    }
}

/// `12235202` (`0xBAB1C2`) as `ARGB.red(..) / 255.0F` and the rest.
const WHITE_ASH_RGB: [f32; 3] = [186.0 / 255.0, 177.0 / 255.0, 194.0 / 255.0];
