use super::*;

impl BlockPos {
    pub const ZERO: Self = Self::new(0, 0, 0);

    pub const PACKED_HORIZONTAL_LENGTH: u32 = PACKED_HORIZONTAL_LENGTH;

    pub const PACKED_Y_LENGTH: u32 = PACKED_Y_LENGTH;

    pub const MAX_HORIZONTAL_COORDINATE: i32 = MAX_HORIZONTAL_COORDINATE;

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Return this immutable position unchanged.
    pub const fn immutable(self) -> Self {
        self
    }

    /// Create mutable state initialized from this position.
    pub const fn mutable(self) -> MutableBlockPos {
        MutableBlockPos::new(self.x, self.y, self.z)
    }

    /// Convert double coordinates to a BlockPos using Java floor semantics.
    pub fn containing(x: f64, y: f64, z: f64) -> Self {
        Self::new(math::floor_f64(x), math::floor_f64(y), math::floor_f64(z))
    }

    /// Convert a Vec3 Position to a BlockPos using Java floor semantics.
    pub fn containing_vec3(value: Vec3) -> Self {
        Self::containing(value.x, value.y, value.z)
    }

    /// Offset coordinates with Java i32 wrapping arithmetic.
    pub fn offset(self, x: i32, y: i32, z: i32) -> Self {
        if x == 0 && y == 0 && z == 0 {
            self
        } else {
            Self::new(
                self.x.wrapping_add(x),
                self.y.wrapping_add(y),
                self.z.wrapping_add(z),
            )
        }
    }

    /// Return Java `BlockPos.offset(Vec3i)` with a block position as delta.
    pub fn offset_by_block_pos(self, delta: Self) -> Self {
        self.offset(delta.x, delta.y, delta.z)
    }

    /// Return Java `BlockPos.subtract(Vec3i)` with wrapping unary negation.
    pub fn subtract_block_pos(self, delta: Self) -> Self {
        self.offset(
            delta.x.wrapping_neg(),
            delta.y.wrapping_neg(),
            delta.z.wrapping_neg(),
        )
    }

    /// Return Java `BlockPos.multiply(int)` with Java i32 wrapping arithmetic.
    pub fn multiply(self, scale: i32) -> Self {
        if scale == 1 {
            self
        } else if scale == 0 {
            Self::ZERO
        } else {
            Self::new(
                self.x.wrapping_mul(scale),
                self.y.wrapping_mul(scale),
                self.z.wrapping_mul(scale),
            )
        }
    }

    /// Rotate around the y axis using Java `BlockPos.rotate(Rotation)`.
    pub fn rotate(self, rotation: BlockRotation) -> Self {
        match rotation {
            BlockRotation::None => self,
            BlockRotation::Clockwise90 => Self::new(self.z.wrapping_neg(), self.y, self.x),
            BlockRotation::Clockwise180 => {
                Self::new(self.x.wrapping_neg(), self.y, self.z.wrapping_neg())
            }
            BlockRotation::CounterClockwise90 => Self::new(self.z, self.y, self.x.wrapping_neg()),
        }
    }

    /// Return Java `BlockPos.cross(Vec3i)` with Java i32 wrapping arithmetic.
    pub fn cross(self, up_vector: Self) -> Self {
        Self::new(
            self.y
                .wrapping_mul(up_vector.z)
                .wrapping_sub(self.z.wrapping_mul(up_vector.y)),
            self.z
                .wrapping_mul(up_vector.x)
                .wrapping_sub(self.x.wrapping_mul(up_vector.z)),
            self.x
                .wrapping_mul(up_vector.y)
                .wrapping_sub(self.y.wrapping_mul(up_vector.x)),
        )
    }

    /// Return a copy with only the y coordinate replaced.
    pub fn at_y(self, y: i32) -> Self {
        Self::new(self.x, y, self.z)
    }

    /// Return Java `Vec3i.distSqr(Vec3i)` for block coordinates.
    pub fn dist_sqr(self, pos: Self) -> f64 {
        self.dist_to_low_corner_sqr(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z))
    }

    /// Return Java `Vec3i.distToLowCornerSqr(double, double, double)`.
    pub fn dist_to_low_corner_sqr(self, x: f64, y: f64, z: f64) -> f64 {
        let dx = f64::from(self.x) - x;
        let dy = f64::from(self.y) - y;
        let dz = f64::from(self.z) - z;
        dx * dx + dy * dy + dz * dz
    }

    /// Return Java `Vec3i.distToCenterSqr(double, double, double)`.
    pub fn dist_to_center_sqr(self, x: f64, y: f64, z: f64) -> f64 {
        let dx = f64::from(self.x) + 0.5 - x;
        let dy = f64::from(self.y) + 0.5 - y;
        let dz = f64::from(self.z) + 0.5 - z;
        dx * dx + dy * dy + dz * dz
    }

    /// Return Java `Vec3i.closerThan(Vec3i, double)`.
    pub fn closer_than(self, pos: Self, distance: f64) -> bool {
        self.dist_sqr(pos) < distance * distance
    }

    /// Return Java `Vec3i.distManhattan(Vec3i)` with Java int/float casts.
    pub fn dist_manhattan(self, pos: Self) -> i32 {
        let xd = java_abs_i32(pos.x.wrapping_sub(self.x)) as f32;
        let yd = java_abs_i32(pos.y.wrapping_sub(self.y)) as f32;
        let zd = java_abs_i32(pos.z.wrapping_sub(self.z)) as f32;
        (xd + yd + zd) as i32
    }

    /// Return Java `Vec3i.distChessboard(Vec3i)` with Java i32 arithmetic.
    pub fn dist_chessboard(self, pos: Self) -> i32 {
        let xd = java_abs_i32(self.x.wrapping_sub(pos.x));
        let yd = java_abs_i32(self.y.wrapping_sub(pos.y));
        let zd = java_abs_i32(self.z.wrapping_sub(pos.z));
        xd.max(yd).max(zd)
    }

    /// Return Java `Vec3i.get(Direction.Axis)`.
    pub fn get(self, axis: crate::direction::Axis) -> i32 {
        axis.choose_i32(self.x, self.y, self.z)
    }

    /// Clamp a Vec3 inside this block using Java's mixed-precision bounds.
    pub fn clamp_location_within(self, location: Vec3) -> Vec3 {
        let epsilon = 1.0e-5_f32;
        let lower = |coordinate: i32| (coordinate as f32 + epsilon) as f64;
        let upper = |coordinate: i32| f64::from(coordinate) + 1.0 - f64::from(epsilon);
        Vec3::new(
            math::clamp_f64(location.x, lower(self.x), upper(self.x)),
            math::clamp_f64(location.y, lower(self.y), upper(self.y)),
            math::clamp_f64(location.z, lower(self.z), upper(self.z)),
        )
    }

    /// Return Java `BlockPos.relative(Direction)` and its stepped overload.
    pub fn relative_direction(self, direction: Direction, steps: i32) -> Self {
        if steps == 0 {
            self
        } else {
            let (step_x, step_y, step_z) = direction.step();
            Self::new(
                self.x.wrapping_add(step_x.wrapping_mul(steps)),
                self.y.wrapping_add(step_y.wrapping_mul(steps)),
                self.z.wrapping_add(step_z.wrapping_mul(steps)),
            )
        }
    }

    /// Return Java `BlockPos.relative(Axis, int)`.
    pub fn relative_axis(self, axis: crate::direction::Axis, steps: i32) -> Self {
        if steps == 0 {
            self
        } else {
            let (step_x, step_y, step_z) = match axis {
                crate::direction::Axis::X => (steps, 0, 0),
                crate::direction::Axis::Y => (0, steps, 0),
                crate::direction::Axis::Z => (0, 0, steps),
            };
            Self::new(
                self.x.wrapping_add(step_x),
                self.y.wrapping_add(step_y),
                self.z.wrapping_add(step_z),
            )
        }
    }

    /// Return Java `BlockPos.above()`.
    pub fn above(self) -> Self {
        self.relative_direction(Direction::Up, 1)
    }

    /// Return Java `BlockPos.above(int)`.
    pub fn above_steps(self, steps: i32) -> Self {
        self.relative_direction(Direction::Up, steps)
    }

    /// Return Java `BlockPos.below()`.
    pub fn below(self) -> Self {
        self.relative_direction(Direction::Down, 1)
    }

    /// Return Java `BlockPos.below(int)`.
    pub fn below_steps(self, steps: i32) -> Self {
        self.relative_direction(Direction::Down, steps)
    }

    /// Return Java `BlockPos.north()`.
    pub fn north(self) -> Self {
        self.relative_direction(Direction::North, 1)
    }

    /// Return Java `BlockPos.north(int)`.
    pub fn north_steps(self, steps: i32) -> Self {
        self.relative_direction(Direction::North, steps)
    }

    /// Return Java `BlockPos.south()`.
    pub fn south(self) -> Self {
        self.relative_direction(Direction::South, 1)
    }

    /// Return Java `BlockPos.south(int)`.
    pub fn south_steps(self, steps: i32) -> Self {
        self.relative_direction(Direction::South, steps)
    }

    /// Return Java `BlockPos.west()`.
    pub fn west(self) -> Self {
        self.relative_direction(Direction::West, 1)
    }

    /// Return Java `BlockPos.west(int)`.
    pub fn west_steps(self, steps: i32) -> Self {
        self.relative_direction(Direction::West, steps)
    }

    /// Return Java `BlockPos.east()`.
    pub fn east(self) -> Self {
        self.relative_direction(Direction::East, 1)
    }

    /// Return Java `BlockPos.east(int)`.
    pub fn east_steps(self, steps: i32) -> Self {
        self.relative_direction(Direction::East, steps)
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
        Self::between_corners_in_direction_coordinates(
            math::floor_f64(value.min_x),
            math::floor_f64(value.min_y),
            math::floor_f64(value.min_z),
            math::floor_f64(value.max_x),
            math::floor_f64(value.max_y),
            math::floor_f64(value.max_z),
            direction,
        )
    }

    /// Enumerate the corner traversal between two BlockPos values.
    pub fn between_corners_in_direction(first: Self, second: Self, direction: Vec3) -> Vec<Self> {
        Self::between_corners_in_direction_coordinates(
            first.x, first.y, first.z, second.x, second.y, second.z, direction,
        )
    }

    /// Enumerate normalized integer corners in the direction-prioritized order.
    pub fn between_corners_in_direction_coordinates(
        first_x: i32,
        first_y: i32,
        first_z: i32,
        second_x: i32,
        second_y: i32,
        second_z: i32,
        direction: Vec3,
    ) -> Vec<Self> {
        let min_x = first_x.min(second_x);
        let min_y = first_y.min(second_y);
        let min_z = first_z.min(second_z);
        let max_x = first_x.max(second_x);
        let max_y = first_y.max(second_y);
        let max_z = first_z.max(second_z);
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
        let first_step = axis_direction(first_axis).step();
        let second_step = axis_direction(second_axis).step();
        let third_step = axis_direction(third_axis).step();
        let first_max = first_axis.choose_i32(diff_x, diff_y, diff_z);
        let second_max = second_axis.choose_i32(diff_x, diff_y, diff_z);
        let third_max = third_axis.choose_i32(diff_x, diff_y, diff_z);
        let mut first_index = 0_i32;
        let mut second_index = 0_i32;
        let mut third_index = 0_i32;
        let mut positions = Vec::new();

        loop {
            let x = start_x
                .wrapping_add(first_step.0.wrapping_mul(first_index))
                .wrapping_add(second_step.0.wrapping_mul(second_index))
                .wrapping_add(third_step.0.wrapping_mul(third_index));
            let y = start_y
                .wrapping_add(first_step.1.wrapping_mul(first_index))
                .wrapping_add(second_step.1.wrapping_mul(second_index))
                .wrapping_add(third_step.1.wrapping_mul(third_index));
            let z = start_z
                .wrapping_add(first_step.2.wrapping_mul(first_index))
                .wrapping_add(second_step.2.wrapping_mul(second_index))
                .wrapping_add(third_step.2.wrapping_mul(third_index));
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
