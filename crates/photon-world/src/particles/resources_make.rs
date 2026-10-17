//! The providers of `ParticleResources` that read what their options carry or that make a family
//! of particles: the ones drawn to a place, trails, and the coloured.

use std::sync::Arc;

use photon_core::random::RandomSource;

use super::particle::Particle;
use super::resources::Provider;
use super::sprites::SpriteSet;

/// The providers of the particles drawn to a place or hung in the air (`attract.rs`).
pub(super) fn attract(
    (provider, name): (Provider, &str),
    random: RandomSource,
    motion: ((f64, f64, f64), (f64, f64, f64)),
    sprites: &Arc<SpriteSet>,
    (engine_random, level_random): (&mut RandomSource, &mut RandomSource),
) -> Particle {
    match provider {
        Provider::Portal => Particle::portal(random, motion, sprites, engine_random, false),
        Provider::ReversePortal => Particle::portal(random, motion, sprites, engine_random, true),
        Provider::FlyTowards => {
            Particle::fly_towards_provider(random, motion, sprites, engine_random, false)
        }
        Provider::VaultConnection => {
            Particle::fly_towards_provider(random, motion, sprites, engine_random, true)
        }
        Provider::OminousSpawning => {
            Particle::ominous_spawning(random, motion, sprites, (engine_random, level_random))
        }
        Provider::Suspended => Particle::suspended(name, random, motion.0, sprites, engine_random),
        // `Provider::SuspendedTown`.
        _ => Particle::suspended_town(name, random, motion, sprites, (engine_random, level_random)),
    }
}

/// `TrailParticle.Provider`: nothing for options of another kind.
pub(super) fn trail(
    data: super::engine::ParticleData,
    random: RandomSource,
    motion: ((f64, f64, f64), (f64, f64, f64)),
    (sprites, engine_random): (&Arc<SpriteSet>, &mut RandomSource),
) -> Option<Particle> {
    let super::engine::ParticleData::Trail {
        target,
        color,
        duration,
    } = data
    else {
        return None;
    };
    let sprite = sprites.from_random(engine_random);
    Some(Particle::trail(
        random,
        motion,
        target,
        color,
        duration,
        (sprites, sprite),
    ))
}

/// The providers that read the colours their options carry: nothing for options of another kind.
pub(super) fn coloured(
    provider: Provider,
    data: super::engine::ParticleData,
    random: RandomSource,
    motion: ((f64, f64, f64), (f64, f64, f64)),
    (sprites, spell_random): (&Arc<SpriteSet>, &mut RandomSource),
) -> Option<Particle> {
    Some(match provider {
        Provider::Dust => {
            let super::engine::ParticleData::Dust { color, scale } = data else {
                return None;
            };
            Particle::dust(random, motion, sprites, (color, scale))
        }
        Provider::DustTransition => {
            let super::engine::ParticleData::DustTransition { from, to, scale } = data else {
                return None;
            };
            Particle::dust_transition(random, motion, sprites, (from, to, scale))
        }
        Provider::InstantSpell => {
            let super::engine::ParticleData::Spell { color, power } = data else {
                return None;
            };
            Particle::instant_spell(random, motion, sprites, spell_random, (color, power))
        }
        Provider::MobEffect => {
            let super::engine::ParticleData::Color(color) = data else {
                return None;
            };
            Particle::mob_effect(random, motion, sprites, spell_random, color)
        }
        _ => return None,
    })
}
