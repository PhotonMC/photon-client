//! The coloured particles: `DustParticle` and `DustColorTransitionParticle` (on
//! `DustParticleBase`), `SpellParticle`, `TotemParticle` and `GlowParticle`.

use std::sync::Arc;

use voxel_core::random::RandomSource;

use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// `ARGB.vector3fFromRGB24`: each channel over 255.
fn rgb(color: u32) -> [f32; 3] {
    [16, 8, 0].map(|shift| ((color >> shift) & 0xFF) as f32 / 255.0)
}

impl Particle {
    /// `DustParticleBase(..)`: a tenth of the push, a size and a life by the options' scale.
    fn dust_of(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        scale: f32,
        kind: Kind,
    ) -> Self {
        let mut particle = Self::thrown(random, at, push, (sprites, sprites.first()), kind).quad();
        particle.friction = 0.96;
        particle.speed_up_when_y_motion_is_blocked = true;
        particle.xd *= f64::from(0.1_f32);
        particle.yd *= f64::from(0.1_f32);
        particle.zd *= f64::from(0.1_f32);
        particle.quad_size *= 0.75 * scale;
        let base = (8.0 / (particle.random.next_double() * 0.8 + 0.2)) as i32;
        particle.lifetime = (base as f32 * scale).max(1.0) as i32;
        particle.set_sprite_from_age();
        particle
    }

    /// `DustParticleBase.randomizeColor`.
    fn randomize_colour(&mut self, colour: f32, base: f32) -> f32 {
        (self.random.next_float() * 0.2 + 0.8) * colour * base
    }

    /// `DustParticle.Provider`: dust of a colour, each channel a little darker at random.
    pub fn dust(
        random: RandomSource,
        motion: ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        (color, scale): (u32, f32),
    ) -> Self {
        let mut particle = Self::dust_of(
            random,
            motion,
            sprites,
            scale,
            Kind::Dust { transition: None },
        );
        let base = particle.random.next_float() * 0.4 + 0.6;
        let [r, g, b] = rgb(color);
        particle.r_col = particle.randomize_colour(r, base);
        particle.g_col = particle.randomize_colour(g, base);
        particle.b_col = particle.randomize_colour(b, base);
        particle
    }

    /// `DustColorTransitionParticle.Provider`: dust that goes from one colour to another over its
    /// life, as it is drawn.
    pub fn dust_transition(
        random: RandomSource,
        motion: ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        (from, to, scale): (u32, u32, f32),
    ) -> Self {
        let mut particle = Self::dust_of(
            random,
            motion,
            sprites,
            scale,
            Kind::Dust { transition: None },
        );
        let base = particle.random.next_float() * 0.4 + 0.6;
        let randomized = |color: u32, particle: &mut Self| {
            rgb(color).map(|channel| particle.randomize_colour(channel, base))
        };
        let from = randomized(from, &mut particle);
        let to = randomized(to, &mut particle);
        particle.kind = Kind::Dust {
            transition: Some([from, to]),
        };
        particle
    }

    /// `SpellParticle(..)`: a sideways push from the class's own random source (`RANDOM`), a fifth
    /// of the rise, without physics.
    fn spell_of(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        spell_random: &mut RandomSource,
    ) -> Self {
        let xa = 0.5 - spell_random.next_double();
        let za = 0.5 - spell_random.next_double();
        let mut particle = Self::thrown(
            random,
            at,
            (xa, push.1, za),
            (sprites, sprites.first()),
            Kind::Spell {
                original_alpha: 1.0,
            },
        )
        .quad();
        particle.friction = 0.96;
        particle.gravity = -0.1;
        particle.speed_up_when_y_motion_is_blocked = true;
        particle.yd *= f64::from(0.2_f32);
        if push.0 == 0.0 && push.2 == 0.0 {
            particle.xd *= f64::from(0.1_f32);
            particle.zd *= f64::from(0.1_f32);
        }
        particle.quad_size *= 0.75;
        particle.lifetime = (8.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        particle.has_physics = false;
        particle.set_sprite_from_age();
        particle
    }

    /// `SpellParticle.Provider` (`infested`, `raid_omen`, `trial_omen`).
    pub fn spell(
        random: RandomSource,
        motion: ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        spell_random: &mut RandomSource,
    ) -> Self {
        Self::spell_of(random, motion, sprites, spell_random)
    }

    /// `SpellParticle.InstantProvider` (`effect`, `instant_effect`): the option's colour and power.
    pub fn instant_spell(
        random: RandomSource,
        motion: ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        spell_random: &mut RandomSource,
        (color, power): (u32, f32),
    ) -> Self {
        let mut particle = Self::spell_of(random, motion, sprites, spell_random);
        [particle.r_col, particle.g_col, particle.b_col] = rgb(color);
        particle.set_power(power);
        particle
    }

    /// `SpellParticle.MobEffectProvider` (`entity_effect`): the option's colour and opacity.
    pub fn mob_effect(
        random: RandomSource,
        motion: ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        spell_random: &mut RandomSource,
        color: u32,
    ) -> Self {
        let mut particle = Self::spell_of(random, motion, sprites, spell_random);
        [particle.r_col, particle.g_col, particle.b_col] = rgb(color);
        let alpha = (color >> 24) as f32 / 255.0;
        particle.alpha = alpha;
        particle.kind = Kind::Spell {
            original_alpha: alpha,
        };
        particle
    }

    /// `SpellParticle.WitchProvider`: purple, its brightness from the provider's random.
    pub fn witch(
        random: RandomSource,
        motion: ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        (spell_random, provider_random): (&mut RandomSource, &mut RandomSource),
    ) -> Self {
        let mut particle = Self::spell_of(random, motion, sprites, spell_random);
        let brightness = provider_random.next_float() * 0.5 + 0.35;
        (particle.r_col, particle.g_col, particle.b_col) =
            (brightness, 0.0 * brightness, brightness);
        particle
    }

    /// `TotemParticle.Provider` on `SimpleAnimatedParticle`: green, or one time in four yellow.
    pub fn totem(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let mut particle = Self::at(random, at, (sprites, sprites.first()), Kind::Totem).quad();
        particle.gravity = 1.25;
        particle.friction = 0.6;
        (particle.xd, particle.yd, particle.zd) = push;
        particle.quad_size *= 0.75;
        particle.lifetime = 60 + particle.random.next_int_bound(12).unwrap_or(0);
        particle.set_sprite_from_age();
        let random = &mut particle.random;
        let colour = if random.next_int_bound(4).unwrap_or(0) == 0 {
            let r = 0.6 + random.next_float() * 0.2;
            let g = 0.6 + random.next_float() * 0.3;
            (r, g, random.next_float() * 0.2)
        } else {
            let r = 0.1 + random.next_float() * 0.2;
            let g = 0.4 + random.next_float() * 0.3;
            (r, g, random.next_float() * 0.2)
        };
        (particle.r_col, particle.g_col, particle.b_col) = colour;
        particle
    }

    /// `EndRodParticle.Provider`: a glint that drifts as pushed, falling slowly.
    pub fn end_rod(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let mut particle = Self::at(random, at, (sprites, sprites.first()), Kind::EndRod).quad();
        particle.friction = 0.91;
        particle.gravity = 0.0125;
        (particle.xd, particle.yd, particle.zd) = push;
        particle.quad_size *= 0.75;
        particle.lifetime = 60 + particle.random.next_int_bound(12).unwrap_or(0);
        particle.set_sprite_from_age();
        particle
    }

    /// `GlowParticle(..)`: glows as it ages, without physics.
    fn glow_of(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let mut particle =
            Self::thrown(random, at, push, (sprites, sprites.first()), Kind::Glow).quad();
        particle.friction = 0.96;
        particle.speed_up_when_y_motion_is_blocked = true;
        particle.quad_size *= 0.75;
        particle.has_physics = false;
        particle.set_sprite_from_age();
        particle
    }

    /// The glow providers: `electric_spark`, `glow` (a glow squid's), `scrape`, `wax_on`,
    /// `wax_off`, each with its colours, speed and life, drawn from the provider's random.
    pub fn glow(
        name: &str,
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let still = (0.0, 0.0, 0.0);
        let (xa, ya, za) = push;
        match name {
            "glow" => {
                let push = (
                    0.5 - provider_random.next_double(),
                    ya,
                    0.5 - provider_random.next_double(),
                );
                let mut particle = Self::glow_of(random, (at, push), sprites);
                (particle.r_col, particle.g_col, particle.b_col) = if provider_random.next_boolean()
                {
                    (0.6, 1.0, 0.8)
                } else {
                    (0.08, 0.4, 0.4)
                };
                particle.yd *= f64::from(0.2_f32);
                if xa == 0.0 && za == 0.0 {
                    particle.xd *= f64::from(0.1_f32);
                    particle.zd *= f64::from(0.1_f32);
                }
                particle.lifetime = (8.0 / (provider_random.next_double() * 0.8 + 0.2)) as i32;
                particle
            }
            "electric_spark" => {
                let mut particle = Self::glow_of(random, (at, still), sprites);
                (particle.r_col, particle.g_col, particle.b_col) = (1.0, 0.9, 1.0);
                (particle.xd, particle.yd, particle.zd) = (xa * 0.25, ya * 0.25, za * 0.25);
                particle.lifetime = provider_random.next_int_bound(2).unwrap_or(0) + 2;
                particle
            }
            "scrape" => {
                let mut particle = Self::glow_of(random, (at, still), sprites);
                (particle.r_col, particle.g_col, particle.b_col) = if provider_random.next_boolean()
                {
                    (0.29, 0.58, 0.51)
                } else {
                    (0.43, 0.77, 0.62)
                };
                (particle.xd, particle.yd, particle.zd) = (xa * 0.01, ya * 0.01, za * 0.01);
                particle.lifetime = provider_random.next_int_bound(30).unwrap_or(0) + 10;
                particle
            }
            _ => {
                let mut particle = Self::glow_of(random, (at, still), sprites);
                (particle.r_col, particle.g_col, particle.b_col) = if name == "wax_on" {
                    (0.91, 0.55, 0.08)
                } else {
                    (1.0, 0.9, 1.0)
                };
                (particle.xd, particle.yd, particle.zd) =
                    (xa * 0.01 / 2.0, ya * 0.01, za * 0.01 / 2.0);
                particle.lifetime = provider_random.next_int_bound(30).unwrap_or(0) + 10;
                particle
            }
        }
    }

    /// `DustColorTransitionParticle.lerpColors` at a partial tick: the colour it is drawn in.
    pub(super) fn transition_colour(&self, [from, to]: [[f32; 3]; 2], partial: f32) -> [f32; 3] {
        let t = (self.age as f32 + partial) / (self.lifetime as f32 + 1.0);
        std::array::from_fn(|channel| (to[channel] - from[channel]) * t + from[channel])
    }
}
