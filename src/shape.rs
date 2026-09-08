//! Voxel shapes for the first P01.d geometry slice.
//!
//! A shape stores coordinate planes and the occupied cells between them. This
//! is enough to preserve Java's box, move, supported BooleanOp joins, single
//! shape collision, bounds, and box iteration behavior without choosing a
//! broader collision representation yet.

use crate::aabb::Aabb;
use crate::direction::Axis;

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

/// A voxelized region described by coordinate planes and occupied cells.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelShape {
    xs: Vec<f64>,
    ys: Vec<f64>,
    zs: Vec<f64>,
    cells: Vec<bool>,
}

impl VoxelShape {
    fn new(xs: Vec<f64>, ys: Vec<f64>, zs: Vec<f64>, cells: Vec<bool>) -> Self {
        debug_assert_eq!(cells.len(), Grid::from_coords(&xs, &ys, &zs).cell_count());
        Self { xs, ys, zs, cells }
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

    /// Convert occupied cells to Java's merged box iteration order.
    pub fn to_aabbs(&self) -> Vec<Aabb> {
        let grid = self.grid();
        let mut remaining = self.cells.clone();
        let mut boxes = Vec::new();

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

                        boxes.push(Aabb::new(
                            self.xs[x],
                            self.ys[y],
                            self.zs[start_z],
                            self.xs[end_x + 1],
                            self.ys[end_y + 1],
                            self.zs[z],
                        ));
                    }
                }
            }
        }

        boxes
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
        if x < 0 || y < 0 || z < 0 {
            return false;
        }
        let grid = self.grid();
        let (x, y, z) = (x as usize, y as usize, z as usize);
        x < grid.x_size && y < grid.y_size && z < grid.z_size && self.is_full(x, y, z)
    }
}

/// Constructors and operations for voxel shapes.
pub struct Shapes;

impl Shapes {
    /// Return an empty shape.
    pub fn empty() -> VoxelShape {
        VoxelShape::new(vec![0.0], vec![0.0], vec![0.0], Vec::new())
    }

    /// Return the unit block shape.
    pub fn block() -> VoxelShape {
        Self::box_shape(0.0, 0.0, 0.0, 1.0, 1.0, 1.0).expect("unit block bounds are ordered")
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
        if max_x - min_x < EPSILON || max_y - min_y < EPSILON || max_z - min_z < EPSILON {
            return Ok(Self::empty());
        }

        match (
            find_bits(min_x, max_x),
            find_bits(min_y, max_y),
            find_bits(min_z, max_z),
        ) {
            (Some(x_bits), Some(y_bits), Some(z_bits)) => Ok(cube_box(
                [min_x, min_y, min_z],
                [max_x, max_y, max_z],
                [x_bits, y_bits, z_bits],
            )),
            _ => Ok(VoxelShape::new(
                vec![min_x, max_x],
                vec![min_y, max_y],
                vec![min_z, max_z],
                vec![true],
            )),
        }
    }

    /// Join two shapes by filling every cell occupied by either input.
    pub fn or(first: &VoxelShape, second: &VoxelShape) -> VoxelShape {
        Self::join(first, second, BooleanOp::Or)
    }

    /// Join two shapes with one of the supported Boolean operations.
    pub fn join(first: &VoxelShape, second: &VoxelShape, operation: BooleanOp) -> VoxelShape {
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
    start + (end - start) * 0.5
}

fn collision_axes(axis: Axis) -> (Axis, Axis, Axis) {
    match axis {
        Axis::X => (Axis::X, Axis::Y, Axis::Z),
        Axis::Y => (Axis::Y, Axis::Z, Axis::X),
        Axis::Z => (Axis::Z, Axis::X, Axis::Y),
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
