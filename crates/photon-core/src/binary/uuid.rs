use super::*;

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

/// 128-bit identifier split into big-endian halves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most: u64,
    pub least: u64,
}

impl Uuid {
    pub const fn new(most: u64, least: u64) -> Self {
        Self { most, least }
    }

    /// `UUID.toString`: eight, four, four, four and twelve lowercase hex digits.
    pub fn to_string_java(self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.most >> 32,
            (self.most >> 16) & 0xFFFF,
            self.most & 0xFFFF,
            self.least >> 48,
            self.least & 0xFFFF_FFFF_FFFF
        )
    }
}

#[cfg(test)]
mod uuid_tests {
    use super::Uuid;

    #[test]
    fn a_uuid_reads_as_java_writes_it() {
        assert_eq!(
            Uuid::new(0x5e8c_9a1b_0000_4000, 0x8000_0000_0000_0001).to_string_java(),
            "5e8c9a1b-0000-4000-8000-000000000001"
        );
        assert_eq!(
            Uuid::new(0, 7).to_string_java(),
            "00000000-0000-0000-0000-000000000007"
        );
    }
}
