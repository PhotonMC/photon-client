//! `ParticleResources`: which class makes the particles of a type, and the pictures it
//! is given.

use std::collections::HashMap;
use std::sync::Arc;

use photon_core::random::RandomSource;
use photon_data::particle_type;

use super::super::ClientWorld;
use super::particle::Particle;
use super::resources_make::{attract, coloured, geyser, trail};
use super::sprites::SpriteSet;
use super::terrain::TerrainMaterials;

/// The class a type's provider makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Provider {
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
    BlockMarker,
    Crumbling,
    DustPillar,
    Mark(super::hit::Mark),
    Drip,
    FallingDust,
    EndRod,
    FallingLeaves,
    BreakingItem,
    Vibration,
    Trail,
    GasCloud,
    NoxiousGas,
    SulfurBubbles,
    Geyser,
    GeyserBase,
    GeyserPlume,
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
        "end_rod" => Provider::EndRod,
        "vibration" => Provider::Vibration,
        "trail" => Provider::Trail,
        "item" | "item_slime" | "item_cobweb" | "item_snowball" | "sulfur_cube_goo" => {
            Provider::BreakingItem
        }
        "cherry_leaves" | "pale_oak_leaves" | "tinted_leaves" => Provider::FallingLeaves,
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
        _ => {
            block_provider_of(name).or_else(|| Some(Provider::Mark(super::hit::Mark::of(name)?)))?
        }
    })
}

/// Whether the provider makes its particle from the level or the options rather than from a
/// sprite set alone.
fn makes_its_own(provider: Provider) -> bool {
    matches!(
        provider,
        Provider::GasCloud
            | Provider::NoxiousGas
            | Provider::SulfurBubbles
            | Provider::Geyser
            | Provider::GeyserBase
            | Provider::GeyserPlume
            | Provider::FallingDust
            | Provider::Drip
            | Provider::Mark(_)
            | Provider::FallingLeaves
            | Provider::BreakingItem
            | Provider::Vibration
    )
}

/// The providers of the particles made of a block.
fn block_provider_of(name: &str) -> Option<Provider> {
    Some(match name {
        "block" => Provider::Terrain,
        "block_marker" => Provider::BlockMarker,
        "block_crumble" => Provider::Crumbling,
        "dust_pillar" => Provider::DustPillar,
        "falling_dust" => Provider::FallingDust,
        _ => return sulfur_provider_of(name),
    })
}

/// The providers of a potent sulfur block's particles: its gas and bubbles, and its geyser.
fn sulfur_provider_of(name: &str) -> Option<Provider> {
    Some(match name {
        "noxious_gas_cloud" => Provider::GasCloud,
        "noxious_gas" => Provider::NoxiousGas,
        "sulfur_bubbles" => Provider::SulfurBubbles,
        "geyser" => Provider::Geyser,
        "geyser_base" | "geyser_poof" => Provider::GeyserBase,
        "geyser_plume" => Provider::GeyserPlume,
        _ => return None,
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
        for particle in photon_data::particle_type::PARTICLE_TYPES {
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
            Provider::Terrain | Provider::BlockMarker | Provider::Crumbling | Provider::DustPillar
        ) {
            return self.create_terrain(
                world,
                (provider, &data),
                (at, push),
                random,
                engine_random,
            );
        }
        if makes_its_own(provider) {
            return self.create_special(
                provider,
                world,
                (name, data),
                (at, push),
                random,
                (engine_random, level_random),
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
            Provider::EndRod => Particle::end_rod(random, motion, sprites),
            Provider::Trail => trail(data, random, motion, (sprites, engine_random))?,
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

    /// The providers of a potent sulfur block's particles; the rest of those that make their own
    /// are in `create_own`.
    fn create_special(
        &self,
        provider: Provider,
        world: &ClientWorld,
        (name, data): (&str, super::engine::ParticleData),
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        random: impl FnOnce() -> RandomSource,
        (engine_random, level_random): (&mut RandomSource, &mut RandomSource),
    ) -> Option<Particle> {
        let sprites = self.sprite_sets.get(name);
        match (provider, sprites) {
            (Provider::GasCloud, _) => Some(Particle::gas_cloud(random(), at)),
            (Provider::NoxiousGas, Some(sprites)) => {
                Some(Particle::noxious_gas(random(), at, push, sprites))
            }
            (Provider::SulfurBubbles, Some(sprites)) => Some(Particle::sulfur_bubble(
                random(),
                (at, push),
                sprites,
                engine_random,
            )),
            (Provider::NoxiousGas | Provider::SulfurBubbles, None) => None,
            (Provider::Geyser | Provider::GeyserBase | Provider::GeyserPlume, _) => geyser(
                (provider, data),
                (at, push),
                sprites,
                random,
                (engine_random, level_random),
            ),
            _ => self.create_own(
                provider,
                world,
                (name, data),
                (at, push),
                random,
                engine_random,
            ),
        }
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
        if provider == Provider::BreakingItem {
            // Only the goo has pictures of its own; the items' come from their models.
            let none = Arc::new(SpriteSet::new(Vec::new()));
            let sprites = match self.sprite_sets.get(name) {
                Some(sprites) => sprites,
                None if name != "sulfur_cube_goo" => &none,
                None => return None,
            };
            let materials = self.terrain.as_deref();
            let random = random();
            let options = (data, sprites);
            return Particle::breaking_item_provider(
                name,
                random,
                (at, push),
                options,
                (materials, engine_random),
            );
        }
        if provider == Provider::FallingLeaves {
            let sprites = self.sprite_sets.get(name)?;
            let options = (engine_random, data);
            return Particle::falling_leaves_provider(name, random(), at, sprites, options);
        }
        if provider == Provider::Drip {
            let sprites = self.sprite_sets.get(name)?;
            return Particle::drip_provider(name, random(), at, sprites, engine_random);
        }
        if provider == Provider::Vibration {
            let super::engine::ParticleData::Vibration {
                destination,
                arrival_in_ticks,
            } = data
            else {
                return None;
            };
            let sprites = self.sprite_sets.get(name)?;
            let sprite = sprites.from_random(engine_random);
            return Some(Particle::vibration(
                random(),
                at,
                destination,
                arrival_in_ticks,
                (sprites, sprite),
                world,
            ));
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
        if provider == Provider::BlockMarker {
            return Some(Particle::block_marker(random(), at, state, materials));
        }
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
