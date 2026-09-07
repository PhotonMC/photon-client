//! Fixed-size binary codecs for the pinned reference: UUID and BlockPos.
//!
//! Both travel as big-endian longs with no length prefix. Every 16-byte
//! UUID and every 8-byte position is valid; the only read failure is a
//! short buffer. Each `readLong` is atomic, so a truncated UUID can leave
//! the first half consumed while a truncated position consumes nothing.

/// Short-buffer failure for fixed-size reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    BufferUnderflow,
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
