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

    pub(in super::super) fn slice_at(&self, axis: Axis, slice: isize) -> Self {
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

    pub(in super::super) fn is_cube_like(&self) -> bool {
        self.is_cube_like_along(Axis::X)
            && self.is_cube_like_along(Axis::Y)
            && self.is_cube_like_along(Axis::Z)
    }

    pub(in super::super) fn is_block(&self) -> bool {
        self.cells.len() == 1 && self.cells[0] && self.is_cube_like()
    }

    pub(in super::super) fn is_cube_like_along(&self, axis: Axis) -> bool {
        let coords = self.axis_coords(axis);
        coords.len() == 2 && fuzzy_equals(coords[0], 0.0) && fuzzy_equals(coords[1], 1.0)
    }

    pub(in super::super) fn min_coordinate(&self, axis: Axis) -> f64 {
        self.bounds()
            .map(|bounds| match axis {
                Axis::X => bounds.min_x,
                Axis::Y => bounds.min_y,
                Axis::Z => bounds.min_z,
            })
            .unwrap_or(f64::INFINITY)
    }

    pub(in super::super) fn max_coordinate(&self, axis: Axis) -> f64 {
        self.bounds()
            .map(|bounds| match axis {
                Axis::X => bounds.max_x,
                Axis::Y => bounds.max_y,
                Axis::Z => bounds.max_z,
            })
            .unwrap_or(f64::NEG_INFINITY)
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
}
