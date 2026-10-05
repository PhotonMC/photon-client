//! A painting (`PaintingRenderer`): one slab per block of its size, the picture on the front,
//! the plain back on the other, and the narrow edge where it ends, each block lit by where it is.

use voxel_core::direction::Direction;

use crate::joml::{Pose, Quaternion};

/// A sprite's place in its atlas (`TextureAtlasSprite`'s `u0`, `u1`, `v0` and `v1`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteUv {
    pub u0: f32,
    pub u1: f32,
    pub v0: f32,
    pub v1: f32,
}

impl SpriteUv {
    /// `TextureAtlasSprite.getU(offset)`: a point of the sprite along its width.
    pub fn u(&self, offset: f32) -> f32 {
        let diff = self.u1 - self.u0;
        self.u0 + diff * offset
    }

    /// `TextureAtlasSprite.getV(offset)`.
    pub fn v(&self, offset: f32) -> f32 {
        let diff = self.v1 - self.v0;
        self.v0 + diff * offset
    }
}

/// One vertex of a painting, placed: where, which texel, which way it faces, and how it is lit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaintingVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub light: i32,
}

/// `Painting.calculateBoundingBox`'s centre: where a painting hangs, from the block the server
/// named, the way it faces (a horizontal direction) and its size. A painting an even number of
/// blocks wide or tall is shifted half a block towards its left and up.
pub fn painting_center(
    block: (i32, i32, i32),
    direction: Direction,
    (width, height): (i32, i32),
) -> (f64, f64, f64) {
    let offset_for = |size: i32| if size % 2 == 0 { 0.5 } else { 0.0 };
    // `Vec3.atCenterOf(pos).relative(direction, -0.46875)`.
    let (step_x, _, step_z) = direction.step();
    let mut position = (
        f64::from(block.0) + 0.5 + f64::from(step_x) * -0.46875,
        f64::from(block.1) + 0.5,
        f64::from(block.2) + 0.5 + f64::from(step_z) * -0.46875,
    );
    let left = direction.counter_clockwise_y().unwrap_or(direction);
    let (left_x, _, left_z) = left.step();
    let horizontal = offset_for(width);
    position.0 += f64::from(left_x) * horizontal;
    position.2 += f64::from(left_z) * horizontal;
    position.1 += offset_for(height);
    position
}

/// `PaintingRenderer.submit` up to the geometry: turned to face the way the painting hangs.
/// `direction_2d` is the facing's `get2DDataValue` (south 0, west 1, north 2, east 3).
pub fn painting_pose(base: &Pose, direction_2d: i32) -> Pose {
    let mut pose = *base;
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        (180 - direction_2d * 90) as f32,
    )));
    pose
}

/// `PaintingRenderer.renderPainting` from the pose the painting leaves, in the order the
/// reference's vertex consumer receives the vertices (four to a quad). `lights` holds the
/// light of each block, row by row from the bottom.
pub fn painting_vertices(
    pose: &Pose,
    (width, height): (i32, i32),
    lights: &[i32],
    (front, back): (SpriteUv, SpriteUv),
) -> Vec<PaintingVertex> {
    const HALF_DEPTH: f32 = 0.031_25;
    let offset_x = -width as f32 / 2.0;
    let offset_y = -height as f32 / 2.0;
    let (back_u0, back_u1, back_v0, back_v1) = (back.u0, back.u1, back.v0, back.v1);
    let (top_bottom_u0, top_bottom_u1, top_bottom_v0) = (back.u0, back.u1, back.v0);
    let top_bottom_v1 = back.v(0.0625);
    let (left_right_u0, left_right_v0, left_right_v1) = (back.u0, back.v0, back.v1);
    let left_right_u1 = back.u(0.0625);
    let delta_u = 1.0 / f64::from(width);
    let delta_v = 1.0 / f64::from(height);
    let mut out = Vec::new();
    for segment_x in 0..width {
        for segment_y in 0..height {
            let x0 = offset_x + (segment_x + 1) as f32;
            let x1 = offset_x + segment_x as f32;
            let y0 = offset_y + (segment_y + 1) as f32;
            let y1 = offset_y + segment_y as f32;
            let light = lights[(segment_x + segment_y * width) as usize];
            let front_u0 = front.u((delta_u * f64::from(width - segment_x)) as f32);
            let front_u1 = front.u((delta_u * f64::from(width - (segment_x + 1))) as f32);
            let front_v0 = front.v((delta_v * f64::from(height - segment_y)) as f32);
            let front_v1 = front.v((delta_v * f64::from(height - (segment_y + 1))) as f32);
            let mut quad = |corners: [(f32, f32, f32, f32, f32); 4], normal: [i32; 3]| {
                for (x, y, u, v, z) in corners {
                    out.push(PaintingVertex {
                        position: pose.pose.transform_position(x, y, z),
                        uv: [u, v],
                        normal: pose.transform_normal(normal.map(|axis| axis as f32)),
                        light,
                    });
                }
            };
            quad(
                [
                    (x0, y1, front_u1, front_v0, -HALF_DEPTH),
                    (x1, y1, front_u0, front_v0, -HALF_DEPTH),
                    (x1, y0, front_u0, front_v1, -HALF_DEPTH),
                    (x0, y0, front_u1, front_v1, -HALF_DEPTH),
                ],
                [0, 0, -1],
            );
            quad(
                [
                    (x0, y0, back_u1, back_v0, HALF_DEPTH),
                    (x1, y0, back_u0, back_v0, HALF_DEPTH),
                    (x1, y1, back_u0, back_v1, HALF_DEPTH),
                    (x0, y1, back_u1, back_v1, HALF_DEPTH),
                ],
                [0, 0, 1],
            );
            if segment_y == height - 1 {
                quad(
                    [
                        (x0, y0, top_bottom_u0, top_bottom_v0, -HALF_DEPTH),
                        (x1, y0, top_bottom_u1, top_bottom_v0, -HALF_DEPTH),
                        (x1, y0, top_bottom_u1, top_bottom_v1, HALF_DEPTH),
                        (x0, y0, top_bottom_u0, top_bottom_v1, HALF_DEPTH),
                    ],
                    [0, 1, 0],
                );
            }
            if segment_y == 0 {
                quad(
                    [
                        (x0, y1, top_bottom_u0, top_bottom_v0, HALF_DEPTH),
                        (x1, y1, top_bottom_u1, top_bottom_v0, HALF_DEPTH),
                        (x1, y1, top_bottom_u1, top_bottom_v1, -HALF_DEPTH),
                        (x0, y1, top_bottom_u0, top_bottom_v1, -HALF_DEPTH),
                    ],
                    [0, -1, 0],
                );
            }
            if segment_x == width - 1 {
                quad(
                    [
                        (x0, y0, left_right_u1, left_right_v0, HALF_DEPTH),
                        (x0, y1, left_right_u1, left_right_v1, HALF_DEPTH),
                        (x0, y1, left_right_u0, left_right_v1, -HALF_DEPTH),
                        (x0, y0, left_right_u0, left_right_v0, -HALF_DEPTH),
                    ],
                    [-1, 0, 0],
                );
            }
            if segment_x == 0 {
                quad(
                    [
                        (x1, y0, left_right_u1, left_right_v0, -HALF_DEPTH),
                        (x1, y1, left_right_u1, left_right_v1, -HALF_DEPTH),
                        (x1, y1, left_right_u0, left_right_v1, HALF_DEPTH),
                        (x1, y0, left_right_u0, left_right_v0, HALF_DEPTH),
                    ],
                    [1, 0, 0],
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHOLE: SpriteUv = SpriteUv {
        u0: 0.0,
        u1: 1.0,
        v0: 0.0,
        v1: 1.0,
    };

    #[test]
    fn a_painting_hangs_against_the_wall_it_faces_away_from() {
        // Facing south, it is against the block's north wall, a hair inside the block.
        let (x, y, z) = painting_center((10, 64, -3), Direction::South, (1, 1));
        assert_eq!((x, y, z), (10.5, 64.5, -3.0 + 0.5 - 0.46875));
        // An even width shifts it half a block to its left (east of a south-facing painting).
        let (x, y, _) = painting_center((10, 64, -3), Direction::South, (2, 2));
        assert_eq!((x, y), (11.0, 65.0));
    }

    #[test]
    fn a_one_block_painting_is_two_faces_and_four_edges() {
        let vertices = painting_vertices(&Pose::default(), (1, 1), &[0], (WHOLE, WHOLE));
        assert_eq!(vertices.len(), 6 * 4);
    }

    #[test]
    fn a_wide_painting_has_a_slab_per_block_and_edges_only_at_its_ends() {
        let vertices = painting_vertices(&Pose::default(), (3, 2), &[0; 6], (WHOLE, WHOLE));
        // Six blocks, two faces each; top and bottom edges on every column, side edges on both ends.
        assert_eq!(vertices.len(), (6 * 2 + 3 + 3 + 2 + 2) * 4);
    }
}
