//! Binary codecs for the pinned reference: Vec3, UUID, BlockPos, long arrays and BitSets.
//!
//! UUID and positions travel as big-endian longs with no length prefix.
//! Long arrays prefix a VarInt length; BitSets reuse that framing through
//! trimmed little-endian words. Each `readLong` is atomic, so a truncated
//! UUID can leave the first half consumed while a truncated position
//! consumes nothing.

use crate::aabb::Aabb;
use crate::direction::Direction;
use crate::math::{self, Vec3};
use crate::random::{RandomError, RandomSource};
use crate::wire;

/// Failures for binary reads, using the runner's error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    BufferUnderflow,
    DecoderError,
    TooBig,
}

/// 128-bit identifier split into big-endian halves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uuid {
    pub most: u64,
    pub least: u64,
}

impl Uuid {
    pub const fn new(most: u64, least: u64) -> Self {
        Self { most, least }
    }
}

fn read_double(input: &mut &[u8]) -> Result<f64, ReadError> {
    if input.len() < 8 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(8);
    let value = f64::from_bits(u64::from_be_bytes(head.try_into().expect("checked length")));
    *input = rest;
    Ok(value)
}

/// Read three consecutive big-endian doubles as a Vec3.
pub fn read_vec3(input: &mut &[u8]) -> Result<Vec3, ReadError> {
    let x = read_double(input)?;
    let y = read_double(input)?;
    let z = read_double(input)?;
    Ok(Vec3::new(x, y, z))
}

/// Append three consecutive big-endian doubles for a Vec3.
pub fn write_vec3(value: Vec3, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.x.to_bits().to_be_bytes());
    output.extend_from_slice(&value.y.to_bits().to_be_bytes());
    output.extend_from_slice(&value.z.to_bits().to_be_bytes());
}

const LP_MAX_QUANTIZED_VALUE: u64 = 32766;
const LP_ABS_MAX_VALUE: f64 = 1.7179869183e10;
const LP_ABS_MIN_VALUE: f64 = 3.051944088384301e-5;
const MAX_CHUNK_COORDINATE_VALUE: i32 = 2_097_061;

fn read_byte(input: &mut &[u8]) -> Result<u8, ReadError> {
    let (&head, rest) = input.split_first().ok_or(ReadError::BufferUnderflow)?;
    *input = rest;
    Ok(head)
}

fn read_u32(input: &mut &[u8]) -> Result<u32, ReadError> {
    if input.len() < 4 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(4);
    let value = u32::from_be_bytes(head.try_into().expect("checked length"));
    *input = rest;
    Ok(value)
}

fn lp_unpack(value: u64) -> f64 {
    let quantized = (value & 0x7fff).min(LP_MAX_QUANTIZED_VALUE);
    quantized as f64 * 2.0 / LP_MAX_QUANTIZED_VALUE as f64 - 1.0
}

fn lp_sanitize(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else if value < -LP_ABS_MAX_VALUE {
        -LP_ABS_MAX_VALUE
    } else {
        value.min(LP_ABS_MAX_VALUE)
    }
}

fn lp_pack(value: f64) -> u64 {
    let scaled = (value * 0.5 + 0.5) * LP_MAX_QUANTIZED_VALUE as f64;
    (scaled + 0.5).floor() as u64
}

/// Decode the packed `Vec3.LP_STREAM_CODEC` representation.
pub fn read_lp_vec3(input: &mut &[u8]) -> Result<Vec3, ReadError> {
    let lowest = read_byte(input)?;
    if lowest == 0 {
        return Ok(Vec3::ZERO);
    }
    let middle = read_byte(input)?;
    let highest = read_u32(input)?;
    let buffer = u64::from(highest) << 16 | u64::from(middle) << 8 | u64::from(lowest);
    let mut scale = u64::from(lowest & 3);
    if lowest & 4 != 0 {
        let continuation = wire::read_varint(input).map_err(|error| match error {
            wire::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
            wire::ReadError::TooBig => ReadError::TooBig,
        })?;
        scale |= u64::from(continuation as u32) << 2;
    }
    let scale = scale as f64;
    Ok(Vec3::new(
        lp_unpack(buffer >> 3) * scale,
        lp_unpack(buffer >> 18) * scale,
        lp_unpack(buffer >> 33) * scale,
    ))
}

/// Encode a Vec3 with the packed `Vec3.LP_STREAM_CODEC` representation.
pub fn write_lp_vec3(value: Vec3, output: &mut Vec<u8>) {
    let x = lp_sanitize(value.x);
    let y = lp_sanitize(value.y);
    let z = lp_sanitize(value.z);
    let chessboard_length = x.abs().max(y.abs().max(z.abs()));
    if chessboard_length < LP_ABS_MIN_VALUE {
        output.push(0);
        return;
    }

    let scale = chessboard_length.ceil() as u64;
    let partial = (scale & 3) != scale;
    let markers = if partial { scale & 3 | 4 } else { scale };
    let buffer = markers
        | lp_pack(x / scale as f64) << 3
        | lp_pack(y / scale as f64) << 18
        | lp_pack(z / scale as f64) << 33;
    output.push(buffer as u8);
    output.push((buffer >> 8) as u8);
    output.extend_from_slice(&((buffer >> 16) as u32).to_be_bytes());
    if partial {
        wire::write_varint((scale >> 2) as i32, output);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockRotation {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

impl BlockRotation {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "none" => Some(Self::None),
            "clockwise_90" => Some(Self::Clockwise90),
            "180" => Some(Self::Clockwise180),
            "counterclockwise_90" => Some(Self::CounterClockwise90),
            _ => None,
        }
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
const PACKED_HORIZONTAL_MASK: u64 = (1 << PACKED_HORIZONTAL_LENGTH) - 1;
const PACKED_Y_MASK: u64 = (1 << PACKED_Y_LENGTH) - 1;
const X_OFFSET: u32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;
const Z_OFFSET: u32 = PACKED_Y_LENGTH;
pub const MAX_HORIZONTAL_COORDINATE: i32 = (1 << PACKED_HORIZONTAL_LENGTH) / 2 - 1;

fn java_abs_i32(value: i32) -> i32 {
    if value < 0 {
        value.wrapping_neg()
    } else {
        value
    }
}

impl BlockPos {
    pub const ZERO: Self = Self::new(0, 0, 0);
    pub const PACKED_HORIZONTAL_LENGTH: u32 = PACKED_HORIZONTAL_LENGTH;
    pub const PACKED_Y_LENGTH: u32 = PACKED_Y_LENGTH;
    pub const MAX_HORIZONTAL_COORDINATE: i32 = MAX_HORIZONTAL_COORDINATE;

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
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
    #[allow(clippy::too_many_arguments)]
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

/// Integer chunk coordinates packed as x in the low half and z in the high half.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkPosCodecError {
    WrongLength {
        actual: usize,
        partial: Option<ChunkPos>,
    },
}

impl ChunkPos {
    pub const INVALID_CHUNK_POS: u64 = 1_875_066_u64 | (1_875_066_u64 << 32);
    pub const ZERO: Self = Self::new(0, 0);
    pub const REGION_SIZE: i32 = 32;
    pub const REGION_MAX_INDEX: i32 = 31;

    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Pack signed coordinates into the reference's two 32-bit fields.
    pub fn pack(self) -> u64 {
        u64::from(self.x as u32) | (u64::from(self.z as u32) << 32)
    }

    /// Unpack the two signed 32-bit fields from a packed long.
    pub fn unpack(packed: u64) -> Self {
        Self {
            x: Self::get_x(packed),
            z: Self::get_z(packed),
        }
    }

    /// Read the low signed 32-bit coordinate from a packed ChunkPos.
    pub const fn get_x(packed: u64) -> i32 {
        packed as u32 as i32
    }

    /// Read the high signed 32-bit coordinate from a packed ChunkPos.
    pub const fn get_z(packed: u64) -> i32 {
        (packed >> 32) as u32 as i32
    }

    /// Convert a packed SectionPos node to a packed ChunkPos, discarding y.
    pub fn from_section_node(section_node: u64) -> u64 {
        let section = SectionPos::unpack(section_node);
        Self::new(section.x, section.z).pack()
    }

    /// Decode the fixed two-element list used by Java `ChunkPos.CODEC`.
    pub fn codec_decode(values: &[i32]) -> Result<Self, ChunkPosCodecError> {
        match values {
            [x, z] => Ok(Self::new(*x, *z)),
            values => Err(ChunkPosCodecError::WrongLength {
                actual: values.len(),
                partial: (values.len() >= 2).then(|| Self::new(values[0], values[1])),
            }),
        }
    }

    /// Encode a ChunkPos in the x, z order used by Java `ChunkPos.CODEC`.
    pub const fn codec_encode(self) -> [i32; 2] {
        [self.x, self.z]
    }

    /// Return a block x coordinate using Java's section-to-block shift and add.
    pub fn get_block_x(self, offset: i32) -> i32 {
        self.x.wrapping_shl(4).wrapping_add(offset)
    }

    /// Return a block z coordinate using Java's section-to-block shift and add.
    pub fn get_block_z(self, offset: i32) -> i32 {
        self.z.wrapping_shl(4).wrapping_add(offset)
    }

    pub fn get_min_block_x(self) -> i32 {
        self.get_block_x(0)
    }

    pub fn get_min_block_z(self) -> i32 {
        self.get_block_z(0)
    }

    pub fn get_middle_block_x(self) -> i32 {
        self.get_block_x(8)
    }

    pub fn get_middle_block_z(self) -> i32 {
        self.get_block_z(8)
    }

    pub fn get_max_block_x(self) -> i32 {
        self.get_block_x(15)
    }

    pub fn get_max_block_z(self) -> i32 {
        self.get_block_z(15)
    }

    /// Return the arithmetic-shifted region x coordinate.
    pub fn get_region_x(self) -> i32 {
        self.x >> 5
    }

    /// Return the arithmetic-shifted region z coordinate.
    pub fn get_region_z(self) -> i32 {
        self.z >> 5
    }

    /// Return the region x coordinate from a packed ChunkPos.
    pub fn get_region_x_from_packed(packed: u64) -> i32 {
        Self::unpack(packed).get_region_x()
    }

    /// Return the region z coordinate from a packed ChunkPos.
    pub fn get_region_z_from_packed(packed: u64) -> i32 {
        Self::unpack(packed).get_region_z()
    }

    /// Return the chunk's x index inside its 32-by-32 region.
    pub fn get_region_local_x(self) -> i32 {
        self.x & 31
    }

    /// Return the chunk's z index inside its 32-by-32 region.
    pub fn get_region_local_z(self) -> i32 {
        self.z & 31
    }

    /// Return the first chunk in a region using Java i32 shift wrapping.
    pub fn min_from_region(region_x: i32, region_z: i32) -> Self {
        Self::new(region_x.wrapping_shl(5), region_z.wrapping_shl(5))
    }

    /// Return the last chunk in a region using Java i32 add wrapping.
    pub fn max_from_region(region_x: i32, region_z: i32) -> Self {
        let min = Self::min_from_region(region_x, region_z);
        Self::new(min.x.wrapping_add(31), min.z.wrapping_add(31))
    }

    /// Return a block position at offsets within this chunk using Java i32 wrapping.
    pub fn get_block_at(self, x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos::new(self.get_block_x(x), y, self.get_block_z(z))
    }

    /// Return the chunk's middle block at the requested y coordinate.
    pub fn get_middle_block_position(self, y: i32) -> BlockPos {
        BlockPos::new(self.get_middle_block_x(), y, self.get_middle_block_z())
    }

    /// Return the minimum block position at y=0.
    pub fn get_world_position(self) -> BlockPos {
        BlockPos::new(self.get_min_block_x(), 0, self.get_min_block_z())
    }

    /// Check inclusive horizontal containment; the y coordinate is ignored.
    pub fn contains(self, pos: BlockPos) -> bool {
        pos.x >= self.get_min_block_x()
            && pos.z >= self.get_min_block_z()
            && pos.x <= self.get_max_block_x()
            && pos.z <= self.get_max_block_z()
    }

    /// Find the chunk containing a block position using Java's arithmetic shift.
    pub fn containing(pos: BlockPos) -> Self {
        Self::new(
            math::block_to_section_coord(pos.x),
            math::block_to_section_coord(pos.z),
        )
    }

    /// Pack the chunk containing a block position into the reference long form.
    pub fn pack_block_pos(pos: BlockPos) -> u64 {
        Self::containing(pos).pack()
    }

    /// Check the chunk against Java's world-coordinate validity bound.
    pub fn is_valid(self) -> bool {
        Self::is_valid_coordinates(self.x, self.z)
    }

    /// Check signed chunk coordinates using Java's `Mth.absMax` semantics.
    pub fn is_valid_coordinates(x: i32, z: i32) -> bool {
        math::abs_max(x, z) <= MAX_CHUNK_COORDINATE_VALUE
    }

    /// Return the Java chessboard distance to another chunk.
    pub fn get_chessboard_distance(self, other: Self) -> i32 {
        self.get_chessboard_distance_coordinates(other.x, other.z)
    }

    /// Return the Java chessboard distance to signed chunk coordinates.
    pub fn get_chessboard_distance_coordinates(self, x: i32, z: i32) -> i32 {
        math::chessboard_distance(x, z, self.x, self.z)
    }

    /// Return the Java squared distance to another chunk with i32 wrapping.
    pub fn distance_squared(self, other: Self) -> i32 {
        self.distance_squared_coordinates(other.x, other.z)
    }

    /// Return the Java squared distance to a packed chunk position.
    pub fn distance_squared_from_packed(self, packed: u64) -> i32 {
        let other = Self::unpack(packed);
        self.distance_squared(other)
    }

    fn distance_squared_coordinates(self, x: i32, z: i32) -> i32 {
        let delta_x = x.wrapping_sub(self.x);
        let delta_z = z.wrapping_sub(self.z);
        delta_x
            .wrapping_mul(delta_x)
            .wrapping_add(delta_z.wrapping_mul(delta_z))
    }

    /// Enumerate a centered inclusive range with Java i32 endpoint arithmetic.
    pub fn range_closed(center: Self, radius: i32) -> Vec<Self> {
        Self::range_closed_from_to(
            Self::new(center.x.wrapping_sub(radius), center.z.wrapping_sub(radius)),
            Self::new(center.x.wrapping_add(radius), center.z.wrapping_add(radius)),
        )
    }

    /// Enumerate an inclusive range in Java's x-fastest, then-z order.
    pub fn range_closed_from_to(from: Self, to: Self) -> Vec<Self> {
        let x_diff = if from.x < to.x { 1 } else { -1 };
        let z_diff = if from.z < to.z { 1 } else { -1 };
        let mut positions = Vec::new();
        let mut position: Option<ChunkPos> = None;
        loop {
            let next = match position {
                None => from,
                Some(current) => {
                    if current.x == to.x {
                        if current.z == to.z {
                            break;
                        }
                        Self::new(from.x, current.z.wrapping_add(z_diff))
                    } else {
                        Self::new(current.x.wrapping_add(x_diff), current.z)
                    }
                }
            };
            positions.push(next);
            position = Some(next);
        }
        positions
    }

    /// Compute the Java ChunkPos hash from two signed coordinates.
    pub fn hash(x: i32, z: i32) -> i32 {
        let x_transform = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let z_transform = (z ^ -559_038_737)
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        x_transform ^ z_transform
    }

    /// Return the Java record hashCode result.
    pub fn hash_code_java(self) -> i32 {
        Self::hash(self.x, self.z)
    }

    /// Format the ChunkPos like Java's record toString implementation.
    pub fn to_string_java(self) -> String {
        format!("[{}, {}]", self.x, self.z)
    }
}

/// Read 16 bytes as UUID. A short second half leaves the first half consumed.
pub fn read_uuid(input: &mut &[u8]) -> Result<Uuid, ReadError> {
    if input.len() < 8 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(8);
    let most = u64::from_be_bytes(head.try_into().expect("checked length"));
    *input = rest;
    if input.len() < 8 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(8);
    let least = u64::from_be_bytes(head.try_into().expect("checked length"));
    *input = rest;
    Ok(Uuid { most, least })
}

/// Append 16 bytes for UUID.
pub fn write_uuid(value: Uuid, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.most.to_be_bytes());
    output.extend_from_slice(&value.least.to_be_bytes());
}

/// Read 8 bytes as a packed position without consuming on failure.
pub fn read_blockpos(input: &mut &[u8]) -> Result<BlockPos, ReadError> {
    if input.len() < 8 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(8);
    let packed = u64::from_be_bytes(head.try_into().expect("checked length"));
    *input = rest;
    Ok(BlockPos::unpack(packed))
}

/// Append 8 bytes for a packed position.
pub fn write_blockpos(value: BlockPos, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.pack().to_be_bytes());
}

/// Read 8 bytes as a packed chunk position without consuming on failure.
pub fn read_chunkpos(input: &mut &[u8]) -> Result<ChunkPos, ReadError> {
    if input.len() < 8 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(8);
    let packed = u64::from_be_bytes(head.try_into().expect("checked length"));
    *input = rest;
    Ok(ChunkPos::unpack(packed))
}

/// Append 8 bytes for a packed chunk position.
pub fn write_chunkpos(value: ChunkPos, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.pack().to_be_bytes());
}

/// Bit set stored as trimmed little-endian words, like the reference.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from long words, dropping trailing zero words.
    pub fn from_longs(longs: &[i64]) -> Self {
        let mut words: Vec<u64> = longs.iter().map(|value| *value as u64).collect();
        while words.last() == Some(&0) {
            words.pop();
        }
        Self { words }
    }

    /// Build from ascending bit indices.
    pub fn from_bits(bits: &[u64]) -> Self {
        let mut words = Vec::new();
        for bit in bits {
            let word = (*bit / 64) as usize;
            if words.len() <= word {
                words.resize(word + 1, 0);
            }
            words[word] |= 1 << (bit % 64);
        }
        Self { words }
    }

    /// Words without trailing zeros, matching `toLongArray`.
    pub fn to_longs(&self) -> Vec<i64> {
        self.words.iter().map(|word| *word as i64).collect()
    }

    /// Ascending set bit indices.
    pub fn to_bits(&self) -> Vec<u64> {
        let mut bits = Vec::new();
        for (word_index, word) in self.words.iter().enumerate() {
            let mut remaining = *word;
            while remaining != 0 {
                let bit = remaining.trailing_zeros() as u64;
                bits.push(word_index as u64 * 64 + bit);
                remaining &= remaining - 1;
            }
        }
        bits
    }
}

/// Read a VarInt length followed by big-endian longs.
pub fn read_long_array(input: &mut &[u8]) -> Result<Vec<i64>, ReadError> {
    let length = match wire::read_varint(input) {
        Ok(value) => value,
        Err(wire::ReadError::BufferUnderflow) => return Err(ReadError::BufferUnderflow),
        Err(wire::ReadError::TooBig) => return Err(ReadError::TooBig),
    };
    if length < 0 {
        return Err(ReadError::DecoderError);
    }
    let length = length as usize;
    if length > input.len() / 8 {
        return Err(ReadError::DecoderError);
    }
    let mut values = Vec::with_capacity(length);
    for _ in 0..length {
        let (head, rest) = input.split_at(8);
        values.push(i64::from_be_bytes(head.try_into().expect("checked length")));
        *input = rest;
    }
    Ok(values)
}

/// Append a VarInt length followed by big-endian longs.
pub fn write_long_array(values: &[i64], output: &mut Vec<u8>) {
    wire::write_varint(values.len() as i32, output);
    for value in values {
        output.extend_from_slice(&value.to_be_bytes());
    }
}

/// Read a BitSet through its long array framing.
pub fn read_bitset(input: &mut &[u8]) -> Result<BitSet, ReadError> {
    read_long_array(input).map(|longs| BitSet::from_longs(&longs))
}

/// Append a BitSet through its trimmed long array framing.
pub fn write_bitset(value: &BitSet, output: &mut Vec<u8>) {
    write_long_array(&value.to_longs(), output);
}

/// Failures for fixed-size writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}

impl BitSet {
    /// Build from little-endian bytes without masking excess bits.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut words = vec![0; bytes.len().div_ceil(8)];
        for (index, byte) in bytes.iter().enumerate() {
            words[index / 8] |= u64::from(*byte) << ((index % 8) * 8);
        }
        while words.last() == Some(&0) {
            words.pop();
        }
        Self { words }
    }

    /// Test a single bit.
    pub fn contains(&self, bit: u64) -> bool {
        let word = (bit / 64) as usize;
        self.words
            .get(word)
            .is_some_and(|value| value & (1 << (bit % 64)) != 0)
    }

    /// Highest set bit plus one, or zero when empty.
    pub fn len(&self) -> u64 {
        self.to_bits().last().map_or(0, |bit| bit + 1)
    }

    /// Whether no bits are set.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

/// Read `ceil(size/8)` bytes as a fixed BitSet without consuming on failure.
pub fn read_fixed_bitset(input: &mut &[u8], size: usize) -> Result<BitSet, ReadError> {
    let bytes = size.div_ceil(8);
    if input.len() < bytes {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(bytes);
    let value = BitSet::from_bytes(head);
    *input = rest;
    Ok(value)
}

/// Append `ceil(size/8)` bytes, failing when a set bit meets or exceeds `size`.
pub fn write_fixed_bitset(
    value: &BitSet,
    size: usize,
    output: &mut Vec<u8>,
) -> Result<(), WriteError> {
    if value.len() > size as u64 {
        return Err(WriteError::EncoderError);
    }
    let bytes = size.div_ceil(8);
    for index in 0..bytes {
        let mut byte = 0;
        for bit in 0..8 {
            if value.contains(index as u64 * 8 + bit) {
                byte |= 1 << bit;
            }
        }
        output.push(byte);
    }
    Ok(())
}

/// Integer section coordinates with reference packing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub const PACKED_SECTION_HORIZONTAL_LENGTH: u32 = 22;
pub const PACKED_SECTION_Y_LENGTH: u32 = 20;
const PACKED_SECTION_HORIZONTAL_MASK: u64 = (1 << PACKED_SECTION_HORIZONTAL_LENGTH) - 1;
const PACKED_SECTION_Y_MASK: u64 = (1 << PACKED_SECTION_Y_LENGTH) - 1;
const SECTION_X_OFFSET: u32 = PACKED_SECTION_Y_LENGTH + PACKED_SECTION_HORIZONTAL_LENGTH;
const SECTION_Z_OFFSET: u32 = PACKED_SECTION_Y_LENGTH;

impl SectionPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Convert a block position to the section containing it.
    pub fn from_block_pos(pos: BlockPos) -> Self {
        Self::new(
            math::block_to_section_coord(pos.x),
            math::block_to_section_coord(pos.y),
            math::block_to_section_coord(pos.z),
        )
    }

    /// Build a section position from chunk coordinates and a vertical section.
    pub const fn from_chunk_pos(pos: ChunkPos, section_y: i32) -> Self {
        Self::new(pos.x, section_y, pos.z)
    }

    /// Convert a double position to the section containing it.
    pub fn from_position(pos: Vec3) -> Self {
        Self::new(
            Self::block_to_section_coord_f64(pos.x),
            Self::block_to_section_coord_f64(pos.y),
            Self::block_to_section_coord_f64(pos.z),
        )
    }

    /// Pack a block position's section-relative coordinates into 12 bits.
    pub fn section_relative_pos(pos: BlockPos) -> u16 {
        let x = (pos.x & 15) as u16;
        let y = (pos.y & 15) as u16;
        let z = (pos.z & 15) as u16;
        (x << 8) | (z << 4) | y
    }

    pub fn section_relative_x(relative: u16) -> i32 {
        i32::from((relative >> 8) & 15)
    }

    pub fn section_relative_y(relative: u16) -> i32 {
        i32::from(relative & 15)
    }

    pub fn section_relative_z(relative: u16) -> i32 {
        i32::from((relative >> 4) & 15)
    }

    /// Pack with masking, so out-of-range inputs wrap as the reference does.
    pub fn pack(self) -> u64 {
        ((u64::from(self.x as u32) & PACKED_SECTION_HORIZONTAL_MASK) << SECTION_X_OFFSET)
            | ((u64::from(self.z as u32) & PACKED_SECTION_HORIZONTAL_MASK) << SECTION_Z_OFFSET)
            | (u64::from(self.y as u32) & PACKED_SECTION_Y_MASK)
    }

    /// Pack the section containing a block position.
    pub fn as_long_from_block_pos(pos: BlockPos) -> u64 {
        Self::as_long_coordinates(
            math::block_to_section_coord(pos.x),
            math::block_to_section_coord(pos.y),
            math::block_to_section_coord(pos.z),
        )
    }

    /// Pack section coordinates with the reference field masks.
    pub fn as_long_coordinates(x: i32, y: i32, z: i32) -> u64 {
        Self::new(x, y, z).pack()
    }

    /// Pack this section position through the coordinate overload's path.
    pub fn as_long(self) -> u64 {
        Self::as_long_coordinates(self.x, self.y, self.z)
    }

    /// Convert a double block position to the section coordinate containing it.
    pub fn pos_to_section_coord(pos: f64) -> i32 {
        math::block_to_section_coord(math::floor_f64(pos))
    }

    /// Convert a double block coordinate to its section coordinate.
    pub fn block_to_section_coord_f64(coord: f64) -> i32 {
        math::block_to_section_coord(math::floor_f64(coord))
    }

    /// Offset a packed section node by a cardinal direction's step.
    pub fn offset_packed(section_node: u64, direction: Direction) -> u64 {
        let (step_x, step_y, step_z) = direction.step();
        Self::offset_packed_by(section_node, step_x, step_y, step_z)
    }

    /// Offset a packed section node with Java i32 wrapping arithmetic.
    pub fn offset_packed_by(section_node: u64, step_x: i32, step_y: i32, step_z: i32) -> u64 {
        let section = Self::unpack(section_node);
        Self::as_long_coordinates(
            section.x.wrapping_add(step_x),
            section.y.wrapping_add(step_y),
            section.z.wrapping_add(step_z),
        )
    }

    /// Offset section coordinates, preserving the zero-offset fast path.
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

    /// Enumerate the 16-by-16-by-16 block positions contained by this section.
    pub fn blocks_inside(self) -> Vec<BlockPos> {
        let min_x = self.min_block_x();
        let min_y = self.min_block_y();
        let min_z = self.min_block_z();
        let max_x = self.max_block_x();
        let max_y = self.max_block_y();
        let max_z = self.max_block_z();
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
            positions.push(BlockPos::new(x, y, z));
        }
        positions
    }

    /// Enumerate sections around a block coordinate, including its section.
    pub fn around_and_at_coordinates(block_x: i32, block_y: i32, block_z: i32) -> Vec<u64> {
        let min_section_x = math::block_to_section_coord(block_x.wrapping_sub(1));
        let max_section_x = math::block_to_section_coord(block_x.wrapping_add(1));
        let min_section_y = math::block_to_section_coord(block_y.wrapping_sub(1));
        let max_section_y = math::block_to_section_coord(block_y.wrapping_add(1));
        let min_section_z = math::block_to_section_coord(block_z.wrapping_sub(1));
        let max_section_z = math::block_to_section_coord(block_z.wrapping_add(1));

        if min_section_x == max_section_x
            && min_section_y == max_section_y
            && min_section_z == max_section_z
        {
            return vec![Self::as_long_coordinates(
                min_section_x,
                min_section_y,
                min_section_z,
            )];
        }

        let mut section_nodes = Vec::new();
        for section_x in min_section_x..=max_section_x {
            for section_y in min_section_y..=max_section_y {
                for section_z in min_section_z..=max_section_z {
                    section_nodes.push(Self::as_long_coordinates(section_x, section_y, section_z));
                }
            }
        }
        section_nodes
    }

    /// Enumerate sections around a block position, including its section.
    pub fn around_and_at_block_pos(pos: BlockPos) -> Vec<u64> {
        Self::around_and_at_coordinates(pos.x, pos.y, pos.z)
    }

    /// Enumerate sections around a packed block position, including its section.
    pub fn around_and_at_packed(block_node: u64) -> Vec<u64> {
        Self::around_and_at_block_pos(BlockPos::unpack(block_node))
    }

    /// Enumerate a finite inclusive range in Java's x-fastest, then y, then z order.
    pub fn between_closed(
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

    /// Enumerate sections in the inclusive cube around a center section.
    pub fn cube(center: Self, radius: i32) -> Vec<Self> {
        Self::between_closed(
            center.x.wrapping_sub(radius),
            center.y.wrapping_sub(radius),
            center.z.wrapping_sub(radius),
            center.x.wrapping_add(radius),
            center.y.wrapping_add(radius),
            center.z.wrapping_add(radius),
        )
    }

    /// Enumerate sections around a chunk for an inclusive vertical section range.
    pub fn around_chunk(
        center: ChunkPos,
        radius: i32,
        min_section: i32,
        max_section: i32,
    ) -> Vec<Self> {
        Self::between_closed(
            center.x.wrapping_sub(radius),
            min_section,
            center.z.wrapping_sub(radius),
            center.x.wrapping_add(radius),
            max_section,
            center.z.wrapping_add(radius),
        )
    }

    /// Read the signed 22-bit x coordinate from a packed SectionPos.
    pub const fn get_x(packed: u64) -> i32 {
        ((packed as i64) >> SECTION_X_OFFSET) as i32
    }

    /// Read the signed 20-bit y coordinate from a packed SectionPos.
    pub const fn get_y(packed: u64) -> i32 {
        ((packed << (64 - PACKED_SECTION_Y_LENGTH)) as i64 >> (64 - PACKED_SECTION_Y_LENGTH)) as i32
    }

    /// Read the signed 22-bit z coordinate from a packed SectionPos.
    pub const fn get_z(packed: u64) -> i32 {
        ((packed << (64 - SECTION_Z_OFFSET - PACKED_SECTION_HORIZONTAL_LENGTH)) as i64
            >> (64 - PACKED_SECTION_HORIZONTAL_LENGTH)) as i32
    }

    /// Return the minimum block coordinate on the x axis.
    pub fn min_block_x(self) -> i32 {
        math::section_to_block_coord(self.x)
    }

    /// Return the minimum block coordinate on the y axis.
    pub fn min_block_y(self) -> i32 {
        math::section_to_block_coord(self.y)
    }

    /// Return the minimum block coordinate on the z axis.
    pub fn min_block_z(self) -> i32 {
        math::section_to_block_coord(self.z)
    }

    /// Return the maximum block coordinate on the x axis.
    pub fn max_block_x(self) -> i32 {
        self.min_block_x().wrapping_add(15)
    }

    /// Return the maximum block coordinate on the y axis.
    pub fn max_block_y(self) -> i32 {
        self.min_block_y().wrapping_add(15)
    }

    /// Return the maximum block coordinate on the z axis.
    pub fn max_block_z(self) -> i32 {
        self.min_block_z().wrapping_add(15)
    }

    /// Add a section-relative short's x offset to the section's block origin.
    pub fn relative_to_block_x(self, relative: u16) -> i32 {
        self.min_block_x()
            .wrapping_add(Self::section_relative_x(relative))
    }

    /// Add a section-relative short's y offset to the section's block origin.
    pub fn relative_to_block_y(self, relative: u16) -> i32 {
        self.min_block_y()
            .wrapping_add(Self::section_relative_y(relative))
    }

    /// Add a section-relative short's z offset to the section's block origin.
    pub fn relative_to_block_z(self, relative: u16) -> i32 {
        self.min_block_z()
            .wrapping_add(Self::section_relative_z(relative))
    }

    /// Convert a section-relative short into a block position in this section.
    pub fn relative_to_block_pos(self, relative: u16) -> BlockPos {
        BlockPos::new(
            self.relative_to_block_x(relative),
            self.relative_to_block_y(relative),
            self.relative_to_block_z(relative),
        )
    }

    /// Convert a packed BlockPos node into a packed SectionPos node.
    pub fn block_to_section(block_node: u64) -> u64 {
        let block = BlockPos::unpack(block_node);
        Self::new(
            math::block_to_section_coord(block.x),
            math::block_to_section_coord(block.y),
            math::block_to_section_coord(block.z),
        )
        .pack()
    }

    /// Clear the packed section node's 20-bit y coordinate.
    pub fn get_zero_node(section_node: u64) -> u64 {
        section_node & !PACKED_SECTION_Y_MASK
    }

    /// Pack x/z section coordinates at y=0 and clear the y field.
    pub fn get_zero_node_from_coordinates(x: i32, z: i32) -> u64 {
        Self::get_zero_node(Self::new(x, 0, z).pack())
    }

    /// Convert a packed SectionPos node into a packed ChunkPos node.
    pub fn section_to_chunk(section_node: u64) -> u64 {
        ChunkPos::new(Self::get_x(section_node), Self::get_z(section_node)).pack()
    }

    /// Return the minimum block corner of this section.
    pub fn origin(self) -> BlockPos {
        BlockPos::new(self.min_block_x(), self.min_block_y(), self.min_block_z())
    }

    /// Return the block position at the section center using Java's offset path.
    pub fn center(self) -> BlockPos {
        self.origin().offset(8, 8, 8)
    }

    /// Return the ChunkPos sharing this section's x and z coordinates.
    pub fn chunk(self) -> ChunkPos {
        ChunkPos::new(self.x, self.z)
    }

    /// Unpack with arithmetic shifts, sign-extending each field.
    pub fn unpack(packed: u64) -> Self {
        Self {
            x: Self::get_x(packed),
            y: Self::get_y(packed),
            z: Self::get_z(packed),
        }
    }
}

/// Read 8 bytes as a packed section position without consuming on failure.
pub fn read_sectionpos(input: &mut &[u8]) -> Result<SectionPos, ReadError> {
    if input.len() < 8 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(8);
    let packed = u64::from_be_bytes(head.try_into().expect("checked length"));
    *input = rest;
    Ok(SectionPos::unpack(packed))
}

/// Append 8 bytes for a packed section position.
pub fn write_sectionpos(value: SectionPos, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.pack().to_be_bytes());
}
