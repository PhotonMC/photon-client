//! `java.util.Base64.getDecoder()`/`getEncoder()`: the standard alphabet with `=` padding,
//! nothing else allowed on decode.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `Base64.getEncoder()`: the standard alphabet, `=` padded, on one line.
pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let bits = chunk
            .iter()
            .enumerate()
            .fold(0_u32, |bits, (index, &byte)| {
                bits | u32::from(byte) << (16 - 8 * index)
            });
        for index in 0..4 {
            if index <= chunk.len() {
                out.push(char::from(
                    ALPHABET[(bits >> (18 - 6 * index) & 63) as usize],
                ));
            } else {
                out.push('=');
            }
        }
    }
    out
}

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

    #[test]
    fn encodes_with_padding_and_round_trips() {
        assert_eq!(super::encode(b"Man"), "TWFu");
        assert_eq!(super::encode(b"Ma"), "TWE=");
        assert_eq!(super::encode(b"M"), "TQ==");
        assert_eq!(super::encode(b""), "");
        let bytes: Vec<u8> = (0..=255).collect();
        assert_eq!(super::decode(&super::encode(&bytes)), Some(bytes));
    }
}
