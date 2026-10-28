use super::*;

impl BlockPos {
    /// Traverse a BlockPos graph using Java's FIFO, depth, count, and packed-visited rules.
    pub fn breadth_first_traversal<NeighbourProvider, NodeProcessor>(
        start: Self,
        max_depth: i32,
        max_count: i32,
        mut neighbour_provider: NeighbourProvider,
        mut node_processor: NodeProcessor,
    ) -> i32
    where
        NeighbourProvider: FnMut(Self, &mut dyn FnMut(Self)),
        NodeProcessor: FnMut(Self) -> TraversalNodeStatus,
    {
        let mut nodes = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        nodes.push_back((start, 0_i32));
        let mut count = 0_i32;

        while let Some((current, depth)) = nodes.pop_front() {
            if visited.insert(current.as_long()) {
                match node_processor(current) {
                    TraversalNodeStatus::Skip => {}
                    TraversalNodeStatus::Stop => break,
                    TraversalNodeStatus::Accept => {
                        count = count.wrapping_add(1);
                        if count >= max_count {
                            return count;
                        }

                        if depth < max_depth {
                            let mut neighbours = Vec::new();
                            neighbour_provider(current, &mut |position| neighbours.push(position));
                            for position in neighbours {
                                nodes.push_back((position, depth.wrapping_add(1)));
                            }
                        }
                    }
                }
            }
        }

        count
    }

    /// Enumerate Java's Manhattan reach iterator in its source traversal order.
    pub fn within_manhattan(origin: Self, reach_x: i32, reach_y: i32, reach_z: i32) -> Vec<Self> {
        let max_depth = reach_x.wrapping_add(reach_y).wrapping_add(reach_z);
        let mut positions = Vec::new();
        let mut current_depth: i32 = 0;
        let mut max_x: i32 = 0;
        let mut max_y: i32 = 0;
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut cursor = Self::ZERO;
        let mut z_mirror = false;

        loop {
            if z_mirror {
                z_mirror = false;
                cursor.z = origin.z.wrapping_sub(cursor.z.wrapping_sub(origin.z));
                positions.push(cursor);
                continue;
            }

            loop {
                if y > max_y {
                    x = x.wrapping_add(1);
                    if x > max_x {
                        current_depth = current_depth.wrapping_add(1);
                        if current_depth > max_depth {
                            return positions;
                        }

                        max_x = reach_x.min(current_depth);
                        x = max_x.wrapping_neg();
                    }

                    max_y = reach_y.min(current_depth.wrapping_sub(java_abs_i32(x)));
                    y = max_y.wrapping_neg();
                }

                let xx = x;
                let yy = y;
                let zz = current_depth
                    .wrapping_sub(java_abs_i32(xx))
                    .wrapping_sub(java_abs_i32(yy));
                y = y.wrapping_add(1);
                if zz <= reach_z {
                    z_mirror = zz != 0;
                    cursor = Self::new(
                        origin.x.wrapping_add(xx),
                        origin.y.wrapping_add(yy),
                        origin.z.wrapping_add(zz),
                    );
                    positions.push(cursor);
                    break;
                }
            }
        }
    }

    /// Enumerate Java's spiral iterator from two directions on different axes.
    pub fn spiral_around(
        center: Self,
        radius: i32,
        first_direction: Direction,
        second_direction: Direction,
    ) -> Result<Vec<Self>, SpiralAroundError> {
        if first_direction.axis() == second_direction.axis() {
            return Err(SpiralAroundError::SameAxis);
        }

        let legs = 4_i32.wrapping_mul(radius);
        if legs < 0 {
            return Ok(Vec::new());
        }

        let directions = [
            first_direction,
            second_direction,
            first_direction.opposite(),
            second_direction.opposite(),
        ];
        let mut cursor = center.relative_direction(second_direction, 1);
        let mut last = cursor;
        let mut positions = Vec::new();
        let mut leg: i32 = -1;
        let mut leg_size: i32 = 0;
        let mut leg_index: i32 = 0;

        loop {
            let direction = directions[(leg.wrapping_add(4) % 4) as usize];
            cursor = last.relative_direction(direction, 1);
            last = cursor;
            if leg_index >= leg_size {
                if leg >= legs {
                    return Ok(positions);
                }

                leg = leg.wrapping_add(1);
                leg_index = 0;
                leg_size = leg / 2 + 1;
            }

            leg_index = leg_index.wrapping_add(1);
            positions.push(cursor);
        }
    }

    /// Return the first Manhattan position accepted by a predicate.
    pub fn find_closest_match<F>(
        start_pos: Self,
        horizontal_search_radius: i32,
        vertical_search_radius: i32,
        mut predicate: F,
    ) -> Option<Self>
    where
        F: FnMut(Self) -> bool,
    {
        Self::within_manhattan(
            start_pos,
            horizontal_search_radius,
            vertical_search_radius,
            horizontal_search_radius,
        )
        .into_iter()
        .find(|&position| predicate(position))
    }

    /// Enumerate an inclusive range in Java's x-fastest, then y, then z order.
    pub fn between_closed_coordinates(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Vec<Self> {
        let width = max_x.wrapping_sub(min_x).wrapping_add(1);
        let height = max_y.wrapping_sub(min_y).wrapping_add(1);
        let depth = max_z.wrapping_sub(min_z).wrapping_add(1);
        let end = width.wrapping_mul(height).wrapping_mul(depth);
        if width <= 0 || height <= 0 || depth <= 0 || end <= 0 {
            return Vec::new();
        }

        let mut positions = Vec::with_capacity(end as usize);
        for index in 0..end {
            let x = min_x.wrapping_add(index % width);
            let slice = index / width;
            let y = min_y.wrapping_add(slice % height);
            let z = min_z.wrapping_add(slice / height);
            positions.push(Self::new(x, y, z));
        }
        positions
    }

    /// Enumerate Java's five horizontal neighbor columns with inclusive Y bounds.
    pub fn neighbor_column(start_x: i32, start_y: i32, start_z: i32, end_y: i32) -> Vec<Self> {
        let y_direction = if end_y > start_y { 1 } else { -1 };
        let height = java_abs_i32(end_y.wrapping_sub(start_y)).wrapping_add(1);
        let step_count = 5_i32.wrapping_mul(height);
        if height <= 0 || step_count <= 0 {
            return Vec::new();
        }

        let steps = [(0, 0), (0, -1), (1, 0), (0, 1), (-1, 0)];
        let mut positions = Vec::with_capacity(step_count as usize);
        for index in 0..step_count {
            let y = index % height;
            let step = steps[(index / height) as usize];
            positions.push(Self::new(
                start_x.wrapping_add(step.0),
                start_y.wrapping_add(y.wrapping_mul(y_direction)),
                start_z.wrapping_add(step.1),
            ));
        }
        positions
    }

    /// Return Java's deprecated four-position south-east square stream order.
    pub fn square_out_south_east(self) -> Vec<Self> {
        vec![self, self.south(), self.east(), self.south().east()]
    }

    /// Enumerate Java's bounded random block positions in x, y, z draw order.
    pub fn random_between_closed(
        random: &mut RandomSource,
        limit: i32,
        min: Self,
        max: Self,
    ) -> Result<Vec<Self>, RandomError> {
        let (min_x, min_y, min_z) = (min.x, min.y, min.z);
        let (max_x, max_y, max_z) = (max.x, max.y, max.z);
        let width = max_x.wrapping_sub(min_x).wrapping_add(1);
        let height = max_y.wrapping_sub(min_y).wrapping_add(1);
        let depth = max_z.wrapping_sub(min_z).wrapping_add(1);
        if limit <= 0 {
            return Ok(Vec::new());
        }

        let mut positions = Vec::with_capacity(limit as usize);
        for _ in 0..limit {
            positions.push(Self::new(
                min_x.wrapping_add(random.next_int_bound(width)?),
                min_y.wrapping_add(random.next_int_bound(height)?),
                min_z.wrapping_add(random.next_int_bound(depth)?),
            ));
        }
        Ok(positions)
    }

    /// Enumerate random block positions in the cube centered at `center`.
    pub fn random_in_cube(
        random: &mut RandomSource,
        limit: i32,
        center: Self,
        size_to_scan_in_all_directions: i32,
    ) -> Result<Vec<Self>, RandomError> {
        let reach = size_to_scan_in_all_directions;
        let shifted = |offset: fn(i32, i32) -> i32| {
            Self::new(
                offset(center.x, reach),
                offset(center.y, reach),
                offset(center.z, reach),
            )
        };
        Self::random_between_closed(
            random,
            limit,
            shifted(i32::wrapping_sub),
            shifted(i32::wrapping_add),
        )
    }

    /// Enumerate an inclusive range after componentwise endpoint normalization.
    pub fn between_closed(from: Self, to: Self) -> Vec<Self> {
        Self::between_closed_coordinates(
            from.x.min(to.x),
            from.y.min(to.y),
            from.z.min(to.z),
            from.x.max(to.x),
            from.y.max(to.y),
            from.z.max(to.z),
        )
    }

    /// Enumerate the blocks covered by an AABB after Java floor conversion.
    pub fn between_closed_aabb(value: Aabb) -> Vec<Self> {
        Self::between_closed(
            Self::new(
                math::floor_f64(value.min_x),
                math::floor_f64(value.min_y),
                math::floor_f64(value.min_z),
            ),
            Self::new(
                math::floor_f64(value.max_x),
                math::floor_f64(value.max_y),
                math::floor_f64(value.max_z),
            ),
        )
    }

    /// Forward the BlockPos stream overload to the inclusive range traversal.
    pub fn between_closed_stream(from: Self, to: Self) -> Vec<Self> {
        Self::between_closed(from, to)
    }

    /// Forward the BoundingBox stream overload after componentwise normalization.
    pub fn between_closed_stream_bounding_box(value: BoundingBox) -> Vec<Self> {
        Self::between_closed_coordinates(
            value.min_x.min(value.max_x),
            value.min_y.min(value.max_y),
            value.min_z.min(value.max_z),
            value.min_x.max(value.max_x),
            value.min_y.max(value.max_y),
            value.min_z.max(value.max_z),
        )
    }

    /// Forward the AABB stream overload through Java floor conversion.
    pub fn between_closed_stream_aabb(value: Aabb) -> Vec<Self> {
        Self::between_closed_aabb(value)
    }

    /// Forward the coordinate stream overload to the inclusive range traversal.
    pub fn between_closed_stream_coordinates(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Vec<Self> {
        Self::between_closed_coordinates(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// Enumerate the AABB corner traversal after Java floor conversion.
    pub fn between_corners_in_direction_aabb(value: Aabb, direction: Vec3) -> Vec<Self> {
        Self::between_corners_in_direction(
            Self::new(
                math::floor_f64(value.min_x),
                math::floor_f64(value.min_y),
                math::floor_f64(value.min_z),
            ),
            Self::new(
                math::floor_f64(value.max_x),
                math::floor_f64(value.max_y),
                math::floor_f64(value.max_z),
            ),
            direction,
        )
    }

    /// Enumerate normalized integer corners in the direction-prioritized order.
    pub fn between_corners_in_direction(first: Self, second: Self, direction: Vec3) -> Vec<Self> {
        let (min_x, max_x) = (first.x.min(second.x), first.x.max(second.x));
        let (min_y, max_y) = (first.y.min(second.y), first.y.max(second.y));
        let (min_z, max_z) = (first.z.min(second.z), first.z.max(second.z));
        let diff_x = max_x.wrapping_sub(min_x);
        let diff_y = max_y.wrapping_sub(min_y);
        let diff_z = max_z.wrapping_sub(min_z);
        let start_x = if direction.x >= 0.0 { min_x } else { max_x };
        let start_y = if direction.y >= 0.0 { min_y } else { max_y };
        let start_z = if direction.z >= 0.0 { min_z } else { max_z };
        let [first_axis, second_axis, third_axis] =
            Direction::axis_step_order(direction.x, direction.z);
        let axis_component = |axis| match axis {
            Axis::X => direction.x,
            Axis::Y => direction.y,
            Axis::Z => direction.z,
        };
        let axis_direction = |axis| {
            if axis_component(axis) >= 0.0 {
                axis.get_positive()
            } else {
                axis.get_negative()
            }
        };
        let steps = [first_axis, second_axis, third_axis].map(|axis| axis_direction(axis).step());
        let first_max = first_axis.choose_i32(diff_x, diff_y, diff_z);
        let second_max = second_axis.choose_i32(diff_x, diff_y, diff_z);
        let third_max = third_axis.choose_i32(diff_x, diff_y, diff_z);
        let mut first_index = 0_i32;
        let mut second_index = 0_i32;
        let mut third_index = 0_i32;
        let mut positions = Vec::new();

        loop {
            let indices = [first_index, second_index, third_index];
            let x = offset_along(start_x, steps, indices, |step| step.0);
            let y = offset_along(start_y, steps, indices, |step| step.1);
            let z = offset_along(start_z, steps, indices, |step| step.2);
            positions.push(Self::new(x, y, z));

            if third_index < third_max {
                third_index = third_index.wrapping_add(1);
            } else if second_index < second_max {
                second_index = second_index.wrapping_add(1);
                third_index = 0;
            } else if first_index < first_max {
                first_index = first_index.wrapping_add(1);
                second_index = 0;
                third_index = 0;
            } else {
                break;
            }
        }

        positions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiralAroundError {
    SameAxis,
}

/// Result of processing a node in a bounded BlockPos traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalNodeStatus {
    Accept,
    Skip,
    Stop,
}

/// `start` moved along the three steps by their indices, one wrapping add after another.
fn offset_along(
    start: i32,
    steps: [(i32, i32, i32); 3],
    indices: [i32; 3],
    pick: fn((i32, i32, i32)) -> i32,
) -> i32 {
    steps.iter().zip(indices).fold(start, |at, (step, index)| {
        at.wrapping_add(pick(*step).wrapping_mul(index))
    })
}
