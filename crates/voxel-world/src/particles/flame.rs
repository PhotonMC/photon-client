//! `FlameParticle` on `RisingParticle`.

use std::sync::Arc;

use voxel_core::random::RandomSource;

use super::particle::Particle;
use super::sprites::SpriteSet;

impl Particle {
    /// `FlameParticle.Provider`: its picture is any of the set, from the engine's random source.
    pub fn flame(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        engine_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(engine_random);
        Self::rising_flame(random, at, push, (sprites, sprite))
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
