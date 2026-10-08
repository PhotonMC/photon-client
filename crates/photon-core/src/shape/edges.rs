use super::*;

impl VoxelShape {
    pub(in super::super) fn is_full(&self, x: usize, y: usize, z: usize) -> bool {
        self.cells[index(x, y, z, self.grid())]
    }

    pub(in super::super) fn full_at(&self, x: f64, y: f64, z: f64) -> bool {
        let x_index = interval(&self.xs, x);
        let y_index = interval(&self.ys, y);
        let z_index = interval(&self.zs, z);
        match (x_index, y_index, z_index) {
            (Some(x), Some(y), Some(z)) => self.is_full(x, y, z),
            _ => false,
        }
    }

    pub(in super::super) fn grid(&self) -> Grid {
        Grid::from_coords(&self.xs, &self.ys, &self.zs)
    }

    pub(in super::super) fn axis_size(&self, axis: Axis) -> usize {
        self.axis_coords(axis).len().saturating_sub(1)
    }

    pub(in super::super) fn axis_coord(&self, axis: Axis, index: usize) -> f64 {
        self.axis_coords(axis)[index]
    }

    pub(in super::super) fn axis_coords(&self, axis: Axis) -> &[f64] {
        match axis {
            Axis::X => &self.xs,
            Axis::Y => &self.ys,
            Axis::Z => &self.zs,
        }
    }

    pub(in super::super) fn find_index(&self, axis: Axis, value: f64) -> isize {
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

    pub(in super::super) fn is_full_wide(&self, axis: Axis, a: isize, b: isize, c: isize) -> bool {
        let (x, y, z) = match axis {
            Axis::X => (a, b, c),
            Axis::Y => (c, a, b),
            Axis::Z => (b, c, a),
        };
        self.is_full_wide_xyz(x, y, z)
    }

    pub(in super::super) fn is_full_wide_xyz(&self, x: isize, y: isize, z: isize) -> bool {
        if x < 0 || y < 0 || z < 0 {
            return false;
        }
        let grid = self.grid();
        let (x, y, z) = (x as usize, y as usize, z as usize);
        x < grid.x_size && y < grid.y_size && z < grid.z_size && self.is_full(x, y, z)
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

    pub(in super::super) fn for_all_axis_edges<F>(
        &self,
        consumer: &mut F,
        transform: ShapeAxisCycle,
    ) where
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

    pub(in super::super) fn is_full_wide_cycle(
        &self,
        cycle: ShapeAxisCycle,
        point: [isize; 3],
    ) -> bool {
        self.is_full_wide_xyz(
            cycle.cycle_index(point, Axis::X),
            cycle.cycle_index(point, Axis::Y),
            cycle.cycle_index(point, Axis::Z),
        )
    }

    pub(in super::super) fn consume_edge<F>(
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
}
