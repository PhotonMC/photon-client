//! `SmokeParticle` on `BaseAshSmokeParticle`.

use std::sync::Arc;

use voxel_core::random::RandomSource;

use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

impl Particle {
    /// `BaseAshSmokeParticle(..)`: a particle that rises, darkens at random and follows
    /// its sprite set as it ages.
    #[allow(clippy::too_many_arguments)]
    fn ash_smoke(
        random: RandomSource,
        at: (f64, f64, f64),
        direction: (f32, f32, f32),
        push: (f64, f64, f64),
        scale: f32,
        sprites: &Arc<SpriteSet>,
        (color_random, max_lifetime, gravity, has_physics): (f32, i32, f32, bool),
    ) -> Self {
        let mut particle = Self::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            (sprites, sprites.first()),
            Kind::AshSmoke,
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

    /// `SmokeParticle.Provider`: smoke, which goes where it is pushed and rises.
    pub fn smoke(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        Self::ash_smoke(
            random,
            at,
            (0.1, 0.1, 0.1),
            push,
            1.0,
            sprites,
            (0.3, 8, -0.1, true),
        )
    }
}
