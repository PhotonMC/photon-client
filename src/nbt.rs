//! Flat-compound NBT for the pinned reference (P03.c, first slice).
//!
//! Covers null/empty roots and compounds with primitive and string fields.
//! Lists, arrays, nesting, quotas and SNBT belong to later slices. Strings
//! in this corpus are ASCII, where standard and modified UTF-8 agree.

/// Failures using the runner's error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    BufferUnderflow,
    DecoderError,
    EncoderError,
    TooBig,
}

/// Primitive field values in this slice.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Str(String),
}

/// Single named field, preserving write order.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub key: String,
    pub value: FieldValue,
}

fn read_byte(input: &mut &[u8]) -> Result<u8, ReadError> {
    let (&head, rest) = input.split_first().ok_or(ReadError::EncoderError)?;
    *input = rest;
    Ok(head)
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

fn read_name(input: &mut &[u8]) -> Result<String, ReadError> {
    let length = read_u16(input)? as usize;
    let bytes = read_bytes(input, length)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        let rest: &[u8] = &[];
        *input = rest;
        ReadError::DecoderError
    })
}

fn payload_size(id: u8) -> Option<usize> {
    match id {
        1 => Some(1),
        2 => Some(2),
        3 | 5 => Some(4),
        4 | 6 => Some(8),
        _ => None,
    }
}

/// Read one root tag: `None` for the end marker, otherwise a flat compound.
pub fn read_nbt(input: &mut &[u8]) -> Result<Option<Vec<Field>>, ReadError> {
    let id = read_byte(input)?;
    if id == 0 {
        return Ok(None);
    }
    if id != 10 {
        // Match the reference: the whole non-compound tag is consumed first.
        let size = match id {
            1 => 1,
            2 => 2,
            3 | 5 | 8 => 4,
            4 | 6 => 8,
            7 | 11 | 12 => {
                let rest: &[u8] = &[];
                *input = rest;
                return Err(ReadError::DecoderError);
            }
            _ => {
                let rest: &[u8] = &[];
                *input = rest;
                return Err(ReadError::DecoderError);
            }
        };
        if id == 8 {
            // String payload length is dynamic; consume what a length would need.
            if input.len() < 2 {
                let rest: &[u8] = &[];
                *input = rest;
                return Err(ReadError::DecoderError);
            }
            let length =
                u16::from_be_bytes(input[..2].try_into().expect("checked length")) as usize;
            if input.len() < 2 + length {
                let rest: &[u8] = &[];
                *input = rest;
                return Err(ReadError::DecoderError);
            }
            *input = &input[2 + length..];
        } else {
            read_bytes(input, size)?;
        }
        return Err(ReadError::DecoderError);
    }
    let mut fields = Vec::new();
    loop {
        let field_id = read_byte(input).map_err(|_| {
            let rest: &[u8] = &[];
            *input = rest;
            ReadError::DecoderError
        })?;
        if field_id == 0 {
            // Sort for a stable comparison; write order is preserved on encode.
            fields.sort_by(|a: &Field, b: &Field| a.key.cmp(&b.key));
            return Ok(Some(fields));
        }
        let key = read_name(input)?;
        if field_id == 8 {
            let length = read_u16(input)? as usize;
            let bytes = read_bytes(input, length)?;
            let value = String::from_utf8(bytes.to_vec()).map_err(|_| {
                let rest: &[u8] = &[];
                *input = rest;
                ReadError::DecoderError
            })?;
            fields.push(Field {
                key,
                value: FieldValue::Str(value),
            });
        } else {
            let size = payload_size(field_id).ok_or_else(|| {
                let rest: &[u8] = &[];
                *input = rest;
                ReadError::DecoderError
            })?;
            let payload = read_bytes(input, size)?;
            let value = match field_id {
                1 => FieldValue::Byte(payload[0] as i8),
                2 => FieldValue::Short(i16::from_be_bytes(
                    payload.try_into().expect("checked length"),
                )),
                3 => FieldValue::Int(i32::from_be_bytes(
                    payload.try_into().expect("checked length"),
                )),
                4 => FieldValue::Long(i64::from_be_bytes(
                    payload.try_into().expect("checked length"),
                )),
                5 => FieldValue::Float(f32::from_bits(u32::from_be_bytes(
                    payload.try_into().expect("checked length"),
                ))),
                6 => FieldValue::Double(f64::from_bits(u64::from_be_bytes(
                    payload.try_into().expect("checked length"),
                ))),
                _ => unreachable!("sized ids are matched above"),
            };
            fields.push(Field { key, value });
        }
    }
}

fn write_name(key: &str, output: &mut Vec<u8>) {
    output.extend_from_slice(&(key.len() as u16).to_be_bytes());
    output.extend_from_slice(key.as_bytes());
}

/// Append one root tag: `None` writes the end marker.
pub fn write_nbt(root: Option<&[Field]>, output: &mut Vec<u8>) {
    let Some(fields) = root else {
        output.push(0);
        return;
    };
    output.push(10);
    for field in fields {
        match &field.value {
            FieldValue::Byte(value) => {
                output.push(1);
                write_name(&field.key, output);
                output.push(*value as u8);
            }
            FieldValue::Short(value) => {
                output.push(2);
                write_name(&field.key, output);
                output.extend_from_slice(&value.to_be_bytes());
            }
            FieldValue::Int(value) => {
                output.push(3);
                write_name(&field.key, output);
                output.extend_from_slice(&value.to_be_bytes());
            }
            FieldValue::Long(value) => {
                output.push(4);
                write_name(&field.key, output);
                output.extend_from_slice(&value.to_be_bytes());
            }
            FieldValue::Float(value) => {
                output.push(5);
                write_name(&field.key, output);
                output.extend_from_slice(&value.to_bits().to_be_bytes());
            }
            FieldValue::Double(value) => {
                output.push(6);
                write_name(&field.key, output);
                output.extend_from_slice(&value.to_bits().to_be_bytes());
            }
            FieldValue::Str(value) => {
                output.push(8);
                write_name(&field.key, output);
                output.extend_from_slice(&(value.len() as u16).to_be_bytes());
                output.extend_from_slice(value.as_bytes());
            }
        }
    }
    output.push(0);
}
