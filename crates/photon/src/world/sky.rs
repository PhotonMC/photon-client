//! The sky over the session: the dimension's sky data and the attributes it
//! blends between ticks, the stars, the sun and the moon, the clouds, the
//! weather, the fog rain brings, and the lightmap.

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::VulkanBackend;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::sky::{Attributes, Environment, SkySprites};

use crate::view::CLOUD_RANGE;
use crate::world::atmosphere::{Clouds, Flicker, rain_fog_target, weather, world_border};

/// Where the sky is seen from this frame.
pub struct SkyCamera {
    /// The player's eyes.
    pub eye: Vec3,
    /// The camera's yaw and pitch.
    pub rotation: (f32, f32),
    /// The camera's view and projection at the origin, which the sky and
    /// the clouds are drawn with.
    pub view_projection: [[f32; 4]; 4],
    pub render_distance: i32,
    /// What the camera is in and what the player has, for the fog.
    pub fog: photon_render::sky::FogView,
    /// How far between ticks.
    pub partial: f64,
    /// How many ticks the frame took.
    pub frame_ticks: f32,
    /// The point meshes are measured from.
    pub origin: [i32; 3],
    /// `Options.darknessEffectScale` and `Options.gamma`, which the lightmap extractor reads
    /// each frame.
    pub lightmap_options: (f32, f32),
    /// `Options.hideLightningFlash`, read by the same extractor.
    pub hide_flashes: bool,
}

pub struct Sky {
    /// Where the dimension types and biomes are read from.
    data_root: std::path::PathBuf,
    environment: Environment,
    stars: Vec<[[f32; 3]; 4]>,
    sprites: SkySprites,
    /// The dimension the probe reads.
    dimension: photon_world::DimensionBounds,
    /// The camera's attribute probe at the last two ticks.
    last: Attributes,
    next: Attributes,
    /// A reset found no player to probe at (a login or respawn waits for its position): the
    /// first tick with one starts the probe from its values.
    unprobed: bool,
    rain_fog: f32,
    flicker: Flicker,
    clouds: Clouds,
    /// `ClientLevel.endFlashState`: the End's flashes (`DimensionType.hasEndFlashes`).
    end_flash: Option<photon_world::end_flash::EndFlashState>,
    /// The angles of a flash that started, until its sound is asked for.
    flash_start: Option<(f32, f32)>,
}

/// The world border's wall for the frame, animated by the clock's milliseconds.
fn set_world_border(
    backend: &mut VulkanBackend,
    session: &PlaySession,
    camera: &SkyCamera,
    partial: f32,
) -> Result<(), photon_render::VulkanError> {
    let time_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as i64);
    let border_vertices = world_border(
        session,
        camera.eye,
        camera.origin,
        partial,
        (camera.render_distance, CLOUD_RANGE),
        time_millis,
    );
    backend.set_world_border(&border_vertices)
}

/// The End's flashes, in a dimension that has them.
fn end_flash_of(environment: &Environment) -> Option<photon_world::end_flash::EndFlashState> {
    (environment.skybox() == photon_render::sky::Skybox::End).then(Default::default)
}

impl Sky {
    /// `ClientLevel.tick`'s `endFlashState.tick(getDefaultClockTime())`: the dimension's own clock,
    /// zero when it has none.
    /// What the sky quad of the End's flash is drawn from: the intensity between two ticks and
    /// the angles.
    fn flash_view(&self, partial: f32) -> Option<(f32, f32, f32)> {
        let flash = self.end_flash.as_ref()?;
        Some((flash.intensity(partial), flash.x_angle(), flash.y_angle()))
    }

    /// The angles of a flash that started since the last call, for its sound.
    /// The sun angle and moon phase at the camera that a clock reads.
    pub fn clock(&self) -> (f32, u8) {
        self.next.clock()
    }

    pub fn take_flash_start(&mut self) -> Option<(f32, f32)> {
        self.flash_start.take()
    }

    fn tick_end_flash(&mut self, session: &PlaySession) {
        let clock = self
            .environment
            .default_clock()
            .and_then(|name| session.clock_ticks(name))
            .unwrap_or(0);
        if let Some(flash) = self.end_flash.as_mut() {
            flash.tick(clock);
            if flash.flash_started_this_tick() {
                self.flash_start = Some((flash.x_angle(), flash.y_angle()));
            }
        }
    }

    /// What the dimension's data says at this tick for the blocks that animate.
    pub fn ambience<'a>(&'a self, session: &PlaySession) -> photon_render::sky::SkyAmbience<'a> {
        photon_render::sky::SkyAmbience::new(&self.environment, |clock| {
            session.clock_ticks(clock).unwrap_or(0)
        })
    }

    /// The dimension's sky data from the data pack, the probe's first
    /// values at the player's eyes, and the clouds texture.
    pub fn load(
        data_root: &std::path::Path,
        root: &str,
        session: &PlaySession,
        sprites: SkySprites,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let environment = Environment::load(
            data_root,
            session.dimension_type().unwrap_or("minecraft:overworld"),
            session.registry_names("minecraft:worldgen/biome"),
        )
        .ok_or("the dimension's sky data could not be read")?;
        let stars = photon_render::sky::build_stars();
        let dimension = session.world().dimension();
        let first = probe(&environment, dimension, session, false);
        let cloud_image = crate::play::texture(root, "environment/clouds", "clouds")?;
        let end_flash = end_flash_of(&environment);
        Ok(Self {
            data_root: data_root.to_path_buf(),
            environment,
            stars,
            sprites,
            dimension,
            last: first.clone(),
            next: first,
            unprobed: false,
            rain_fog: 0.0,
            flicker: Flicker::new(),
            clouds: Clouds::new(photon_render::sky::clouds::CloudCells::from_image(
                &cloud_image,
            )),
            end_flash,
            flash_start: None,
        })
    }

    /// A respawn or a change of dimension: the new dimension's sky data is read, the probe starts
    /// from its values at the camera and the clouds are built again.
    pub fn reset(&mut self, session: &PlaySession) {
        let loaded = Environment::load(
            &self.data_root,
            session.dimension_type().unwrap_or("minecraft:overworld"),
            session.registry_names("minecraft:worldgen/biome"),
        );
        if let Some(environment) = loaded {
            self.environment = environment;
        }
        self.end_flash = end_flash_of(&self.environment);
        self.dimension = session.world().dimension();
        self.start_probe(session);
        self.clouds.reset();
    }

    /// The probe starts from its values at the camera, or waits for the player that has none yet.
    fn start_probe(&mut self, session: &PlaySession) {
        self.unprobed = session.player().is_none();
        if !self.unprobed {
            let first = probe(&self.environment, self.dimension, session, false);
            self.last = first.clone();
            self.next = first;
        }
    }

    /// One client tick: the block light flicker, and the camera's attribute
    /// probe while there is a player.
    pub fn tick(&mut self, session: &PlaySession, options: &photon_game::options::Options) {
        let hide_lightning_flashes = options.hide_lightning_flashes;
        self.flicker.tick();
        self.tick_end_flash(session);
        if self.unprobed {
            self.start_probe(session);
        } else if session.player().is_some() {
            let next = probe(
                &self.environment,
                self.dimension,
                session,
                hide_lightning_flashes,
            );
            self.last = std::mem::replace(&mut self.next, next);
        }
    }

    /// The frame's weather, lightmap, clouds, fog and sky; the colour the
    /// frame is cleared to.
    pub fn draw(
        &mut self,
        backend: &mut VulkanBackend,
        session: &PlaySession,
        tints: &BiomeTints,
        camera: &SkyCamera,
    ) -> Result<[f32; 4], Box<dyn std::error::Error>> {
        let partial = camera.partial as f32;
        let attributes = self.last.lerp(&self.next, partial);
        // `AtmosphericFogEnvironment.updateRainFogState`, easing a fifth of
        // the way to its target per tick.
        let target = rain_fog_target(session, tints, &self.environment, camera.eye);
        self.rain_fog += (target - self.rain_fog) * camera.frame_ticks * 0.2;
        let weather = weather(session, tints, camera.eye, camera.origin, partial);
        backend.set_weather(&weather.rain, &weather.snow)?;
        set_world_border(backend, session, camera, partial)?;
        let (y_rot, x_rot) = camera.rotation;
        let look = Vec3::direction_from_rotation_components(x_rot, y_rot);
        let sky = photon_render::sky::build_sky_frame(
            &attributes,
            &photon_render::sky::SkyView {
                forward: [look.x as f32, look.y as f32, look.z as f32],
                eye_above_horizon: camera.eye.y - 63.0,
                fog: camera.fog,
                render_distance: camera.render_distance,
                end_flash: self.flash_view(partial),
                cloud_range: CLOUD_RANGE,
                weather: photon_render::sky::Weather {
                    rain: session.rain_level(),
                    thunder: session.thunder_level(),
                },
                rain_fog: self.rain_fog,
            },
            &self.stars,
            &self.sprites,
            self.environment.skybox(),
        );
        self.set_lightmap(backend, session, (partial, camera));
        self.clouds.draw(
            backend,
            &sky,
            (camera.eye, session.game_time(), partial),
            camera.view_projection,
            CLOUD_RANGE,
        )?;
        backend.set_fog(sky.fog_colour, sky.terrain_fog);
        backend.set_sky(
            &sky.vertices,
            &sky.batches,
            camera.view_projection,
            sky.fog_colour,
            sky.sky_fog_end,
        )?;
        Ok(sky.clear_colour)
    }

    /// The lightmap's inputs, extracted once per tick from the probe's values at the end of the
    /// tick rather than blended within it, and what the player's effects make of them.
    fn set_lightmap(
        &self,
        backend: &mut VulkanBackend,
        session: &PlaySession,
        (partial, camera): (f32, &SkyCamera),
    ) {
        let lightmap = self.next.lightmap();
        let flash = photon_render::sky::end_flash_boost(
            self.end_flash
                .as_ref()
                .map(|flash| flash.intensity(partial)),
            camera.hide_flashes,
            camera.fog.boss_fog,
        );
        let view = super::fog::lightmap_view(
            session,
            partial,
            (camera.fog.darken_world, flash),
            camera.lightmap_options,
        );
        let effects = lightmap.effects(&view, partial);
        backend.set_level_lighting(
            session
                .world()
                .dimension_environment()
                .cardinal_light_type(),
        );
        backend.set_lightmap(&lightmap, 1.4 + self.flicker.value(), &effects);
    }
}

/// The camera's attribute probe at the player's eyes (where they stand at
/// the tick, not between ticks).
fn probe(
    environment: &Environment,
    dimension: photon_world::DimensionBounds,
    session: &PlaySession,
    hide_lightning_flashes: bool,
) -> Attributes {
    let player = session.player().expect("a loaded session has a player");
    let feet = player.position();
    let eye = [feet.x, feet.y + f64::from(player.eye_height()), feet.z];
    environment.evaluate(
        eye,
        |x, y, z| {
            let y = dimension.clamp_quart_y(y);
            session.world().biome_at(x * 4, y * 4, z * 4)
        },
        |clock| session.clock_ticks(clock).unwrap_or(0),
        photon_render::sky::Weather {
            rain: session.rain_level(),
            thunder: session.thunder_level(),
        },
        // `ClientLevel.getSkyFlashTime`: the option hides the flash.
        session.world().sky_flash_time() > 0 && !hide_lightning_flashes,
    )
}
