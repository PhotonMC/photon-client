//! Rain and snow, the way the reference's weather effect renderer lays
//! them out.
//!
//! Around the camera, within the weather radius, every column where it rains
//! or snows gets one quad standing from the terrain (or the camera's reach
//! below) to the camera's reach above, turned to face the centre of the
//! radius. Rain scrolls down its texture at a speed seeded per column; snow
//! drifts by a seeded wander. Columns fade towards the edge of the radius and
//! with the rain level.

use crate::block::biome_tint::Precipitation;
use crate::scene::Vertex;
use photon_core::random::RandomSource;

/// The weather radius option's default, in blocks.
pub const RADIUS: i32 = 10;

/// What the weather columns need from the world.
pub struct WeatherWorld<'a> {
    /// The top of the MOTION_BLOCKING heightmap at a column.
    pub height: &'a dyn Fn(i32, i32) -> i32,
    /// What falls at a position, none where no chunk is loaded.
    pub precipitation: &'a dyn Fn(i32, i32, i32) -> Precipitation,
    /// Packed light coordinates at a position.
    pub light_coords: &'a dyn Fn(i32, i32, i32) -> i32,
}

/// The rain and snow quads, as triangles, placed from `origin`.
#[derive(Debug, Default)]
pub struct WeatherMesh {
    pub rain: Vec<Vertex>,
    pub snow: Vec<Vertex>,
}

/// One column: `WeatherEffectRenderer.ColumnInstance`.
struct Column {
    x: i32,
    z: i32,
    bottom: i32,
    top: i32,
    u_offset: f32,
    v_offset: f32,
    light: i32,
}

/// `WeatherEffectRenderer`'s column facing table: for each cell of a 32 by
/// 32 square around the camera, the quad's half-width direction.
fn column_size(x: i32, z: i32) -> (f32, f32) {
    let delta_x = (x - 16) as f32;
    let delta_z = (z - 16) as f32;
    let distance = (delta_x * delta_x + delta_z * delta_z).sqrt();
    (-delta_z / distance, delta_x / distance)
}

/// Build the frame's weather: `extractRenderState` and `renderInstances`.
pub fn build(
    world: &WeatherWorld,
    camera: [f64; 3],
    origin: [i32; 3],
    game_time: i64,
    partial_tick: f32,
    intensity: f32,
) -> WeatherMesh {
    let mut mesh = WeatherMesh::default();
    if intensity <= 0.0 {
        return mesh;
    }
    let camera_block = camera.map(|value| value.floor() as i32);
    let mut rain = Vec::new();
    let mut snow = Vec::new();
    for z in camera_block[2] - RADIUS..=camera_block[2] + RADIUS {
        for x in camera_block[0] - RADIUS..=camera_block[0] + RADIUS {
            let terrain = (world.height)(x, z);
            let bottom = (camera_block[1] - RADIUS).max(terrain);
            let top = (camera_block[1] + RADIUS).max(terrain);
            if top - bottom == 0 {
                continue;
            }
            let precipitation = (world.precipitation)(x, camera_block[1], z);
            if precipitation == Precipitation::None {
                continue;
            }
            let seed = (x.wrapping_mul(x).wrapping_mul(3121))
                .wrapping_add(x.wrapping_mul(45_238_971))
                ^ (z.wrapping_mul(z).wrapping_mul(418_711)).wrapping_add(z.wrapping_mul(13761));
            let mut random = RandomSource::create(i64::from(seed));
            let light = (world.light_coords)(x, camera_block[1].max(terrain), z);
            let wrapped_ticks = (game_time & 131_071) as i32;
            if precipitation == Precipitation::Rain {
                let tick_offset = (x.wrapping_mul(x).wrapping_mul(3121))
                    .wrapping_add(x.wrapping_mul(45_238_971))
                    .wrapping_add(z.wrapping_mul(z).wrapping_mul(418_711))
                    .wrapping_add(z.wrapping_mul(13761))
                    & 0xFF;
                let speed = 3.0 + random.next_float();
                let offset = -((wrapped_ticks + tick_offset) as f32 + partial_tick) / 32.0 * speed;
                rain.push(Column {
                    x,
                    z,
                    bottom,
                    top,
                    u_offset: 0.0,
                    v_offset: offset % 32.0,
                    light,
                });
            } else {
                let time = wrapped_ticks as f32 + partial_tick;
                let u_base = random.next_double();
                let u = (u_base + f64::from(time * 0.01 * random.next_gaussian() as f32)) as f32;
                let v_base = random.next_double();
                let v = (v_base + f64::from(time * random.next_gaussian() as f32 * 0.001)) as f32;
                let v_offset = -((game_time & 511) as f32 + partial_tick) / 512.0;
                let block = (light & 0xFFFF) >> 4;
                let sky = (light >> 20) & 0xF;
                let brightened =
                    (((block * 3 + 15) / 4) << 4) | ((((sky * 3 + 15) / 4) << 4) << 16);
                snow.push(Column {
                    x,
                    z,
                    bottom,
                    top,
                    u_offset: u,
                    v_offset: v_offset + v,
                    light: brightened,
                });
            }
        }
    }
    let place = |columns: &[Column], max_alpha: f32, out: &mut Vec<Vertex>| {
        let radius_squared = (RADIUS * RADIUS) as f32;
        for column in columns {
            let relative_x = (f64::from(column.x) + 0.5 - camera[0]) as f32;
            let relative_z = (f64::from(column.z) + 0.5 - camera[2]) as f32;
            let distance_squared = relative_x * relative_x + relative_z * relative_z;
            let fade = (distance_squared / radius_squared).min(1.0);
            let alpha = (max_alpha + fade * (0.5 - max_alpha)) * intensity;
            // `ARGB.white`: the alpha as an 8-bit channel.
            let alpha = (alpha * 255.0).floor().clamp(0.0, 255.0) / 255.0;
            let (size_x, size_z) = column_size(
                column.x - camera_block[0] + 16,
                column.z - camera_block[2] + 16,
            );
            let (half_x, half_z) = (size_x / 2.0, size_z / 2.0);
            // From the camera, then moved to the mesh origin.
            let shift = [
                (camera[0] - f64::from(origin[0])) as f32,
                (camera[1] - f64::from(origin[1])) as f32,
                (camera[2] - f64::from(origin[2])) as f32,
            ];
            let x0 = relative_x - half_x + shift[0];
            let x1 = relative_x + half_x + shift[0];
            let y1 = (f64::from(column.top) - camera[1]) as f32 + shift[1];
            let y0 = (f64::from(column.bottom) - camera[1]) as f32 + shift[1];
            let z0 = relative_z - half_z + shift[2];
            let z1 = relative_z + half_z + shift[2];
            let u0 = column.u_offset;
            let u1 = column.u_offset + 1.0;
            let v0 = column.bottom as f32 * 0.25 + column.v_offset;
            let v1 = column.top as f32 * 0.25 + column.v_offset;
            let light = crate::block::light::light_uv(column.light);
            let corner = |position: [f32; 3], uv: [f32; 2]| Vertex {
                position,
                uv,
                shade: 1.0,
                tint: [1.0, 1.0, 1.0, alpha],
                light,
            };
            let corners = [
                corner([x0, y1, z0], [u0, v0]),
                corner([x1, y1, z1], [u1, v0]),
                corner([x1, y0, z1], [u1, v1]),
                corner([x0, y0, z0], [u0, v1]),
            ];
            for index in [0, 1, 2, 2, 3, 0] {
                out.push(corners[index]);
            }
        }
    };
    place(&rain, 1.0, &mut mesh.rain);
    place(&snow, 0.8, &mut mesh.snow);
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_facing_table_turns_across_the_radius() {
        let (x, z) = column_size(16 + 3, 16);
        assert_eq!((x, z), (0.0, 1.0));
        let (x, z) = column_size(16, 16 + 2);
        assert_eq!((x, z), (-1.0, 0.0));
    }

    #[test]
    fn rain_stands_from_the_terrain_to_the_camera_reach() {
        let world = WeatherWorld {
            height: &|_, _| 64,
            precipitation: &|_, _, _| Precipitation::Rain,
            light_coords: &|_, _, _| 15 << 20,
        };
        let mesh = build(&world, [0.5, 70.0, 0.5], [0, 0, 0], 0, 0.0, 1.0);
        assert_eq!(mesh.rain.len(), 21 * 21 * 6);
        assert!(mesh.snow.is_empty());
        let heights: Vec<f32> = mesh.rain.iter().map(|vertex| vertex.position[1]).collect();
        assert!(heights.iter().all(|y| *y == 64.0 || *y == 80.0));
    }
}
