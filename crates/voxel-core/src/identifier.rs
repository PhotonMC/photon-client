//! Resource identifiers used by protocol payloads.

use crate::{binary, utf8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError;

/// A Java-style resource identifier with a namespace and path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub const DEFAULT_NAMESPACE: &'static str = "minecraft";

    pub fn parse(value: &str) -> Result<Self, ParseError> {
        let (namespace, path) = match value.find(':') {
            Some(separator) if separator != 0 => (&value[..separator], &value[separator + 1..]),
            Some(separator) => (Self::DEFAULT_NAMESPACE, &value[separator + 1..]),
            None => (Self::DEFAULT_NAMESPACE, value),
        };
        if !is_valid_namespace(namespace) || !is_valid_path(path) {
            return Err(ParseError);
        }
        Ok(Self {
            namespace: namespace.to_owned(),
            path: path.to_owned(),
        })
    }

    pub fn from_namespace_and_path(
        namespace: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Self, ParseError> {
        let namespace = namespace.into();
        let path = path.into();
        if !is_valid_namespace(&namespace) || !is_valid_path(&path) {
            return Err(ParseError);
        }
        Ok(Self { namespace, path })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn as_string(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }

    pub(crate) fn java_hash_code(&self) -> i32 {
        self.namespace
            .java_string_hash()
            .wrapping_mul(31)
            .wrapping_add(self.path.java_string_hash())
    }
}

pub fn write_identifier(
    identifier: &Identifier,
    output: &mut Vec<u8>,
) -> Result<(), utf8::WriteError> {
    let units: Vec<u16> = identifier.as_string().encode_utf16().collect();
    utf8::write_utf8(output, &units, 32_767)
}

pub fn read_identifier(input: &mut &[u8]) -> Result<Identifier, binary::ReadError> {
    let units = utf8::read_utf8(input, 32_767).map_err(map_utf8_error)?;
    let value = String::from_utf16(&units).map_err(|_| binary::ReadError::DecoderError)?;
    Identifier::parse(&value).map_err(|_| binary::ReadError::DecoderError)
}

pub fn java_hash_set_order(features: Vec<Identifier>, initial_capacity: usize) -> Vec<Identifier> {
    java_hash_map_order(
        features.into_iter().map(|feature| (feature, ())).collect(),
        initial_capacity,
    )
    .into_iter()
    .map(|(feature, ())| feature)
    .collect()
}

pub(crate) fn java_hash_map_order<T>(
    entries: Vec<(Identifier, T)>,
    initial_capacity: usize,
) -> Vec<(Identifier, T)> {
    if entries.is_empty() {
        return entries;
    }

    let capacity = if initial_capacity == 0 {
        16
    } else {
        table_size_for(initial_capacity)
    };
    let mut table: Vec<Vec<(Identifier, T)>> = (0..capacity).map(|_| Vec::new()).collect();
    let mut threshold = table.len() * 3 / 4;
    let mut size = 0;
    for (key, value) in entries {
        let mut existing = None;
        for (bucket_index, bucket) in table.iter().enumerate() {
            if let Some(entry_index) = bucket.iter().position(|entry| entry.0 == key) {
                existing = Some((bucket_index, entry_index));
                break;
            }
        }
        if let Some((bucket_index, entry_index)) = existing {
            table[bucket_index][entry_index].1 = value;
            continue;
        }

        let index = hash_bucket(&key, table.len());
        table[index].push((key, value));
        size += 1;
        if size > threshold {
            let mut resized: Vec<Vec<(Identifier, T)>> =
                (0..table.len() * 2).map(|_| Vec::new()).collect();
            for bucket in table {
                for entry in bucket {
                    let index = hash_bucket(&entry.0, resized.len());
                    resized[index].push(entry);
                }
            }
            threshold = resized.len() * 3 / 4;
            table = resized;
        }
    }
    table.into_iter().flatten().collect()
}

trait JavaStringHash {
    fn java_string_hash(&self) -> i32;
}

impl JavaStringHash for str {
    fn java_string_hash(&self) -> i32 {
        self.encode_utf16().fold(0_i32, |hash, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }
}

fn is_valid_namespace(value: &str) -> bool {
    value != ".." && value.chars().all(is_valid_namespace_char)
}

fn is_valid_path(value: &str) -> bool {
    value.chars().all(is_valid_path_char)
}

fn is_valid_namespace_char(value: char) -> bool {
    value.is_ascii_lowercase() || value.is_ascii_digit() || matches!(value, '_' | '-' | '.')
}

fn is_valid_path_char(value: char) -> bool {
    is_valid_namespace_char(value) || value == '/'
}

fn map_utf8_error(error: utf8::ReadError) -> binary::ReadError {
    match error {
        utf8::ReadError::DecoderError => binary::ReadError::DecoderError,
        utf8::ReadError::BufferUnderflow => binary::ReadError::BufferUnderflow,
        utf8::ReadError::TooBig => binary::ReadError::TooBig,
    }
}

fn hash_bucket(identifier: &Identifier, capacity: usize) -> usize {
    let hash = identifier.java_hash_code() as u32;
    ((hash ^ (hash >> 16)) as usize) & (capacity - 1)
}

fn table_size_for(value: usize) -> usize {
    value.max(1).next_power_of_two()
}
