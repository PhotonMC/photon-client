//! NBT values for the pinned reference (P03.c).
//!
//! This slice covers null/empty roots, primitive and string fields, nested
//! compounds, homogeneous lists, and byte/int/long arrays. Quotas, depth
//! limits, and SNBT are separate concerns. Strings in the current corpus are
//! ASCII, where standard and modified UTF-8 agree.

/// Failures using the runner's error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    BufferUnderflow,
    DecoderError,
    EncoderError,
    TooBig,
}

/// A named NBT value.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub key: String,
    pub value: FieldValue,
}

/// NBT values supported by this behavior slice.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    Str(String),
    List(Vec<FieldValue>),
    Compound(Vec<Field>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

fn decoder_error(input: &mut &[u8]) -> ReadError {
    let rest: &[u8] = &[];
    *input = rest;
    ReadError::DecoderError
}

fn read_byte(input: &mut &[u8]) -> Result<u8, ReadError> {
    let (&head, rest) = input.split_first().ok_or(ReadError::EncoderError)?;
    *input = rest;
    Ok(head)
}

fn read_required_byte(input: &mut &[u8]) -> Result<u8, ReadError> {
    read_byte(input).map_err(|_| decoder_error(input))
}

fn read_bytes<'a>(input: &mut &'a [u8], count: usize) -> Result<&'a [u8], ReadError> {
    if input.len() < count {
        let rest: &[u8] = &[];
        *input = rest;
        return Err(ReadError::DecoderError);
    }
    let (head, rest) = input.split_at(count);
    *input = rest;
    Ok(head)
}

fn read_u16(input: &mut &[u8]) -> Result<u16, ReadError> {
    let bytes = read_bytes(input, 2)?;
    Ok(u16::from_be_bytes(
        bytes.try_into().expect("checked length"),
    ))
}

fn read_i32(input: &mut &[u8]) -> Result<i32, ReadError> {
    let bytes = read_bytes(input, 4)?;
    Ok(i32::from_be_bytes(
        bytes.try_into().expect("checked length"),
    ))
}

fn read_name(input: &mut &[u8]) -> Result<String, ReadError> {
    let length = read_u16(input)? as usize;
    let bytes = read_bytes(input, length)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| decoder_error(input))
}

fn read_string(input: &mut &[u8]) -> Result<String, ReadError> {
    let length = read_u16(input)? as usize;
    let bytes = read_bytes(input, length)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| decoder_error(input))
}

fn read_payload(input: &mut &[u8], id: u8) -> Result<FieldValue, ReadError> {
    match id {
        1 => Ok(FieldValue::Byte(read_bytes(input, 1)?[0] as i8)),
        2 => {
            let bytes = read_bytes(input, 2)?;
            Ok(FieldValue::Short(i16::from_be_bytes(
                bytes.try_into().expect("checked length"),
            )))
        }
        3 => Ok(FieldValue::Int(read_i32(input)?)),
        4 => {
            let bytes = read_bytes(input, 8)?;
            Ok(FieldValue::Long(i64::from_be_bytes(
                bytes.try_into().expect("checked length"),
            )))
        }
        5 => {
            let bytes = read_bytes(input, 4)?;
            Ok(FieldValue::Float(f32::from_bits(u32::from_be_bytes(
                bytes.try_into().expect("checked length"),
            ))))
        }
        6 => {
            let bytes = read_bytes(input, 8)?;
            Ok(FieldValue::Double(f64::from_bits(u64::from_be_bytes(
                bytes.try_into().expect("checked length"),
            ))))
        }
        7 => {
            let length = read_i32(input)?;
            if length < 0 {
                return Err(decoder_error(input));
            }
            let bytes = read_bytes(input, length as usize)?;
            Ok(FieldValue::ByteArray(
                bytes.iter().map(|byte| *byte as i8).collect(),
            ))
        }
        8 => Ok(FieldValue::Str(read_string(input)?)),
        9 => read_list(input),
        10 => Ok(FieldValue::Compound(read_compound(input)?)),
        11 => read_int_array(input),
        12 => read_long_array(input),
        _ => Err(decoder_error(input)),
    }
}

fn read_list(input: &mut &[u8]) -> Result<FieldValue, ReadError> {
    let element_id = read_required_byte(input)?;
    let count = read_i32(input)?;
    if count < 0 {
        return Err(decoder_error(input));
    }

    let mut values = Vec::new();
    for _ in 0..count {
        let value = read_payload(input, element_id)?;
        values.push(unwrap_list_element(element_id, value));
    }
    Ok(FieldValue::List(values))
}

fn unwrap_list_element(element_id: u8, value: FieldValue) -> FieldValue {
    if element_id == 10
        && let FieldValue::Compound(mut fields) = value
    {
        if fields.len() == 1 && fields[0].key.is_empty() {
            return fields.remove(0).value;
        }
        return FieldValue::Compound(fields);
    }
    value
}

fn read_int_array(input: &mut &[u8]) -> Result<FieldValue, ReadError> {
    let length = read_i32(input)?;
    if length < 0 {
        return Err(decoder_error(input));
    }
    let byte_count = (length as usize)
        .checked_mul(4)
        .ok_or_else(|| decoder_error(input))?;
    let bytes = read_bytes(input, byte_count)?;
    let (chunks, remainder) = bytes.as_chunks::<4>();
    debug_assert!(remainder.is_empty());
    let values = chunks
        .iter()
        .map(|chunk| i32::from_be_bytes(*chunk))
        .collect();
    Ok(FieldValue::IntArray(values))
}

fn read_long_array(input: &mut &[u8]) -> Result<FieldValue, ReadError> {
    let length = read_i32(input)?;
    if length < 0 {
        return Err(decoder_error(input));
    }
    let byte_count = (length as usize)
        .checked_mul(8)
        .ok_or_else(|| decoder_error(input))?;
    let bytes = read_bytes(input, byte_count)?;
    let (chunks, remainder) = bytes.as_chunks::<8>();
    debug_assert!(remainder.is_empty());
    let values = chunks
        .iter()
        .map(|chunk| i64::from_be_bytes(*chunk))
        .collect();
    Ok(FieldValue::LongArray(values))
}

fn read_compound(input: &mut &[u8]) -> Result<Vec<Field>, ReadError> {
    let mut fields: Vec<Field> = Vec::new();
    loop {
        let field_id = read_required_byte(input)?;
        if field_id == 0 {
            fields.sort_by(|a, b| a.key.cmp(&b.key));
            return Ok(fields);
        }
        let key = read_name(input)?;
        let value = read_payload(input, field_id)?;
        fields.push(Field { key, value });
    }
}

/// Read one root tag: `None` for the end marker, otherwise any NBT payload.
pub fn read_any_nbt(input: &mut &[u8]) -> Result<Option<FieldValue>, ReadError> {
    let id = read_byte(input)?;
    if id == 0 {
        return Ok(None);
    }
    read_payload(input, id).map(Some)
}

/// Read one root tag: `None` for the end marker, otherwise a compound.
pub fn read_nbt(input: &mut &[u8]) -> Result<Option<Vec<Field>>, ReadError> {
    let value = read_any_nbt(input)?;
    match value {
        None => Ok(None),
        Some(FieldValue::Compound(fields)) => Ok(Some(fields)),
        _ => Err(ReadError::DecoderError),
    }
}

fn write_name(key: &str, output: &mut Vec<u8>) {
    output.extend_from_slice(&(key.len() as u16).to_be_bytes());
    output.extend_from_slice(key.as_bytes());
}

fn type_id(value: &FieldValue) -> u8 {
    match value {
        FieldValue::Byte(_) => 1,
        FieldValue::Short(_) => 2,
        FieldValue::Int(_) => 3,
        FieldValue::Long(_) => 4,
        FieldValue::Float(_) => 5,
        FieldValue::Double(_) => 6,
        FieldValue::ByteArray(_) => 7,
        FieldValue::Str(_) => 8,
        FieldValue::List(_) => 9,
        FieldValue::Compound(_) => 10,
        FieldValue::IntArray(_) => 11,
        FieldValue::LongArray(_) => 12,
    }
}

fn write_named_field(field: &Field, output: &mut Vec<u8>) {
    output.push(type_id(&field.value));
    write_name(&field.key, output);
    write_payload(&field.value, output);
}

fn write_compound(fields: &[Field], output: &mut Vec<u8>) {
    for field in fields {
        write_named_field(field, output);
    }
    output.push(0);
}

fn list_type_id(values: &[FieldValue]) -> u8 {
    let Some(first) = values.first() else {
        return 0;
    };
    let first_id = type_id(first);
    if values.iter().all(|value| type_id(value) == first_id) {
        first_id
    } else {
        10
    }
}

fn write_list(values: &[FieldValue], output: &mut Vec<u8>) {
    let element_id = list_type_id(values);
    output.push(element_id);
    output.extend_from_slice(&(values.len() as i32).to_be_bytes());
    for value in values {
        if element_id == 10 && type_id(value) != 10 {
            let wrapper = FieldValue::Compound(vec![Field {
                key: String::new(),
                value: value.clone(),
            }]);
            write_payload(&wrapper, output);
        } else {
            write_payload(value, output);
        }
    }
}

fn write_payload(value: &FieldValue, output: &mut Vec<u8>) {
    match value {
        FieldValue::Byte(value) => output.push(*value as u8),
        FieldValue::Short(value) => output.extend_from_slice(&value.to_be_bytes()),
        FieldValue::Int(value) => output.extend_from_slice(&value.to_be_bytes()),
        FieldValue::Long(value) => output.extend_from_slice(&value.to_be_bytes()),
        FieldValue::Float(value) => output.extend_from_slice(&value.to_bits().to_be_bytes()),
        FieldValue::Double(value) => output.extend_from_slice(&value.to_bits().to_be_bytes()),
        FieldValue::ByteArray(values) => {
            output.extend_from_slice(&(values.len() as i32).to_be_bytes());
            output.extend(values.iter().map(|value| *value as u8));
        }
        FieldValue::Str(value) => {
            output.extend_from_slice(&(value.len() as u16).to_be_bytes());
            output.extend_from_slice(value.as_bytes());
        }
        FieldValue::List(values) => write_list(values, output),
        FieldValue::Compound(fields) => write_compound(fields, output),
        FieldValue::IntArray(values) => {
            output.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for value in values {
                output.extend_from_slice(&value.to_be_bytes());
            }
        }
        FieldValue::LongArray(values) => {
            output.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for value in values {
                output.extend_from_slice(&value.to_be_bytes());
            }
        }
    }
}

/// Append one root tag: `None` writes the end marker.
pub fn write_any_nbt(root: Option<&FieldValue>, output: &mut Vec<u8>) {
    let Some(value) = root else {
        output.push(0);
        return;
    };
    output.push(type_id(value));
    write_payload(value, output);
}

/// Append one compound root tag: `None` writes the end marker.
pub fn write_nbt(root: Option<&[Field]>, output: &mut Vec<u8>) {
    let value = root.map(|fields| FieldValue::Compound(fields.to_vec()));
    write_any_nbt(value.as_ref(), output);
}

/// A numeric tag as an `i32` (the way the reference's `getIntOr` reads one).
pub fn numeric_nbt_value(value: &FieldValue) -> Option<i32> {
    match value {
        FieldValue::Byte(value) => Some(i32::from(*value)),
        FieldValue::Short(value) => Some(i32::from(*value)),
        FieldValue::Int(value) => Some(*value),
        FieldValue::Long(value) => Some(*value as i32),
        FieldValue::Float(value) => Some(*value as i32),
        FieldValue::Double(value) => Some(*value as i32),
        _ => None,
    }
}
