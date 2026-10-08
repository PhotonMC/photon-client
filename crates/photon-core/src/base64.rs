//! `java.util.Base64.getDecoder()`: the standard alphabet with `=` padding, nothing else allowed.

/// The bytes `text` encodes, or none when it is not valid base64.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    let value = |byte: u8| -> Option<u32> {
        Some(u32::from(match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        }))
    };
    let trimmed = match bytes {
        [rest @ .., b'=', b'='] => rest,
        [rest @ .., b'='] => rest,
        _ => bytes,
    };
    // A padded input must be whole quads; one quad's worth of a single character is never valid.
    if (trimmed.len() != bytes.len() && !bytes.len().is_multiple_of(4)) || trimmed.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(trimmed.len() * 3 / 4);
    for chunk in trimmed.chunks(4) {
        let mut bits = 0_u32;
        for &byte in chunk {
            bits = bits << 6 | value(byte)?;
        }
        bits <<= 6 * (4 - chunk.len() as u32);
        let produced = chunk.len() * 6 / 8;
        out.extend_from_slice(&bits.to_be_bytes()[1..=produced]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn decodes_padded_and_unpadded_text() {
        assert_eq!(super::decode("TWFu").as_deref(), Some(&b"Man"[..]));
        assert_eq!(super::decode("TWE=").as_deref(), Some(&b"Ma"[..]));
        assert_eq!(super::decode("TQ==").as_deref(), Some(&b"M"[..]));
        assert_eq!(super::decode("TQ").as_deref(), Some(&b"M"[..]));
        assert_eq!(super::decode("T"), None);
        assert_eq!(super::decode("T!=="), None);
    }
}
