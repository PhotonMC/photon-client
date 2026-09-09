//! Binary codecs for the pinned reference: Vec3, UUID, BlockPos, long arrays and BitSets.
//!
//! UUID and positions travel as big-endian longs with no length prefix.
//! Long arrays prefix a VarInt length; BitSets reuse that framing through
//! trimmed little-endian words. Each `readLong` is atomic, so a truncated
//! UUID can leave the first half consumed while a truncated position
//! consumes nothing.

use crate::math::Vec3;
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

impl BlockPos {
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
        let x = ((packed << (64 - X_OFFSET - PACKED_HORIZONTAL_LENGTH)) as i64
            >> (64 - PACKED_HORIZONTAL_LENGTH)) as i32;
        let y = ((packed << (64 - PACKED_Y_LENGTH)) as i64 >> (64 - PACKED_Y_LENGTH)) as i32;
        let z = ((packed << (64 - Z_OFFSET - PACKED_HORIZONTAL_LENGTH)) as i64
            >> (64 - PACKED_HORIZONTAL_LENGTH)) as i32;
        Self { x, y, z }
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
            x: packed as u32 as i32,
            z: (packed >> 32) as u32 as i32,
        }
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

    /// Unpack with arithmetic shifts, sign-extending each field.
    pub fn unpack(packed: u64) -> Self {
        let x = ((packed << (64 - SECTION_X_OFFSET - PACKED_SECTION_HORIZONTAL_LENGTH)) as i64
            >> (64 - PACKED_SECTION_HORIZONTAL_LENGTH)) as i32;
        let y = ((packed << (64 - PACKED_SECTION_Y_LENGTH)) as i64
            >> (64 - PACKED_SECTION_Y_LENGTH)) as i32;
        let z = ((packed << (64 - SECTION_Z_OFFSET - PACKED_SECTION_HORIZONTAL_LENGTH)) as i64
            >> (64 - PACKED_SECTION_HORIZONTAL_LENGTH)) as i32;
        Self { x, y, z }
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
