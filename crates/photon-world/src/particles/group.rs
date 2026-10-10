//! `ParticleGroup`: the particles drawn the same way, ticked in the order they were added.

use std::collections::VecDeque;

use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::engine::Emitted;
use super::particle::Particle;

/// `ParticleGroup.MAX_PARTICLES`.
const MAX_PARTICLES: usize = 16_384;
/// `ParticleGroup.RESERVOIR_START`: from this many on, a particle is let in with a falling chance.
const RESERVOIR_START: usize = 12_288;
/// `ParticleGroup.RESERVOIR_SIZE`.
const RESERVOIR_SIZE: f32 = 4096.0;

#[derive(Debug, Clone)]
pub(super) struct ParticleGroup<T = Particle> {
    particles: VecDeque<T>,
}

impl<T> Default for ParticleGroup<T> {
    fn default() -> Self {
        Self {
            particles: VecDeque::new(),
        }
    }
}

impl ParticleGroup {
    /// `ParticleGroup.tickParticles`: each particle ticks, and the ones that died are dropped;
    /// what they asked the level for, in order, and how many limited ones died.
    pub(super) fn tick(
        &mut self,
        world: &ClientWorld,
        camera: photon_core::math::Vec3,
    ) -> (Vec<Emitted>, i32) {
        let mut spawns = Vec::new();
        let mut limited_gone = 0;
        self.particles.retain_mut(|particle| {
            let asked = if particle.is_firework_starter() {
                particle.firework_starter_tick(camera)
            } else {
                particle.tick(world)
            };
            spawns.extend(asked.into_iter().map(Emitted::Particle));
            if let Some(sound) = particle.take_sound() {
                spawns.push(Emitted::Sound(sound));
            }
            let alive = particle.is_alive();
            if !alive && particle.is_limited() {
                limited_gone += 1;
            }
            alive
        });
        (spawns, limited_gone)
    }
}

impl<T> ParticleGroup<T> {
    /// Each one ticks, and the ones that are done are dropped.
    pub(super) fn retain_mut(&mut self, keep: impl FnMut(&mut T) -> bool) {
        self.particles.retain_mut(keep);
    }

    /// `ParticleGroup.add`: whether the particle got in. Past the reservoir's start the chance
    /// is the free part of it, squared.
    pub(super) fn add(&mut self, particle: T, random: &mut RandomSource) -> bool {
        let size = self.particles.len();
        if size >= MAX_PARTICLES {
            return false;
        }
        if size >= RESERVOIR_START {
            let free_space = (MAX_PARTICLES - size) as f32 / RESERVOIR_SIZE;
            if random.next_float() >= free_space * free_space {
                return false;
            }
        }
        self.particles.push_back(particle);
        true
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
        self.particles.iter()
    }

    pub(super) fn len(&self) -> usize {
        self.particles.len()
    }

    pub(super) fn clear(&mut self) {
        self.particles.clear();
    }
}
