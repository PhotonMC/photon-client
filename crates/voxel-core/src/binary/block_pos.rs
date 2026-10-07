use super::*;

impl BlockPos {
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
}

/// Integer block coordinates with reference packing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub const PACKED_HORIZONTAL_LENGTH: u32 = 26;

pub const PACKED_Y_LENGTH: u32 = 12;

pub(super) const PACKED_HORIZONTAL_MASK: u64 = (1 << PACKED_HORIZONTAL_LENGTH) - 1;

pub(super) const PACKED_Y_MASK: u64 = (1 << PACKED_Y_LENGTH) - 1;

pub(super) const X_OFFSET: u32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;

pub(super) const Z_OFFSET: u32 = PACKED_Y_LENGTH;

pub const MAX_HORIZONTAL_COORDINATE: i32 = (1 << PACKED_HORIZONTAL_LENGTH) / 2 - 1;
