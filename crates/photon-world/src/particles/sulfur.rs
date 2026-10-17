//! The particles of a potent sulfur block: the cloud that sends its gas out, the gas, and the
//! bubbles that rise through the water above it.

use std::sync::Arc;

use photon_core::math::Vec3;
use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::hit::Emitter;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// `NoxiousGasCloudParticle`'s life.
const CLOUD_LIFETIME: i32 = 20;
/// `SulfurBubbleParticle.SIZE_END`.
const SIZE_END: f32 = 0.15;

/// What a `SulfurBubbleParticle` keeps of where it started and where it was.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SulfurBubble {
    y_start: f64,
    y_end: f64,
    size_start: f32,
    y_prev: f64,
}

impl Particle {
    /// `NoxiousGasCloudParticle(..)`: never drawn, it lives a second.
    pub fn gas_cloud(random: RandomSource, at: (f64, f64, f64)) -> Self {
        let sprites = Arc::new(SpriteSet::new(Vec::new()));
        let mut particle = Self::at(random, at, (&sprites, 0), Kind::Emitter(Emitter::GasCloud));
        particle.lifetime = CLOUD_LIFETIME;
        particle
    }

    /// `NoxiousGasCloudParticle.tick`: on every second tick it asks the level for a place near it
    /// to send gas to; the level picks it from its random source, which the particle has no hand
    /// in.
    pub(super) fn gas_cloud_tick(&mut self) {
        let aged = self.age >= self.lifetime;
        self.age += 1;
        if aged {
            self.removed = true;
        }
        if self.age % 2 == 0 {
            self.gas_probe = Some([self.x, self.y, self.z].map(photon_core::math::floor_f64));
        }
    }

    /// `NoxiousGasParticle.Provider`: smoke that stays white and thins out in its second half.
    pub fn noxious_gas(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let scale = 3.0_f32;
        let mut particle = Self::ash_smoke(
            random,
            at,
            (0.1, 0.1, 0.1),
            push,
            scale,
            sprites,
            (0.3, 5, -0.02, true),
        );
        particle.r_col = 1.0;
        particle.g_col = 1.0;
        particle.b_col = 1.0;
        particle.lifetime =
            (6.0 / (f64::from(particle.random.next_float()) * 0.5 + 0.5) * f64::from(scale)) as i32;
        particle.kind = Kind::NoxiousGas {
            fade_start: particle.lifetime as f32 / 2.0,
        };
        particle
    }

    /// `NoxiousGasParticle.tick`, after the smoke's: once past half its life it fades out.
    pub(super) fn noxious_gas_fade(&mut self, fade_start: f32) {
        if self.age as f32 > fade_start {
            let since = self.age as f32 - fade_start;
            self.alpha = (self.lifetime as f32 - since) / self.lifetime as f32;
        }
    }

    /// `SulfurBubbleParticle.Provider`: its picture from the provider's random, its sideways push
    /// taken from the push's first and second parts.
    pub fn sulfur_bubble(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        let mut particle = Self::at(random, at, (sprites, sprite), Kind::Suspended).quad();
        particle.gravity = -0.04;
        particle.friction = 0.85;
        particle.set_size(0.02, 0.02);
        let jitter = |push: f64, random: &mut RandomSource| {
            push * f64::from(0.2_f32) + f64::from((random.next_float() * 2.0 - 1.0) * 0.02)
        };
        particle.xd = jitter(push.0, &mut particle.random);
        particle.zd = jitter(push.1, &mut particle.random);
        let size_start = 0.02 + 0.02 * particle.random.next_float();
        particle.quad_size = size_start;
        particle.lifetime = i32::MAX;
        particle.kind = Kind::SulfurBubble(SulfurBubble {
            y_start: particle.yo,
            y_end: particle.yo + 4.0 - 1.0,
            size_start,
            y_prev: at.1,
        });
        particle
    }

    /// `SulfurBubbleParticle.tick`: it rises, and goes where the water ends, or the height it
    /// was given, or when it stops rising; the rest of the tick is done all the same.
    pub(super) fn sulfur_bubble_tick(&mut self, world: &ClientWorld, mut bubble: SulfurBubble) {
        self.base_tick(world);
        let cell = [self.x, self.y, self.z].map(photon_core::math::floor_f64);
        if !self.removed && !world.is_water_source_at((cell[0], cell[1], cell[2])) {
            self.removed = true;
        }
        if !self.removed && self.y >= bubble.y_end {
            self.removed = true;
        }
        if !self.removed && self.y <= bubble.y_prev {
            self.removed = true;
        }
        self.xd += self.random_horizontal_wiggling();
        self.zd += self.random_horizontal_wiggling();
        self.move_by(world, self.xd, 0.0, self.zd);
        let progress = ((self.y - bubble.y_start) / (bubble.y_end - bubble.y_start)) as f32;
        self.quad_size = bubble.size_start + progress * (SIZE_END - bubble.size_start);
        bubble.y_prev = self.y;
        self.kind = Kind::SulfurBubble(bubble);
    }

    fn random_horizontal_wiggling(&mut self) -> f64 {
        let size = self.random.next_float() * 0.003;
        let sign = if self.random.next_boolean() {
            1.0
        } else {
            -1.0
        };
        f64::from(size * sign) * 0.5
    }
}

impl super::engine::ParticleEngine {
    /// What a gas cloud asks of the level: a place within three blocks of its cell, along a
    /// direction and a distance drawn from the level's random source, and gas there if the place
    /// is one the gas reaches.
    pub(super) fn send_gas(
        &mut self,
        world: &ClientWorld,
        cell: [i32; 3],
        view: &super::engine::AddView,
        level_random: &mut RandomSource,
        particle_random: &mut impl FnMut() -> RandomSource,
    ) {
        let x = level_random.next_float() - 0.5;
        let z = level_random.next_float() - 0.5;
        let direction = Vec3::new(f64::from(x), 0.0, f64::from(z)).normalized();
        let distance = f64::from(level_random.next_float() * 3.0);
        let centre = (
            f64::from(cell[0]) + 0.5,
            f64::from(cell[1]) + 0.5,
            f64::from(cell[2]) + 0.5,
        );
        let at = Vec3::new(
            centre.0 + direction.x * distance,
            centre.1 + direction.y * distance - 0.25,
            centre.2 + direction.z * distance,
        );
        if !world.noxious_gas_reaches((cell[0], cell[1], cell[2]), at) {
            return;
        }
        let Some(type_id) = photon_data::particle_type::id("noxious_gas") else {
            return;
        };
        let spawn = super::engine::Spawn {
            type_id,
            data: super::engine::ParticleData::Plain,
            override_limiter: false,
            always_show: true,
            at: (at.x, at.y, at.z),
            push: (0.0, 0.0, 0.0),
        };
        self.add_particle(world, &spawn, view, level_random, particle_random);
    }
}

impl super::engine::ParticleEngine {
    /// `Level.addParticle(new GeyserParticleOptions(GEYSER, waterBlocks), ..)`.
    pub(super) fn add_geyser<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut super::level_events::LevelEventContext<'_, F>,
        at: (f64, f64, f64),
        water_blocks: i32,
    ) {
        let Some(type_id) = photon_data::particle_type::id("geyser") else {
            return;
        };
        let spawn = super::engine::Spawn {
            type_id,
            data: super::engine::ParticleData::Geyser { water_blocks },
            override_limiter: photon_data::particle_type::overrides_limiter(type_id),
            always_show: false,
            at,
            push: (0.0, 0.0, 0.0),
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
