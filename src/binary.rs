//! Binary codecs for the pinned reference: UUID, BlockPos, long arrays and BitSets.
//!
//! UUID and positions travel as big-endian longs with no length prefix.
//! Long arrays prefix a VarInt length; BitSets reuse that framing through
//! trimmed little-endian words. Each `readLong` is atomic, so a truncated
//! UUID can leave the first half consumed while a truncated position
//! consumes nothing.

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
