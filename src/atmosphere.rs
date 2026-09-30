//! The weather around the camera: rain and snow, and the fog rain brings.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::biome_tint::{BiomeTints, Precipitation};

/// `AtmosphericFogEnvironment.updateRainFogState`'s target: rain pulls the
/// fog in where the camera sees the sky, half as much in a biome without
/// precipitation.
pub fn rain_fog_target(
    session: &PlaySession,
    tints: &BiomeTints,
    environment: &voxel_render::sky::Environment,
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
) -> voxel_render::weather::WeatherMesh {
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
    let lighter = voxel_render::block_light::Lighter {
        state: |[x, y, z]: [i32; 3]| {
            world
                .block_state_at(x, y, z)
                .and_then(|state| u32::try_from(state).ok())
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
    voxel_render::weather::build(
        &voxel_render::weather::WeatherWorld {
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
