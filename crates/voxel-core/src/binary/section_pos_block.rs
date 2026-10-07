use super::*;

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
