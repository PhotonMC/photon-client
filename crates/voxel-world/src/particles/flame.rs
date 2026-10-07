//! `FlameParticle` on `RisingParticle`.

use std::sync::Arc;

use voxel_core::random::RandomSource;

use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

impl Particle {
    /// `FlameParticle(..)` through `RisingParticle(..)`: a slow rise from a spot a little
    /// off where it was asked for, living a while longer than most.
    fn flame_of(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
    ) -> Self {
        let mut particle = Self::thrown(random, at, push, sprite, Kind::Flame).quad();
        particle.friction = 0.96;
        particle.xd = particle.xd * f64::from(0.01_f32) + push.0;
        particle.yd = particle.yd * f64::from(0.01_f32) + push.1;
        particle.zd = particle.zd * f64::from(0.01_f32) + push.2;
        particle.x +=
            f64::from((particle.random.next_float() - particle.random.next_float()) * 0.05);
        particle.y +=
            f64::from((particle.random.next_float() - particle.random.next_float()) * 0.05);
        particle.z +=
            f64::from((particle.random.next_float() - particle.random.next_float()) * 0.05);
        particle.lifetime =
            (8.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32 + 4;
        particle
    }

    /// `FlameParticle.Provider`: its picture is any of the set, from the engine's random source.
    pub fn flame(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        engine_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(engine_random);
        Self::flame_of(random, at, push, (sprites, sprite))
    }

    /// `FlameParticle.SmallFlameProvider`: half the size.
    pub fn small_flame(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        engine_random: &mut RandomSource,
    ) -> Self {
        let mut particle = Self::flame(random, at, push, sprites, engine_random);
        particle.scale(0.5);
        particle
    }
}
