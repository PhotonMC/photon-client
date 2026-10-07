//! The level's particles on the client: the engine, the random sources the reference seeds from
//! the clock, and the sprite lists of `particles/*.json`.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use voxel_core::math::Vec3;
use voxel_core::random::RandomSource;
use voxel_world::ClientWorld;
use voxel_world::particles::{
    AddView, LevelParticles, ParticleEngine, ParticleResources, ParticleStatus, SpriteSet,
};

/// `RandomSupport.SEED_UNIQUIFIER`'s start and step.
const UNIQUIFIER_START: i64 = 8_682_522_807_148_012;
const UNIQUIFIER_STEP: i64 = 1_181_783_497_276_652_981;

pub(crate) struct Particles {
    engine: ParticleEngine,
    /// The listener's random source (the spread of a counted packet) and the level's (which of
    /// the particles the option lets in).
    listener_random: RandomSource,
    level_random: RandomSource,
    uniquifier: i64,
    /// Where the camera was when the last frame was drawn.
    camera: Vec3,
}

/// `RandomSupport.generateUniqueSeed`: a counter times a constant, with the clock mixed in.
fn unique_seed(uniquifier: &mut i64) -> i64 {
    *uniquifier = uniquifier.wrapping_mul(UNIQUIFIER_STEP);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos() as i64);
    *uniquifier ^ nanos
}

impl Particles {
    /// The engine with the sprite lists the assets give each type (`root/minecraft/particles`).
    pub(crate) fn load(root: &str) -> Self {
        let mut resources = ParticleResources::new();
        for particle in voxel_data::particle_type::PARTICLE_TYPES {
            let path = Path::new(root)
                .join("minecraft/particles")
                .join(format!("{}.json", particle.name));
            let Some(description) = voxel_core::json::read_object(&path) else {
                continue;
            };
            let names = description
                .array("textures")
                .unwrap_or_default()
                .iter()
                .filter_map(|texture| match texture {
                    voxel_core::json::JsonValue::String(units) => String::from_utf16(units).ok(),
                    _ => None,
                })
                .collect();
            resources.set_sprites(particle.name, SpriteSet::new(names));
        }
        let mut uniquifier = UNIQUIFIER_START;
        Self {
            engine: ParticleEngine::new(
                RandomSource::create(unique_seed(&mut uniquifier)),
                resources,
            ),
            listener_random: RandomSource::create(unique_seed(&mut uniquifier)),
            level_random: RandomSource::create(unique_seed(&mut uniquifier)),
            uniquifier,
            camera: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    /// The particle packets that arrived: each particle asked for is made when the option and
    /// the camera's distance let it in.
    pub(crate) fn receive(&mut self, packets: Vec<LevelParticles>, status: ParticleStatus) {
        let view = AddView {
            camera: self.camera,
            status,
        };
        for packet in packets {
            let mut uniquifier = self.uniquifier;
            self.engine.level_particles(
                &packet,
                &view,
                (&mut self.listener_random, &mut self.level_random),
                || RandomSource::create(unique_seed(&mut uniquifier)),
            );
            self.uniquifier = uniquifier;
        }
    }

    /// Where the camera is, as the last frame drew it.
    pub(crate) fn set_camera(&mut self, camera: Vec3) {
        self.camera = camera;
    }

    /// One client tick: `ParticleEngine.tick`.
    pub(crate) fn tick(&mut self, world: &ClientWorld) {
        self.engine.tick(world);
    }

    /// A new level: nothing of the old one stays.
    pub(crate) fn clear(&mut self) {
        self.engine.clear();
    }
}
