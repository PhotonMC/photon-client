//! `ClientboundExplodePacket` on the client: its sound and particle at once, and the block
//! particles `ClientExplosionTracker` scatters about it on the level's next tick.

use photon_core::random::RandomSource;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::engine::{ParticleData, ParticleEngine, ParticleStatus, Spawn};
use super::level_events::LevelEventContext;

/// One kind of particle an explosion scatters (`ExplosionParticleInfo`), with its weight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionParticle {
    pub type_id: i32,
    pub data: ParticleData,
    pub scaling: f32,
    pub speed: f32,
    pub weight: i32,
}

/// An explosion the server reports.
#[derive(Debug, Clone, PartialEq)]
pub struct Explosion {
    pub center: (f64, f64, f64),
    pub radius: f32,
    pub block_count: i32,
    /// The particle at its centre.
    pub particle: (i32, ParticleData),
    /// Its sound; none for one the client does not know by name.
    pub sound: Option<&'static str>,
    pub block_particles: Vec<ExplosionParticle>,
}

/// `ClientExplosionTracker.MAX_PARTICLES_PER_TICK`.
const MAX_PARTICLES_PER_TICK: i32 = 512;

impl ParticleEngine {
    /// `ClientPacketListener.handleExplosion`'s part of the level: the sound, pitched by the level's
    /// random, the particle at the centre, and the blocks' particles kept for the next tick.
    pub(super) fn explode<F: FnMut() -> RandomSource>(
        &mut self,
        explosion: Explosion,
        context: &mut LevelEventContext<'_, F>,
    ) -> Vec<LocalSound> {
        let mut sounds = Vec::new();
        let random = &mut *context.level_random;
        let pitch = (1.0 + (random.next_float() - random.next_float()) * 0.2) * 0.7;
        // `playLocalSound`'s seed.
        random.next_long();
        if let Some(event) = explosion.sound {
            sounds.push(LocalSound {
                relative: false,
                delay_ticks: 0,
                follows_player: false,
                event,
                source: LocalSoundSource::Blocks,
                volume: 4.0,
                pitch,
                position: explosion.center,
            });
        }
        let (type_id, data) = explosion.particle;
        let spawn = Spawn {
            type_id,
            data,
            override_limiter: false,
            always_show: false,
            at: explosion.center,
            push: (1.0, 0.0, 0.0),
        };
        let world = context.world;
        self.add_particle(
            world,
            &spawn,
            context.view,
            context.level_random,
            &mut context.particle_random,
        );
        // `ClientExplosionTracker.track`.
        if !explosion.block_particles.is_empty() {
            self.explosions.push(explosion);
        }
        sounds
    }

    /// `ClientExplosionTracker.tick`, in the level's tick: with every particle let in, up to 512
    /// particles, each about an explosion picked by its block count.
    pub fn tick_explosions<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let explosions = std::mem::take(&mut self.explosions);
        if context.view.status != ParticleStatus::All {
            return;
        }
        let total: i32 = explosions
            .iter()
            .map(|explosion| explosion.block_count)
            .sum();
        for _ in 0..total.min(MAX_PARTICLES_PER_TICK) {
            // `WeightedRandom.getRandomItem`.
            if total <= 0 {
                break;
            }
            let mut index = context.level_random.next_int_bound(total).unwrap_or(0);
            let picked = explosions.iter().find(|explosion| {
                index -= explosion.block_count;
                index < 0
            });
            if let Some(explosion) = picked {
                self.explosion_particle(explosion, context);
            }
        }
    }

    /// `ClientExplosionTracker.addParticle`: somewhere within the radius, where there is air, a
    /// particle of the explosion's kinds flies out.
    fn explosion_particle<F: FnMut() -> RandomSource>(
        &mut self,
        explosion: &Explosion,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let random = &mut *context.level_random;
        let mut unit = || f64::from(random.next_float() * 2.0 - 1.0);
        let (dx, dy, dz) = (unit(), unit(), unit());
        let length = (dx * dx + dy * dy + dz * dz).sqrt();
        // `Vec3.normalize`: nothing for a length under 1.0E-5.
        let direction = if length < 1.0E-5 {
            (0.0, 0.0, 0.0)
        } else {
            (dx / length, dy / length, dz / length)
        };
        let radius = (f64::from(random.next_float()).cbrt() as f32) * explosion.radius;
        let local = (
            direction.0 * f64::from(radius),
            direction.1 * f64::from(radius),
            direction.2 * f64::from(radius),
        );
        let (cx, cy, cz) = explosion.center;
        let at = [cx + local.0, cy + local.1, cz + local.2].map(photon_core::math::floor_f64);
        let state = context.world.block_state_or_air(at[0], at[1], at[2]);
        if !crate::block::state_or_air(state).block().air {
            return;
        }
        let speed =
            0.5 / (radius / explosion.radius + 0.1) * random.next_float() * random.next_float()
                + 0.3;
        // `WeightedList.getRandomOrThrow`.
        let total: i32 = explosion
            .block_particles
            .iter()
            .map(|kind| kind.weight)
            .sum();
        let mut index = random.next_int_bound(total.max(1)).unwrap_or(0);
        let Some(kind) = explosion.block_particles.iter().find(|kind| {
            index -= kind.weight;
            index < 0
        }) else {
            return;
        };
        let scaling = f64::from(kind.scaling);
        let velocity = f64::from(speed * kind.speed);
        let spawn = Spawn {
            type_id: kind.type_id,
            data: kind.data,
            override_limiter: false,
            always_show: false,
            at: (
                cx + local.0 * scaling,
                cy + local.1 * scaling,
                cz + local.2 * scaling,
            ),
            push: (
                direction.0 * velocity,
                direction.1 * velocity,
                direction.2 * velocity,
            ),
        };
        self.add_particle(
            context.world,
            &spawn,
            context.view,
            context.level_random,
            &mut context.particle_random,
        );
    }
}
