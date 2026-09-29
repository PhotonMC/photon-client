//! Cuboid face geometry: which corners a quad takes and how it maps a texture.
//!
//! Every block quad starts here. The corner order decides the winding the GPU
//! sees, and the default UV rectangle decides where a face lands on its sprite
//! when the model does not say. Both are pure functions of the element's box
//! and the face's direction, so they are pinned before anything is drawn.

use voxel_core::direction::Direction;

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

/// Why a quad's winding could not be repaired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindingError;

/// The triangle normal for three corners, as the reference's maths library
/// computes it: the cross product of the two edges leaving the first corner,
/// then normalised.
///
/// The component order is kept because it decides the sign of a zero, and a
/// degenerate triangle divides by zero and yields `NaN` rather than failing.
pub fn triangle_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let a = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let b = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let mut normal = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    // The reference's inverse square root takes the root in double and the
    // reciprocal in float.
    let length_squared = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
    let inverse_length = 1.0_f32 / (f64::from(length_squared).sqrt() as f32);
    for component in normal.iter_mut() {
        *component *= inverse_length;
    }
    normal
}

/// Java `FaceBakery.findClosestDirection`.
///
/// A non-finite normal has no direction at all. Otherwise the best candidate is
/// the one with the largest strictly positive dot product, so a face exactly
/// edge-on to an axis never claims it, and ties keep the earlier direction.
pub fn closest_direction(normal: [f32; 3]) -> Option<Direction> {
    if !normal.iter().all(|component| component.is_finite()) {
        return None;
    }
    const CANDIDATES: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];
    let mut best = None;
    let mut closest = 0.0_f32;
    for candidate in CANDIDATES {
        let unit = direction_unit(candidate);
        let product = normal[0] * unit[0] + normal[1] * unit[1] + normal[2] * unit[2];
        if product >= 0.0 && product > closest {
            closest = product;
            best = Some(candidate);
        }
    }
    best
}

/// The unit vector pointing along one direction.
pub(crate) fn direction_unit(direction: Direction) -> [f32; 3] {
    match direction {
        Direction::Down => [0.0, -1.0, 0.0],
        Direction::Up => [0.0, 1.0, 0.0],
        Direction::North => [0.0, 0.0, -1.0],
        Direction::South => [0.0, 0.0, 1.0],
        Direction::West => [-1.0, 0.0, 0.0],
        Direction::East => [1.0, 0.0, 0.0],
    }
}

/// Java `FaceBakery.recalculateWinding`.
///
/// After a rotation the four corners may be in any order. This puts them back
/// into the order the given direction expects, by finding which corner of the
/// quad's own bounding box each slot wants and swapping it into place. A quad
/// whose corners are not corners of that box - anything not axis aligned - has
/// no such vertex, and the reference raises rather than guessing.
pub fn recalculate_winding(
    positions: &mut [[f32; 3]; 4],
    payloads: &mut [u64; 4],
    direction: Direction,
) -> Result<(), WindingError> {
    // The reference seeds the search with these sentinels rather than with the
    // first vertex, so a quad outside them would not be bounded.
    let mut minimum = [999.0_f32; 3];
    let mut maximum = [-999.0_f32; 3];
    for position in positions.iter() {
        for axis in 0..3 {
            if position[axis] < minimum[axis] {
                minimum[axis] = position[axis];
            }
            if position[axis] > maximum[axis] {
                maximum[axis] = position[axis];
            }
        }
    }

    let corners = face_corners(direction);
    for (slot, selectors) in corners.iter().enumerate() {
        let wanted: [f32; 3] = std::array::from_fn(|axis| match selectors[axis] {
            Extent::MinX => minimum[0],
            Extent::MinY => minimum[1],
            Extent::MinZ => minimum[2],
            Extent::MaxX => maximum[0],
            Extent::MaxY => maximum[1],
            Extent::MaxZ => maximum[2],
        });
        let found = (slot..4).find(|index| positions[*index] == wanted);
        let Some(found) = found else {
            return Err(WindingError);
        };
        if found != slot {
            positions.swap(found, slot);
            payloads.swap(found, slot);
        }
    }
    Ok(())
}

/// A face's texture rotation, which the model gives in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quadrant {
    /// No rotation.
    R0,
    /// A quarter turn.
    R90,
    /// A half turn.
    R180,
    /// Three quarters.
    R270,
}

impl Quadrant {
    /// Java `Quadrant.parseJson`.
    ///
    /// The angle wraps into a positive range first, so a negative or
    /// over-full turn still names a quadrant; anything that is not a multiple
    /// of ninety is rejected rather than rounded.
    pub fn from_degrees(degrees: i32) -> Option<Self> {
        match voxel_core::math::positive_modulo_i32(degrees, 360) {
            0 => Some(Self::R0),
            90 => Some(Self::R90),
            180 => Some(Self::R180),
            270 => Some(Self::R270),
            _ => None,
        }
    }

    /// How many vertex slots the rotation advances by.
    pub fn shift(self) -> u32 {
        match self {
            Self::R0 => 0,
            Self::R90 => 1,
            Self::R180 => 2,
            Self::R270 => 3,
        }
    }

    /// Java `Quadrant.rotateVertexIndex`.
    fn rotate_vertex_index(self, index: u32) -> u32 {
        (index + self.shift()) % 4
    }
}

/// The texture coordinate one face vertex reads, in atlas sixteenths scaled to
/// the unit square.
///
/// The rectangle's corners map to vertex slots in a fixed order - the first two
/// take the low `u`, the first and last take the low `v` - and the rotation
/// picks which slot each vertex reads.
pub fn face_vertex_uv(uvs: FaceUvs, rotation: Quadrant, vertex: u32) -> (f32, f32) {
    let index = rotation.rotate_vertex_index(vertex);
    let u = if index != 0 && index != 1 {
        uvs.max_u
    } else {
        uvs.min_u
    };
    let v = if index != 0 && index != 3 {
        uvs.max_v
    } else {
        uvs.min_v
    };
    (u / 16.0, v / 16.0)
}

/// Java `Direction.getApproximateNearest`: which face a vector points at.
///
/// This is not the same search as [`closest_direction`]. The bar starts at
/// Java's smallest positive float rather than at zero, and there is no
/// fallback to nothing: a vector whose every dot product falls below the bar —
/// the zero vector, or one with a NaN in it — comes back as north, because
/// that is what the search was holding before it began.
pub fn approximate_nearest_direction(vector: [f32; 3]) -> Direction {
    const CANDIDATES: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];
    let mut result = Direction::North;
    // Java's smallest positive float, which is subnormal and so is not the
    // same as Rust's smallest *normal* positive.
    let mut highest = f32::from_bits(1);
    for candidate in CANDIDATES {
        let unit = direction_unit(candidate);
        let product = vector[0] * unit[0] + vector[1] * unit[1] + vector[2] * unit[2];
        if product > highest {
            highest = product;
            result = candidate;
        }
    }
    result
}
