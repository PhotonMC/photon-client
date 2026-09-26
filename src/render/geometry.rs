//! Cuboid face geometry: which corners a quad takes and how it maps a texture.
//!
//! Every block quad starts here. The corner order decides the winding the GPU
//! sees, and the default UV rectangle decides where a face lands on its sprite
//! when the model does not say. Both are pure functions of the element's box
//! and the face's direction, so they are pinned before anything is drawn.

use crate::direction::Direction;

/// The eight corner selectors a face vertex can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Extent {
    MinX,
    MinY,
    MinZ,
    MaxX,
    MaxY,
    MaxZ,
}

impl Extent {
    fn select(self, from: [f32; 3], to: [f32; 3]) -> f32 {
        match self {
            Self::MinX => from[0],
            Self::MinY => from[1],
            Self::MinZ => from[2],
            Self::MaxX => to[0],
            Self::MaxY => to[1],
            Self::MaxZ => to[2],
        }
    }
}

/// The four corners of one face, in the reference's winding order.
///
/// The order is per direction and is what the whole pipeline depends on; a
/// face wound the other way is culled rather than mirrored.
const fn face_corners(facing: Direction) -> [[Extent; 3]; 4] {
    use Extent::{MaxX, MaxY, MaxZ, MinX, MinY, MinZ};
    match facing {
        Direction::Down => [
            [MinX, MinY, MaxZ],
            [MinX, MinY, MinZ],
            [MaxX, MinY, MinZ],
            [MaxX, MinY, MaxZ],
        ],
        Direction::Up => [
            [MinX, MaxY, MinZ],
            [MinX, MaxY, MaxZ],
            [MaxX, MaxY, MaxZ],
            [MaxX, MaxY, MinZ],
        ],
        Direction::North => [
            [MaxX, MaxY, MinZ],
            [MaxX, MinY, MinZ],
            [MinX, MinY, MinZ],
            [MinX, MaxY, MinZ],
        ],
        Direction::South => [
            [MinX, MaxY, MaxZ],
            [MinX, MinY, MaxZ],
            [MaxX, MinY, MaxZ],
            [MaxX, MaxY, MaxZ],
        ],
        Direction::West => [
            [MinX, MaxY, MinZ],
            [MinX, MinY, MinZ],
            [MinX, MinY, MaxZ],
            [MinX, MaxY, MaxZ],
        ],
        Direction::East => [
            [MaxX, MaxY, MaxZ],
            [MaxX, MinY, MaxZ],
            [MaxX, MinY, MinZ],
            [MaxX, MaxY, MinZ],
        ],
    }
}

/// The texture rectangle a face uses when the model names none.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceUvs {
    /// Left edge in sixteenths.
    pub min_u: f32,
    /// Top edge in sixteenths.
    pub min_v: f32,
    /// Right edge in sixteenths.
    pub max_u: f32,
    /// Bottom edge in sixteenths.
    pub max_v: f32,
}

/// Java `FaceBakery.defaultFaceUV`.
///
/// Each direction reads a different pair of axes, and four of the six flip one
/// of them, which is what keeps a block's texture upright from every side.
pub fn default_face_uv(from: [f32; 3], to: [f32; 3], facing: Direction) -> FaceUvs {
    let (min_u, min_v, max_u, max_v) = match facing {
        Direction::Down => (from[0], 16.0 - to[2], to[0], 16.0 - from[2]),
        Direction::Up => (from[0], from[2], to[0], to[2]),
        Direction::North => (16.0 - to[0], 16.0 - to[1], 16.0 - from[0], 16.0 - from[1]),
        Direction::South => (from[0], 16.0 - to[1], to[0], 16.0 - from[1]),
        Direction::West => (from[2], 16.0 - to[1], to[2], 16.0 - from[1]),
        Direction::East => (16.0 - to[2], 16.0 - to[1], 16.0 - from[2], 16.0 - from[1]),
    };
    FaceUvs {
        min_u,
        min_v,
        max_u,
        max_v,
    }
}

/// The four vertex positions of one face, in block units.
///
/// The reference divides by sixteen as soon as it has selected the corner, so
/// the division happens here too rather than at a later stage.
pub fn face_vertices(from: [f32; 3], to: [f32; 3], facing: Direction) -> [[f32; 3]; 4] {
    let corners = face_corners(facing);
    let mut out = [[0.0_f32; 3]; 4];
    for (vertex, selectors) in out.iter_mut().zip(corners) {
        for (axis, selector) in vertex.iter_mut().zip(selectors) {
            *axis = selector.select(from, to) / 16.0;
        }
    }
    out
}
