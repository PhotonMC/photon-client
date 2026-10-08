//! The particles of hits, bursts and moods: `CritParticle`, `AttackSweepParticle`,
//! `ExplodeParticle`, `HugeExplosionParticle`, `SonicBoomParticle`, `HeartParticle`,
//! `NoteParticle`, `GustParticle`, `SculkChargeParticle`, `SculkChargePopParticle`, and the
//! emitters that leave bursts and gusts about them (`HugeExplosionSeedParticle`,
//! `GustSeedParticle`).

use std::sync::Arc;

use photon_core::math::mth_sin_f64;
use photon_core::random::RandomSource;
use photon_data::particle_type;

use super::super::ClientWorld;
use super::engine::{ParticleData, Spawn};
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// The providers of this file, by type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mark {
    Crit,
    DamageIndicator,
    EnchantedHit,
    Sweep,
    Poof,
    Explosion,
    SonicBoom,
    Heart,
    AngryVillager,
    Note,
    Gust,
    SmallGust,
    SculkCharge,
    SculkChargePop,
    ExplosionEmitter,
    GustEmitterLarge,
    GustEmitterSmall,
}

impl Mark {
    /// `ParticleResources.registerProviders`, for these types.
    pub(super) fn of(name: &str) -> Option<Self> {
        Some(match name {
            "crit" => Self::Crit,
            "damage_indicator" => Self::DamageIndicator,
            "enchanted_hit" => Self::EnchantedHit,
            "sweep_attack" => Self::Sweep,
            "poof" => Self::Poof,
            "explosion" => Self::Explosion,
            "sonic_boom" => Self::SonicBoom,
            "heart" => Self::Heart,
            "angry_villager" => Self::AngryVillager,
            "note" => Self::Note,
            "gust" => Self::Gust,
            "small_gust" => Self::SmallGust,
            "sculk_charge" => Self::SculkCharge,
            "sculk_charge_pop" => Self::SculkChargePop,
            "explosion_emitter" => Self::ExplosionEmitter,
            "gust_emitter_large" => Self::GustEmitterLarge,
            "gust_emitter_small" => Self::GustEmitterSmall,
            _ => return None,
        })
    }

    /// Whether its particles are drawn: the emitters (`NoRenderParticle`) are not, and have no
    /// pictures.
    pub(super) fn is_emitter(self) -> bool {
        matches!(
            self,
            Self::ExplosionEmitter | Self::GustEmitterLarge | Self::GustEmitterSmall
        )
    }
}

/// What a `NoRenderParticle` leaves about it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Emitter {
    /// `HugeExplosionSeedParticle`: six explosions each tick for eight ticks.
    Explosion,
    /// `GustSeedParticle`: three gusts every so many ticks, within a spread.
    Gust { scale: f64, delay: i32 },
}

impl Particle {
    /// The provider of a type of this file by its name; nothing for another type, or where its
    /// options are not the ones it reads.
    #[allow(clippy::too_many_arguments)]
    pub fn hit_provider(
        name: &str,
        world: &ClientWorld,
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
        data: ParticleData,
    ) -> Option<Self> {
        let mark = Mark::of(name)?;
        let (xa, ya, za) = push;
        Some(match mark {
            Mark::Crit => Self::crit(world, random, (at, push), sprites, provider_random),
            Mark::DamageIndicator => {
                let push = (xa, ya + 1.0, za);
                let mut particle = Self::crit(world, random, (at, push), sprites, provider_random);
                particle.lifetime = 20;
                particle
            }
            Mark::EnchantedHit => {
                let mut particle = Self::crit(world, random, (at, push), sprites, provider_random);
                particle.r_col *= 0.3;
                particle.g_col *= 0.8;
                particle
            }
            Mark::Sweep => {
                let mut particle = Self::burst(random, at, sprites);
                particle.lifetime = 4;
                particle.burst_colour(0.6, 0.4);
                particle.quad_size = 1.0 - xa as f32 * 0.5;
                particle.set_sprite_from_age();
                particle
            }
            Mark::Poof => Self::poof(random, (at, push), sprites),
            Mark::Explosion | Mark::SonicBoom => {
                let mut particle = Self::burst(random, at, sprites);
                particle.lifetime = 6 + particle.random.next_int_bound(4).unwrap_or(0);
                particle.burst_colour(0.6, 0.4);
                particle.quad_size = 2.0 * (1.0 - xa as f32 * 0.5);
                particle.set_sprite_from_age();
                if mark == Mark::SonicBoom {
                    particle.lifetime = 16;
                    particle.quad_size = 1.5;
                    particle.set_sprite_from_age();
                }
                particle
            }
            Mark::Heart | Mark::AngryVillager => {
                let at = if mark == Mark::AngryVillager {
                    (at.0, at.1 + 0.5, at.2)
                } else {
                    at
                };
                let sprite = sprites.from_random(provider_random);
                let mut particle = Self::bobbing(random, at, (sprites, sprite), 0.86, 0.1);
                particle.lifetime = 16;
                particle.has_physics = false;
                particle
            }
            Mark::Note => {
                let sprite = sprites.from_random(provider_random);
                let mut particle = Self::bobbing(random, at, (sprites, sprite), 0.66, 0.2);
                // `Mth.sin(((float)color + offset) * (float)(Math.PI * 2))`, held above zero.
                let channel = |offset: f32| {
                    let turn = (xa as f32 + offset) * (std::f64::consts::PI * 2.0) as f32;
                    (mth_sin_f64(f64::from(turn)) * 0.65 + 0.35).max(0.0)
                };
                particle.r_col = channel(0.0);
                particle.g_col = channel(0.333_333_34);
                particle.b_col = channel(0.666_666_7);
                particle.lifetime = 6;
                particle
            }
            Mark::Gust | Mark::SmallGust => {
                let mut particle =
                    Self::at(random, at, (sprites, sprites.first()), Kind::Burst).quad();
                particle.set_sprite_from_age();
                particle.lifetime = 12 + particle.random.next_int_bound(4).unwrap_or(0);
                particle.quad_size = 1.0;
                particle.set_size(1.0, 1.0);
                if mark == Mark::SmallGust {
                    particle.scale(0.15);
                }
                particle
            }
            Mark::SculkCharge | Mark::SculkChargePop => {
                return Self::sculk_charge(random, (at, push), sprites, provider_random, data);
            }
            Mark::ExplosionEmitter | Mark::GustEmitterLarge | Mark::GustEmitterSmall => {
                Self::emitter(mark, random, at, sprites)
            }
        })
    }

    /// `SculkChargeParticle.Provider` (with its roll) and `SculkChargePopParticle.Provider`.
    fn sculk_charge(
        random: RandomSource,
        (at, (xa, ya, za)): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
        data: ParticleData,
    ) -> Option<Self> {
        let roll = match data {
            ParticleData::Roll(roll) => Some(roll),
            ParticleData::Plain => None,
            _ => return None,
        };
        let push = (xa, ya, za);
        let mut particle = Self::thrown(
            random,
            at,
            push,
            (sprites, sprites.first()),
            Kind::SculkCharge,
        )
        .quad();
        particle.friction = 0.96;
        particle.scale(if roll.is_some() { 1.5 } else { 1.0 });
        particle.has_physics = false;
        particle.set_sprite_from_age();
        particle.alpha = 1.0;
        (particle.xd, particle.yd, particle.zd) = push;
        if let Some(roll) = roll {
            (particle.o_roll, particle.roll) = (roll, roll);
        }
        particle.lifetime = if roll.is_some() {
            provider_random.next_int_bound(12).unwrap_or(0) + 8
        } else {
            provider_random.next_int_bound(4).unwrap_or(0) + 6
        };
        Some(particle)
    }

    /// `HugeExplosionSeedParticle(..)` and `GustSeedParticle(..)` with their providers' spread,
    /// life and pause.
    fn emitter(
        mark: Mark,
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let (emitter, lifetime) = match mark {
            Mark::ExplosionEmitter => (Emitter::Explosion, 8),
            Mark::GustEmitterLarge => (
                Emitter::Gust {
                    scale: 3.0,
                    delay: 0,
                },
                7,
            ),
            _ => (
                Emitter::Gust {
                    scale: 1.0,
                    delay: 2,
                },
                3,
            ),
        };
        let mut particle = Self::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            (sprites, 0),
            Kind::Emitter(emitter),
        );
        particle.lifetime = lifetime;
        particle
    }

    /// `CritParticle(..)`: a spark thrown slow, greying and fading to red, that ticks once as it
    /// is made.
    fn crit(
        world: &ClientWorld,
        random: RandomSource,
        (at, (xa, ya, za)): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        let mut particle =
            Self::thrown(random, at, (0.0, 0.0, 0.0), (sprites, sprite), Kind::Crit).quad();
        particle.friction = 0.7;
        particle.gravity = 0.5;
        particle.xd *= f64::from(0.1_f32);
        particle.yd *= f64::from(0.1_f32);
        particle.zd *= f64::from(0.1_f32);
        particle.xd += xa * 0.4;
        particle.yd += ya * 0.4;
        particle.zd += za * 0.4;
        let colour = particle.random.next_float() * 0.3 + 0.6;
        (particle.r_col, particle.g_col, particle.b_col) = (colour, colour, colour);
        particle.quad_size *= 0.75;
        particle.lifetime =
            ((6.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.6)) as i32).max(1);
        particle.has_physics = false;
        particle.tick(world);
        particle
    }

    /// `ExplodeParticle(..)`: a puff that rises and slows, following its pictures.
    fn poof(
        random: RandomSource,
        (at, (xa, ya, za)): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let mut particle = Self::at(random, at, (sprites, sprites.first()), Kind::Poof).quad();
        particle.gravity = -0.1;
        particle.friction = 0.9;
        let mut spread = || f64::from((particle.random.next_float() * 2.0 - 1.0) * 0.05);
        let (sx, sy, sz) = (spread(), spread(), spread());
        (particle.xd, particle.yd, particle.zd) = (xa + sx, ya + sy, za + sz);
        let colour = particle.random.next_float() * 0.3 + 0.7;
        (particle.r_col, particle.g_col, particle.b_col) = (colour, colour, colour);
        let size = particle.random.next_float() * particle.random.next_float() * 6.0 + 1.0;
        particle.quad_size = 0.1 * size;
        particle.lifetime =
            (16.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32 + 2;
        particle.set_sprite_from_age();
        particle
    }

    /// The still, full bright quads that follow their pictures (`AttackSweepParticle`,
    /// `HugeExplosionParticle`), before their own life and size.
    fn burst(random: RandomSource, at: (f64, f64, f64), sprites: &Arc<SpriteSet>) -> Self {
        Self::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            (sprites, sprites.first()),
            Kind::Burst,
        )
        .quad()
    }

    /// A grey drawn from its own random: `random * spread + low`.
    fn burst_colour(&mut self, spread: f32, low: f32) {
        let colour = self.random.next_float() * spread + low;
        (self.r_col, self.g_col, self.b_col) = (colour, colour, colour);
    }

    /// `HeartParticle(..)` and `NoteParticle(..)`: a sign that bobs up and grows in.
    fn bobbing(
        random: RandomSource,
        at: (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
        friction: f32,
        rise: f64,
    ) -> Self {
        let mut particle = Self::thrown(random, at, (0.0, 0.0, 0.0), sprite, Kind::Bobbing).quad();
        particle.speed_up_when_y_motion_is_blocked = true;
        particle.friction = friction;
        particle.xd *= f64::from(0.01_f32);
        particle.yd *= f64::from(0.01_f32);
        particle.zd *= f64::from(0.01_f32);
        particle.yd += rise;
        particle.quad_size *= 1.5;
        particle
    }

    /// `AttackSweepParticle.tick` and the rest of the still ones: older, a picture further.
    pub(super) fn burst_tick(&mut self) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let aged = self.age >= self.lifetime;
        self.age += 1;
        if aged {
            self.removed = true;
        } else {
            self.set_sprite_from_age();
        }
    }

    /// `HugeExplosionSeedParticle.tick` and `GustSeedParticle.tick`: what the emitter leaves
    /// about it this tick, through the level.
    pub(super) fn emitter_tick(&mut self, emitter: Emitter) -> Vec<Spawn> {
        let (name, count, scale, emits) = match emitter {
            Emitter::Explosion => ("explosion", 6, 4.0, true),
            Emitter::Gust { scale, delay } => ("gust", 3, scale, self.age % (delay + 1) == 0),
        };
        let mut spawns = Vec::new();
        if emits {
            let type_id = particle_type::id(name).expect("a carried type");
            let power = self.age as f32 / self.lifetime as f32;
            for _ in 0..count {
                let mut spread = || (self.random.next_double() - self.random.next_double()) * scale;
                let (sx, sy, sz) = (spread(), spread(), spread());
                let at = (self.x + sx, self.y + sy, self.z + sz);
                spawns.push(Spawn {
                    type_id,
                    data: ParticleData::Plain,
                    override_limiter: false,
                    always_show: false,
                    at,
                    push: (f64::from(power), 0.0, 0.0),
                });
            }
        }
        let age = self.age;
        self.age += 1;
        let end = match emitter {
            Emitter::Explosion => self.age,
            Emitter::Gust { .. } => age,
        };
        if end == self.lifetime {
            self.removed = true;
        }
        spawns
    }
}
