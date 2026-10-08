//! SHA-1, which the reference names a server's icon texture with
//! (`Hashing.sha1().hashUnencodedChars`).

/// The digest of `bytes`.
pub fn digest(bytes: &[u8]) -> [u8; 20] {
    let mut state: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&(bytes.len() as u64 * 8).to_be_bytes());
    for block in padded.as_chunks::<64>().0 {
        let mut words = [0_u32; 80];
        for (word, bytes) in words.iter_mut().zip(block.as_chunks::<4>().0) {
            *word = u32::from_be_bytes(*bytes);
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = state;
        for (index, word) in words.iter().enumerate() {
            let (mix, constant) = match index {
                0..20 => ((b & c) | (!b & d), 0x5A82_7999),
                20..40 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..60 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let next = a
                .rotate_left(5)
                .wrapping_add(mix)
                .wrapping_add(e)
                .wrapping_add(constant)
                .wrapping_add(*word);
            (e, d, c, b, a) = (d, c, b.rotate_left(30), a, next);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut out = [0; 20];
    for (bytes, word) in out.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        *bytes = word.to_be_bytes();
    }
    out
}

/// `HashCode.toString` of `Hashing.sha1().hashUnencodedChars(text)`: the hash of the text's UTF-16
/// code units, each low byte first, as lowercase hex.
pub fn hash_unencoded_chars(text: &str) -> String {
    let bytes: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn matches_the_reference_vectors() {
        let hex = |bytes: &[u8]| -> String {
            super::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect()
        };
        assert_eq!(hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(
            hex(&[b'a'; 1000]),
            "291e9a6c66994949b57ba5e650361e98fc36b1ba"
        );
    }

    /// The icon of the oracle's `127.0.0.1:1` servers, as the reference's recorded draw names it.
    #[test]
    fn names_a_server_address_like_the_reference() {
        assert_eq!(
            super::hash_unencoded_chars("127.0.0.1:1"),
            "4a76554f38707bbaaa61c88f6943d9d2515ecd9b"
        );
    }
}
