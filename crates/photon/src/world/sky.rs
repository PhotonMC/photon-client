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
    rain_fog: f32,
    flicker: Flicker,
    clouds: Clouds,
}

impl Sky {
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
        Ok(Self {
            data_root: data_root.to_path_buf(),
            environment,
            stars,
            sprites,
            dimension,
            last: first.clone(),
            next: first,
            rain_fog: 0.0,
            flicker: Flicker::new(),
            clouds: Clouds::new(photon_render::sky::clouds::CloudCells::from_image(
                &cloud_image,
            )),
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
        self.dimension = session.world().dimension();
        let first = probe(&self.environment, self.dimension, session, false);
        self.last = first.clone();
        self.next = first;
        self.clouds.reset();
    }

    /// One client tick: the block light flicker, and the camera's attribute
    /// probe while there is a player.
    pub fn tick(&mut self, session: &PlaySession, hide_lightning_flashes: bool) {
        self.flicker.tick();
        if session.player().is_some() {
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
        let time_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis() as i64);
        let border_vertices = world_border(
            session,
            camera.eye,
            camera.origin,
            partial,
            camera.render_distance,
            CLOUD_RANGE,
            time_millis,
        );
        backend.set_world_border(&border_vertices)?;
        let (y_rot, x_rot) = camera.rotation;
        let look = Vec3::direction_from_rotation_components(x_rot, y_rot);
        let sky = photon_render::sky::build_sky_frame(
            &attributes,
            &photon_render::sky::SkyView {
                forward: [look.x as f32, look.y as f32, look.z as f32],
                eye_above_horizon: camera.eye.y - 63.0,
                fog: camera.fog,
                render_distance: camera.render_distance,
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
        self.set_lightmap(backend, session, (partial, camera.fog.darken_world));
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
        (partial, boss_darkening): (f32, f32),
    ) {
        let lightmap = self.next.lightmap();
        let view = super::fog::lightmap_view(session, partial, boss_darkening);
        let effects = lightmap.effects(&view, partial);
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
