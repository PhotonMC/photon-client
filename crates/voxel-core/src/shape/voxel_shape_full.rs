use super::*;

impl VoxelShape {
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

    pub(super) fn slice_at(&self, axis: Axis, slice: isize) -> Self {
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

    pub(super) fn is_cube_like(&self) -> bool {
        self.is_cube_like_along(Axis::X)
            && self.is_cube_like_along(Axis::Y)
            && self.is_cube_like_along(Axis::Z)
    }

    pub(super) fn is_block(&self) -> bool {
        self.cells.len() == 1 && self.cells[0] && self.is_cube_like()
    }

    pub(super) fn is_cube_like_along(&self, axis: Axis) -> bool {
        let coords = self.axis_coords(axis);
        coords.len() == 2 && fuzzy_equals(coords[0], 0.0) && fuzzy_equals(coords[1], 1.0)
    }

    pub(super) fn min_coordinate(&self, axis: Axis) -> f64 {
        self.bounds()
            .map(|bounds| match axis {
                Axis::X => bounds.min_x,
                Axis::Y => bounds.min_y,
                Axis::Z => bounds.min_z,
            })
            .unwrap_or(f64::INFINITY)
    }

    pub(super) fn max_coordinate(&self, axis: Axis) -> f64 {
        self.bounds()
            .map(|bounds| match axis {
                Axis::X => bounds.max_x,
                Axis::Y => bounds.max_y,
                Axis::Z => bounds.max_z,
            })
            .unwrap_or(f64::NEG_INFINITY)
    }

    pub(super) fn is_full(&self, x: usize, y: usize, z: usize) -> bool {
        self.cells[index(x, y, z, self.grid())]
    }

    pub(super) fn full_at(&self, x: f64, y: f64, z: f64) -> bool {
        let x_index = interval(&self.xs, x);
        let y_index = interval(&self.ys, y);
        let z_index = interval(&self.zs, z);
        match (x_index, y_index, z_index) {
            (Some(x), Some(y), Some(z)) => self.is_full(x, y, z),
            _ => false,
        }
    }

    pub(super) fn grid(&self) -> Grid {
        Grid::from_coords(&self.xs, &self.ys, &self.zs)
    }

    pub(super) fn axis_size(&self, axis: Axis) -> usize {
        self.axis_coords(axis).len().saturating_sub(1)
    }

    pub(super) fn axis_coord(&self, axis: Axis, index: usize) -> f64 {
        self.axis_coords(axis)[index]
    }

    pub(super) fn axis_coords(&self, axis: Axis) -> &[f64] {
        match axis {
            Axis::X => &self.xs,
            Axis::Y => &self.ys,
            Axis::Z => &self.zs,
        }
    }

    pub(super) fn find_index(&self, axis: Axis, value: f64) -> isize {
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

    pub(super) fn is_full_wide(&self, axis: Axis, a: isize, b: isize, c: isize) -> bool {
        let (x, y, z) = match axis {
            Axis::X => (a, b, c),
            Axis::Y => (c, a, b),
            Axis::Z => (b, c, a),
        };
        self.is_full_wide_xyz(x, y, z)
    }

    pub(super) fn is_full_wide_xyz(&self, x: isize, y: isize, z: isize) -> bool {
        if x < 0 || y < 0 || z < 0 {
            return false;
        }
        let grid = self.grid();
        let (x, y, z) = (x as usize, y as usize, z as usize);
        x < grid.x_size && y < grid.y_size && z < grid.z_size && self.is_full(x, y, z)
    }
}
