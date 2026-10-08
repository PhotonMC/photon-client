//! Minecraft's signed variable-width integers (not ZigZag encoding).
//!
//! Reads advance the input slice even on error. Framing and retry/rollback policy
//! belong to the caller; this module matches the reference decoder's consumption.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    BufferUnderflow,
    TooBig,
}

/// The first `N` bytes of the input, taken off it; `None`, leaving the
/// input as it was, when fewer remain.
pub fn take<const N: usize>(input: &mut &[u8]) -> Option<[u8; N]> {
    let (head, rest) = input.split_first_chunk::<N>()?;
    *input = rest;
    Some(*head)
}

/// Decode an i32, accepting the reference's noncanonical representations.
pub fn read_varint(input: &mut &[u8]) -> Result<i32, ReadError> {
    read_unsigned(input, 5).map(|value| value as i32)
}

/// Decode an i64, accepting the reference's noncanonical representations.
pub fn read_varlong(input: &mut &[u8]) -> Result<i64, ReadError> {
    read_unsigned(input, 10).map(|value| value as i64)
}

fn read_unsigned(input: &mut &[u8], limit: u32) -> Result<u64, ReadError> {
    let mut value = 0_u64;
    for group in 0..=limit {
        let (&byte, remaining) = input.split_first().ok_or(ReadError::BufferUnderflow)?;
        *input = remaining;
        // A byte beyond the limit is consumed before reporting TooBig. If no
        // such byte exists, the reference reports underflow instead.
        if group == limit {
            return Err(ReadError::TooBig);
        }
        value |= u64::from(byte & 0x7f) << (group * 7);
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    unreachable!("the extra group always returns TooBig")
}

/// Append a VarInt; negative i32 values occupy five bytes.
pub fn write_varint(value: i32, output: &mut Vec<u8>) {
    write_unsigned(u64::from(value as u32), output);
}

/// Append a VarLong; negative i64 values occupy ten bytes.
pub fn write_varlong(value: i64, output: &mut Vec<u8>) {
    write_unsigned(value as u64, output);
}

fn write_unsigned(mut value: u64, output: &mut Vec<u8>) {
    loop {
        let payload = (value & 0x7f) as u8;
        value >>= 7;
        output.push(payload | if value == 0 { 0 } else { 0x80 });
        if value == 0 {
            return;
        }
    }
}

pub fn varint_size(value: i32) -> usize {
    unsigned_size(u64::from(value as u32))
}

pub fn varlong_size(value: i64) -> usize {
    unsigned_size(value as u64)
}

fn unsigned_size(value: u64) -> usize {
    (64 - value.leading_zeros()).max(1).div_ceil(7) as usize
}
