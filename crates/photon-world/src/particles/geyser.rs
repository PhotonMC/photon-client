//! The geyser of an erupting potent sulfur block: the emitter that sends the rest up, the spray
//! at its foot, and the plume that rises.

use std::sync::Arc;

use photon_core::math::fdlibm_pow;
use photon_core::random::RandomSource;

use super::engine::{ParticleData, Spawn};
use super::hit::Emitter;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// `GeyserEruptionParticle.BASE_PARTICLE_FREQUENCY` and the rest of its pace.
const BASE_BURST_IMPULSE: f32 = 1.5;
const POOF_BURST_IMPULSE: f32 = 2.0;

/// What a `GeyserPlumeParticle` keeps of how it rises.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Plume {
    start_y: f64,
    max_y: f64,
    initial_propulsion: f32,
    spray: (f32, f32),
    sizes: (f32, f32),
    done: bool,
}

/// `Level.addParticle` of one of the geyser's types, from the emitter's place.
fn spawn(name: &str, data: ParticleData, at: (f64, f64, f64), push: (f64, f64, f64)) -> Spawn {
    Spawn {
        type_id: photon_data::particle_type::id(name).expect("a carried type"),
        data,
        override_limiter: false,
        always_show: false,
        at,
        push,
    }
}

impl Particle {
    /// `GeyserEruptionParticle.Provider`: never drawn, it sends the geyser up for a second.
    pub fn geyser_eruption(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        water_blocks: i32,
    ) -> Self {
        let sprites = Arc::new(SpriteSet::new(Vec::new()));
        let emitter = Emitter::Geyser { water_blocks, push };
        let mut particle = Self::at(random, at, (&sprites, 0), Kind::Emitter(emitter));
        particle.lifetime = 20;
        particle
    }

    /// `GeyserEruptionParticle.tick`: the spray every other tick, the plume on every one, the
    /// poofs every tenth.
    pub(super) fn geyser_tick(&mut self, water_blocks: i32, push: (f64, f64, f64)) -> Vec<Spawn> {
        let aged = self.age >= self.lifetime;
        self.age += 1;
        if aged {
            self.removed = true;
        }
        let at = (self.x, self.y, self.z);
        let base = |name: &str, burst: f32| {
            let data = ParticleData::GeyserBase {
                water_blocks,
                burst,
            };
            spawn(name, data, at, push)
        };
        let mut spawns = Vec::new();
        if self.age % 2 == 0 {
            spawns.extend((0..2).map(|_| base("geyser_base", BASE_BURST_IMPULSE)));
        }
        let plume = spawn(
            "geyser_plume",
            ParticleData::Geyser { water_blocks },
            at,
            push,
        );
        spawns.extend((0..water_blocks + 2).map(|_| plume));
        if self.age % 10 == 0 {
            spawns.extend((0..20).map(|_| base("geyser_poof", POOF_BURST_IMPULSE)));
        }
        spawns
    }

    /// `GeyserBaseParticle.Provider`: smoke thrown out about the foot of the geyser, white,
    /// that lives about a second.
    pub fn geyser_base(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        (water_blocks, burst_base): (i32, f32),
        sprites: &Arc<SpriteSet>,
        (provider_random, level_random): (&mut RandomSource, &mut RandomSource),
    ) -> Self {
        let spread = |random: &mut RandomSource| f64::from((random.next_float() - 0.5) * 0.5);
        let x = at.0 + spread(provider_random);
        let y = at.1 + spread(provider_random) + f64::from(0.2_f32);
        let z = at.2 + spread(provider_random);
        let burst = burst_base + 0.25 * water_blocks as f32;
        let size = 3.0 + 0.125 * water_blocks as f32;
        let mut particle = Self::ash_smoke(
            random,
            (x, y, z),
            (burst, burst, burst),
            push,
            (size, sprites),
            (0.0, 0, 0.0, true),
        );
        particle.friction = 0.725;
        particle.r_col = 1.0;
        particle.g_col = 1.0;
        particle.b_col = 1.0;
        particle.yd = particle.yd.abs();
        let factor = 0.8 + 0.2 * level_random.next_float();
        particle.lifetime = (25.0 * factor) as i32;
        particle
    }

    /// `GeyserPlumeParticle.Provider`: a puff that shoots up through the water to the height the
    /// geyser has, its spray to the sides drawn from the level's random.
    pub fn geyser_plume(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        water_blocks: i32,
        sprites: &Arc<SpriteSet>,
        (provider_random, level_random): (&mut RandomSource, &mut RandomSource),
    ) -> Self {
        let x = at.0 + f64::from((provider_random.next_float() - 0.5) * 0.2);
        let y = at.1 + f64::from(provider_random.next_float());
        let z = at.2 + f64::from((provider_random.next_float() - 0.5) * 0.2);
        let mut particle = Self::thrown(
            random,
            (x, y, z),
            push,
            (sprites, sprites.first()),
            Kind::Suspended,
        )
        .quad();
        let height = 5 * water_blocks.max(1);
        particle.has_physics = true;
        particle.speed_up_when_y_motion_is_blocked = true;
        particle.lifetime = height * 5;
        particle.yd = 0.0;
        let spray_x = (level_random.next_float() - 0.5) * 0.2;
        let spray_z = (level_random.next_float() - 0.5) * 0.2;
        particle.friction = 1.0;
        let boost = if water_blocks == 1 { 1.5 } else { 1.0 };
        let initial_propulsion = boost * height as f32 * 1.45;
        particle.gravity = -initial_propulsion;
        let size = particle.quad_size * 0.75;
        let min_size = size * (2.0 + height as f32 / 8.0);
        let max_size = size * (3.0 + height as f32 / 8.0);
        particle.quad_size = min_size;
        particle.kind = Kind::GeyserPlume(Plume {
            start_y: y,
            max_y: y + f64::from(height) - 1.0,
            initial_propulsion,
            spray: (spray_x, spray_z),
            sizes: (min_size, max_size),
            done: false,
        });
        particle.set_sprite_from_age();
        particle
    }

    /// `GeyserPlumeParticle.tick`: it rises, slowed more the higher it is; at the top, or when it
    /// stops, it has five ticks left.
    pub(super) fn geyser_plume_tick(
        &mut self,
        world: &super::super::ClientWorld,
        mut plume: Plume,
    ) {
        self.base_tick(world);
        if !plume.done && (self.yd < 0.0 || self.y > plume.max_y || self.y == self.yo) {
            self.lifetime = self.lifetime.min(self.age + 5);
            self.friction = 0.0;
            plume.done = true;
        }
        let progress = ((self.y - plume.start_y) / (plume.max_y - plume.start_y)).clamp(0.0, 1.0);
        let exponential = fdlibm_pow(progress, 3.0);
        self.gravity = plume.initial_propulsion * exponential as f32 * 0.12;
        self.xd = progress * f64::from(plume.spray.0);
        self.zd = progress * f64::from(plume.spray.1);
        self.set_sprite_from_age();
        let (min_size, max_size) = plume.sizes;
        self.quad_size = min_size + (progress * f64::from(max_size - min_size)) as f32;
        self.kind = Kind::GeyserPlume(plume);
    }
}
