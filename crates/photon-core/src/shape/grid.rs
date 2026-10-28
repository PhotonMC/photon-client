use super::*;
use std::ops::Range;

pub(crate) fn point_distance_squared(first: Vec3, second: Vec3) -> f64 {
    let dx = first.x - second.x;
    let dy = first.y - second.y;
    let dz = first.z - second.z;
    dx * dx + dy * dy + dz * dz
}

pub(crate) fn fuzzy_equals(first: f64, second: f64) -> bool {
    (first - second).abs() <= EPSILON
}

pub(crate) fn axis_component(point: Vec3, axis: usize) -> f64 {
    match axis {
        0 => point.x,
        1 => point.y,
        _ => point.z,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Grid {
    pub(crate) x_size: usize,
    pub(crate) y_size: usize,
    pub(crate) z_size: usize,
}

pub(crate) fn cube_box(min: [f64; 3], max: [f64; 3], bits: [u32; 3]) -> VoxelShape {
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

pub(crate) fn cube_coords(size: usize) -> Vec<f64> {
    (0..=size).map(|index| index as f64 / size as f64).collect()
}

pub(crate) fn find_bits(min: f64, max: f64) -> Option<u32> {
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

pub(crate) fn java_round(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}

pub(crate) fn index(x: usize, y: usize, z: usize, grid: Grid) -> usize {
    (x * grid.y_size + y) * grid.z_size + z
}

pub(crate) fn midpoint(start: f64, end: f64) -> f64 {
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

pub(crate) fn collision_axes(axis: Axis) -> (Axis, Axis, Axis) {
    match axis {
        Axis::X => (Axis::X, Axis::Y, Axis::Z),
        Axis::Y => (Axis::Y, Axis::Z, Axis::X),
        Axis::Z => (Axis::Z, Axis::X, Axis::Y),
    }
}

pub(crate) fn column_axes(axis: Axis) -> (Axis, Axis) {
    match axis {
        Axis::X => (Axis::Y, Axis::Z),
        Axis::Y => (Axis::Z, Axis::X),
        Axis::Z => (Axis::X, Axis::Y),
    }
}

pub(crate) fn aabb_axis(value: Aabb, axis: Axis, min: bool) -> f64 {
    match (axis, min) {
        (Axis::X, true) => value.min_x,
        (Axis::X, false) => value.max_x,
        (Axis::Y, true) => value.min_y,
        (Axis::Y, false) => value.max_y,
        (Axis::Z, true) => value.min_z,
        (Axis::Z, false) => value.max_z,
    }
}

pub(crate) fn interval(coords: &[f64], value: f64) -> Option<usize> {
    coords
        .windows(2)
        .position(|pair| pair[0] <= value && value < pair[1])
}

pub(crate) fn merged_coords(first: &[f64], second: &[f64]) -> Vec<f64> {
    let mut coords = Vec::with_capacity(first.len() + second.len());
    coords.extend_from_slice(first);
    coords.extend_from_slice(second);
    coords.sort_by(f64::total_cmp);
    coords.dedup_by(|left, right| *left == *right);
    coords
}

pub(crate) fn clear_z_strip(
    cells: &mut [bool],
    start_z: usize,
    end_z: usize,
    x: usize,
    y: usize,
    grid: Grid,
) {
    for z in start_z..end_z {
        cells[index(x, y, z, grid)] = false;
    }
}

pub(crate) fn is_z_strip_full(
    cells: &[bool],
    start_z: usize,
    end_z: usize,
    x: usize,
    y: usize,
    grid: Grid,
) -> bool {
    x < grid.x_size && y < grid.y_size && (start_z..end_z).all(|z| cells[index(x, y, z, grid)])
}

pub(crate) fn is_xz_rectangle_full(
    cells: &[bool],
    xs: Range<usize>,
    zs: Range<usize>,
    y: usize,
    grid: Grid,
) -> bool {
    y < grid.y_size
        && xs
            .into_iter()
            .all(|x| is_z_strip_full(cells, zs.start, zs.end, x, y, grid))
}

impl Grid {
    pub(crate) fn from_coords(xs: &[f64], ys: &[f64], zs: &[f64]) -> Self {
        Self {
            x_size: xs.len().saturating_sub(1),
            y_size: ys.len().saturating_sub(1),
            z_size: zs.len().saturating_sub(1),
        }
    }

    pub(crate) fn cell_count(self) -> usize {
        self.x_size
            .saturating_mul(self.y_size)
            .saturating_mul(self.z_size)
    }
}
