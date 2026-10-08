//! `ParticleGroup`: the particles drawn the same way, ticked in the order they were added.

use std::collections::VecDeque;

use voxel_core::random::RandomSource;

use super::super::ClientWorld;
use super::engine::Spawn;
use super::particle::Particle;

/// `ParticleGroup.MAX_PARTICLES`.
const MAX_PARTICLES: usize = 16_384;
/// `ParticleGroup.RESERVOIR_START`: from this many on, a particle is let in with a falling chance.
const RESERVOIR_START: usize = 12_288;
/// `ParticleGroup.RESERVOIR_SIZE`.
const RESERVOIR_SIZE: f32 = 4096.0;

#[derive(Debug, Clone, Default)]
pub(super) struct ParticleGroup {
    particles: VecDeque<Particle>,
}

impl ParticleGroup {
    /// `ParticleGroup.tickParticles`: each particle ticks, and the ones that died are dropped;
    /// what they asked the level for, in order.
    pub(super) fn tick(&mut self, world: &ClientWorld) -> Vec<Spawn> {
        let mut spawns = Vec::new();
        self.particles.retain_mut(|particle| {
            spawns.extend(particle.tick(world));
            particle.is_alive()
        });
        spawns
    }

    /// `ParticleGroup.add`: whether the particle got in. Past the reservoir's start the chance
    /// is the free part of it, squared.
    pub(super) fn add(&mut self, particle: Particle, random: &mut RandomSource) -> bool {
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

    pub(super) fn iter(&self) -> impl Iterator<Item = &Particle> {
        self.particles.iter()
    }

    pub(super) fn len(&self) -> usize {
        self.particles.len()
    }

    pub(super) fn clear(&mut self) {
        self.particles.clear();
    }
}
