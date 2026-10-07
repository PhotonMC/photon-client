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
    // The six coordinates mirror Java's public overload exactly.
    #[expect(
        clippy::too_many_arguments,
        reason = "the reference method's parameters, in its order"
    )]
    pub fn random_between_closed(
        random: &mut RandomSource,
        limit: i32,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Result<Vec<Self>, RandomError> {
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
        Self::random_between_closed(
            random,
            limit,
            center.x.wrapping_sub(size_to_scan_in_all_directions),
            center.y.wrapping_sub(size_to_scan_in_all_directions),
            center.z.wrapping_sub(size_to_scan_in_all_directions),
            center.x.wrapping_add(size_to_scan_in_all_directions),
            center.y.wrapping_add(size_to_scan_in_all_directions),
            center.z.wrapping_add(size_to_scan_in_all_directions),
        )
    }

    /// Offset a packed block position by a cardinal direction's step.
    pub fn offset_packed(block_node: u64, direction: Direction) -> u64 {
        Self::offset_packed_by(
            block_node,
            direction.get_step_x(),
            direction.get_step_y(),
            direction.get_step_z(),
        )
    }

    /// Offset a packed block position with Java i32 wrapping arithmetic.
    pub fn offset_packed_by(block_node: u64, step_x: i32, step_y: i32, step_z: i32) -> u64 {
        let position = Self::unpack(block_node);
        Self::as_long_coordinates(
            position.x.wrapping_add(step_x),
            position.y.wrapping_add(step_y),
            position.z.wrapping_add(step_z),
        )
    }

    /// Read the signed x field from a packed block position.
    pub const fn get_x(block_node: u64) -> i32 {
        ((block_node << (64 - X_OFFSET - PACKED_HORIZONTAL_LENGTH)) as i64
            >> (64 - PACKED_HORIZONTAL_LENGTH)) as i32
    }

    /// Read the signed y field from a packed block position.
    pub const fn get_y(block_node: u64) -> i32 {
        ((block_node << (64 - PACKED_Y_LENGTH)) as i64 >> (64 - PACKED_Y_LENGTH)) as i32
    }

    /// Read the signed z field from a packed block position.
    pub const fn get_z(block_node: u64) -> i32 {
        ((block_node << (64 - Z_OFFSET - PACKED_HORIZONTAL_LENGTH)) as i64
            >> (64 - PACKED_HORIZONTAL_LENGTH)) as i32
    }

    /// Return Java `BlockPos.of(long)`.
    pub const fn of(block_node: u64) -> Self {
        Self::new(
            Self::get_x(block_node),
            Self::get_y(block_node),
            Self::get_z(block_node),
        )
    }

    /// Return Java `BlockPos.getFlatIndex(long)`.
    pub const fn get_flat_index(neighbor_block_node: u64) -> u64 {
        neighbor_block_node & !15
    }

    /// Return Java `BlockPos.asLong()`.
    pub fn as_long(self) -> u64 {
        self.pack()
    }

    /// Return Java `BlockPos.asLong(int, int, int)`.
    pub fn as_long_coordinates(x: i32, y: i32, z: i32) -> u64 {
        Self::new(x, y, z).pack()
    }

    /// Componentwise minimum, matching `BlockPos.min`.
    pub fn min(a: Self, b: Self) -> Self {
        Self::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
    }

    /// Componentwise maximum, matching `BlockPos.max`.
    pub fn max(a: Self, b: Self) -> Self {
        Self::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
    }

    /// Pack with masking, so out-of-range inputs wrap as the reference does.
    pub fn pack(self) -> u64 {
        ((u64::from(self.x as u32) & PACKED_HORIZONTAL_MASK) << X_OFFSET)
            | ((u64::from(self.z as u32) & PACKED_HORIZONTAL_MASK) << Z_OFFSET)
            | (u64::from(self.y as u32) & PACKED_Y_MASK)
    }

    /// Unpack with arithmetic shifts, sign-extending each field.
    pub fn unpack(packed: u64) -> Self {
        Self::of(packed)
    }
}
