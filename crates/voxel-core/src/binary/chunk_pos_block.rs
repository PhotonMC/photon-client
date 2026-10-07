use super::*;

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

    pub(super) fn distance_squared_coordinates(self, x: i32, z: i32) -> i32 {
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
