//! The weather around the camera: rain and snow, and the fog rain brings.

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::{BiomeTints, Precipitation};
use photon_render::scene::Vertex;

/// `AtmosphericFogEnvironment.updateRainFogState`'s target: rain pulls the
/// fog in where the camera sees the sky, half as much in a biome without
/// precipitation.
pub fn rain_fog_target(
    session: &PlaySession,
    tints: &BiomeTints,
    environment: &photon_render::sky::Environment,
    eye: Vec3,
) -> f32 {
    let world = session.world();
    let [x, y, z] = [eye.x, eye.y, eye.z].map(|value| value.floor() as i32);
    let biome = tints.biome_at(x, y, z, &|x, y, z| world.noise_biome(x, y, z));
    let precipitation = environment.has_precipitation(i32::try_from(biome).ok());
    let sky_light = f32::from(world.sky_brightness(x, y, z));
    session.rain_level()
        * ((sky_light - 8.0) / 7.0).clamp(0.0, 1.0)
        * if precipitation { 1.0 } else { 0.5 }
}

/// Rain and snow around the camera.
pub fn weather(
    session: &PlaySession,
    tints: &BiomeTints,
    eye: Vec3,
    origin: [i32; 3],
    partial: f32,
) -> photon_render::sky::weather::WeatherMesh {
    let world = session.world();
    let noise_biome = |x: i32, y: i32, z: i32| world.noise_biome(x, y, z);
    let sea_level = world.sea_level();
    let height = |x: i32, z: i32| world.motion_blocking_height(x, z);
    let precipitation = |x: i32, y: i32, z: i32| {
        if !world.contains_chunk(x.div_euclid(16), z.div_euclid(16)) {
            return Precipitation::None;
        }
        tints.precipitation_at([x, y, z], &noise_biome, sea_level)
    };
    let lighter = photon_render::block::light::Lighter {
        state: |[x, y, z]: [i32; 3]| {
            world
                .block_state_at(x, y, z)
                .filter(|state| *state >= 0)
                .map(photon_core::ids::BlockStateId)
        },
        light: |[x, y, z]: [i32; 3]| {
            (
                world.block_brightness(x, y, z),
                world.sky_brightness(x, y, z),
            )
        },
    };
    let light_coords =
        |x: i32, y: i32, z: i32| lighter.light_coords((lighter.state)([x, y, z]), [x, y, z]);
    photon_render::sky::weather::build(
        &photon_render::sky::weather::WeatherWorld {
            height: &height,
            precipitation: &precipitation,
            light_coords: &light_coords,
        },
        [eye.x, eye.y, eye.z],
        origin,
        session.game_time(),
        partial,
        session.rain_level(),
    )
}

/// The world border overlay's triangles, placed like the terrain's:
/// `WorldBorderRenderer.extract` and the quads its `render` lays out, with
/// nothing to draw once `extract` leaves the border transparent.
pub fn world_border(
    session: &PlaySession,
    eye: Vec3,
    origin: [i32; 3],
    partial: f32,
    render_distance_chunks: i32,
    cloud_range_chunks: i32,
    time_millis: i64,
) -> Vec<Vertex> {
    let border = session.world().world_border();
    let min_x = border.min_x(partial);
    let max_x = border.max_x(partial);
    let min_z = border.min_z(partial);
    let max_z = border.max_z(partial);
    let render_distance = f64::from(render_distance_chunks * 16);
    let extract = photon_render::border::extract(border, partial, (eye.x, eye.z), render_distance);
    if extract.alpha <= 0.0 {
        return Vec::new();
    }
    // `Camera.update`'s `depthFar`: four times the render distance, or the
    // cloud range if that is farther.
    let half_height_y =
        ((render_distance_chunks * 16 * 4) as f32).max((cloud_range_chunks * 16) as f32);
    let mut vertices = Vec::new();
    for (_, quad) in photon_render::border::quads(
        min_x,
        max_x,
        min_z,
        max_z,
        [eye.x, eye.y, eye.z],
        origin,
        render_distance,
        half_height_y,
        time_millis,
        extract.tint,
        extract.alpha as f32,
    ) {
        vertices.extend(photon_render::border::triangulate(quad));
    }
    vertices
}

/// `LightmapRenderStateExtractor`'s block light flicker: it wanders by a
/// small random step each tick and decays.
pub struct Flicker {
    random: photon_core::random::RandomSource,
    value: f32,
}

impl Flicker {
    /// The reference seeds this source uniquely per run.
    pub fn new() -> Self {
        Self {
            random: photon_core::random::RandomSource::create(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|elapsed| elapsed.as_nanos() as i64)
                    .unwrap_or(0),
            ),
            value: 0.0,
        }
    }

    pub fn tick(&mut self) {
        let random = &mut self.random;
        self.value += (random.next_float() - random.next_float())
            * random.next_float()
            * random.next_float()
            * 0.1;
        self.value *= 0.9;
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

/// The cloud layer: its cells from the clouds texture, and which cell and
/// side of the layer its faces were last built for.
pub struct Clouds {
    cells: photon_render::sky::clouds::CloudCells,
    built_for: Option<((i32, i32), photon_render::sky::clouds::RelativeCamera)>,
}

impl Clouds {
    pub fn new(cells: photon_render::sky::clouds::CloudCells) -> Self {
        Self {
            cells,
            built_for: None,
        }
    }

    /// Build the faces again on the next frame, as after a respawn.
    pub fn reset(&mut self) {
        self.built_for = None;
    }

    /// The clouds for the frame, when their colour is not fully
    /// transparent; their faces are rebuilt when the camera enters another
    /// cell or crosses the layer.
    pub fn draw(
        &mut self,
        backend: &mut photon_render::VulkanBackend,
        sky: &photon_render::sky::SkyFrame,
        (eye, game_time, partial): (Vec3, i64, f32),
        view_projection: [[f32; 4]; 4],
        cloud_range: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let visible = sky.cloud_colour[3] > 0.0;
        if !visible {
            backend.set_cloud_frame(None);
            return Ok(());
        }
        let placement = photon_render::sky::clouds::place(
            &self.cells,
            sky.cloud_height,
            [eye.x, eye.y, eye.z],
            game_time,
            partial,
        );
        let key = (placement.cell, placement.relative);
        if self.built_for != Some(key) {
            self.built_for = Some(key);
            let faces = self.cells.build(
                placement.relative,
                placement.cell.0,
                placement.cell.1,
                photon_render::sky::clouds::radius_cells(cloud_range),
            );
            backend.set_cloud_faces(&faces)?;
        }
        backend.set_cloud_frame(Some(photon_render::vulkan::CloudFrame {
            view_projection,
            offset: placement.offset,
            fog_end: sky.cloud_fog_end,
            colour: sky.cloud_colour,
        }));
        Ok(())
    }
}
