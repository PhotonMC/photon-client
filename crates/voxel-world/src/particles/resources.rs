//! `ParticleResources`: which class makes the particles of a type, and the pictures it
//! is given.

use std::collections::HashMap;
use std::sync::Arc;

use voxel_core::random::RandomSource;
use voxel_data::particle_type;

use super::super::ClientWorld;
use super::particle::Particle;
use super::sprites::SpriteSet;
use super::terrain::TerrainMaterials;

/// The class a type's provider makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    Smoke,
    LargeSmoke,
    WhiteSmoke,
    Ash,
    WhiteAsh,
    DustPlume,
    Flame,
    SmallFlame,
    Soul,
    SculkSoul,
    Lava,
    CosySmoke,
    SignalSmoke,
    Cloud,
    Sneeze,
    Bubble,
    BubbleColumnUp,
    BubblePop,
    Rain,
    Splash,
    Fishing,
    CurrentDown,
    SquidInk,
    GlowSquidInk,
    Terrain,
    Crumbling,
    DustPillar,
}

/// `ParticleResources.registerProviders`, for the types whose particles are carried.
fn provider_of(name: &str) -> Option<Provider> {
    Some(match name {
        "smoke" => Provider::Smoke,
        "large_smoke" => Provider::LargeSmoke,
        "white_smoke" => Provider::WhiteSmoke,
        "ash" => Provider::Ash,
        "white_ash" => Provider::WhiteAsh,
        "dust_plume" => Provider::DustPlume,
        "flame" | "soul_fire_flame" | "copper_fire_flame" => Provider::Flame,
        "small_flame" => Provider::SmallFlame,
        "soul" => Provider::Soul,
        "sculk_soul" => Provider::SculkSoul,
        "lava" => Provider::Lava,
        "campfire_cosy_smoke" => Provider::CosySmoke,
        "campfire_signal_smoke" => Provider::SignalSmoke,
        "cloud" => Provider::Cloud,
        "sneeze" => Provider::Sneeze,
        "bubble" => Provider::Bubble,
        "bubble_column_up" => Provider::BubbleColumnUp,
        "bubble_pop" => Provider::BubblePop,
        "rain" => Provider::Rain,
        "splash" => Provider::Splash,
        "fishing" => Provider::Fishing,
        "current_down" => Provider::CurrentDown,
        "squid_ink" => Provider::SquidInk,
        "glow_squid_ink" => Provider::GlowSquidInk,
        "block" => Provider::Terrain,
        "block_crumble" => Provider::Crumbling,
        "dust_pillar" => Provider::DustPillar,
        _ => return None,
    })
}

/// The providers, each with the sprite set its `particles/*.json` lists.
#[derive(Debug, Clone, Default)]
pub struct ParticleResources {
    sprite_sets: HashMap<&'static str, Arc<SpriteSet>>,
    terrain: Option<Arc<dyn TerrainMaterials>>,
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

    /// The block models and tints the flakes of blocks take their pictures and colours from.
    pub fn set_terrain(&mut self, materials: Arc<dyn TerrainMaterials>) {
        self.terrain = Some(materials);
    }

    /// What the flakes of blocks are made with.
    pub(super) fn terrain(&self) -> Option<&Arc<dyn TerrainMaterials>> {
        self.terrain.as_ref()
    }

    /// `ParticleProvider.createParticle` of the type with this id: nothing where the type has no
    /// provider here, no sprite set was bound, or the provider makes none.
    pub(super) fn create(
        &self,
        world: &ClientWorld,
        (type_id, block_state): (i32, Option<i32>),
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        random: impl FnOnce() -> RandomSource,
        engine_random: &mut RandomSource,
    ) -> Option<Particle> {
        let name = particle_type::name(type_id)?;
        let provider = provider_of(name)?;
        if matches!(
            provider,
            Provider::Terrain | Provider::Crumbling | Provider::DustPillar
        ) {
            let materials = self.terrain.as_deref()?;
            let state = block_state?;
            if !super::terrain::makes_terrain_particles(state) {
                return None;
            }
            let env = (world, materials);
            let random = random();
            return match provider {
                Provider::Terrain => Particle::terrain_provider(random, (at, push), state, env),
                Provider::Crumbling => {
                    Particle::crumbling_provider(random, (at, push), state, env, engine_random)
                }
                _ => Particle::dust_pillar_provider(random, (at, push), state, env, engine_random),
            };
        }
        let sprites = self.sprite_sets.get(name)?;
        let random = random();
        Some(match provider {
            Provider::Smoke => Particle::smoke(random, at, push, sprites, 1.0),
            Provider::LargeSmoke => Particle::smoke(random, at, push, sprites, 2.5),
            Provider::WhiteSmoke => Particle::white_smoke(random, at, push, sprites),
            Provider::Ash => Particle::ash(random, at, sprites),
            Provider::WhiteAsh => Particle::white_ash(random, at, sprites, engine_random),
            Provider::DustPlume => Particle::dust_plume(random, at, push, sprites),
            Provider::Flame => Particle::flame(random, at, push, sprites, engine_random),
            Provider::Soul => Particle::soul(random, at, push, sprites, false),
            Provider::SculkSoul => Particle::soul(random, at, push, sprites, true),
            Provider::Lava => Particle::lava(random, at, sprites, engine_random),
            Provider::CosySmoke => {
                Particle::campfire_smoke(random, at, push, (sprites, false), engine_random)
            }
            Provider::SignalSmoke => {
                Particle::campfire_smoke(random, at, push, (sprites, true), engine_random)
            }
            Provider::Cloud => Particle::player_cloud(random, at, push, sprites, false),
            Provider::Sneeze => Particle::player_cloud(random, at, push, sprites, true),
            Provider::Bubble => Particle::bubble(random, (at, push), sprites, engine_random),
            Provider::BubbleColumnUp => {
                Particle::bubble_column_up(random, (at, push), sprites, engine_random)
            }
            Provider::BubblePop => Particle::bubble_pop(random, (at, push), sprites),
            Provider::Rain => Particle::rain(random, at, sprites, engine_random),
            Provider::Splash => Particle::splash(random, (at, push), sprites, engine_random),
            Provider::Fishing => Particle::wake(random, (at, push), sprites),
            Provider::CurrentDown => Particle::current_down(random, at, sprites, engine_random),
            Provider::SquidInk => Particle::squid_ink(random, (at, push), sprites, false),
            Provider::GlowSquidInk => Particle::squid_ink(random, (at, push), sprites, true),
            _ => Particle::small_flame(random, at, push, sprites, engine_random),
        })
    }
}
