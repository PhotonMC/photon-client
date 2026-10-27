//! The level's particles on the client: the engine, the random sources the reference seeds from
//! the clock, and the sprite lists of `particles/*.json`.

mod tracking;

use photon_core::identifier::Identifier;
use photon_core::ids::BlockStateId;
use photon_core::ids::ItemId;
use photon_core::resources::Resources;
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use photon_core::math::Vec3;
use photon_core::random::RandomSource;
use photon_render::entity::textures::{EntityTextures, TextureRect};
use photon_render::particles::{ParticleCamera, extract};
use photon_render::scene::Vertex;
use photon_world::ClientWorld;
use std::sync::Arc;

use photon_core::binary::BlockPos;
use photon_core::direction::Direction;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_world::particles::{
    AddView, LevelParticles, ParticleAtlas, ParticleEngine, ParticleResources, ParticleStatus,
    SpriteSet, TerrainMaterials,
};

/// `RandomSupport.SEED_UNIQUIFIER`'s start and step.
const UNIQUIFIER_START: i64 = 8_682_522_807_148_012;
const UNIQUIFIER_STEP: i64 = 1_181_783_497_276_652_981;

/// Where the local player is for a tick: its block, and the place a pickup heads for.
pub(crate) type PlayerPlace = (Option<(i32, i32, i32)>, Option<(f64, f64, f64)>);

pub(crate) struct Particles {
    engine: ParticleEngine,
    /// The listener's random source (the spread of a counted packet) and the level's (which of
    /// the particles the option lets in).
    listener_random: RandomSource,
    level_random: RandomSource,
    /// `animateTick`'s own (`RandomSource.createThreadLocalInstance`), the blocks' draws.
    animate_random: RandomSource,
    uniquifier: i64,
    /// What the dimension of the last level events says about dripstone.
    dripstone: photon_world::particles::DripstoneAttributes,
    /// The local player as the last level events found them.
    local_player: Option<photon_world::particles::PlayerSpot>,
    /// Where the camera was when the last frame was drawn.
    camera: Vec3,
    /// The textures (under `textures/`) the types carried here are drawn in, and where each
    /// sits in the packed entity textures once those are made.
    textures: Vec<String>,
    rects: HashMap<String, TextureRect>,
    /// The block models, for where the flakes' pictures are in the block atlas.
    renderer: Option<Arc<BlockRenderer>>,
    /// What a scripted run reports: the packets that came, and the most particles alive at once.
    packets_heard: usize,
    most_alive: usize,
    /// The elder guardian's model the particle draws, baked when the first one flashes.
    elder_model: Option<photon_render::entity::mob_model::MobModel>,
}

/// What the flakes of blocks are made with: the models' particle textures and the tint sources.
struct BlockMaterials {
    renderer: Arc<BlockRenderer>,
    tints: Arc<BiomeTints>,
}

impl std::fmt::Debug for BlockMaterials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("BlockMaterials")
    }
}

impl TerrainMaterials for BlockMaterials {
    fn particle_texture(&self, state: BlockStateId) -> String {
        self.renderer
            .particle_sprite(state)
            .unwrap_or("minecraft:missingno")
            .to_owned()
    }

    fn item_particle_textures(&self, item: ItemId) -> Vec<String> {
        self.renderer.item_particle_sprites(item)
    }

    fn rains_at(&self, world: &ClientWorld, pos: BlockPos) -> bool {
        let noise_biome = |x: i32, y: i32, z: i32| world.noise_biome(x, y, z);
        let precipitation =
            self.tints
                .precipitation_at([pos.x, pos.y, pos.z], &noise_biome, world.sea_level());
        precipitation == photon_render::block::biome_tint::Precipitation::Rain
    }

    fn tint(&self, world: &ClientWorld, state: BlockStateId, pos: BlockPos) -> Option<i32> {
        // `BiomeColors.getAverage...Color`: the biome colours blended around the block.
        let noise_biome = |x: i32, y: i32, z: i32| world.noise_biome(x, y, z);
        let biome_tint = |kind, position: [i32; 3]| self.tints.tint(kind, position, &noise_biome);
        self.renderer
            .particle_tint(state, [pos.x, pos.y, pos.z], &biome_tint)
    }
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
        let stack = crate::resources::shared(Path::new(root));
        for particle in photon_data::particle_type::PARTICLE_TYPES {
            let id = format!("minecraft:particles/{}.json", particle.name);
            let Some(description) = Identifier::parse(&id)
                .ok()
                .and_then(|id| stack.read(&id))
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .and_then(|text| photon_core::json::parse_object(&text))
            else {
                continue;
            };
            let names = description
                .array("textures")
                .unwrap_or_default()
                .iter()
                .filter_map(|texture| match texture {
                    photon_core::json::JsonValue::String(units) => String::from_utf16(units).ok(),
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
        resources.set_spell_random(RandomSource::create(unique_seed(&mut uniquifier)));
        Self {
            engine: ParticleEngine::new(
                RandomSource::create(unique_seed(&mut uniquifier)),
                resources,
            ),
            listener_random: RandomSource::create(unique_seed(&mut uniquifier)),
            level_random: RandomSource::create(unique_seed(&mut uniquifier)),
            animate_random: RandomSource::create(unique_seed(&mut uniquifier)),
            uniquifier,
            dripstone: photon_world::particles::DripstoneAttributes::default(),
            local_player: None,
            camera: Vec3::new(0.0, 0.0, 0.0),
            textures,
            rects: HashMap::new(),
            renderer: None,
            packets_heard: 0,
            most_alive: 0,
            elder_model: None,
        }
    }

    /// The sound event of each jukebox song, in the order the server synchronised them.
    pub(crate) fn set_jukebox_songs(&mut self, registries: &photon_game::registries::Registries) {
        let songs = registries.jukebox_songs.iter();
        self.engine
            .set_jukebox_songs(songs.map(|song| song.sound_event.clone()).collect());
    }

    /// The songs jukeboxes started and stopped since the last call.
    pub(crate) fn take_jukebox_changes(&mut self) -> Vec<photon_world::particles::JukeboxChange> {
        self.engine.take_jukebox_changes()
    }

    /// The textures the particles are drawn in, to pack with the entity textures.
    pub(crate) fn textures(&self) -> &[String] {
        &self.textures
    }

    /// The block models and tints the flakes of blocks are made with, once they are baked.
    pub(crate) fn set_terrain(&mut self, renderer: Arc<BlockRenderer>, tints: Arc<BiomeTints>) {
        self.engine.set_terrain(Arc::new(BlockMaterials {
            renderer: Arc::clone(&renderer),
            tints,
        }));
        self.renderer = Some(renderer);
    }

    /// What the level's particles need to know of where they are made: what the dimension says
    /// about dripstone, and the local player.
    fn look_about(&mut self, session: &photon_net::connection::PlaySession) {
        self.engine
            .set_marker_block(session.marker_particle_block());
        let dimension = session.dimension_type().unwrap_or("minecraft:overworld");
        self.dripstone = photon_world::particles::DripstoneAttributes::of_dimension_type(dimension);
        self.local_player = session
            .profile_id()
            .zip(session.player())
            .map(|(uuid, player)| photon_world::particles::PlayerSpot {
                uuid,
                position: player.position(),
                height: player.bb_height(),
            });
    }

    /// The level events since the last tick (`LevelEventHandler.levelEvent`): their particles are
    /// made, and the sounds they play given back in order.
    pub(crate) fn level_events(
        &mut self,
        session: &photon_net::connection::PlaySession,
        events: Vec<photon_world::LevelEvent>,
    ) -> Vec<photon_world::local_player::LocalSound> {
        self.look_about(session);
        let world = session.world();
        let view = AddView {
            camera: self.camera,
            status: ParticleStatus::All,
        };
        let mut uniquifier = self.uniquifier;
        let mut context = photon_world::particles::LevelEventContext {
            world,
            view: &view,
            level_random: &mut self.level_random,
            particle_random: || RandomSource::create(unique_seed(&mut uniquifier)),
            dripstone: self.dripstone,
            local_player: self.local_player,
            ambience: &photon_world::particles::NoAmbience,
        };
        let mut sounds = Vec::new();
        for event in events {
            sounds.extend(self.engine.level_event(event, &mut context));
        }
        self.uniquifier = uniquifier;
        sounds
    }

    /// The blocks the player swung at breaking: each makes a flake on the face aimed at.
    pub(crate) fn breaking(
        &mut self,
        world: &ClientWorld,
        effects: Vec<((i32, i32, i32), Direction)>,
    ) {
        for ((x, y, z), face) in effects {
            let mut uniquifier = self.uniquifier;
            self.engine.add_breaking_block_effect(
                world,
                (BlockPos::new(x, y, z), face),
                &mut self.level_random,
                || RandomSource::create(unique_seed(&mut uniquifier)),
            );
            self.uniquifier = uniquifier;
        }
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

    /// The entities taken, to fly to their takers.
    pub(crate) fn add_pickups(&mut self, pickups: Vec<photon_world::particles::ItemPickup>) {
        for pickup in pickups {
            self.engine.add_pickup(pickup);
        }
    }

    /// The elder guardians flashing over the screen, as the frame draws them
    /// (`ElderGuardianParticleGroup`): the model of the elder guardian, unposed, placed against the
    /// camera and blended by the opacity of the particle's age.
    pub(crate) fn elder_guardian_vertices(
        &mut self,
        textures: &EntityTextures,
        (partial, view): (f32, &crate::overlays::FrameView),
    ) -> Vec<photon_render::scene::Vertex> {
        use photon_render::entity::elder_guardian_particle::{LAYER, LIGHT, TEXTURE, place};
        let mut vertices = Vec::new();
        if self.engine.elder_guardians().next().is_none() {
            return vertices;
        }
        let Some(rect) = textures.rect(TEXTURE) else {
            return vertices;
        };
        if self.elder_model.is_none() {
            self.elder_model = photon_data::entity_model::layer(LAYER)
                .map(photon_render::entity::mob_model::MobModel::bake);
        }
        let Some(model) = self.elder_model.as_ref() else {
            return vertices;
        };
        let orientation =
            photon_render::entity::name_tag::camera_orientation(view.rotation.0, view.rotation.1);
        let mut base = photon_render::joml::Pose::default();
        base.translate(view.camera[0], view.camera[1], view.camera[2]);
        for particle in self.engine.elder_guardians() {
            let placed = place(
                &base,
                &orientation,
                (particle.age(), particle.lifetime()),
                partial,
            );
            vertices.extend(photon_render::entity::mob_model::mob_vertices_blended(
                model,
                &placed.pose,
                rect,
                (LIGHT, (placed.color >> 24) as u8),
                view.camera,
            ));
        }
        vertices
    }

    /// The entities flying to whoever took them.
    pub(crate) fn pickups(&self) -> impl Iterator<Item = &photon_world::particles::ItemPickup> {
        self.engine.pickups()
    }

    /// `ClientLevel.animateTick` around a block: what the blocks about it show, and the sounds
    /// they play.
    fn animate_blocks(
        &mut self,
        world: &ClientWorld,
        center: (i32, i32, i32),
        ambience: &dyn photon_world::particles::Ambience,
    ) -> Vec<photon_world::local_player::LocalSound> {
        let view = AddView {
            camera: self.camera,
            status: ParticleStatus::All,
        };
        let mut uniquifier = self.uniquifier;
        let mut context = photon_world::particles::LevelEventContext {
            world,
            view: &view,
            level_random: &mut self.level_random,
            particle_random: || RandomSource::create(unique_seed(&mut uniquifier)),
            dripstone: self.dripstone,
            local_player: self.local_player,
            ambience,
        };
        let sounds =
            self.engine
                .animate_tick((center, 667), &mut self.animate_random, &mut context);
        self.uniquifier = uniquifier;
        sounds
    }

    /// `ClientLevel.tickWeatherEffects` around the camera.
    fn tick_weather(
        &mut self,
        world: &ClientWorld,
        ambience: &dyn photon_world::particles::Ambience,
    ) -> Option<photon_world::local_player::LocalSound> {
        let view = AddView {
            camera: self.camera,
            status: ParticleStatus::All,
        };
        let mut uniquifier = self.uniquifier;
        let mut context = photon_world::particles::LevelEventContext {
            world,
            view: &view,
            level_random: &mut self.level_random,
            particle_random: || RandomSource::create(unique_seed(&mut uniquifier)),
            dripstone: self.dripstone,
            local_player: self.local_player,
            ambience,
        };
        let camera = (self.camera.x, self.camera.y, self.camera.z);
        let sound = self.engine.tick_weather(camera, &mut context);
        self.uniquifier = uniquifier;
        sound
    }

    /// One client tick: `ClientLevel.animateTick` around the player's block (`player`), then
    /// `ParticleEngine.tick`; the pickups head for where their takers are (`local`, the local
    /// player's place to head for, when there is one). The sounds the blocks played, in order.
    pub(crate) fn tick(
        &mut self,
        session: &photon_net::connection::PlaySession,
        (player, local): PlayerPlace,
        ambience: &dyn photon_world::particles::Ambience,
    ) -> Vec<photon_world::local_player::LocalSound> {
        self.look_about(session);
        self.follow_local_player(session);
        let world = session.world();
        let mut sounds = Vec::new();
        // `ClientLevel.tick`'s explosion tracker, before the blocks animate.
        {
            let view = AddView {
                camera: self.camera,
                status: ParticleStatus::All,
            };
            let mut uniquifier = self.uniquifier;
            let mut context = photon_world::particles::LevelEventContext {
                world,
                view: &view,
                level_random: &mut self.level_random,
                particle_random: || RandomSource::create(unique_seed(&mut uniquifier)),
                dripstone: self.dripstone,
                local_player: self.local_player,
                ambience,
            };
            self.engine.tick_explosions(&mut context);
            self.uniquifier = uniquifier;
        }
        if let Some(center) = player {
            sounds = self.animate_blocks(world, center, ambience);
        }
        sounds.extend(self.tick_weather(world, ambience));
        self.engine.tick_pickups(|target| match target {
            None => local,
            Some(id) => world.entities().at(id).map(|entity| {
                photon_world::particles::pickup_target(entity.position(), entity.eye_height())
            }),
        });
        // The particles option is not kept yet: all of them, as `receive` is given.
        let view = AddView {
            camera: self.camera,
            status: ParticleStatus::All,
        };
        let mut uniquifier = self.uniquifier;
        sounds.extend(self.engine.tick(world, &view, &mut self.level_random, || {
            RandomSource::create(unique_seed(&mut uniquifier))
        }));
        self.uniquifier = uniquifier;
        self.most_alive = self.most_alive.max(self.engine.particle_count());
        sounds
    }

    /// The packets that came and the most particles alive at once, for a scripted run's report.
    pub(crate) fn report(&self) -> (usize, usize) {
        (self.packets_heard, self.most_alive)
    }

    /// A new level: nothing of the old one stays.
    pub(crate) fn clear(&mut self) {
        self.engine.clear();
    }

    /// A new level: no record plays.
    pub(crate) fn forget_jukeboxes(&mut self) {
        self.engine.forget_jukeboxes();
    }

    /// The triangles of the particles in view, over the particle textures and over the block
    /// atlas: each quad turned to the camera, lit by the level and cut out by its texture,
    /// measured from `shift` (where the camera is from the origin the frame's vertices are
    /// measured from).
    pub(crate) fn vertices(
        &self,
        world: &ClientWorld,
        camera: &ParticleCamera,
        (frustum, shift, partial): (&photon_render::visibility::Frustum, [f32; 3], f32),
    ) -> (Vec<Vertex>, Vec<Vertex>) {
        let (mut particles, mut blocks) = (Vec::new(), Vec::new());
        for particle in self.engine.particles() {
            let at = particle.position();
            if !particle.is_drawn() || !frustum.is_visible([at.x, at.y, at.z], [at.x, at.y, at.z]) {
                continue;
            }
            let (rect, window) = match particle.atlas() {
                ParticleAtlas::Particles => {
                    let Some(rect) = self.rects.get(&texture_path(particle.sprite_name())) else {
                        continue;
                    };
                    (*rect, particle.sprite_window())
                }
                ParticleAtlas::Blocks => {
                    let Some(rect) = self
                        .renderer
                        .as_ref()
                        .and_then(|renderer| renderer.sprite_rect(particle.sprite_name()))
                    else {
                        continue;
                    };
                    (rect, particle.sprite_window())
                }
            };
            // `TextureAtlasSprite.getU` and `getV`: a place within the sprite.
            let along = |low: f32, high: f32, fraction: f32| low + (high - low) * fraction;
            let uvs = match window {
                None => [rect[0], rect[2], rect[1], rect[3]],
                Some([u0, u1, v0, v1]) => [
                    along(rect[0], rect[2], u0),
                    along(rect[0], rect[2], u1),
                    along(rect[1], rect[3], v0),
                    along(rect[1], rect[3], v1),
                ],
            };
            let light = particle.light_coords_at(world, partial);
            let quad = extract(particle, camera, partial, (uvs, light));
            let (target, blocks_atlas) = match particle.atlas() {
                ParticleAtlas::Particles => (&mut particles, false),
                ParticleAtlas::Blocks => (&mut blocks, true),
            };
            for index in [0, 1, 2, 0, 2, 3] {
                let corner = quad[index];
                let color = corner.color as u32;
                let rgb = color & 0xFF_FFFF;
                let position = [
                    corner.position[0] + shift[0],
                    corner.position[1] + shift[1],
                    corner.position[2] + shift[2],
                ];
                let light = photon_render::block::light::light_uv(corner.light);
                target.push(if blocks_atlas {
                    // The block atlas' shader: the colour as it is, no shade.
                    let channel = |shift: u32| ((color >> shift) & 0xFF) as f32 / 255.0;
                    Vertex {
                        position,
                        uv: corner.uv,
                        shade: 1.0,
                        tint: [channel(16), channel(8), channel(0), channel(24)],
                        light,
                    }
                } else {
                    Vertex {
                        position,
                        uv: corner.uv,
                        // A vertex lit by the world and blended: 64 and the alpha in 255ths.
                        shade: 64.0 + (color >> 24) as f32,
                        // No normal, so no shading by face; the fourth is the colour (a negative
                        // one is black, since zero means none).
                        tint: [0.0, 0.0, 0.0, if rgb == 0 { -1.0 } else { rgb as f32 }],
                        light,
                    }
                });
            }
        }
        (particles, blocks)
    }
}
