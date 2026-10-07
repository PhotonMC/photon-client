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

/// Read 16 bytes as UUID. A short second half leaves the first half consumed.
pub fn read_uuid(input: &mut &[u8]) -> Result<Uuid, ReadError> {
    let most = read_u64(input)?;
    let least = read_u64(input)?;
    Ok(Uuid { most, least })
}

/// Append 16 bytes for UUID.
pub fn write_uuid(value: Uuid, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.most.to_be_bytes());
    output.extend_from_slice(&value.least.to_be_bytes());
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

/// Read a VarInt length followed by big-endian longs.
pub fn read_long_array(input: &mut &[u8]) -> Result<Vec<i64>, ReadError> {
    let length = wire::read_varint(input)?;
    if length < 0 {
        return Err(ReadError::DecoderError);
    }
    let length = length as usize;
    if length > input.len() / 8 {
        return Err(ReadError::DecoderError);
    }
    let mut values = Vec::with_capacity(length);
    for _ in 0..length {
        values.push(read_u64(input)? as i64);
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

/// Read 8 bytes as a packed section position without consuming on failure.
pub fn read_sectionpos(input: &mut &[u8]) -> Result<SectionPos, ReadError> {
    read_u64(input).map(SectionPos::unpack)
}

/// Append 8 bytes for a packed section position.
pub fn write_sectionpos(value: SectionPos, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.pack().to_be_bytes());
}
