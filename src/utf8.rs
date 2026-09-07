//! Java-compatible UTF-8 strings for the pinned reference.
//!
//! Length limits count UTF-16 code units, not scalar values or bytes.
//! Unpaired surrogates can appear in write inputs, so the API uses
//! UTF-16 units rather than `&str`. Decoded output never contains
//! unpaired surrogates: malformed sequences become U+FFFD.

use crate::wire;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    DecoderError,
    BufferUnderflow,
    TooBig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}

/// Maximum encoded bytes for a UTF-16 length, as the reference computes it.
pub fn utf8_max_bytes(max_length: usize) -> usize {
    max_length.saturating_mul(3)
}

/// Read a length-prefixed string, advancing the input even on error.
/// VarInt prefix failures keep wire consumption; encoded-length and
/// availability failures consume only the prefix; decoded-length
/// failures consume the body as well.
pub fn read_utf8(input: &mut &[u8], max_length: usize) -> Result<Vec<u16>, ReadError> {
    let max_encoded = utf8_max_bytes(max_length);
    let length = match wire::read_varint(input) {
        Ok(value) => value,
        Err(wire::ReadError::BufferUnderflow) => return Err(ReadError::BufferUnderflow),
        Err(wire::ReadError::TooBig) => return Err(ReadError::TooBig),
    };
    if length < 0 {
        return Err(ReadError::DecoderError);
    }
    let length = length as usize;
    if length > max_encoded {
        return Err(ReadError::DecoderError);
    }
    if input.len() < length {
        return Err(ReadError::DecoderError);
    }
    let (body, rest) = input.split_at(length);
    let decoded = decode_java_utf8(body);
    *input = rest;
    if decoded.len() > max_length {
        return Err(ReadError::DecoderError);
    }
    Ok(decoded)
}

/// Append a length-prefixed string. On length failures nothing is appended,
/// preserving any prefix bytes already in the output.
pub fn write_utf8(
    output: &mut Vec<u8>,
    value: &[u16],
    max_length: usize,
) -> Result<(), WriteError> {
    if value.len() > max_length {
        return Err(WriteError::EncoderError);
    }
    let encoded = encode_netty_utf8(value);
    if encoded.len() > utf8_max_bytes(max_length) {
        return Err(WriteError::EncoderError);
    }
    wire::write_varint(encoded.len() as i32, output);
    output.extend_from_slice(&encoded);
    Ok(())
}

fn is_continuation(byte: u8) -> bool {
    (0x80..=0xBF).contains(&byte)
}

fn decode_java_utf8(bytes: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        if b0 < 0x80 {
            out.push(u16::from(b0));
            i += 1;
        } else if (0xC2..=0xDF).contains(&b0) {
            if i + 1 >= bytes.len() {
                out.push(0xFFFD);
                i = bytes.len();
            } else {
                let b1 = bytes[i + 1];
                if !is_continuation(b1) {
                    out.push(0xFFFD);
                    i += 1;
                } else {
                    let code = (u16::from(b0 & 0x1F) << 6) | u16::from(b1 & 0x3F);
                    out.push(code);
                    i += 2;
                }
            }
        } else if (0xE0..=0xEF).contains(&b0) {
            if i + 1 >= bytes.len() {
                out.push(0xFFFD);
                i = bytes.len();
            } else {
                let b1 = bytes[i + 1];
                if !is_continuation(b1) || (b0 == 0xE0 && !(0xA0..=0xBF).contains(&b1)) {
                    out.push(0xFFFD);
                    i += 1;
                } else if i + 2 >= bytes.len() {
                    // Valid two-byte prefix but the third byte is missing.
                    out.push(0xFFFD);
                    i = bytes.len();
                } else {
                    let b2 = bytes[i + 2];
                    if !is_continuation(b2) {
                        out.push(0xFFFD);
                        i += 2;
                    } else {
                        let code = (u16::from(b0 & 0x0F) << 12)
                            | (u16::from(b1 & 0x3F) << 6)
                            | u16::from(b2 & 0x3F);
                        if (0xD800..=0xDFFF).contains(&code) {
                            out.push(0xFFFD);
                        } else {
                            out.push(code);
                        }
                        i += 3;
                    }
                }
            }
        } else if (0xF0..=0xF4).contains(&b0) {
            if i + 1 >= bytes.len() {
                out.push(0xFFFD);
                i = bytes.len();
            } else {
                let b1 = bytes[i + 1];
                if !is_continuation(b1)
                    || (b0 == 0xF0 && !(0x90..=0xBF).contains(&b1))
                    || (b0 == 0xF4 && !(0x80..=0x8F).contains(&b1))
                {
                    out.push(0xFFFD);
                    i += 1;
                } else if i + 2 >= bytes.len() {
                    out.push(0xFFFD);
                    i = bytes.len();
                } else {
                    let b2 = bytes[i + 2];
                    if !is_continuation(b2) {
                        out.push(0xFFFD);
                        i += 2;
                    } else if i + 3 >= bytes.len() {
                        out.push(0xFFFD);
                        i = bytes.len();
                    } else {
                        let b3 = bytes[i + 3];
                        if !is_continuation(b3) {
                            out.push(0xFFFD);
                            i += 3;
                        } else {
                            let code = (u32::from(b0 & 0x07) << 18)
                                | (u32::from(b1 & 0x3F) << 12)
                                | (u32::from(b2 & 0x3F) << 6)
                                | u32::from(b3 & 0x3F);
                            // The prefix checks above keep this inside the
                            // supplementary range; unpaired surrogates cannot
                            // arise from four-byte sequences.
                            let code = code - 0x10000;
                            out.push(0xD800 | ((code >> 10) as u16));
                            out.push(0xDC00 | ((code & 0x3FF) as u16));
                            i += 4;
                        }
                    }
                }
            }
        } else {
            // C0, C1, F5-FF and lone continuations each become one replacement.
            out.push(0xFFFD);
            i += 1;
        }
    }
    out
}

fn is_high_surrogate(unit: u16) -> bool {
    (0xD800..=0xDBFF).contains(&unit)
}

fn is_low_surrogate(unit: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&unit)
}

fn encode_netty_utf8(value: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len().saturating_mul(3));
    let mut i = 0;
    while i < value.len() {
        let unit = value[i];
        if unit < 0x80 {
            out.push(unit as u8);
            i += 1;
        } else if unit < 0x800 {
            out.push(0xC0 | (unit >> 6) as u8);
            out.push(0x80 | (unit & 0x3F) as u8);
            i += 1;
        } else if is_high_surrogate(unit) || is_low_surrogate(unit) {
            if is_low_surrogate(unit) {
                out.push(b'?');
                i += 1;
            } else {
                i += 1;
                if i == value.len() {
                    out.push(b'?');
                    break;
                }
                let next = value[i];
                if !is_low_surrogate(next) {
                    out.push(b'?');
                    if is_high_surrogate(next) {
                        out.push(b'?');
                    } else {
                        // Matches the reference writer: the unit after an
                        // unpaired high surrogate is emitted as a single raw
                        // byte, not re-encoded as UTF-8.
                        out.push((next & 0xFF) as u8);
                    }
                    i += 1;
                } else {
                    let high = u32::from(unit - 0xD800);
                    let low = u32::from(next - 0xDC00);
                    let code = 0x10000 + (high << 10) + low;
                    // Valid surrogate pairs always form a supplementary scalar.
                    match char::from_u32(code) {
                        Some(ch) => {
                            let mut buf = [0; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        None => {
                            out.push(b'?');
                            out.push(b'?');
                        }
                    }
                    i += 1;
                }
            }
        } else {
            out.push(0xE0 | (unit >> 12) as u8);
            out.push(0x80 | ((unit >> 6) & 0x3F) as u8);
            out.push(0x80 | (unit & 0x3F) as u8);
            i += 1;
        }
    }
    out
}
