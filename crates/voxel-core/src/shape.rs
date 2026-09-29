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

impl VoxelShape {
    fn new(xs: Vec<f64>, ys: Vec<f64>, zs: Vec<f64>, cells: Vec<bool>) -> Self {
        debug_assert_eq!(cells.len(), Grid::from_coords(&xs, &ys, &zs).cell_count());
        Self {
            xs,
            ys,
            zs,
            cells,
            identity: Arc::new(()),
        }
    }

    /// Compare object identity like Java `VoxelShape.equals(Object)`.
    pub fn equals_java(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }

    /// Format the shape like Java `VoxelShape.toString()`.
    pub fn to_string_java(&self) -> String {
        if self.is_empty() {
            return "EMPTY".to_owned();
        }
        let bounds = self.bounds().expect("nonempty shape has bounds");
        format!("VoxelShape[{}]", bounds.to_string_java())
    }

    /// Return whether no voxel cell is occupied.
    pub fn is_empty(&self) -> bool {
        !self.cells.iter().any(|cell| *cell)
    }

    /// Move every coordinate plane by the supplied delta.
    pub fn move_by(&self, dx: f64, dy: f64, dz: f64) -> Self {
        if self.is_empty() {
            return Shapes::empty();
        }
        Self::new(
            self.xs.iter().map(|value| value + dx).collect(),
            self.ys.iter().map(|value| value + dy).collect(),
            self.zs.iter().map(|value| value + dz).collect(),
            self.cells.clone(),
        )
    }

    /// Move by a double-precision vector.
    pub fn move_by_vec3(&self, delta: Vec3) -> Self {
        self.move_by(delta.x, delta.y, delta.z)
    }

    /// Move by integer block coordinates converted to double precision.
    pub fn move_by_block_pos(&self, delta: BlockPos) -> Self {
        self.move_by(f64::from(delta.x), f64::from(delta.y), f64::from(delta.z))
    }

    /// Return the coordinate planes for an axis in ascending order.
    pub fn coords(&self, axis: Axis) -> &[f64] {
        self.axis_coords(axis)
    }

    /// Rebuild the shape from its merged boxes using Java's optimize order.
    pub fn optimize(&self) -> Self {
        let mut optimized = Shapes::empty();
        for bounds in self.to_aabbs() {
            let box_shape = Shapes::box_shape(
                bounds.min_x,
                bounds.min_y,
                bounds.min_z,
                bounds.max_x,
                bounds.max_y,
                bounds.max_z,
            )
            .expect("merged shape boxes have ordered bounds");
            optimized = if optimized.is_empty() {
                box_shape
            } else {
                Shapes::join_unoptimized(&optimized, &box_shape, BooleanOp::Or)
            };
        }
        optimized
    }

    /// Visit merged boundary edges in Java's axis-cycle order.
    pub fn for_all_edges<F>(&self, mut consumer: F)
    where
        F: FnMut(f64, f64, f64, f64, f64, f64),
    {
        for transform in [
            ShapeAxisCycle::None,
            ShapeAxisCycle::Forward,
            ShapeAxisCycle::Backward,
        ] {
            self.for_all_axis_edges(&mut consumer, transform);
        }
    }

    fn for_all_axis_edges<F>(&self, consumer: &mut F, transform: ShapeAxisCycle)
    where
        F: FnMut(f64, f64, f64, f64, f64, f64),
    {
        let inverse = transform.inverse();
        let a_size = self.axis_size(inverse.cycle_axis(Axis::X));
        let b_size = self.axis_size(inverse.cycle_axis(Axis::Y));
        let c_size = self.axis_size(inverse.cycle_axis(Axis::Z));

        for a in 0..=a_size {
            for b in 0..=b_size {
                let mut last_start = None;

                for c in 0..=c_size {
                    let mut full_sectors = 0;
                    let mut odd_sectors = 0;
                    for da in 0..=1isize {
                        for db in 0..=1isize {
                            if self.is_full_wide_cycle(
                                inverse,
                                [a as isize + da - 1, b as isize + db - 1, c as isize],
                            ) {
                                full_sectors += 1;
                                odd_sectors ^= da ^ db;
                            }
                        }
                    }

                    let is_edge = full_sectors == 1
                        || full_sectors == 3
                        || full_sectors == 2 && odd_sectors & 1 == 0;
                    if is_edge {
                        if last_start.is_none() {
                            last_start = Some(c);
                        }
                    } else if let Some(start) = last_start.take() {
                        self.consume_edge(consumer, inverse, a, b, start, c);
                    }
                }
            }
        }
    }

    fn is_full_wide_cycle(&self, cycle: ShapeAxisCycle, point: [isize; 3]) -> bool {
        self.is_full_wide_xyz(
            cycle.cycle_index(point, Axis::X),
            cycle.cycle_index(point, Axis::Y),
            cycle.cycle_index(point, Axis::Z),
        )
    }

    fn consume_edge<F>(
        &self,
        consumer: &mut F,
        cycle: ShapeAxisCycle,
        a: usize,
        b: usize,
        start: usize,
        end: usize,
    ) where
        F: FnMut(f64, f64, f64, f64, f64, f64),
    {
        let first = [a as isize, b as isize, start as isize];
        let second = [a as isize, b as isize, end as isize];
        let first = [
            cycle.cycle_index(first, Axis::X),
            cycle.cycle_index(first, Axis::Y),
            cycle.cycle_index(first, Axis::Z),
        ];
        let second = [
            cycle.cycle_index(second, Axis::X),
            cycle.cycle_index(second, Axis::Y),
            cycle.cycle_index(second, Axis::Z),
        ];
        consumer(
            self.axis_coord(Axis::X, first[0] as usize),
            self.axis_coord(Axis::Y, first[1] as usize),
            self.axis_coord(Axis::Z, first[2] as usize),
            self.axis_coord(Axis::X, second[0] as usize),
            self.axis_coord(Axis::Y, second[1] as usize),
            self.axis_coord(Axis::Z, second[2] as usize),
        );
    }

    /// Return the smallest AABB containing every occupied cell.
    pub fn bounds(&self) -> Option<Aabb> {
        let mut min_x = usize::MAX;
        let mut min_y = usize::MAX;
        let mut min_z = usize::MAX;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut max_z = 0;

        let grid = self.grid();
        for x in 0..grid.x_size {
            for y in 0..grid.y_size {
                for z in 0..grid.z_size {
                    if self.is_full(x, y, z) {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        min_z = min_z.min(z);
                        max_x = max_x.max(x + 1);
                        max_y = max_y.max(y + 1);
                        max_z = max_z.max(z + 1);
                    }
                }
            }
        }

        (min_x != usize::MAX).then(|| {
            Aabb::new(
                self.xs[min_x],
                self.ys[min_y],
                self.zs[min_z],
                self.xs[max_x],
                self.ys[max_y],
                self.zs[max_z],
            )
        })
    }

    /// Return the first occupied coordinate plane for an axis.
    pub fn min(&self, axis: Axis) -> f64 {
        self.bounds()
            .map(|bounds| match axis {
                Axis::X => bounds.min_x,
                Axis::Y => bounds.min_y,
                Axis::Z => bounds.min_z,
            })
            .unwrap_or(f64::INFINITY)
    }

    /// Return the last occupied coordinate plane for an axis.
    pub fn max(&self, axis: Axis) -> f64 {
        self.bounds()
            .map(|bounds| match axis {
                Axis::X => bounds.max_x,
                Axis::Y => bounds.max_y,
                Axis::Z => bounds.max_z,
            })
            .unwrap_or(f64::NEG_INFINITY)
    }

    /// Return the first occupied coordinate plane in the requested column.
    pub fn min_at(&self, axis: Axis, b: f64, c: f64) -> f64 {
        let (b_axis, c_axis) = column_axes(axis);
        let b_index = self.find_index(b_axis, b);
        let c_index = self.find_index(c_axis, c);
        for a in 0..self.axis_size(axis) {
            if self.is_full_wide(axis, a as isize, b_index, c_index) {
                return self.axis_coord(axis, a);
            }
        }
        f64::INFINITY
    }

    /// Return the last occupied coordinate plane in the requested column.
    pub fn max_at(&self, axis: Axis, b: f64, c: f64) -> f64 {
        let (b_axis, c_axis) = column_axes(axis);
        let b_index = self.find_index(b_axis, b);
        let c_index = self.find_index(c_axis, c);
        for a in (0..self.axis_size(axis)).rev() {
            if self.is_full_wide(axis, a as isize, b_index, c_index) {
                return self.axis_coord(axis, a + 1);
            }
        }
        f64::NEG_INFINITY
    }

    /// Collapse a shape to one box spanning its occupied coordinate bounds.
    pub fn single_encompassing(&self) -> Self {
        let Some(bounds) = self.bounds() else {
            return Shapes::empty();
        };
        Shapes::box_shape(
            bounds.min_x,
            bounds.min_y,
            bounds.min_z,
            bounds.max_x,
            bounds.max_y,
            bounds.max_z,
        )
        .expect("occupied shape bounds are ordered")
    }

    /// Limit an axis-aligned moving box's distance against this shape.
    pub fn collide(&self, axis: Axis, moving: Aabb, distance: f64) -> f64 {
        if self.is_empty() {
            return distance;
        }
        if distance.abs() < EPSILON {
            return 0.0;
        }

        let (a_axis, b_axis, c_axis) = collision_axes(axis);
        let max_a = aabb_axis(moving, a_axis, false);
        let min_a = aabb_axis(moving, a_axis, true);
        let a_min = self.find_index(a_axis, min_a + EPSILON);
        let a_max = self.find_index(a_axis, max_a - EPSILON);
        let b_min = self
            .find_index(b_axis, aabb_axis(moving, b_axis, true) + EPSILON)
            .max(0) as usize;
        let b_max = (self.find_index(b_axis, aabb_axis(moving, b_axis, false) - EPSILON) + 1)
            .min(self.axis_size(b_axis) as isize)
            .max(0) as usize;
        let c_min = self
            .find_index(c_axis, aabb_axis(moving, c_axis, true) + EPSILON)
            .max(0) as usize;
        let c_max = (self.find_index(c_axis, aabb_axis(moving, c_axis, false) - EPSILON) + 1)
            .min(self.axis_size(c_axis) as isize)
            .max(0) as usize;
        let a_size = self.axis_size(a_axis);

        if distance > 0.0 {
            let start = (a_max + 1).max(0) as usize;
            for a in start..a_size {
                for b in b_min..b_max {
                    for c in c_min..c_max {
                        if self.is_full_wide(axis, a as isize, b as isize, c as isize) {
                            let new_distance = self.axis_coord(a_axis, a) - max_a;
                            if new_distance >= -EPSILON {
                                return distance.min(new_distance);
                            }
                            return distance;
                        }
                    }
                }
            }
        } else if distance < 0.0 {
            let mut a = a_min - 1;
            while a >= 0 {
                for b in b_min..b_max {
                    for c in c_min..c_max {
                        if self.is_full_wide(axis, a, b as isize, c as isize) {
                            let new_distance = self.axis_coord(a_axis, (a + 1) as usize) - min_a;
                            if new_distance <= EPSILON {
                                return distance.max(new_distance);
                            }
                            return distance;
                        }
                    }
                }
                a -= 1;
            }
        }

        distance
    }

    /// Visit occupied cells as Java's merged boxes in iteration order.
    pub fn for_all_boxes<F>(&self, mut consumer: F)
    where
        F: FnMut(f64, f64, f64, f64, f64, f64),
    {
        let grid = self.grid();
        let mut remaining = self.cells.clone();

        for y in 0..grid.y_size {
            for x in 0..grid.x_size {
                let mut start_z = None;

                for z in 0..=grid.z_size {
                    let full = z < grid.z_size && remaining[index(x, y, z, grid)];
                    if full {
                        if start_z.is_none() {
                            start_z = Some(z);
                        }
                    } else if let Some(start_z) = start_z.take() {
                        let mut end_x = x;
                        let mut end_y = y;
                        clear_z_strip(&mut remaining, start_z, z, x, y, grid);

                        while is_z_strip_full(&remaining, start_z, z, end_x + 1, y, grid) {
                            end_x += 1;
                            clear_z_strip(&mut remaining, start_z, z, end_x, y, grid);
                        }

                        while is_xz_rectangle_full(
                            &remaining,
                            x,
                            end_x + 1,
                            start_z,
                            z,
                            end_y + 1,
                            grid,
                        ) {
                            for cx in x..=end_x {
                                clear_z_strip(&mut remaining, start_z, z, cx, end_y + 1, grid);
                            }
                            end_y += 1;
                        }

                        consumer(
                            self.xs[x],
                            self.ys[y],
                            self.zs[start_z],
                            self.xs[end_x + 1],
                            self.ys[end_y + 1],
                            self.zs[z],
                        );
                    }
                }
            }
        }
    }

    /// Convert occupied cells to Java's merged box iteration order.
    pub fn to_aabbs(&self) -> Vec<Aabb> {
        let mut boxes = Vec::new();
        self.for_all_boxes(|min_x, min_y, min_z, max_x, max_y, max_z| {
            boxes.push(Aabb::new(min_x, min_y, min_z, max_x, max_y, max_z));
        });
        boxes
    }

    /// Clip a segment against this shape and return the nearest block hit.
    pub fn clip(&self, from: Vec3, to: Vec3, pos: BlockPos) -> Option<BlockHitResult> {
        if self.is_empty() {
            return None;
        }

        let diff = Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z);
        if diff.length_squared() < EPSILON {
            return None;
        }

        let test_point = Vec3::new(
            from.x + diff.x * 0.001,
            from.y + diff.y * 0.001,
            from.z + diff.z * 0.001,
        );
        let x = self.find_index(Axis::X, test_point.x - f64::from(pos.x));
        let y = self.find_index(Axis::Y, test_point.y - f64::from(pos.y));
        let z = self.find_index(Axis::Z, test_point.z - f64::from(pos.z));
        if self.is_full_wide_xyz(x, y, z) {
            Some(BlockHitResult::new(
                test_point,
                Direction::approximate_nearest(diff.x, diff.y, diff.z).opposite(),
                pos,
                true,
            ))
        } else {
            let boxes = self.to_aabbs();
            Aabb::clip_many(&boxes, from, to, pos)
        }
    }

    /// Return the closest point in the shape, preserving box iteration ties.
    pub fn closest_point_to(&self, point: Vec3) -> Option<Vec3> {
        if self.is_empty() {
            return None;
        }

        let mut closest = None;
        for aabb in self.to_aabbs() {
            let candidate = Vec3::new(
                clamp_f64(point.x, aabb.min_x, aabb.max_x),
                clamp_f64(point.y, aabb.min_y, aabb.max_y),
                clamp_f64(point.z, aabb.min_z, aabb.max_z),
            );
            if closest.is_none_or(|current| {
                point_distance_squared(point, candidate) < point_distance_squared(point, current)
            }) {
                closest = Some(candidate);
            }
        }
        closest
    }

    /// Return the selected boundary face using Java's normalized slice axis.
    pub fn face_shape(&self, direction: Direction) -> Self {
        if self.is_empty() || self.is_cube_like_along(direction.axis()) {
            return self.clone();
        }

        let axis = direction.axis();
        let sample = match direction.axis_direction() {
            AxisDirection::Positive => 0.9999999,
            AxisDirection::Negative => 1.0e-7,
        };
        let slice = self.slice_at(axis, self.find_index(axis, sample));
        if slice.is_empty() {
            Shapes::empty()
        } else if slice.is_cube_like() {
            Shapes::block()
        } else {
            slice
        }
    }

    fn slice_at(&self, axis: Axis, slice: isize) -> Self {
        let axis_size = self.axis_size(axis) as isize;
        if slice < 0 || slice >= axis_size {
            return Shapes::empty();
        }

        let parent_grid = self.grid();
        let face_grid = Grid {
            x_size: if axis == Axis::X {
                1
            } else {
                parent_grid.x_size
            },
            y_size: if axis == Axis::Y {
                1
            } else {
                parent_grid.y_size
            },
            z_size: if axis == Axis::Z {
                1
            } else {
                parent_grid.z_size
            },
        };
        let mut cells = vec![false; face_grid.cell_count()];
        for x in 0..face_grid.x_size {
            for y in 0..face_grid.y_size {
                for z in 0..face_grid.z_size {
                    let parent_x = if axis == Axis::X { slice as usize } else { x };
                    let parent_y = if axis == Axis::Y { slice as usize } else { y };
                    let parent_z = if axis == Axis::Z { slice as usize } else { z };
                    cells[index(x, y, z, face_grid)] = self.is_full(parent_x, parent_y, parent_z);
                }
            }
        }

        let (xs, ys, zs) = match axis {
            Axis::X => (vec![0.0, 1.0], self.ys.clone(), self.zs.clone()),
            Axis::Y => (self.xs.clone(), vec![0.0, 1.0], self.zs.clone()),
            Axis::Z => (self.xs.clone(), self.ys.clone(), vec![0.0, 1.0]),
        };
        Self::new(xs, ys, zs, cells)
    }

    fn is_cube_like(&self) -> bool {
        self.is_cube_like_along(Axis::X)
            && self.is_cube_like_along(Axis::Y)
            && self.is_cube_like_along(Axis::Z)
    }

    fn is_block(&self) -> bool {
        self.cells.len() == 1 && self.cells[0] && self.is_cube_like()
    }

    fn is_cube_like_along(&self, axis: Axis) -> bool {
        let coords = self.axis_coords(axis);
        coords.len() == 2 && fuzzy_equals(coords[0], 0.0) && fuzzy_equals(coords[1], 1.0)
    }

    fn min_coordinate(&self, axis: Axis) -> f64 {
        self.bounds()
            .map(|bounds| match axis {
                Axis::X => bounds.min_x,
                Axis::Y => bounds.min_y,
                Axis::Z => bounds.min_z,
            })
            .unwrap_or(f64::INFINITY)
    }

    fn max_coordinate(&self, axis: Axis) -> f64 {
        self.bounds()
            .map(|bounds| match axis {
                Axis::X => bounds.max_x,
                Axis::Y => bounds.max_y,
                Axis::Z => bounds.max_z,
            })
            .unwrap_or(f64::NEG_INFINITY)
    }

    fn is_full(&self, x: usize, y: usize, z: usize) -> bool {
        self.cells[index(x, y, z, self.grid())]
    }

    fn full_at(&self, x: f64, y: f64, z: f64) -> bool {
        let x_index = interval(&self.xs, x);
        let y_index = interval(&self.ys, y);
        let z_index = interval(&self.zs, z);
        match (x_index, y_index, z_index) {
            (Some(x), Some(y), Some(z)) => self.is_full(x, y, z),
            _ => false,
        }
    }

    fn grid(&self) -> Grid {
        Grid::from_coords(&self.xs, &self.ys, &self.zs)
    }

    fn axis_size(&self, axis: Axis) -> usize {
        self.axis_coords(axis).len().saturating_sub(1)
    }

    fn axis_coord(&self, axis: Axis, index: usize) -> f64 {
        self.axis_coords(axis)[index]
    }

    fn axis_coords(&self, axis: Axis) -> &[f64] {
        match axis {
            Axis::X => &self.xs,
            Axis::Y => &self.ys,
            Axis::Z => &self.zs,
        }
    }

    fn find_index(&self, axis: Axis, value: f64) -> isize {
        let coords = self.axis_coords(axis);
        let mut low = 0;
        let mut high = coords.len();
        while low < high {
            let middle = low + (high - low) / 2;
            if value < coords[middle] {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        low as isize - 1
    }

    fn is_full_wide(&self, axis: Axis, a: isize, b: isize, c: isize) -> bool {
        let (x, y, z) = match axis {
            Axis::X => (a, b, c),
            Axis::Y => (c, a, b),
            Axis::Z => (b, c, a),
        };
        self.is_full_wide_xyz(x, y, z)
    }

    fn is_full_wide_xyz(&self, x: isize, y: isize, z: isize) -> bool {
        if x < 0 || y < 0 || z < 0 {
            return false;
        }
        let grid = self.grid();
        let (x, y, z) = (x as usize, y as usize, z as usize);
        x < grid.x_size && y < grid.y_size && z < grid.z_size && self.is_full(x, y, z)
    }
}

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

impl Shapes {
    /// Minimum shape extent used by Java's shape constructors.
    pub const EPSILON: f64 = 1.0e-7;

    /// Larger fuzzy comparison extent exposed by Java's shape helpers.
    pub const BIG_EPSILON: f64 = 1.0e-6;

    /// Return an empty shape.
    pub fn empty() -> VoxelShape {
        EMPTY_SHAPE
            .get_or_init(|| VoxelShape::new(vec![0.0], vec![0.0], vec![0.0], Vec::new()))
            .clone()
    }

    /// Return the unit block shape.
    pub fn block() -> VoxelShape {
        BLOCK_SHAPE
            .get_or_init(|| cube_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0, 0, 0]))
            .clone()
    }

    /// Return the shape spanning negative to positive infinity on every axis.
    pub fn infinity() -> VoxelShape {
        INFINITY_SHAPE
            .get_or_init(|| {
                Self::box_shape(
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::INFINITY,
                )
                .expect("infinity bounds are ordered")
            })
            .clone()
    }

    /// Construct a shape from raw coordinates like Java `Shapes.create(double...)`.
    pub fn create_from_coords(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> VoxelShape {
        if max_x - min_x < EPSILON || max_y - min_y < EPSILON || max_z - min_z < EPSILON {
            return Self::empty();
        }

        match (
            find_bits(min_x, max_x),
            find_bits(min_y, max_y),
            find_bits(min_z, max_z),
        ) {
            (Some(0), Some(0), Some(0)) => Self::block(),
            (Some(x_bits), Some(y_bits), Some(z_bits)) => cube_box(
                [min_x, min_y, min_z],
                [max_x, max_y, max_z],
                [x_bits, y_bits, z_bits],
            ),
            _ => VoxelShape::new(
                vec![min_x, max_x],
                vec![min_y, max_y],
                vec![min_z, max_z],
                vec![true],
            ),
        }
    }

    /// Construct a box, applying Java's minimum extent threshold.
    pub fn box_shape(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> Result<VoxelShape, ShapeError> {
        if min_x > max_x || min_y > max_y || min_z > max_z {
            return Err(ShapeError::ReversedBounds);
        }
        Ok(Self::create_from_coords(
            min_x, min_y, min_z, max_x, max_y, max_z,
        ))
    }

    /// Construct a shape from normalized AABB endpoints like Java `Shapes.create(AABB)`.
    pub fn create_from_aabb(aabb: Aabb) -> Result<VoxelShape, ShapeError> {
        Self::box_shape(
            aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
        )
    }

    /// Join two shapes by filling every cell occupied by either input.
    pub fn or(first: &VoxelShape, second: &VoxelShape) -> VoxelShape {
        Self::join(first, second, BooleanOp::Or)
    }

    /// Join a first shape with Java's left-to-right varargs OR tail.
    pub fn or_many(first: &VoxelShape, tail: &[VoxelShape]) -> VoxelShape {
        tail.iter()
            .fold(first.clone(), |joined, shape| Self::or(&joined, shape))
    }

    /// Join two shapes and normalize the result through Java's optimize path.
    pub fn join(first: &VoxelShape, second: &VoxelShape, operation: BooleanOp) -> VoxelShape {
        Self::join_unoptimized(first, second, operation).optimize()
    }

    /// Join two shapes while preserving the merged coordinate grid.
    pub fn join_unoptimized(
        first: &VoxelShape,
        second: &VoxelShape,
        operation: BooleanOp,
    ) -> VoxelShape {
        if first.equals_java(second) {
            return if operation.apply(true, true) {
                first.clone()
            } else {
                Self::empty()
            };
        }

        let first_only_matters = operation.apply(true, false);
        let second_only_matters = operation.apply(false, true);
        if first.is_empty() {
            return if second_only_matters {
                second.clone()
            } else {
                Self::empty()
            };
        }
        if second.is_empty() {
            return if first_only_matters {
                first.clone()
            } else {
                Self::empty()
            };
        }

        let xs = merged_coords(&first.xs, &second.xs);
        let ys = merged_coords(&first.ys, &second.ys);
        let zs = merged_coords(&first.zs, &second.zs);
        let grid = Grid::from_coords(&xs, &ys, &zs);
        let mut cells = vec![false; grid.cell_count()];

        for x in 0..grid.x_size {
            for y in 0..grid.y_size {
                for z in 0..grid.z_size {
                    let center_x = midpoint(xs[x], xs[x + 1]);
                    let center_y = midpoint(ys[y], ys[y + 1]);
                    let center_z = midpoint(zs[z], zs[z + 1]);
                    cells[index(x, y, z, grid)] = operation.apply(
                        first.full_at(center_x, center_y, center_z),
                        second.full_at(center_x, center_y, center_z),
                    );
                }
            }
        }

        VoxelShape::new(xs, ys, zs, cells)
    }

    /// Return whether a BooleanOp join contains any occupied cell.
    pub fn join_is_not_empty(
        first: &VoxelShape,
        second: &VoxelShape,
        operation: BooleanOp,
    ) -> bool {
        !Self::join(first, second, operation).is_empty()
    }

    /// Return whether two shapes occupy the same cells.
    pub fn equal(first: &VoxelShape, second: &VoxelShape) -> bool {
        !Self::join_is_not_empty(first, second, BooleanOp::NotSame)
    }

    /// Rotate a shape around the unit block center.
    pub fn rotate(shape: &VoxelShape, rotation: ShapeRotation) -> VoxelShape {
        Self::rotate_with_center(shape, rotation, Vec3::new(0.5, 0.5, 0.5))
    }

    /// Rotate a shape around an explicit point.
    pub fn rotate_with_center(
        shape: &VoxelShape,
        rotation: ShapeRotation,
        rotation_center: Vec3,
    ) -> VoxelShape {
        Self::rotate_transform_with_center(shape, rotation.transform(), rotation_center)
    }

    fn rotate_transform_with_center(
        shape: &VoxelShape,
        rotation: ShapeTransform,
        rotation_center: Vec3,
    ) -> VoxelShape {
        if rotation.is_identity() {
            return shape.clone();
        }

        let old_sizes = [shape.xs.len() - 1, shape.ys.len() - 1, shape.zs.len() - 1];
        let old_coords = [&shape.xs, &shape.ys, &shape.zs];
        let permutation = rotation.permutation;
        let inverted = rotation.inverted;
        let new_sizes = [
            old_sizes[permutation[0]],
            old_sizes[permutation[1]],
            old_sizes[permutation[2]],
        ];
        let shifts = [
            if inverted[0] && new_sizes[0] > 0 {
                new_sizes[0] as isize - 1
            } else {
                0
            },
            if inverted[1] && new_sizes[1] > 0 {
                new_sizes[1] as isize - 1
            } else {
                0
            },
            if inverted[2] && new_sizes[2] > 0 {
                new_sizes[2] as isize - 1
            } else {
                0
            },
        ];
        let grid = Grid {
            x_size: new_sizes[0],
            y_size: new_sizes[1],
            z_size: new_sizes[2],
        };
        let mut cells = vec![false; grid.cell_count()];

        for x in 0..old_sizes[0] {
            for y in 0..old_sizes[1] {
                for z in 0..old_sizes[2] {
                    if !shape.is_full(x, y, z) {
                        continue;
                    }
                    let old_position = [x as isize, y as isize, z as isize];
                    let new_position = [
                        shifts[0]
                            + if inverted[0] {
                                -old_position[permutation[0]]
                            } else {
                                old_position[permutation[0]]
                            },
                        shifts[1]
                            + if inverted[1] {
                                -old_position[permutation[1]]
                            } else {
                                old_position[permutation[1]]
                            },
                        shifts[2]
                            + if inverted[2] {
                                -old_position[permutation[2]]
                            } else {
                                old_position[permutation[2]]
                            },
                    ];
                    cells[index(
                        new_position[0] as usize,
                        new_position[1] as usize,
                        new_position[2] as usize,
                        grid,
                    )] = true;
                }
            }
        }

        let new_coords = [
            rotate_axis_coords(
                old_coords[permutation[0]],
                inverted[0],
                axis_component(rotation_center, permutation[0]),
                axis_component(rotation_center, 0),
            ),
            rotate_axis_coords(
                old_coords[permutation[1]],
                inverted[1],
                axis_component(rotation_center, permutation[1]),
                axis_component(rotation_center, 1),
            ),
            rotate_axis_coords(
                old_coords[permutation[2]],
                inverted[2],
                axis_component(rotation_center, permutation[2]),
                axis_component(rotation_center, 2),
            ),
        ];
        VoxelShape::new(
            new_coords[0].clone(),
            new_coords[1].clone(),
            new_coords[2].clone(),
            cells,
        )
    }

    /// Return the horizontal-axis variants of a shape in Java EnumMap order.
    pub fn rotate_horizontal_axis(z_axis: &VoxelShape) -> [(Axis, VoxelShape); 2] {
        Self::rotate_horizontal_axis_with_center(z_axis, Vec3::new(0.5, 0.5, 0.5))
    }

    /// Return horizontal-axis variants around an explicit point.
    pub fn rotate_horizontal_axis_with_center(
        z_axis: &VoxelShape,
        rotation_center: Vec3,
    ) -> [(Axis, VoxelShape); 2] {
        [
            (
                Axis::X,
                Self::rotate_with_center(z_axis, ShapeRotation::Rot90YNeg, rotation_center),
            ),
            (Axis::Z, z_axis.clone()),
        ]
    }

    /// Return the all-axis variants of a shape in Java EnumMap order.
    pub fn rotate_all_axis(north: &VoxelShape) -> [(Axis, VoxelShape); 3] {
        Self::rotate_all_axis_with_center(north, Vec3::new(0.5, 0.5, 0.5))
    }

    /// Return all-axis variants around an explicit point.
    pub fn rotate_all_axis_with_center(
        north: &VoxelShape,
        rotation_center: Vec3,
    ) -> [(Axis, VoxelShape); 3] {
        [
            (
                Axis::X,
                Self::rotate_with_center(north, ShapeRotation::Rot90YNeg, rotation_center),
            ),
            (
                Axis::Y,
                Self::rotate_with_center(north, ShapeRotation::Rot90XNeg, rotation_center),
            ),
            (Axis::Z, north.clone()),
        ]
    }

    fn rotate_horizontal_with_transform(
        north: &VoxelShape,
        initial: ShapeTransform,
    ) -> [(Direction, VoxelShape); 4] {
        let y_90 = ShapeRotation::Rot90YNeg.transform();
        let y_180 = ShapeRotation::Rot180FaceXz.transform();
        let y_270 = ShapeRotation::Rot90YPos.transform();
        let center = Vec3::new(0.5, 0.5, 0.5);
        [
            (
                Direction::North,
                Self::rotate_transform_with_center(north, initial, center),
            ),
            (
                Direction::South,
                Self::rotate_transform_with_center(north, y_180.compose(initial), center),
            ),
            (
                Direction::West,
                Self::rotate_transform_with_center(north, y_270.compose(initial), center),
            ),
            (
                Direction::East,
                Self::rotate_transform_with_center(north, y_90.compose(initial), center),
            ),
        ]
    }

    /// Return the floor, wall, and ceiling direction variants in Java enum order.
    fn rotate_attach_face_with_transform(
        north: &VoxelShape,
        initial: ShapeTransform,
    ) -> [(AttachFace, [(Direction, VoxelShape); 4]); 3] {
        let x_270 = ShapeRotation::Rot90XPos.transform();
        let x_90 = ShapeRotation::Rot90XNeg.transform();
        let y_180 = ShapeRotation::Rot180FaceXz.transform();
        [
            (
                AttachFace::Floor,
                Self::rotate_horizontal_with_transform(north, x_270.compose(initial)),
            ),
            (
                AttachFace::Wall,
                Self::rotate_horizontal_with_transform(north, initial),
            ),
            (
                AttachFace::Ceiling,
                Self::rotate_horizontal_with_transform(north, y_180.compose(x_90).compose(initial)),
            ),
        ]
    }

    /// Return the floor, wall, and ceiling direction variants in Java enum order.
    pub fn rotate_attach_face(
        north: &VoxelShape,
    ) -> [(AttachFace, [(Direction, VoxelShape); 4]); 3] {
        Self::rotate_attach_face_with_transform(north, ShapeRotation::Identity.transform())
    }

    /// Return attach-face variants for a supported initial rotation.
    pub fn rotate_attach_face_with_initial(
        north: &VoxelShape,
        initial: ShapeRotation,
    ) -> [(AttachFace, [(Direction, VoxelShape); 4]); 3] {
        Self::rotate_attach_face_with_transform(north, initial.transform())
    }

    /// Return the horizontal direction variants of a shape in Java EnumMap order.
    pub fn rotate_horizontal(north: &VoxelShape) -> [(Direction, VoxelShape); 4] {
        [
            (Direction::North, north.clone()),
            (
                Direction::South,
                Self::rotate(north, ShapeRotation::Rot180FaceXz),
            ),
            (
                Direction::West,
                Self::rotate(north, ShapeRotation::Rot90YPos),
            ),
            (
                Direction::East,
                Self::rotate(north, ShapeRotation::Rot90YNeg),
            ),
        ]
    }

    /// Return horizontal direction variants for a supported initial rotation.
    pub fn rotate_horizontal_with_initial(
        north: &VoxelShape,
        initial: ShapeRotation,
    ) -> [(Direction, VoxelShape); 4] {
        Self::rotate_horizontal_with_transform(north, initial.transform())
    }

    /// Return all direction variants of a shape in Java EnumMap order.
    pub fn rotate_all(north: &VoxelShape) -> [(Direction, VoxelShape); 6] {
        Self::rotate_all_with_center(north, Vec3::new(0.5, 0.5, 0.5))
    }

    fn rotate_all_with_transform(
        north: &VoxelShape,
        initial: ShapeTransform,
        rotation_center: Vec3,
    ) -> [(Direction, VoxelShape); 6] {
        let x_90 = ShapeRotation::Rot90XNeg.transform();
        let x_270 = ShapeRotation::Rot90XPos.transform();
        let y_90 = ShapeRotation::Rot90YNeg.transform();
        let y_180 = ShapeRotation::Rot180FaceXz.transform();
        let y_270 = ShapeRotation::Rot90YPos.transform();
        [
            (
                Direction::Down,
                Self::rotate_transform_with_center(north, x_90.compose(initial), rotation_center),
            ),
            (
                Direction::Up,
                Self::rotate_transform_with_center(north, x_270.compose(initial), rotation_center),
            ),
            (
                Direction::North,
                Self::rotate_transform_with_center(north, initial, Vec3::new(0.5, 0.5, 0.5)),
            ),
            (
                Direction::South,
                Self::rotate_transform_with_center(north, y_180.compose(initial), rotation_center),
            ),
            (
                Direction::West,
                Self::rotate_transform_with_center(north, y_270.compose(initial), rotation_center),
            ),
            (
                Direction::East,
                Self::rotate_transform_with_center(north, y_90.compose(initial), rotation_center),
            ),
        ]
    }

    /// Return all direction variants around an explicit point.
    pub fn rotate_all_with_initial(
        north: &VoxelShape,
        initial: ShapeRotation,
        rotation_center: Vec3,
    ) -> [(Direction, VoxelShape); 6] {
        Self::rotate_all_with_transform(north, initial.transform(), rotation_center)
    }

    /// Return all direction variants around an explicit point.
    pub fn rotate_all_with_center(
        north: &VoxelShape,
        rotation_center: Vec3,
    ) -> [(Direction, VoxelShape); 6] {
        Self::rotate_all_with_transform(north, ShapeRotation::Identity.transform(), rotation_center)
    }

    /// Return whether two block-local shapes occlude across a direction.
    pub fn block_occludes(shape: &VoxelShape, occluder: &VoxelShape, direction: Direction) -> bool {
        let axis = direction.axis();
        let (first, second, operation) =
            if matches!(direction.axis_direction(), AxisDirection::Positive) {
                (shape, occluder, BooleanOp::OnlyFirst)
            } else {
                (occluder, shape, BooleanOp::OnlySecond)
            };

        if !fuzzy_equals(first.max_coordinate(axis), 1.0)
            || !fuzzy_equals(second.min_coordinate(axis), 0.0)
        {
            return false;
        }

        let first_slice = first.slice_at(axis, first.axis_size(axis) as isize - 1);
        let second_slice = second.slice_at(axis, 0);
        !Self::join_is_not_empty(&first_slice, &second_slice, operation)
    }

    /// Return whether the merged boundary faces cover a unit block.
    pub fn merged_face_occludes(
        shape: &VoxelShape,
        occluder: &VoxelShape,
        direction: Direction,
    ) -> bool {
        if shape.is_block() || occluder.is_block() {
            return true;
        }

        let axis = direction.axis();
        let (first, second) = if matches!(direction.axis_direction(), AxisDirection::Positive) {
            (shape, occluder)
        } else {
            (occluder, shape)
        };
        let first = if fuzzy_equals(first.max_coordinate(axis), 1.0) {
            first.clone()
        } else {
            Self::empty()
        };
        let second = if fuzzy_equals(second.min_coordinate(axis), 0.0) {
            second.clone()
        } else {
            Self::empty()
        };
        let first_slice = first.slice_at(axis, first.axis_size(axis) as isize - 1);
        let second_slice = second.slice_at(axis, 0);
        let merged = Self::or(&first_slice, &second_slice);
        !Self::join_is_not_empty(&Self::block(), &merged, BooleanOp::OnlyFirst)
    }

    /// Return whether two face shapes cover a unit block together.
    pub fn face_shape_occludes(shape: &VoxelShape, occluder: &VoxelShape) -> bool {
        if shape.is_block() || occluder.is_block() {
            return true;
        }
        if shape.is_empty() && occluder.is_empty() {
            return false;
        }

        let merged = Self::or(shape, occluder);
        !Self::join_is_not_empty(&Self::block(), &merged, BooleanOp::OnlyFirst)
    }

    /// Limit a moving box against each shape in iteration order.
    pub fn collide(axis: Axis, moving: Aabb, shapes: &[VoxelShape], mut distance: f64) -> f64 {
        for shape in shapes {
            if distance.abs() < EPSILON {
                return 0.0;
            }
            distance = shape.collide(axis, moving, distance);
        }
        distance
    }
}

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
