//! `ParticleResources`: which class makes the particles of a type, and the pictures it
//! is given.

use std::collections::HashMap;
use std::sync::Arc;

use voxel_core::random::RandomSource;
use voxel_data::particle_type;

use super::particle::Particle;
use super::sprites::SpriteSet;

/// The class a type's provider makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    Smoke,
    Flame,
    SmallFlame,
}

/// `ParticleResources.registerProviders`, for the types whose particles are carried.
fn provider_of(name: &str) -> Option<Provider> {
    Some(match name {
        "smoke" => Provider::Smoke,
        "flame" | "soul_fire_flame" | "copper_fire_flame" => Provider::Flame,
        "small_flame" => Provider::SmallFlame,
        _ => return None,
    })
}

/// The providers, each with the sprite set its `particles/<name>.json` lists.
#[derive(Debug, Clone, Default)]
pub struct ParticleResources {
    sprite_sets: HashMap<&'static str, Arc<SpriteSet>>,
}

impl ParticleResources {
    pub fn new() -> Self {
        Self::default()
    }

    /// The textures every bound type lists, each once, in the order they are listed.
    pub fn texture_names(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for particle in voxel_data::particle_type::PARTICLE_TYPES {
            if let Some(sprites) = self.sprite_sets.get(particle.name) {
                for name in sprites.names() {
                    if !names.contains(name) {
                        names.push(name.clone());
                    }
                }
            }
        }
        names
    }

    /// Binds the textures of a type (`MutableSpriteSet.rebind`); a type that is not carried is ignored.
    pub fn set_sprites(&mut self, name: &str, sprites: SpriteSet) {
        if let Some(entry) = particle_type::PARTICLE_TYPES
            .iter()
            .find(|candidate| candidate.name == name && provider_of(candidate.name).is_some())
        {
            self.sprite_sets.insert(entry.name, Arc::new(sprites));
        }
    }

    /// `ParticleProvider.createParticle` of the type with this id: nothing where the type has no
    /// provider here or no sprite set was bound.
    pub(super) fn create(
        &self,
        type_id: i32,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        random: impl FnOnce() -> RandomSource,
        engine_random: &mut RandomSource,
    ) -> Option<Particle> {
        let name = particle_type::name(type_id)?;
        let provider = provider_of(name)?;
        let sprites = self.sprite_sets.get(name)?;
        let random = random();
        Some(match provider {
            Provider::Smoke => Particle::smoke(random, at, push, sprites),
            Provider::Flame => Particle::flame(random, at, push, sprites, engine_random),
            Provider::SmallFlame => Particle::small_flame(random, at, push, sprites, engine_random),
        })
    }
}
