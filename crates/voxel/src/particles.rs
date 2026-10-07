//! The level's particles on the client: the engine, the random sources the reference seeds from
//! the clock, and the sprite lists of `particles/*.json`.

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use voxel_core::math::Vec3;
use voxel_core::random::RandomSource;
use voxel_render::entity::textures::{EntityTextures, TextureRect};
use voxel_render::particles::{ParticleCamera, extract};
use voxel_render::scene::Vertex;
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
    /// The textures (under `textures/`) the types carried here are drawn in, and where each
    /// sits in the packed entity textures once those are made.
    textures: Vec<String>,
    rects: HashMap<String, TextureRect>,
    /// What a scripted run reports: the packets that came, and the most particles alive at once.
    packets_heard: usize,
    most_alive: usize,
}

/// The path under `textures/` of a sprite the particle lists name (`minecraft:generic_3`).
fn texture_path(sprite: &str) -> String {
    format!(
        "particle/{}",
        sprite.strip_prefix("minecraft:").unwrap_or(sprite)
    )
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
        let textures = resources
            .texture_names()
            .iter()
            .map(|name| texture_path(name))
            .collect();
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
            textures,
            rects: HashMap::new(),
            packets_heard: 0,
            most_alive: 0,
        }
    }

    /// The textures the particles are drawn in, to pack with the entity textures.
    pub(crate) fn textures(&self) -> &[String] {
        &self.textures
    }

    /// Where the textures were packed.
    pub(crate) fn bind_textures(&mut self, packed: &EntityTextures) {
        self.rects = self
            .textures
            .iter()
            .filter_map(|path| Some((path.clone(), packed.rect(path)?)))
            .collect();
    }

    /// The particle packets that arrived: each particle asked for is made when the option and
    /// the camera's distance let it in.
    pub(crate) fn receive(
        &mut self,
        packets: Vec<LevelParticles>,
        world: &ClientWorld,
        status: ParticleStatus,
    ) {
        self.packets_heard += packets.len();
        let view = AddView {
            camera: self.camera,
            status,
        };
        for packet in packets {
            let mut uniquifier = self.uniquifier;
            self.engine.level_particles(
                world,
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
        self.most_alive = self.most_alive.max(self.engine.particle_count());
    }

    /// The packets that came and the most particles alive at once, for a scripted run's report.
    pub(crate) fn report(&self) -> (usize, usize) {
        (self.packets_heard, self.most_alive)
    }

    /// A new level: nothing of the old one stays.
    pub(crate) fn clear(&mut self) {
        self.engine.clear();
    }

    /// The triangles of the particles in view: each quad turned to the camera, lit by the level
    /// and cut out by its texture, measured from `shift` (where the camera is from the origin
    /// the frame's vertices are measured from).
    pub(crate) fn vertices(
        &self,
        world: &ClientWorld,
        camera: &ParticleCamera,
        (frustum, shift, partial): (&voxel_render::visibility::Frustum, [f32; 3], f32),
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        for particle in self.engine.particles() {
            let at = particle.position();
            if !frustum.is_visible([at.x, at.y, at.z], [at.x, at.y, at.z]) {
                continue;
            }
            let Some(rect) = self.rects.get(&texture_path(particle.sprite_name())) else {
                continue;
            };
            let light = particle.light_coords_at(world, partial);
            let quad = extract(
                particle,
                camera,
                partial,
                ([rect[0], rect[2], rect[1], rect[3]], light),
            );
            for index in [0, 1, 2, 0, 2, 3] {
                let corner = quad[index];
                let color = corner.color as u32;
                let rgb = color & 0xFF_FFFF;
                vertices.push(Vertex {
                    position: [
                        corner.position[0] + shift[0],
                        corner.position[1] + shift[1],
                        corner.position[2] + shift[2],
                    ],
                    uv: corner.uv,
                    // A vertex lit by the world and blended: 64 and the alpha in 255ths.
                    shade: 64.0 + (color >> 24) as f32,
                    // No normal, so no shading by face; the fourth is the colour (a negative
                    // one is black, since zero means none).
                    tint: [0.0, 0.0, 0.0, if rgb == 0 { -1.0 } else { rgb as f32 }],
                    light: voxel_render::block::light::light_uv(corner.light),
                });
            }
        }
        vertices
    }
}
