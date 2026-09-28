//! The light a block's quads are drawn with: which light levels each corner
//! reads and how dark its ambient occlusion makes it.
//!
//! This is the reference's model lighter. With smooth lighting on, every
//! corner of a quad blends the light and the occlusion shade of the blocks
//! around it, weighted by where a partial face sits in its block; a block that
//! emits light, or a model that asks for no occlusion, is lit flat from the
//! one block its face looks into. Light levels are packed as the reference
//! packs them: the block level in the low half, the sky level in the high
//! half, each in sixteenths.

use crate::direction::Direction;
use crate::world::block_light_table::{BLOCKS, LightTraits, TRAITS};
use crate::world::block_registry;

/// Full brightness in both halves.
pub const FULL_BRIGHT: i32 = 15_728_880;

fn traits(state: Option<u32>) -> &'static LightTraits {
    let index = state
        .and_then(block_registry::block_index_for_state)
        .map(|(block, offset)| {
            let states = BLOCKS[block];
            if states.len() == 1 {
                states[0]
            } else {
                states[offset as usize]
            }
        })
        .unwrap_or(0);
    &TRAITS[usize::from(index)]
}

/// The lighting traits of a state: emission, emissive, shade, view blocking,
/// dampening, solid rendering and a full collision shape.
pub fn light_traits(state: Option<u32>) -> &'static LightTraits {
    traits(state)
}

fn pack(block: i32, sky: i32) -> i32 {
    (block << 4) | (sky << 20)
}

fn block_of(packed: i32) -> i32 {
    (packed >> 4) & 15
}

fn sky_of(packed: i32) -> i32 {
    (packed >> 20) & 15
}

/// `LightCoordsUtil.smoothBlend`.
fn smooth_blend(mut first: i32, mut second: i32, mut third: i32, centre: i32) -> i32 {
    if sky_of(centre) > 2 || block_of(centre) > 2 {
        for neighbour in [&mut first, &mut second, &mut third] {
            if *neighbour == 0 {
                *neighbour = centre;
            } else if sky_of(*neighbour) == 0 {
                *neighbour |= centre & 0xFF_0000;
            }
        }
    }
    ((first + second + third + centre) >> 2) & 0xFF_00FF
}

/// `LightCoordsUtil.smoothWeightedBlend`.
fn smooth_weighted_blend(coords: [i32; 4], weights: [f32; 4]) -> i32 {
    let sky: f32 = coords
        .iter()
        .zip(weights)
        .map(|(coords, weight)| ((coords >> 16) & 0xFF) as f32 * weight)
        .fold(0.0, |sum, value| sum + value);
    let block: f32 = coords
        .iter()
        .zip(weights)
        .map(|(coords, weight)| (coords & 0xFF) as f32 * weight)
        .fold(0.0, |sum, value| sum + value);
    ((block as i32) & 0xFF) | (((sky as i32) & 0xFF) << 16)
}

/// The side shade the default dimension gives each face.
pub fn cardinal(direction: Direction) -> f32 {
    match direction {
        Direction::Down => 0.5,
        Direction::Up => 1.0,
        Direction::North | Direction::South => 0.8,
        Direction::West | Direction::East => 0.6,
    }
}

// The face shape's slots: the bounds, then one minus each bound.
const DOWN: usize = 0;
const UP: usize = 1;
const NORTH: usize = 2;
const SOUTH: usize = 3;
const WEST: usize = 4;
const EAST: usize = 5;
const FLIP_DOWN: usize = 6;
const FLIP_UP: usize = 7;
const FLIP_NORTH: usize = 8;
const FLIP_SOUTH: usize = 9;
const FLIP_WEST: usize = 10;
const FLIP_EAST: usize = 11;

/// `AdjacencyInfo`: the four sides to step to around a face, and each
/// corner's weights, as pairs of face shape slots.
struct Adjacency {
    corners: [Direction; 4],
    weights: [[usize; 8]; 4],
}

fn adjacency(direction: Direction) -> Adjacency {
    use Direction as D;
    match direction {
        D::Down => Adjacency {
            corners: [D::West, D::East, D::North, D::South],
            weights: [
                [
                    FLIP_WEST, SOUTH, FLIP_WEST, FLIP_SOUTH, WEST, FLIP_SOUTH, WEST, SOUTH,
                ],
                [
                    FLIP_WEST, NORTH, FLIP_WEST, FLIP_NORTH, WEST, FLIP_NORTH, WEST, NORTH,
                ],
                [
                    FLIP_EAST, NORTH, FLIP_EAST, FLIP_NORTH, EAST, FLIP_NORTH, EAST, NORTH,
                ],
                [
                    FLIP_EAST, SOUTH, FLIP_EAST, FLIP_SOUTH, EAST, FLIP_SOUTH, EAST, SOUTH,
                ],
            ],
        },
        D::Up => Adjacency {
            corners: [D::East, D::West, D::North, D::South],
            weights: [
                [
                    EAST, SOUTH, EAST, FLIP_SOUTH, FLIP_EAST, FLIP_SOUTH, FLIP_EAST, SOUTH,
                ],
                [
                    EAST, NORTH, EAST, FLIP_NORTH, FLIP_EAST, FLIP_NORTH, FLIP_EAST, NORTH,
                ],
                [
                    WEST, NORTH, WEST, FLIP_NORTH, FLIP_WEST, FLIP_NORTH, FLIP_WEST, NORTH,
                ],
                [
                    WEST, SOUTH, WEST, FLIP_SOUTH, FLIP_WEST, FLIP_SOUTH, FLIP_WEST, SOUTH,
                ],
            ],
        },
        D::North => Adjacency {
            corners: [D::Up, D::Down, D::East, D::West],
            weights: [
                [UP, FLIP_WEST, UP, WEST, FLIP_UP, WEST, FLIP_UP, FLIP_WEST],
                [UP, FLIP_EAST, UP, EAST, FLIP_UP, EAST, FLIP_UP, FLIP_EAST],
                [
                    DOWN, FLIP_EAST, DOWN, EAST, FLIP_DOWN, EAST, FLIP_DOWN, FLIP_EAST,
                ],
                [
                    DOWN, FLIP_WEST, DOWN, WEST, FLIP_DOWN, WEST, FLIP_DOWN, FLIP_WEST,
                ],
            ],
        },
        D::South => Adjacency {
            corners: [D::West, D::East, D::Down, D::Up],
            weights: [
                [UP, FLIP_WEST, FLIP_UP, FLIP_WEST, FLIP_UP, WEST, UP, WEST],
                [
                    DOWN, FLIP_WEST, FLIP_DOWN, FLIP_WEST, FLIP_DOWN, WEST, DOWN, WEST,
                ],
                [
                    DOWN, FLIP_EAST, FLIP_DOWN, FLIP_EAST, FLIP_DOWN, EAST, DOWN, EAST,
                ],
                [UP, FLIP_EAST, FLIP_UP, FLIP_EAST, FLIP_UP, EAST, UP, EAST],
            ],
        },
        D::West => Adjacency {
            corners: [D::Up, D::Down, D::North, D::South],
            weights: [
                [
                    UP, SOUTH, UP, FLIP_SOUTH, FLIP_UP, FLIP_SOUTH, FLIP_UP, SOUTH,
                ],
                [
                    UP, NORTH, UP, FLIP_NORTH, FLIP_UP, FLIP_NORTH, FLIP_UP, NORTH,
                ],
                [
                    DOWN, NORTH, DOWN, FLIP_NORTH, FLIP_DOWN, FLIP_NORTH, FLIP_DOWN, NORTH,
                ],
                [
                    DOWN, SOUTH, DOWN, FLIP_SOUTH, FLIP_DOWN, FLIP_SOUTH, FLIP_DOWN, SOUTH,
                ],
            ],
        },
        D::East => Adjacency {
            corners: [D::Down, D::Up, D::North, D::South],
            weights: [
                [
                    FLIP_DOWN, SOUTH, FLIP_DOWN, FLIP_SOUTH, DOWN, FLIP_SOUTH, DOWN, SOUTH,
                ],
                [
                    FLIP_DOWN, NORTH, FLIP_DOWN, FLIP_NORTH, DOWN, FLIP_NORTH, DOWN, NORTH,
                ],
                [
                    FLIP_UP, NORTH, FLIP_UP, FLIP_NORTH, UP, FLIP_NORTH, UP, NORTH,
                ],
                [
                    FLIP_UP, SOUTH, FLIP_UP, FLIP_SOUTH, UP, FLIP_SOUTH, UP, SOUTH,
                ],
            ],
        },
    }
}

/// `AmbientVertexRemap`: which quad corner each computed corner lands on.
fn remap(direction: Direction) -> [usize; 4] {
    match direction {
        Direction::Down | Direction::South => [0, 1, 2, 3],
        Direction::Up => [2, 3, 0, 1],
        Direction::North | Direction::West => [3, 0, 1, 2],
        Direction::East => [1, 2, 3, 0],
    }
}

fn offset(position: [i32; 3], direction: Direction) -> [i32; 3] {
    let (x, y, z) = direction.step();
    [position[0] + x, position[1] + y, position[2] + z]
}

/// The level as the lighter reads it.
pub struct Lighter<S, L> {
    /// The block state at a position, or nothing where no chunk is loaded.
    pub state: S,
    /// The block and sky light levels at a position.
    pub light: L,
}

/// One quad's lighting: each corner's shade (before the side shade) and its
/// packed light coordinates, in the quad's own corner order.
pub struct QuadLight {
    pub shade: [f32; 4],
    pub light: [i32; 4],
}

impl<S, L> Lighter<S, L>
where
    S: Fn([i32; 3]) -> Option<u32>,
    L: Fn([i32; 3]) -> (u8, u8),
{
    /// `LightCoordsUtil.getLightCoords` for a state standing at a position.
    pub fn light_coords(&self, state: Option<u32>, position: [i32; 3]) -> i32 {
        let traits = traits(state);
        if traits.emissive {
            return FULL_BRIGHT;
        }
        let (block, sky) = (self.light)(position);
        let packed = pack(i32::from(block), i32::from(sky));
        let emission = i32::from(traits.emission);
        if i32::from(block) < emission {
            (packed & 0xFF_0000) | (emission << 4)
        } else {
            packed
        }
    }

    fn light_at(&self, position: [i32; 3]) -> i32 {
        self.light_coords((self.state)(position), position)
    }

    fn shade_at(&self, position: [i32; 3]) -> f32 {
        f32::from_bits(traits((self.state)(position)).shade)
    }

    /// The lit colour and light of a quad drawn at `position` for `state`.
    ///
    /// `positions` are the quad's corners inside its block. `cull` is the
    /// side the face is hidden behind, if it names one; `smooth` is whether
    /// smooth lighting applies to this block and model.
    pub fn light_quad(
        &self,
        state: u32,
        position: [i32; 3],
        positions: &[[f32; 3]; 4],
        direction: Direction,
        cull: Option<Direction>,
        smooth: bool,
    ) -> QuadLight {
        let traits_here = traits(Some(state));
        let mut min = [32.0_f32; 3];
        let mut max = [-32.0_f32; 3];
        for corner in positions {
            for axis in 0..3 {
                min[axis] = min[axis].min(corner[axis]);
                max[axis] = max[axis].max(corner[axis]);
            }
        }
        let full = traits_here.collision_full_block;
        let face_cubic = match direction {
            Direction::Down => min[1] == max[1] && (min[1] < 1.0e-4 || full),
            Direction::Up => min[1] == max[1] && (max[1] > 0.9999 || full),
            Direction::North => min[2] == max[2] && (min[2] < 1.0e-4 || full),
            Direction::South => min[2] == max[2] && (max[2] > 0.9999 || full),
            Direction::West => min[0] == max[0] && (min[0] < 1.0e-4 || full),
            Direction::East => min[0] == max[0] && (max[0] > 0.9999 || full),
        };

        if !smooth {
            // Flat: a face hidden behind a side reads that side's block; any
            // other reads the block it faces if it lies on the block's edge,
            // else its own. Both read with this block's own emission.
            let light_position = match cull {
                Some(side) => offset(position, side),
                None if face_cubic => offset(position, direction),
                None => position,
            };
            let light = self.light_coords(Some(state), light_position);
            return QuadLight {
                shade: [1.0; 4],
                light: [light; 4],
            };
        }

        let face_partial = match direction {
            Direction::Down | Direction::Up => {
                min[0] >= 1.0e-4 || min[2] >= 1.0e-4 || max[0] <= 0.9999 || max[2] <= 0.9999
            }
            Direction::North | Direction::South => {
                min[0] >= 1.0e-4 || min[1] >= 1.0e-4 || max[0] <= 0.9999 || max[1] <= 0.9999
            }
            Direction::West | Direction::East => {
                min[1] >= 1.0e-4 || min[2] >= 1.0e-4 || max[1] <= 0.9999 || max[2] <= 0.9999
            }
        };
        let mut shape = [0.0_f32; 12];
        shape[WEST] = min[0];
        shape[EAST] = max[0];
        shape[DOWN] = min[1];
        shape[UP] = max[1];
        shape[NORTH] = min[2];
        shape[SOUTH] = max[2];
        shape[FLIP_WEST] = 1.0 - min[0];
        shape[FLIP_EAST] = 1.0 - max[0];
        shape[FLIP_DOWN] = 1.0 - min[1];
        shape[FLIP_UP] = 1.0 - max[1];
        shape[FLIP_NORTH] = 1.0 - min[2];
        shape[FLIP_SOUTH] = 1.0 - max[2];

        let base = if face_cubic {
            offset(position, direction)
        } else {
            position
        };
        let info = adjacency(direction);
        let around: [[i32; 3]; 4] = info.corners.map(|side| offset(base, side));
        let light = around.map(|p| self.light_at(p));
        let shade = around.map(|p| self.shade_at(p));
        let translucent = around.map(|p| {
            let beyond = offset(p, direction);
            let traits = traits((self.state)(beyond));
            !traits.view_blocking || traits.dampening == 0
        });
        // The reference falls back to the first corner's values for every
        // diagonal whose two sides are both opaque.
        let diagonal = |first: usize, second: usize| {
            if !translucent[second] && !translucent[first] {
                (shade[0], light[0])
            } else {
                let p = offset(around[first], info.corners[second]);
                (self.shade_at(p), self.light_at(p))
            }
        };
        let (shade02, light02) = diagonal(0, 2);
        let (shade03, light03) = diagonal(0, 3);
        let (shade12, light12) = diagonal(1, 2);
        let (shade13, light13) = diagonal(1, 3);

        let mut light_centre = self.light_coords(Some(state), position);
        let next = offset(position, direction);
        let next_state = (self.state)(next);
        if face_cubic || !traits(next_state).solid_render {
            light_centre = self.light_coords(next_state, next);
        }
        let shade_centre = if face_cubic {
            self.shade_at(base)
        } else {
            self.shade_at(position)
        };
        let order = remap(direction);

        let level1 = (shade[3] + shade[0] + shade03 + shade_centre) * 0.25;
        let level2 = (shade[2] + shade[0] + shade02 + shade_centre) * 0.25;
        let level3 = (shade[2] + shade[1] + shade12 + shade_centre) * 0.25;
        let level4 = (shade[3] + shade[1] + shade13 + shade_centre) * 0.25;
        let blend1 = smooth_blend(light[3], light[0], light03, light_centre);
        let blend2 = smooth_blend(light[2], light[0], light02, light_centre);
        let blend3 = smooth_blend(light[2], light[1], light12, light_centre);
        let blend4 = smooth_blend(light[3], light[1], light13, light_centre);

        let mut out = QuadLight {
            shade: [0.0; 4],
            light: [0; 4],
        };
        if face_partial {
            for (vertex, weights) in info.weights.iter().enumerate() {
                let w = [
                    shape[weights[0]] * shape[weights[1]],
                    shape[weights[2]] * shape[weights[3]],
                    shape[weights[4]] * shape[weights[5]],
                    shape[weights[6]] * shape[weights[7]],
                ];
                let value = level1 * w[0] + level2 * w[1] + level3 * w[2] + level4 * w[3];
                out.shade[order[vertex]] = value.clamp(0.0, 1.0);
                out.light[order[vertex]] =
                    smooth_weighted_blend([blend1, blend2, blend3, blend4], w);
            }
        } else {
            let levels = [level1, level2, level3, level4];
            let blends = [blend1, blend2, blend3, blend4];
            for vertex in 0..4 {
                out.shade[order[vertex]] = levels[vertex];
                out.light[order[vertex]] = blends[vertex];
            }
        }
        out
    }
}

/// The two lightmap coordinates a packed light value gives a vertex.
pub fn light_uv(packed: i32) -> [f32; 2] {
    [(packed & 0xFFFF) as f32, ((packed >> 16) & 0xFFFF) as f32]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stone() -> u32 {
        block_registry::block("minecraft:stone").expect("stone").default_state_id()
    }

    #[test]
    fn an_open_top_face_is_unshaded_and_fully_sky_lit() {
        let lighter = Lighter {
            state: |p: [i32; 3]| (p == [0, 0, 0]).then(stone),
            light: |_: [i32; 3]| (0, 15),
        };
        let top = [[0.0, 1.0, 0.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]];
        let lit = lighter.light_quad(stone(), [0, 0, 0], &top, Direction::Up, Some(Direction::Up), true);
        assert_eq!(lit.shade, [1.0; 4]);
        assert_eq!(lit.light.map(light_uv), [[0.0, 240.0]; 4]);
    }

    #[test]
    fn a_face_beside_a_wall_is_darkened_along_it() {
        let lighter = Lighter {
            state: |p: [i32; 3]| (p == [0, 0, 0] || p == [1, 1, 0]).then(stone),
            light: |_: [i32; 3]| (0, 15),
        };
        let top = [[0.0, 1.0, 0.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]];
        let lit = lighter.light_quad(stone(), [0, 0, 0], &top, Direction::Up, Some(Direction::Up), true);
        // The two corners along the east wall take its occlusion shade.
        assert!(lit.shade[2] < 1.0 && lit.shade[3] < 1.0);
        assert_eq!(lit.shade[0], 1.0);
        assert_eq!(smooth_blend(0, 0, 0, pack(0, 15)), pack(0, 15));
    }
}
