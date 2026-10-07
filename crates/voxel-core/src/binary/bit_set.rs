use super::*;

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

/// Bit set stored as trimmed little-endian words, like the reference.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BitSet {
    pub(super) words: Vec<u64>,
}

impl BitSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from long words, dropping trailing zero words.
    pub fn from_longs(longs: &[i64]) -> Self {
        let mut words: Vec<u64> = longs.iter().map(|value| *value as u64).collect();
        while words.last() == Some(&0) {
            words.pop();
        }
        Self { words }
    }

    /// Build from ascending bit indices.
    pub fn from_bits(bits: &[u64]) -> Self {
        let mut words = Vec::new();
        for bit in bits {
            let word = (*bit / 64) as usize;
            if words.len() <= word {
                words.resize(word + 1, 0);
            }
            words[word] |= 1 << (bit % 64);
        }
        Self { words }
    }

    /// Words without trailing zeros, matching `toLongArray`.
    pub fn to_longs(&self) -> Vec<i64> {
        self.words.iter().map(|word| *word as i64).collect()
    }

    /// Ascending set bit indices.
    pub fn to_bits(&self) -> Vec<u64> {
        let mut bits = Vec::new();
        for (word_index, word) in self.words.iter().enumerate() {
            let mut remaining = *word;
            while remaining != 0 {
                let bit = remaining.trailing_zeros() as u64;
                bits.push(word_index as u64 * 64 + bit);
                remaining &= remaining - 1;
            }
        }
        bits
    }

    /// Build from little-endian bytes without masking excess bits.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut words = vec![0; bytes.len().div_ceil(8)];
        for (index, byte) in bytes.iter().enumerate() {
            words[index / 8] |= u64::from(*byte) << ((index % 8) * 8);
        }
        while words.last() == Some(&0) {
            words.pop();
        }
        Self { words }
    }

    /// Test a single bit.
    pub fn contains(&self, bit: u64) -> bool {
        let word = (bit / 64) as usize;
        self.words
            .get(word)
            .is_some_and(|value| value & (1 << (bit % 64)) != 0)
    }

    /// Highest set bit plus one, or zero when empty.
    pub fn len(&self) -> u64 {
        self.to_bits().last().map_or(0, |bit| bit + 1)
    }

    /// Whether no bits are set.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}
