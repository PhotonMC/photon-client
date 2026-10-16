//! What an entity's shadow rests on (`EntityRenderer.extractShadow`): the
//! squares of the blocks beneath it that are lit enough and whole enough to
//! take a shadow, and how dark each is.

use photon_core::math::floor_f64;

use crate::ClientWorld;
use crate::block::state;

/// What the shadow of one entity is worked out from.
#[derive(Debug, Clone, Copy)]
pub struct ShadowInput {
    /// Where the entity is (the render state's `x`, `y`, `z`).
    pub position: (f64, f64, f64),
    /// The renderer's `getShadowRadius`, before the cap at 32.
    pub radius: f32,
    /// The renderer's `getShadowStrength`.
    pub strength: f32,
    /// `distanceToCameraSq`.
    pub distance_sq: f64,
    /// `Level.getSkyDarken`: how much the sky's light is dimmed.
    pub sky_darken: i32,
}

/// One block's square of shadow (`EntityRenderState.ShadowPiece`): where its
/// cell is from the entity, the bounds of the shape below, and the alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowPiece {
    pub relative: [f32; 3],
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub alpha: f32,
}

/// Whether the block's `getRenderShape` is `INVISIBLE`.
pub(crate) fn is_invisible_render_shape(name: &str) -> bool {
    INVISIBLE.contains(&name)
}

/// Blocks whose `getRenderShape` is `INVISIBLE`: they take no shadow.
const INVISIBLE: [&str; 12] = [
    "minecraft:air",
    "minecraft:cave_air",
    "minecraft:void_air",
    "minecraft:barrier",
    "minecraft:bubble_column",
    "minecraft:end_portal",
    "minecraft:end_gateway",
    "minecraft:light",
    "minecraft:water",
    "minecraft:lava",
    "minecraft:structure_void",
    "minecraft:moving_piston",
];

/// `Lightmap.getBrightness`: how bright a light level looks.
fn brightness(ambient_light: f32, level: i32) -> f32 {
    let v = level as f32 / 15.0;
    let curved = v / (4.0 - 3.0 * v);
    curved + ambient_light * (1.0 - curved)
}

impl ClientWorld {
    /// How bright the light at a point looks (`Lightmap.getBrightness` of
    /// `getMaxLocalRawBrightness` at the block the point is in), for what is laid over the view.
    /// The sky's dimming (`gameplay/sky_light_level`) is not carried.
    pub fn eye_brightness(&self, point: [f64; 3]) -> f32 {
        let at = (
            floor_f64(point[0]),
            floor_f64(point[1]),
            floor_f64(point[2]),
        );
        brightness(
            self.dimension_environment().ambient_light(),
            self.max_local_raw_brightness(at, 0),
        )
    }

    /// `getMaxLocalRawBrightness(pos, skyDarken)`.
    fn max_local_raw_brightness(&self, (x, y, z): (i32, i32, i32), sky_darken: i32) -> i32 {
        if !(-30_000_000..30_000_000).contains(&x) || !(-30_000_000..30_000_000).contains(&z) {
            return 15;
        }
        let sky = i32::from(self.sky_brightness(x, y, z)) - sky_darken;
        sky.max(i32::from(self.block_brightness(x, y, z)))
    }

    /// `EntityRenderer.extractShadow` for a visible entity with the shadows
    /// option on: the pieces, in the order the reference finds them.
    pub fn entity_shadow_pieces(&self, input: ShadowInput) -> Vec<ShadowPiece> {
        let mut pieces = Vec::new();
        let radius = input.radius.min(32.0);
        if radius <= 0.0 {
            return pieces;
        }
        let (x, y, z) = input.position;
        let pow = ((1.0 - input.distance_sq / 256.0) * f64::from(input.strength)) as f32;
        if pow <= 0.0 {
            return pieces;
        }
        let x0 = floor_f64(x - f64::from(radius));
        let x1 = floor_f64(x + f64::from(radius));
        let z0 = floor_f64(z - f64::from(radius));
        let z1 = floor_f64(z + f64::from(radius));
        let depth = (pow / 0.5 - 1.0).min(radius);
        let y0 = floor_f64(y - f64::from(depth));
        let y1 = floor_f64(y);
        for block_z in z0..=z1 {
            for block_x in x0..=x1 {
                for block_y in y0..=y1 {
                    let power_at_depth = pow - (y - f64::from(block_y)) as f32 * 0.5;
                    let Some((below_id, below)) = self
                        .block_state_at(block_x, block_y - 1, block_z)
                        .and_then(|id| Some((id, state(id)?)))
                    else {
                        continue;
                    };
                    if INVISIBLE.contains(&below.block().name) {
                        continue;
                    }
                    let level = self
                        .max_local_raw_brightness((block_x, block_y, block_z), input.sky_darken);
                    if level <= 3 || !below.collision_full_block {
                        continue;
                    }
                    let shape = crate::outline_shape(below_id, [block_x, block_y - 1, block_z]);
                    let Some(bounds) = shape.bounds().filter(|_| !shape.is_empty()) else {
                        continue;
                    };
                    let alpha = (power_at_depth
                        * 0.5
                        * brightness(self.dimension_environment().ambient_light(), level))
                    .clamp(0.0, 1.0);
                    pieces.push(ShadowPiece {
                        relative: [
                            (f64::from(block_x) - x) as f32,
                            (f64::from(block_y) - y) as f32,
                            (f64::from(block_z) - z) as f32,
                        ],
                        min: [
                            bounds.min_x as f32,
                            bounds.min_y as f32,
                            bounds.min_z as f32,
                        ],
                        max: [
                            bounds.max_x as f32,
                            bounds.max_y as f32,
                            bounds.max_z as f32,
                        ],
                        alpha,
                    });
                }
            }
        }
        pieces
    }
}
