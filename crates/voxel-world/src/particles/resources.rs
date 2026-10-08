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
    Dust,
    DustTransition,
    Spell,
    InstantSpell,
    MobEffect,
    Witch,
    Totem,
    Glow,
    Portal,
    ReversePortal,
    FlyTowards,
    VaultConnection,
    OminousSpawning,
    Suspended,
    SuspendedTown,
    Terrain,
    Crumbling,
    DustPillar,
    Mark(super::hit::Mark),
    Drip,
    FallingDust,
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
        "dust" => Provider::Dust,
        "dust_color_transition" => Provider::DustTransition,
        "infested" | "raid_omen" | "trial_omen" => Provider::Spell,
        "effect" | "instant_effect" => Provider::InstantSpell,
        "entity_effect" => Provider::MobEffect,
        "witch" => Provider::Witch,
        "totem_of_undying" => Provider::Totem,
        "glow" | "electric_spark" | "scrape" | "wax_on" | "wax_off" => Provider::Glow,
        "portal" => Provider::Portal,
        "reverse_portal" => Provider::ReversePortal,
        "enchant" | "nautilus" => Provider::FlyTowards,
        "vault_connection" => Provider::VaultConnection,
        "ominous_spawning" => Provider::OminousSpawning,
        "underwater" | "crimson_spore" | "warped_spore" | "spore_blossom_air" => {
            Provider::Suspended
        }
        "mycelium" | "happy_villager" | "egg_crack" | "composter" | "dolphin" => {
            Provider::SuspendedTown
        }
        "block" => Provider::Terrain,
        "block_crumble" => Provider::Crumbling,
        "dust_pillar" => Provider::DustPillar,
        "falling_dust" => Provider::FallingDust,
        name if name.starts_with("dripping_")
            || name.starts_with("landing_")
            || matches!(
                name,
                "falling_lava"
                    | "falling_water"
                    | "falling_honey"
                    | "falling_nectar"
                    | "falling_spore_blossom"
                    | "falling_obsidian_tear"
                    | "falling_dripstone_water"
                    | "falling_dripstone_lava"
            ) =>
        {
            Provider::Drip
        }
        _ => Provider::Mark(super::hit::Mark::of(name)?),
    })
}

/// The providers, each with the sprite set its `particles/*.json` lists.
#[derive(Debug, Clone)]
pub struct ParticleResources {
    sprite_sets: HashMap<&'static str, Arc<SpriteSet>>,
    terrain: Option<Arc<dyn TerrainMaterials>>,
    /// `SpellParticle.RANDOM`: the class's own random source, which pushes its particles aside.
    spell_random: RandomSource,
}

impl Default for ParticleResources {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleResources {
    pub fn new() -> Self {
        Self {
            sprite_sets: HashMap::new(),
            terrain: None,
            spell_random: RandomSource::create(0),
        }
    }

    /// The spell particles' own random source, seeded (`RandomSource.create()` seeds it from the
    /// clock).
    pub fn set_spell_random(&mut self, random: RandomSource) {
        self.spell_random = random;
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
        &mut self,
        world: &ClientWorld,
        (type_id, data): (i32, super::engine::ParticleData),
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        random: impl FnOnce() -> RandomSource,
        (engine_random, level_random): (&mut RandomSource, &mut RandomSource),
    ) -> Option<Particle> {
        let name = particle_type::name(type_id)?;
        let provider = provider_of(name)?;
        if matches!(
            provider,
            Provider::Terrain | Provider::Crumbling | Provider::DustPillar
        ) {
            return self.create_terrain(
                world,
                (provider, &data),
                (at, push),
                random,
                engine_random,
            );
        }
        if matches!(
            provider,
            Provider::FallingDust | Provider::Drip | Provider::Mark(_)
        ) {
            return self.create_own(
                provider,
                world,
                (name, data),
                (at, push),
                random,
                engine_random,
            );
        }
        let sprites = self.sprite_sets.get(name)?;
        let spell_random = &mut self.spell_random;
        let random = random();
        let motion = (at, push);
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
            Provider::Dust | Provider::DustTransition => {
                coloured(provider, data, random, motion, (sprites, spell_random))?
            }
            Provider::Spell => Particle::spell(random, motion, sprites, spell_random),
            Provider::InstantSpell | Provider::MobEffect => {
                coloured(provider, data, random, motion, (sprites, spell_random))?
            }
            Provider::Witch => {
                Particle::witch(random, motion, sprites, (spell_random, engine_random))
            }
            Provider::Totem => Particle::totem(random, motion, sprites),
            Provider::Glow => Particle::glow(name, random, motion, sprites, engine_random),
            Provider::Portal
            | Provider::ReversePortal
            | Provider::FlyTowards
            | Provider::VaultConnection
            | Provider::OminousSpawning
            | Provider::Suspended
            | Provider::SuspendedTown => attract(
                (provider, name),
                random,
                motion,
                sprites,
                (engine_random, level_random),
            ),
            _ => Particle::small_flame(random, at, push, sprites, engine_random),
        })
    }

    /// The providers that make their own particle from the level or the options: falling dust,
    /// drips, and the hits and bursts.
    fn create_own(
        &self,
        provider: Provider,
        world: &ClientWorld,
        (name, data): (&str, super::engine::ParticleData),
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        random: impl FnOnce() -> RandomSource,
        engine_random: &mut RandomSource,
    ) -> Option<Particle> {
        if provider == Provider::FallingDust {
            let state = data.block_state()?;
            let sprites = self.sprite_sets.get(name)?;
            let materials = self.terrain.as_deref();
            return Particle::falling_dust_provider(
                random,
                at,
                (state, sprites),
                (world, materials),
            );
        }
        if provider == Provider::Drip {
            let sprites = self.sprite_sets.get(name)?;
            return Particle::drip_provider(name, random(), at, sprites, engine_random);
        }
        if let Provider::Mark(mark) = provider {
            let sprites = if mark.is_emitter() {
                // The emitters have no pictures.
                &Arc::new(SpriteSet::new(Vec::new()))
            } else {
                self.sprite_sets.get(name)?
            };
            let motion = (at, push);
            return Particle::hit_provider(
                name,
                world,
                random(),
                motion,
                sprites,
                engine_random,
                data,
            );
        }
        None
    }

    /// The providers of flakes of blocks: nothing without the materials or for a block that
    /// makes none.
    fn create_terrain(
        &self,
        world: &ClientWorld,
        (provider, data): (Provider, &super::engine::ParticleData),
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        random: impl FnOnce() -> RandomSource,
        engine_random: &mut RandomSource,
    ) -> Option<Particle> {
        let materials = self.terrain.as_deref()?;
        let state = data.block_state()?;
        if !super::terrain::makes_terrain_particles(state) {
            return None;
        }
        let env = (world, materials);
        let random = random();
        match provider {
            Provider::Terrain => Particle::terrain_provider(random, (at, push), state, env),
            Provider::Crumbling => {
                Particle::crumbling_provider(random, (at, push), state, env, engine_random)
            }
            _ => Particle::dust_pillar_provider(random, (at, push), state, env, engine_random),
        }
    }
}

/// The providers of the particles drawn to a place or hung in the air (`attract.rs`).
fn attract(
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

/// The providers that read the colours their options carry: nothing for options of another kind.
fn coloured(
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
