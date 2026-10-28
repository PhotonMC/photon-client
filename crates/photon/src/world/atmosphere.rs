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
        state: |[x, y, z]: [i32; 3]| world.block_state_at(x, y, z),
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
    let wall = photon_render::border::Wall {
        bounds: [min_x, max_x, min_z, max_z],
        render_distance,
        half_height_y,
        time_millis,
        tint: extract.rgba(),
    };
    for (_, quad) in photon_render::border::quads(&wall, [eye.x, eye.y, eye.z], origin) {
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

/// The faces a thread is building for a cell and side of the layer.
type CloudBuild = (
    CloudKey,
    std::sync::mpsc::Receiver<Vec<photon_render::sky::clouds::CloudVertex>>,
);
type CloudKey = ((i32, i32), photon_render::sky::clouds::RelativeCamera);

/// The cloud layer: its cells from the clouds texture, which cell and side of the layer its
/// faces were last built for, and the build under way.
pub struct Clouds {
    cells: std::sync::Arc<photon_render::sky::clouds::CloudCells>,
    built_for: Option<CloudKey>,
    building: Option<CloudBuild>,
}

impl Clouds {
    pub fn new(cells: photon_render::sky::clouds::CloudCells) -> Self {
        Self {
            cells: std::sync::Arc::new(cells),
            built_for: None,
            building: None,
        }
    }

    /// Build the faces again on the next frame, as after a respawn.
    pub fn reset(&mut self) {
        self.built_for = None;
        self.building = None;
    }

    /// Faces for `key`: made on a thread while the old ones stay up, except where there are
    /// none yet.
    fn build(
        &mut self,
        key: CloudKey,
        cloud_range: i32,
    ) -> Option<Vec<photon_render::sky::clouds::CloudVertex>> {
        let radius = photon_render::sky::clouds::radius_cells(cloud_range);
        let cells = self.cells.clone();
        let make = move || cells.build(key.1, key.0.0, key.0.1, radius);
        if self.built_for.is_none() {
            self.building = None;
            return Some(make());
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("Cloud Builder".to_owned())
            .spawn(move || {
                let _ = sender.send(make());
            });
        if spawned.is_err() {
            return Some(self.cells.build(key.1, key.0.0, key.0.1, radius));
        }
        self.building = Some((key, receiver));
        None
    }

    /// The faces a finished build made, taken in.
    fn finished(&mut self) -> Option<(CloudKey, Vec<photon_render::sky::clouds::CloudVertex>)> {
        let (key, receiver) = self.building.as_ref()?;
        match receiver.try_recv() {
            Ok(faces) => {
                let key = *key;
                self.building = None;
                Some((key, faces))
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.building = None;
                None
            }
        }
    }

    /// Where the built cell's corner sits from the camera: the placement's offset, moved by the
    /// cells between (the grid wraps).
    fn offset_for_built(&self, placement: &photon_render::sky::clouds::CloudPlacement) -> [f32; 3] {
        let Some(((built_x, built_z), _)) = self.built_for else {
            return placement.offset;
        };
        let wrap = |delta: i32, width: i32| {
            if delta > width / 2 {
                delta - width
            } else if delta < -(width / 2) {
                delta + width
            } else {
                delta
            }
        };
        let (width, height) = (self.cells.width(), self.cells.height());
        let step = photon_render::sky::clouds::CELL_SIZE;
        let mut offset = placement.offset;
        offset[0] -= wrap(placement.cell.0 - built_x, width) as f32 * step;
        offset[2] -= wrap(placement.cell.1 - built_z, height) as f32 * step;
        offset
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
        if let Some((built, faces)) = self.finished() {
            self.built_for = Some(built);
            backend.set_cloud_faces(&faces)?;
        }
        let wanted = self.built_for != Some(key)
            && self.building.as_ref().map(|(pending, _)| *pending) != Some(key);
        if wanted && let Some(faces) = self.build(key, cloud_range) {
            self.built_for = Some(key);
            backend.set_cloud_faces(&faces)?;
        }
        backend.set_cloud_frame(Some(photon_render::vulkan::CloudFrame {
            view_projection,
            offset: self.offset_for_built(&placement),
            fog_end: sky.cloud_fog_end,
            colour: sky.cloud_colour,
        }));
        Ok(())
    }
}
