use super::*;

pub(super) fn read_u64(input: &mut &[u8]) -> Result<u64, ReadError> {
    wire::take(input)
        .map(u64::from_be_bytes)
        .ok_or(ReadError::BufferUnderflow)
}

pub(super) fn read_double(input: &mut &[u8]) -> Result<f64, ReadError> {
    read_u64(input).map(f64::from_bits)
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

pub(super) fn read_byte(input: &mut &[u8]) -> Result<u8, ReadError> {
    let (&head, rest) = input.split_first().ok_or(ReadError::BufferUnderflow)?;
    *input = rest;
    Ok(head)
}

pub(super) fn read_u32(input: &mut &[u8]) -> Result<u32, ReadError> {
    wire::take(input)
        .map(u32::from_be_bytes)
        .ok_or(ReadError::BufferUnderflow)
}

pub(super) fn lp_unpack(value: u64) -> f64 {
    let quantized = (value & 0x7fff).min(LP_MAX_QUANTIZED_VALUE);
    quantized as f64 * 2.0 / LP_MAX_QUANTIZED_VALUE as f64 - 1.0
}

pub(super) fn lp_sanitize(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else if value < -LP_ABS_MAX_VALUE {
        -LP_ABS_MAX_VALUE
    } else {
        value.min(LP_ABS_MAX_VALUE)
    }
}

pub(super) fn lp_pack(value: f64) -> u64 {
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
        let continuation = wire::read_varint(input)?;
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

pub(super) fn java_abs_i32(value: i32) -> i32 {
    if value < 0 {
        value.wrapping_neg()
    } else {
        value
    }
}

/// Read 8 bytes as a packed position without consuming on failure.
pub fn read_blockpos(input: &mut &[u8]) -> Result<BlockPos, ReadError> {
    read_u64(input).map(BlockPos::unpack)
}

/// Append 8 bytes for a packed position.
pub fn write_blockpos(value: BlockPos, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.pack().to_be_bytes());
}

/// Read 8 bytes as a packed chunk position without consuming on failure.
pub fn read_chunkpos(input: &mut &[u8]) -> Result<ChunkPos, ReadError> {
    read_u64(input).map(ChunkPos::unpack)
}

/// Append 8 bytes for a packed chunk position.
pub fn write_chunkpos(value: ChunkPos, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.pack().to_be_bytes());
}

/// Read 8 bytes as a packed section position without consuming on failure.
pub fn read_sectionpos(input: &mut &[u8]) -> Result<SectionPos, ReadError> {
    read_u64(input).map(SectionPos::unpack)
}

/// Append 8 bytes for a packed section position.
pub fn write_sectionpos(value: SectionPos, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.pack().to_be_bytes());
}

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

pub(super) const LP_MAX_QUANTIZED_VALUE: u64 = 32766;

pub(super) const LP_ABS_MAX_VALUE: f64 = 1.7179869183e10;

pub(super) const LP_ABS_MIN_VALUE: f64 = 3.051944088384301e-5;

pub(super) const MAX_CHUNK_COORDINATE_VALUE: i32 = 2_097_061;

/// Failures for fixed-size writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}
