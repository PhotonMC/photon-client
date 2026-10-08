//! The particles that are drawn to a place or hang in the air: `PortalParticle`,
//! `ReversePortalParticle`, `FlyTowardsPositionParticle` (`enchant`, `nautilus`,
//! `vault_connection`), `FlyStraightTowardsParticle` (`ominous_spawning`), `SuspendedParticle`
//! and `SuspendedTownParticle`.

use std::sync::Arc;

use photon_core::random::RandomSource;

use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// What a particle drawn to a place keeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Attract {
    /// `PortalParticle` (`reverse`: `ReversePortalParticle`): from where it started, along its
    /// motion.
    Portal { start: [f64; 3], reverse: bool },
    /// `FlyTowardsPositionParticle`: toward where it started; a glowing one fades by its
    /// `LifetimeAlpha` (start, end, from, to).
    FlyTowards {
        start: [f64; 3],
        glowing: bool,
        alpha: [f32; 4],
    },
    /// `FlyStraightTowardsParticle`: straight to where it started, between two colours.
    FlyStraight { start: [f64; 3], colours: [u32; 2] },
    /// `SuspendedTownParticle`: drifts, slowing, through blocks.
    Town,
}

/// `Particle.LifetimeAlpha.ALWAYS_OPAQUE`.
const ALWAYS_OPAQUE: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

impl Particle {
    /// `PortalParticle.Provider` and `ReversePortalParticle.ReversePortalProvider`.
    pub fn portal(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
        reverse: bool,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        let start = [at.0, at.1, at.2];
        let mut particle = Self::at(
            random,
            at,
            (sprites, sprite),
            Kind::Attract(Attract::Portal { start, reverse }),
        )
        .quad();
        (particle.xd, particle.yd, particle.zd) = push;
        particle.quad_size = 0.1 * (particle.random.next_float() * 0.2 + 0.5);
        let brightness = particle.random.next_float() * 0.6 + 0.4;
        (particle.r_col, particle.g_col, particle.b_col) =
            (brightness * 0.9, brightness * 0.3, brightness);
        particle.lifetime = (particle.random.next_float() * 10.0) as i32 + 40;
        if reverse {
            particle.quad_size *= 1.5;
            particle.lifetime = (particle.random.next_float() * 2.0) as i32 + 60;
        }
        particle
    }

    /// `FlyTowardsPositionParticle(..)`: it starts at the place pushed to and comes back.
    fn fly_towards(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprite: (&Arc<SpriteSet>, usize),
        (glowing, alpha): (bool, [f32; 4]),
    ) -> Self {
        let start = [at.0, at.1, at.2];
        let mut particle = Self::at(
            random,
            at,
            sprite,
            Kind::Attract(Attract::FlyTowards {
                start,
                glowing,
                alpha,
            }),
        )
        .quad();
        particle.alpha = alpha[0];
        (particle.xd, particle.yd, particle.zd) = push;
        (particle.xo, particle.yo, particle.zo) = (at.0 + push.0, at.1 + push.1, at.2 + push.2);
        (particle.x, particle.y, particle.z) = (particle.xo, particle.yo, particle.zo);
        particle.quad_size = 0.1 * (particle.random.next_float() * 0.5 + 0.2);
        let brightness = particle.random.next_float() * 0.6 + 0.4;
        (particle.r_col, particle.g_col, particle.b_col) =
            (0.9 * brightness, 0.9 * brightness, brightness);
        particle.has_physics = false;
        particle.lifetime = (particle.random.next_float() * 10.0) as i32 + 30;
        particle
    }

    /// `FlyTowardsPositionParticle.EnchantProvider` and `NautilusProvider`, and
    /// `VaultConnectionProvider` (`vault`: glowing, fading in, half again as big).
    pub fn fly_towards_provider(
        random: RandomSource,
        motion: ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
        vault: bool,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        if vault {
            let mut particle = Self::fly_towards(
                random,
                motion,
                (sprites, sprite),
                (true, [0.0, 0.6, 0.25, 1.0]),
            );
            particle.scale(1.5);
            particle
        } else {
            Self::fly_towards(random, motion, (sprites, sprite), (false, ALWAYS_OPAQUE))
        }
    }

    /// `FlyStraightTowardsParticle.OminousSpawnProvider`: blue to white, three to five times as
    /// big by the level's random.
    pub fn ominous_spawning(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        (provider_random, level_random): (&mut RandomSource, &mut RandomSource),
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        let start = [at.0, at.1, at.2];
        let mut particle = Self::at(
            random,
            at,
            (sprites, sprite),
            Kind::Attract(Attract::FlyStraight {
                start,
                colours: [(-12_210_434_i32) as u32, u32::MAX],
            }),
        )
        .quad();
        (particle.xd, particle.yd, particle.zd) = push;
        (particle.xo, particle.yo, particle.zo) = (at.0 + push.0, at.1 + push.1, at.2 + push.2);
        (particle.x, particle.y, particle.z) = (particle.xo, particle.yo, particle.zo);
        particle.quad_size = 0.1 * (particle.random.next_float() * 0.5 + 0.2);
        particle.has_physics = false;
        particle.lifetime = (particle.random.next_float() * 5.0) as i32 + 25;
        // `Mth.randomBetween(level.getRandom(), 3, 5)`.
        particle.scale(level_random.next_float() * 2.0 + 3.0);
        particle
    }

    /// `SuspendedParticle(..)`: an eighth of a block lower, a speck that hangs (`push`: the
    /// constructor with a motion, a bigger quad).
    fn suspended_of(
        random: RandomSource,
        at: (f64, f64, f64),
        push: Option<(f64, f64, f64)>,
        sprite: (&Arc<SpriteSet>, usize),
    ) -> Self {
        let at = (at.0, at.1 - 0.125, at.2);
        let mut particle = match push {
            Some(push) => Self::thrown(random, at, push, sprite, Kind::Suspended).quad(),
            None => Self::at(random, at, sprite, Kind::Suspended).quad(),
        };
        particle.set_size(0.01, 0.01);
        let base = if push.is_some() { 0.6 } else { 0.2 };
        particle.quad_size *= particle.random.next_float() * 0.6 + base;
        particle.lifetime = (16.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        particle.has_physics = false;
        particle.friction = 1.0;
        particle.gravity = 0.0;
        particle
    }

    /// `SuspendedParticle`'s providers: `underwater`, `crimson_spore`, `warped_spore`,
    /// `spore_blossom_air`, their motion, colour and life from the provider's random.
    pub fn suspended(
        name: &str,
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        // The motion is drawn before the picture, as the providers' arguments are.
        let sprite = |random: &mut RandomSource| (sprites, sprites.from_random(random));
        match name {
            "crimson_spore" => {
                let xa = provider_random.next_gaussian() * f64::from(1.0e-6_f32);
                let ya = provider_random.next_gaussian() * f64::from(1.0e-4_f32);
                let za = provider_random.next_gaussian() * f64::from(1.0e-6_f32);
                let sprite = sprite(provider_random);
                let mut particle = Self::suspended_of(random, at, Some((xa, ya, za)), sprite);
                (particle.r_col, particle.g_col, particle.b_col) = (0.9, 0.4, 0.5);
                particle
            }
            "spore_blossom_air" => {
                let push = (0.0, f64::from(-0.8_f32), 0.0);
                let sprite = sprite(provider_random);
                let mut particle = Self::suspended_of(random, at, Some(push), sprite);
                particle.limited = true;
                particle.lifetime = provider_random.next_int_bound(501).unwrap_or(0) + 500;
                particle.gravity = 0.01;
                (particle.r_col, particle.g_col, particle.b_col) = (0.32, 0.5, 0.22);
                particle
            }
            "warped_spore" => {
                let ya = f64::from(provider_random.next_float())
                    * -1.9
                    * f64::from(provider_random.next_float())
                    * 0.1;
                let sprite = sprite(provider_random);
                let mut particle = Self::suspended_of(random, at, Some((0.0, ya, 0.0)), sprite);
                (particle.r_col, particle.g_col, particle.b_col) = (0.1, 0.1, 0.3);
                particle.set_size(0.001, 0.001);
                particle
            }
            _ => {
                let sprite = sprite(provider_random);
                let mut particle = Self::suspended_of(random, at, None, sprite);
                (particle.r_col, particle.g_col, particle.b_col) = (0.4, 0.4, 0.7);
                particle
            }
        }
    }

    /// `SuspendedTownParticle`'s providers: `mycelium`, `happy_villager`, `egg_crack`,
    /// `composter` (living three to seven ticks by the level's random) and `dolphin` (blue,
    /// faint at random, half the life).
    pub fn suspended_town(
        name: &str,
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        (provider_random, level_random): (&mut RandomSource, &mut RandomSource),
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        let mut particle = Self::thrown(
            random,
            at,
            push,
            (sprites, sprite),
            Kind::Attract(Attract::Town),
        )
        .quad();
        let brightness = particle.random.next_float() * 0.1 + 0.2;
        (particle.r_col, particle.g_col, particle.b_col) = (brightness, brightness, brightness);
        particle.set_size(0.02, 0.02);
        particle.quad_size *= particle.random.next_float() * 0.6 + 0.5;
        particle.xd *= f64::from(0.02_f32);
        particle.yd *= f64::from(0.02_f32);
        particle.zd *= f64::from(0.02_f32);
        particle.lifetime = (20.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        match name {
            "composter" => {
                (particle.r_col, particle.g_col, particle.b_col) = (1.0, 1.0, 1.0);
                particle.lifetime = 3 + level_random.next_int_bound(5).unwrap_or(0);
            }
            "dolphin" => {
                (particle.r_col, particle.g_col, particle.b_col) = (0.3, 0.5, 1.0);
                particle.alpha = 1.0 - provider_random.next_float() * 0.7;
                particle.lifetime /= 2;
            }
            "egg_crack" | "happy_villager" => {
                (particle.r_col, particle.g_col, particle.b_col) = (1.0, 1.0, 1.0);
            }
            _ => {}
        }
        particle
    }

    /// The tick of a particle drawn to a place.
    pub(super) fn attract_tick(&mut self, attract: Attract) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        if let Attract::Town = attract {
            let left = self.lifetime;
            self.lifetime -= 1;
            if left <= 0 {
                self.removed = true;
                return;
            }
            // `move` through blocks.
            self.bounding_box_move(self.xd, self.yd, self.zd);
            self.xd *= 0.99;
            self.yd *= 0.99;
            self.zd *= 0.99;
            return;
        }
        let age = self.age;
        self.age += 1;
        if age >= self.lifetime {
            self.removed = true;
            return;
        }
        let normalized = self.age as f32 / self.lifetime as f32;
        match attract {
            Attract::Portal {
                start,
                reverse: false,
            } => {
                let mut pos = -normalized + normalized * normalized * 2.0;
                pos = 1.0 - pos;
                self.x = start[0] + self.xd * f64::from(pos);
                self.y = start[1] + self.yd * f64::from(pos) + f64::from(1.0 - normalized);
                self.z = start[2] + self.zd * f64::from(pos);
            }
            Attract::Portal { reverse: true, .. } => {
                self.x += self.xd * f64::from(normalized);
                self.y += self.yd * f64::from(normalized);
                self.z += self.zd * f64::from(normalized);
            }
            Attract::FlyTowards { start, .. } => {
                let pos = 1.0 - normalized;
                let mut pp = 1.0 - pos;
                pp *= pp;
                pp *= pp;
                self.x = start[0] + self.xd * f64::from(pos);
                self.y = start[1] + self.yd * f64::from(pos) - f64::from(pp * 1.2);
                self.z = start[2] + self.zd * f64::from(pos);
            }
            Attract::FlyStraight { start, colours } => {
                let pos = 1.0 - normalized;
                self.x = start[0] + self.xd * f64::from(pos);
                self.y = start[1] + self.yd * f64::from(pos);
                self.z = start[2] + self.zd * f64::from(pos);
                let colour = srgb_lerp(normalized, colours[0], colours[1]);
                let channel = |shift: u32| ((colour >> shift) & 0xFF) as f32 / 255.0;
                (self.r_col, self.g_col, self.b_col) = (channel(16), channel(8), channel(0));
                self.alpha = channel(24);
            }
            Attract::Town => {}
        }
    }

    /// `Particle.LifetimeAlpha.currentAlphaForAge` of a glowing one, at a partial tick.
    pub(super) fn lifetime_alpha(&self, [start, end, from, to]: [f32; 4], partial: f32) -> f32 {
        if (end - start).abs() < 1.0e-5 {
            return start;
        }
        let value = (self.age as f32 + partial) / self.lifetime as f32;
        let t = (value - from) / (to - from);
        if t < 0.0 {
            start
        } else if t > 1.0 {
            end
        } else {
            start + t * (end - start)
        }
    }
}

/// `ARGB.srgbLerp`: each channel `p0 + floor(t * (p1 - p0))`.
fn srgb_lerp(t: f32, from: u32, to: u32) -> u32 {
    [24, 16, 8, 0]
        .map(|shift| {
            let (a, b) = (
                ((from >> shift) & 0xFF) as i32,
                ((to >> shift) & 0xFF) as i32,
            );
            let channel = a + (t * (b - a) as f32).floor() as i32;
            ((channel & 0xFF) as u32) << shift
        })
        .into_iter()
        .fold(0, |colour, channel| colour | channel)
}
