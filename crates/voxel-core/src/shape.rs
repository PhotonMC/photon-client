//! Voxel shapes for the first P01.d geometry slice.
//!
//! A shape stores coordinate planes and the occupied cells between them. This
//! is enough to preserve Java's box, move, supported BooleanOp joins, single
//! shape collision, VoxelShape clip hits, closest points, face/occlusion, rotations, axis variants, bounds,
//! box iteration, object identity, and text formatting behavior
//! without choosing a broader collision representation yet.

use crate::aabb::{Aabb, BlockHitResult};
use crate::binary::BlockPos;
use crate::direction::{Axis, AxisDirection, Direction};
use crate::math::{Vec3, clamp_f64};
use std::sync::{Arc, OnceLock};

const EPSILON: f64 = 1.0e-7;

/// Errors raised while constructing a shape with reversed bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeError {
    ReversedBounds,
}

/// Boolean operation supported by the current shape join slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    And,
    False,
    First,
    NotSame,
    OnlyFirst,
    OnlySecond,
    Or,
    Second,
}

impl BooleanOp {
    fn apply(self, first: bool, second: bool) -> bool {
        match self {
            Self::And => first && second,
            Self::False => false,
            Self::First => first,
            Self::NotSame => first != second,
            Self::OnlyFirst => first && !second,
            Self::OnlySecond => second && !first,
            Self::Or => first || second,
            Self::Second => second,
        }
    }
}

/// Signed axis permutations supported by the current shape rotation slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeRotation {
    Identity,
    Rot90XNeg,
    Rot90XPos,
    Rot90YNeg,
    Rot90YPos,
    Rot90ZNeg,
    Rot90ZPos,
    Rot180FaceXy,
    Rot180FaceXz,
    Rot180FaceYz,
    /// `OctahedralGroup.INVERT_Y`: the mirror that turns a shape upside down.
    InvertY,
    /// `OctahedralGroup.INVERT_X`.
    InvertX,
}

impl ShapeRotation {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "IDENTITY" => Some(Self::Identity),
            "ROT_90_X_NEG" => Some(Self::Rot90XNeg),
            "ROT_90_X_POS" => Some(Self::Rot90XPos),
            "ROT_90_Y_NEG" => Some(Self::Rot90YNeg),
            "ROT_90_Y_POS" => Some(Self::Rot90YPos),
            "ROT_90_Z_NEG" => Some(Self::Rot90ZNeg),
            "ROT_90_Z_POS" => Some(Self::Rot90ZPos),
            "ROT_180_FACE_XY" => Some(Self::Rot180FaceXy),
            "ROT_180_FACE_XZ" => Some(Self::Rot180FaceXz),
            "ROT_180_FACE_YZ" => Some(Self::Rot180FaceYz),
            "INVERT_Y" => Some(Self::InvertY),
            "INVERT_X" => Some(Self::InvertX),
            _ => None,
        }
    }

    fn axes(self) -> ([usize; 3], [bool; 3]) {
        match self {
            Self::Identity => ([0, 1, 2], [false, false, false]),
            Self::Rot90XNeg => ([0, 2, 1], [false, false, true]),
            Self::Rot90XPos => ([0, 2, 1], [false, true, false]),
            Self::Rot90YNeg => ([2, 1, 0], [true, false, false]),
            Self::Rot90YPos => ([2, 1, 0], [false, false, true]),
            Self::Rot90ZNeg => ([1, 0, 2], [false, true, false]),
            Self::Rot90ZPos => ([1, 0, 2], [true, false, false]),
            Self::Rot180FaceXy => ([0, 1, 2], [true, true, false]),
            Self::Rot180FaceXz => ([0, 1, 2], [true, false, true]),
            Self::Rot180FaceYz => ([0, 1, 2], [false, true, true]),
            Self::InvertY => ([0, 1, 2], [false, true, false]),
            Self::InvertX => ([0, 1, 2], [true, false, false]),
        }
    }

    fn transform(self) -> ShapeTransform {
        let (permutation, inverted) = self.axes();
        ShapeTransform {
            permutation,
            inverted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShapeTransform {
    permutation: [usize; 3],
    inverted: [bool; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShapeAxisCycle {
    None,
    Forward,
    Backward,
}

impl ShapeAxisCycle {
    fn inverse(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }

    fn cycle_axis(self, axis: Axis) -> Axis {
        match (self, axis) {
            (Self::None, axis) => axis,
            (Self::Forward, Axis::X) => Axis::Y,
            (Self::Forward, Axis::Y) => Axis::Z,
            (Self::Forward, Axis::Z) => Axis::X,
            (Self::Backward, Axis::X) => Axis::Z,
            (Self::Backward, Axis::Y) => Axis::X,
            (Self::Backward, Axis::Z) => Axis::Y,
        }
    }

    fn cycle_index(self, point: [isize; 3], axis: Axis) -> isize {
        match (self, axis) {
            (Self::None, Axis::X) => point[0],
            (Self::None, Axis::Y) => point[1],
            (Self::None, Axis::Z) => point[2],
            (Self::Forward, Axis::X) => point[2],
            (Self::Forward, Axis::Y) => point[0],
            (Self::Forward, Axis::Z) => point[1],
            (Self::Backward, Axis::X) => point[1],
            (Self::Backward, Axis::Y) => point[2],
            (Self::Backward, Axis::Z) => point[0],
        }
    }
}

impl ShapeTransform {
    fn is_identity(self) -> bool {
        self.permutation == [0, 1, 2] && self.inverted == [false, false, false]
    }

    /// Compose in the same order as Java OctahedralGroup.compose.
    fn compose(self, that: Self) -> Self {
        Self {
            permutation: [
                that.permutation[self.permutation[0]],
                that.permutation[self.permutation[1]],
                that.permutation[self.permutation[2]],
            ],
            inverted: [
                self.inverted[0] ^ that.inverted[self.permutation[0]],
                self.inverted[1] ^ that.inverted[self.permutation[1]],
                self.inverted[2] ^ that.inverted[self.permutation[2]],
            ],
        }
    }
}

/// The surface to which an attachable block shape is oriented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachFace {
    Floor,
    Wall,
    Ceiling,
}

impl AttachFace {
    pub fn name(self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Wall => "wall",
            Self::Ceiling => "ceiling",
        }
    }
}

/// A voxelized region described by coordinate planes and occupied cells.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelShape {
    xs: Vec<f64>,
    ys: Vec<f64>,
    zs: Vec<f64>,
    cells: Vec<bool>,
    identity: Arc<()>,
}

static EMPTY_SHAPE: OnceLock<VoxelShape> = OnceLock::new();
static BLOCK_SHAPE: OnceLock<VoxelShape> = OnceLock::new();
static INFINITY_SHAPE: OnceLock<VoxelShape> = OnceLock::new();

fn point_distance_squared(first: Vec3, second: Vec3) -> f64 {
    let dx = first.x - second.x;
    let dy = first.y - second.y;
    let dz = first.z - second.z;
    dx * dx + dy * dy + dz * dz
}

fn fuzzy_equals(first: f64, second: f64) -> bool {
    (first - second).abs() <= EPSILON
}

fn axis_component(point: Vec3, axis: usize) -> f64 {
    match axis {
        0 => point.x,
        1 => point.y,
        _ => point.z,
    }
}

fn rotate_axis_coords(
    coords: &[f64],
    invert: bool,
    source_center: f64,
    output_center: f64,
) -> Vec<f64> {
    if !invert && source_center == output_center {
        return coords.to_vec();
    }
    if invert {
        coords
            .iter()
            .rev()
            .map(|value| -(*value - source_center) + output_center)
            .collect()
    } else {
        coords
            .iter()
            .map(|value| *value - source_center + output_center)
            .collect()
    }
}

/// Constructors and operations for voxel shapes.
pub struct Shapes;

#[derive(Clone, Copy)]
struct Grid {
    x_size: usize,
    y_size: usize,
    z_size: usize,
}

impl Grid {
    fn from_coords(xs: &[f64], ys: &[f64], zs: &[f64]) -> Self {
        Self {
            x_size: xs.len().saturating_sub(1),
            y_size: ys.len().saturating_sub(1),
            z_size: zs.len().saturating_sub(1),
        }
    }

    fn cell_count(self) -> usize {
        self.x_size
            .saturating_mul(self.y_size)
            .saturating_mul(self.z_size)
    }
}

fn cube_box(min: [f64; 3], max: [f64; 3], bits: [u32; 3]) -> VoxelShape {
    let [min_x, min_y, min_z] = min;
    let [max_x, max_y, max_z] = max;
    let [x_bits, y_bits, z_bits] = bits;
    let x_size = 1usize << x_bits;
    let y_size = 1usize << y_bits;
    let z_size = 1usize << z_bits;
    let x_min = java_round(min_x * x_size as f64) as usize;
    let y_min = java_round(min_y * y_size as f64) as usize;
    let z_min = java_round(min_z * z_size as f64) as usize;
    let x_max = java_round(max_x * x_size as f64) as usize;
    let y_max = java_round(max_y * y_size as f64) as usize;
    let z_max = java_round(max_z * z_size as f64) as usize;
    let xs = cube_coords(x_size);
    let ys = cube_coords(y_size);
    let zs = cube_coords(z_size);
    let grid = Grid {
        x_size,
        y_size,
        z_size,
    };
    let mut cells = vec![false; grid.cell_count()];

    for x in x_min..x_max {
        for y in y_min..y_max {
            for z in z_min..z_max {
                cells[index(x, y, z, grid)] = true;
            }
        }
    }

    VoxelShape::new(xs, ys, zs, cells)
}

fn cube_coords(size: usize) -> Vec<f64> {
    (0..=size).map(|index| index as f64 / size as f64).collect()
}

fn find_bits(min: f64, max: f64) -> Option<u32> {
    if min < -EPSILON || max > 1.0000001 {
        return None;
    }

    (0..=3).find(|bits| {
        let intervals = 1u32 << bits;
        let scale = intervals as f64;
        let scaled_min = min * scale;
        let scaled_max = max * scale;
        (scaled_min - java_round(scaled_min) as f64).abs() < EPSILON * scale
            && (scaled_max - java_round(scaled_max) as f64).abs() < EPSILON * scale
    })
}

fn java_round(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}

fn index(x: usize, y: usize, z: usize, grid: Grid) -> usize {
    (x * grid.y_size + y) * grid.z_size + z
}

fn midpoint(start: f64, end: f64) -> f64 {
    if start == f64::NEG_INFINITY {
        return if end == f64::INFINITY {
            0.0
        } else {
            end - end.abs().max(1.0)
        };
    }
    if end == f64::INFINITY {
        return start + start.abs().max(1.0);
    }
    start + (end - start) * 0.5
}

fn collision_axes(axis: Axis) -> (Axis, Axis, Axis) {
    match axis {
        Axis::X => (Axis::X, Axis::Y, Axis::Z),
        Axis::Y => (Axis::Y, Axis::Z, Axis::X),
        Axis::Z => (Axis::Z, Axis::X, Axis::Y),
    }
}

fn column_axes(axis: Axis) -> (Axis, Axis) {
    match axis {
        Axis::X => (Axis::Y, Axis::Z),
        Axis::Y => (Axis::Z, Axis::X),
        Axis::Z => (Axis::X, Axis::Y),
    }
}

fn aabb_axis(value: Aabb, axis: Axis, min: bool) -> f64 {
    match (axis, min) {
        (Axis::X, true) => value.min_x,
        (Axis::X, false) => value.max_x,
        (Axis::Y, true) => value.min_y,
        (Axis::Y, false) => value.max_y,
        (Axis::Z, true) => value.min_z,
        (Axis::Z, false) => value.max_z,
    }
}

fn interval(coords: &[f64], value: f64) -> Option<usize> {
    coords
        .windows(2)
        .position(|pair| pair[0] <= value && value < pair[1])
}

fn merged_coords(first: &[f64], second: &[f64]) -> Vec<f64> {
    let mut coords = Vec::with_capacity(first.len() + second.len());
    coords.extend_from_slice(first);
    coords.extend_from_slice(second);
    coords.sort_by(f64::total_cmp);
    coords.dedup_by(|left, right| *left == *right);
    coords
}

fn clear_z_strip(cells: &mut [bool], start_z: usize, end_z: usize, x: usize, y: usize, grid: Grid) {
    for z in start_z..end_z {
        cells[index(x, y, z, grid)] = false;
    }
}

fn is_z_strip_full(
    cells: &[bool],
    start_z: usize,
    end_z: usize,
    x: usize,
    y: usize,
    grid: Grid,
) -> bool {
    x < grid.x_size && y < grid.y_size && (start_z..end_z).all(|z| cells[index(x, y, z, grid)])
}

fn is_xz_rectangle_full(
    cells: &[bool],
    start_x: usize,
    end_x: usize,
    start_z: usize,
    end_z: usize,
    y: usize,
    grid: Grid,
) -> bool {
    y < grid.y_size && (start_x..end_x).all(|x| is_z_strip_full(cells, start_z, end_z, x, y, grid))
}

mod voxel_shape_full;
mod voxel_shape_move_ops;

mod shapes_rotate;
mod shapes_rotate_2;
