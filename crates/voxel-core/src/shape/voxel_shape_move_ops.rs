use super::*;

impl VoxelShape {
    pub(super) fn new(xs: Vec<f64>, ys: Vec<f64>, zs: Vec<f64>, cells: Vec<bool>) -> Self {
        debug_assert_eq!(cells.len(), Grid::from_coords(&xs, &ys, &zs).cell_count());
        Self {
            xs,
            ys,
            zs,
            cells,
            identity: Arc::new(()),
        }
    }

    /// A number no other live shape shares, like Java's identity hash: the
    /// key for caches the reference keeps per shape object.
    pub fn identity(&self) -> usize {
        Arc::as_ptr(&self.identity) as usize
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

    pub(super) fn for_all_axis_edges<F>(&self, consumer: &mut F, transform: ShapeAxisCycle)
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

    pub(super) fn is_full_wide_cycle(&self, cycle: ShapeAxisCycle, point: [isize; 3]) -> bool {
        self.is_full_wide_xyz(
            cycle.cycle_index(point, Axis::X),
            cycle.cycle_index(point, Axis::Y),
            cycle.cycle_index(point, Axis::Z),
        )
    }

    pub(super) fn consume_edge<F>(
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
}
