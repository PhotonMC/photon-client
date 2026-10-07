//! Binary codecs for the pinned reference: Vec3, UUID, BlockPos, long arrays and BitSets.
//!
//! UUID and positions travel as big-endian longs with no length prefix.
//! Long arrays prefix a VarInt length; BitSets reuse that framing through
//! trimmed little-endian words. Each `readLong` is atomic, so a truncated
//! UUID can leave the first half consumed while a truncated position
//! consumes nothing.

use crate::aabb::{Aabb, BoundingBox};
use crate::axis_cycle::AxisCycle;
use crate::direction::{Axis, Direction};
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

impl From<wire::ReadError> for ReadError {
    fn from(error: wire::ReadError) -> Self {
        match error {
            wire::ReadError::BufferUnderflow => Self::BufferUnderflow,
            wire::ReadError::TooBig => Self::TooBig,
        }
    }
}

impl From<crate::utf8::ReadError> for ReadError {
    fn from(error: crate::utf8::ReadError) -> Self {
        match error {
            crate::utf8::ReadError::DecoderError => Self::DecoderError,
            crate::utf8::ReadError::BufferUnderflow => Self::BufferUnderflow,
            crate::utf8::ReadError::TooBig => Self::TooBig,
        }
    }
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

    /// `UUID.toString`: eight, four, four, four and twelve lowercase hex digits.
    pub fn to_string_java(self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.most >> 32,
            (self.most >> 16) & 0xFFFF,
            self.most & 0xFFFF,
            self.least >> 48,
            self.least & 0xFFFF_FFFF_FFFF
        )
    }
}

const LP_MAX_QUANTIZED_VALUE: u64 = 32766;
const LP_ABS_MAX_VALUE: f64 = 1.7179869183e10;
const LP_ABS_MIN_VALUE: f64 = 3.051944088384301e-5;
const MAX_CHUNK_COORDINATE_VALUE: i32 = 2_097_061;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiralAroundError {
    SameAxis,
}

/// Integer block coordinates with reference packing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Mutable BlockPos state used by Java's cursor-based helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Result of processing a node in a bounded BlockPos traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalNodeStatus {
    Accept,
    Skip,
    Stop,
}

pub const PACKED_HORIZONTAL_LENGTH: u32 = 26;
pub const PACKED_Y_LENGTH: u32 = 12;
const PACKED_HORIZONTAL_MASK: u64 = (1 << PACKED_HORIZONTAL_LENGTH) - 1;
const PACKED_Y_MASK: u64 = (1 << PACKED_Y_LENGTH) - 1;
const X_OFFSET: u32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;
const Z_OFFSET: u32 = PACKED_Y_LENGTH;
pub const MAX_HORIZONTAL_COORDINATE: i32 = (1 << PACKED_HORIZONTAL_LENGTH) / 2 - 1;

impl MutableBlockPos {
    /// Create mutable BlockPos state from integer coordinates.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Create mutable BlockPos state using Java floor conversion.
    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self::new(math::floor_f64(x), math::floor_f64(y), math::floor_f64(z))
    }

    /// Return an immutable snapshot of the current mutable coordinates.
    pub const fn immutable(self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    /// Set all three coordinates.
    pub const fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    /// Set all three coordinates after Java floor conversion.
    pub fn set_f64(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.set(math::floor_f64(x), math::floor_f64(y), math::floor_f64(z))
    }

    /// Set all three coordinates from an immutable BlockPos.
    pub const fn set_block_pos(&mut self, value: BlockPos) -> &mut Self {
        self.set(value.x, value.y, value.z)
    }

    /// Set all three coordinates from a packed BlockPos long.
    pub const fn set_packed(&mut self, value: u64) -> &mut Self {
        self.set_block_pos(BlockPos::of(value))
    }

    /// Set coordinates after applying a Java AxisCycle to each output axis.
    pub fn set_axis_cycle(&mut self, cycle: AxisCycle, x: i32, y: i32, z: i32) -> &mut Self {
        self.set(
            cycle.cycle_i32(x, y, z, Axis::X),
            cycle.cycle_i32(x, y, z, Axis::Y),
            cycle.cycle_i32(x, y, z, Axis::Z),
        )
    }

    /// Set to a position plus one cardinal direction step.
    pub fn set_with_offset_direction(
        &mut self,
        position: BlockPos,
        direction: Direction,
    ) -> &mut Self {
        let (x, y, z) = direction.step();
        self.set(
            position.x.wrapping_add(x),
            position.y.wrapping_add(y),
            position.z.wrapping_add(z),
        )
    }

    /// Set to a position plus explicit integer offsets.
    pub const fn set_with_offset_components(
        &mut self,
        position: BlockPos,
        x: i32,
        y: i32,
        z: i32,
    ) -> &mut Self {
        self.set(
            position.x.wrapping_add(x),
            position.y.wrapping_add(y),
            position.z.wrapping_add(z),
        )
    }

    /// Set to a position plus an immutable BlockPos offset.
    pub const fn set_with_offset_block_pos(
        &mut self,
        position: BlockPos,
        offset: BlockPos,
    ) -> &mut Self {
        self.set_with_offset_components(position, offset.x, offset.y, offset.z)
    }

    /// Move by a cardinal direction and signed number of steps.
    pub fn move_direction(&mut self, direction: Direction, steps: i32) -> &mut Self {
        let (x, y, z) = direction.step();
        self.move_components(
            x.wrapping_mul(steps),
            y.wrapping_mul(steps),
            z.wrapping_mul(steps),
        )
    }

    /// Move by explicit integer offsets.
    pub const fn move_components(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = self.x.wrapping_add(x);
        self.y = self.y.wrapping_add(y);
        self.z = self.z.wrapping_add(z);
        self
    }

    /// Move by an immutable BlockPos offset.
    pub const fn move_block_pos(&mut self, value: BlockPos) -> &mut Self {
        self.move_components(value.x, value.y, value.z)
    }

    /// Clamp one selected axis and preserve the other two.
    pub fn clamp(&mut self, axis: Axis, minimum: i32, maximum: i32) -> &mut Self {
        match axis {
            Axis::X => self.x = math::clamp_i32(self.x, minimum, maximum),
            Axis::Y => self.y = math::clamp_i32(self.y, minimum, maximum),
            Axis::Z => self.z = math::clamp_i32(self.z, minimum, maximum),
        }
        self
    }

    /// Set the x coordinate.
    pub const fn set_x(&mut self, value: i32) -> &mut Self {
        self.x = value;
        self
    }

    /// Set the y coordinate.
    pub const fn set_y(&mut self, value: i32) -> &mut Self {
        self.y = value;
        self
    }

    /// Set the z coordinate.
    pub const fn set_z(&mut self, value: i32) -> &mut Self {
        self.z = value;
        self
    }

    /// Return an immutable offset result without changing this mutable state.
    pub fn offset(self, x: i32, y: i32, z: i32) -> BlockPos {
        self.immutable().offset(x, y, z)
    }

    /// Return an immutable scaled result without changing this mutable state.
    pub fn multiply(self, scale: i32) -> BlockPos {
        self.immutable().multiply(scale)
    }

    /// Return an immutable cardinal-relative result without changing this mutable state.
    pub fn relative_direction(self, direction: Direction, steps: i32) -> BlockPos {
        self.immutable().relative_direction(direction, steps)
    }

    /// Return an immutable axis-relative result without changing this mutable state.
    pub fn relative_axis(self, axis: Axis, steps: i32) -> BlockPos {
        self.immutable().relative_axis(axis, steps)
    }

    /// Return an immutable rotated result without changing this mutable state.
    pub fn rotate(self, rotation: BlockRotation) -> BlockPos {
        self.immutable().rotate(rotation)
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

#[cfg(test)]
mod uuid_tests {
    use super::Uuid;

    #[test]
    fn a_uuid_reads_as_java_writes_it() {
        assert_eq!(
            Uuid::new(0x5e8c_9a1b_0000_4000, 0x8000_0000_0000_0001).to_string_java(),
            "5e8c9a1b-0000-4000-8000-000000000001"
        );
        assert_eq!(
            Uuid::new(0, 7).to_string_java(),
            "00000000-0000-0000-0000-000000000007"
        );
    }
}

mod block_pos_between;
mod block_pos_between_2;

mod vec3;
pub use vec3::*;

mod section_pos_block;

mod chunk_pos_block;
