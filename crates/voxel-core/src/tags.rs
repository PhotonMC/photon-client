//! Network tag payload codecs for the pinned Java protocol.

use crate::binary;
use crate::identifier::{self, Identifier};
use crate::wire;

pub type ReadError = binary::ReadError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}

/// One tag identifier and its registry entry ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagEntry {
    pub id: Identifier,
    pub values: Vec<i32>,
}

impl TagEntry {
    pub fn new(id: Identifier, values: Vec<i32>) -> Self {
        Self { id, values }
    }
}

/// Tags belonging to one registry key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryTags {
    pub registry: Identifier,
    pub tags: Vec<TagEntry>,
}

impl RegistryTags {
    pub fn new(registry: Identifier, tags: Vec<TagEntry>) -> Self {
        Self {
            registry,
            tags: order_tags(tags, 0),
        }
    }
}

/// Clientbound common update-tags packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateTagsPacket {
    pub registries: Vec<RegistryTags>,
}

impl ClientboundUpdateTagsPacket {
    pub fn new(registries: Vec<RegistryTags>) -> Self {
        Self {
            registries: deduplicate_registries(registries),
        }
    }

    /// Encode the registry map and each identifier-to-id-list payload.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        wire::write_varint(self.registries.len() as i32, output);
        for registry in &self.registries {
            write_identifier(&registry.registry, output)?;
            wire::write_varint(registry.tags.len() as i32, output);
            for tag in &registry.tags {
                write_identifier(&tag.id, output)?;
                wire::write_varint(tag.values.len() as i32, output);
                for value in &tag.values {
                    wire::write_varint(*value, output);
                }
            }
        }
        Ok(())
    }

    /// Decode the registry map and preserve the Java map iteration order.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let registry_count = read_count(input)?;
        let mut registries = Vec::new();
        for _ in 0..registry_count {
            let registry = identifier::read_identifier(input)?;
            let tag_count = read_count(input)?;
            let mut tags = Vec::new();
            for _ in 0..tag_count {
                let id = identifier::read_identifier(input)?;
                let value_count = wire::read_varint(input).map_err(ReadError::from)?;
                let mut values = Vec::new();
                if value_count >= 0 {
                    for _ in 0..value_count {
                        values.push(wire::read_varint(input).map_err(ReadError::from)?);
                    }
                }
                tags.push(TagEntry::new(id, values));
            }
            registries.push(RegistryTags {
                registry,
                tags: order_tags(tags, guava_expected_size(tag_count)),
            });
        }
        Ok(Self {
            registries: deduplicate_registries(registries),
        })
    }
}

fn write_identifier(identifier: &Identifier, output: &mut Vec<u8>) -> Result<(), WriteError> {
    identifier::write_identifier(identifier, output).map_err(|_| WriteError::EncoderError)
}

fn read_count(input: &mut &[u8]) -> Result<usize, ReadError> {
    let count = wire::read_varint(input).map_err(ReadError::from)?;
    usize::try_from(count).map_err(|_| ReadError::DecoderError)
}

fn order_tags(tags: Vec<TagEntry>, initial_capacity: usize) -> Vec<TagEntry> {
    identifier::java_hash_map_order(
        tags.into_iter().map(|tag| (tag.id.clone(), tag)).collect(),
        initial_capacity,
    )
    .into_iter()
    .map(|(_, tag)| tag)
    .collect()
}

fn deduplicate_registries(registries: Vec<RegistryTags>) -> Vec<RegistryTags> {
    let mut result = Vec::new();
    for registry in registries {
        if let Some(existing) = result
            .iter_mut()
            .find(|existing: &&mut RegistryTags| existing.registry == registry.registry)
        {
            existing.tags = registry.tags;
        } else {
            result.push(registry);
        }
    }
    result
}

fn guava_expected_size(value: usize) -> usize {
    if value < 3 {
        value + 1
    } else if value < (1 << 30) {
        (value as f64 / 0.75 + 1.0) as usize
    } else {
        usize::MAX
    }
}
