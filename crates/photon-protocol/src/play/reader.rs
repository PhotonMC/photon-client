use super::*;

pub fn read_long_array(input: &mut &[u8], name: &'static str) -> Result<Vec<i64>, ConnectionError> {
    photon_core::binary::read_long_array(input).map_err(|_| ConnectionError::Codec(name))
}

pub fn read_bitset(input: &mut &[u8], name: &'static str) -> Result<BitSet, ConnectionError> {
    photon_core::binary::read_bitset(input).map_err(|_| ConnectionError::Codec(name))
}

pub fn read_length_prefixed_bytes<'a>(
    input: &mut &'a [u8],
    max_length: usize,
    name: &'static str,
) -> Result<&'a [u8], ConnectionError> {
    let length = read_count(input, name)?;
    if length > max_length || input.len() < length {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(length);
    *input = rest;
    Ok(head)
}

pub(super) fn read_identifier_string(
    input: &mut &[u8],
    name: &'static str,
) -> Result<String, ConnectionError> {
    photon_core::identifier::read_identifier(input)
        .map(|identifier| identifier.as_string())
        .map_err(|_| ConnectionError::Codec(name))
}

pub(super) fn skip_bytes(
    input: &mut &[u8],
    count: usize,
    name: &'static str,
) -> Result<(), ConnectionError> {
    if input.len() < count {
        return Err(ConnectionError::Codec(name));
    }
    *input = &input[count..];
    Ok(())
}

pub(super) fn read_lp_vec3(
    input: &mut &[u8],
    name: &'static str,
) -> Result<(f64, f64, f64), ConnectionError> {
    let movement =
        photon_core::binary::read_lp_vec3(input).map_err(|_| ConnectionError::Codec(name))?;
    Ok((movement.x, movement.y, movement.z))
}

pub(super) fn decode_packed_degrees(value: u8) -> f32 {
    i8::from_ne_bytes([value]) as f32 * 360.0 / 256.0
}

pub(super) fn read_nbt_bytes(
    input: &mut &[u8],
    name: &'static str,
) -> Result<Option<Vec<u8>>, ConnectionError> {
    let before = *input;
    let value = photon_core::nbt::read_nbt(input).map_err(|_| ConnectionError::Codec(name))?;
    let consumed = before.len() - input.len();
    Ok(value.map(|_| before[..consumed].to_vec()))
}

pub fn read_varint(input: &mut &[u8], name: &'static str) -> Result<i32, ConnectionError> {
    wire::read_varint(input).map_err(|_| ConnectionError::Codec(name))
}

pub fn read_count(input: &mut &[u8], name: &'static str) -> Result<usize, ConnectionError> {
    let count = read_varint(input, name)?;
    usize::try_from(count).map_err(|_| ConnectionError::Codec(name))
}

pub fn read_byte(input: &mut &[u8], name: &'static str) -> Result<u8, ConnectionError> {
    let Some((&byte, rest)) = input.split_first() else {
        return Err(ConnectionError::Codec(name));
    };
    *input = rest;
    Ok(byte)
}

pub fn read_bool(input: &mut &[u8], name: &'static str) -> Result<bool, ConnectionError> {
    Ok(read_byte(input, name)? != 0)
}

pub fn read_i64(input: &mut &[u8], name: &'static str) -> Result<i64, ConnectionError> {
    wire::take(input)
        .map(i64::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub fn read_i16(input: &mut &[u8], name: &'static str) -> Result<i16, ConnectionError> {
    wire::take(input)
        .map(i16::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub fn read_i32(input: &mut &[u8], name: &'static str) -> Result<i32, ConnectionError> {
    wire::take(input)
        .map(i32::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub fn read_f32(input: &mut &[u8], name: &'static str) -> Result<f32, ConnectionError> {
    wire::take(input)
        .map(f32::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub fn read_f64(input: &mut &[u8], name: &'static str) -> Result<f64, ConnectionError> {
    wire::take(input)
        .map(f64::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

/// A big-endian long whose absence is an unexpected end of the stream, as the packets that
/// begin with a section position or a game time report it.
pub fn read_section_position(input: &mut &[u8]) -> Result<i64, ConnectionError> {
    wire::take(input).map(i64::from_be_bytes).ok_or_else(|| {
        ConnectionError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "section position",
        ))
    })
}
