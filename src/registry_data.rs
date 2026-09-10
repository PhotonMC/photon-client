//! Configuration registry-data packet codec for the pinned Java protocol.

use crate::binary;
use crate::identifier::{self, Identifier};
use crate::nbt::{self, FieldValue};
use crate::wire;

/// Registry-data decoder failures using the runner's error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    BufferUnderflow,
    DecoderError,
    EncoderError,
    TooBig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}

/// One registry entry and its optional serialized NBT data.
#[derive(Debug, Clone, PartialEq)]
pub struct PackedRegistryEntry {
    pub id: Identifier,
    pub data: Option<FieldValue>,
}

impl PackedRegistryEntry {
    pub fn new(id: Identifier, data: Option<FieldValue>) -> Self {
        Self { id, data }
    }
}

/// Clientbound configuration registry-data packet.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundRegistryDataPacket {
    pub registry: Identifier,
    pub entries: Vec<PackedRegistryEntry>,
}

impl ClientboundRegistryDataPacket {
    pub fn new(registry: Identifier, entries: Vec<PackedRegistryEntry>) -> Self {
        Self { registry, entries }
    }

    /// Encode the registry identifier, entry list and optional generic NBT tags.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        identifier::write_identifier(&self.registry, output)
            .map_err(|_| WriteError::EncoderError)?;
        write_count(self.entries.len(), output)?;
        for entry in &self.entries {
            identifier::write_identifier(&entry.id, output)
                .map_err(|_| WriteError::EncoderError)?;
            output.push(u8::from(entry.data.is_some()));
            if let Some(data) = &entry.data {
                nbt::write_any_nbt(Some(data), output);
            }
        }
        Ok(())
    }

    /// Decode one registry-data packet and preserve list order.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let registry = identifier::read_identifier(input).map_err(map_binary_error)?;
        let count = read_count(input)?;
        let mut entries = Vec::with_capacity(count.min(65_536));
        for _ in 0..count {
            let id = identifier::read_identifier(input).map_err(map_binary_error)?;
            let has_data = read_byte(input)? != 0;
            let data = if has_data {
                match nbt::read_any_nbt(input).map_err(map_nbt_error)? {
                    Some(value) => Some(value),
                    None => return Err(ReadError::DecoderError),
                }
            } else {
                None
            };
            entries.push(PackedRegistryEntry::new(id, data));
        }
        Ok(Self { registry, entries })
    }
}

fn write_count(count: usize, output: &mut Vec<u8>) -> Result<(), WriteError> {
    let count = i32::try_from(count).map_err(|_| WriteError::EncoderError)?;
    wire::write_varint(count, output);
    Ok(())
}

fn read_count(input: &mut &[u8]) -> Result<usize, ReadError> {
    let count = wire::read_varint(input).map_err(map_wire_error)?;
    if count < 0 {
        return Err(ReadError::DecoderError);
    }
    usize::try_from(count).map_err(|_| ReadError::DecoderError)
}

fn read_byte(input: &mut &[u8]) -> Result<u8, ReadError> {
    let (&head, rest) = input.split_first().ok_or(ReadError::BufferUnderflow)?;
    *input = rest;
    Ok(head)
}

fn map_wire_error(error: wire::ReadError) -> ReadError {
    match error {
        wire::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        wire::ReadError::TooBig => ReadError::TooBig,
    }
}

fn map_binary_error(error: binary::ReadError) -> ReadError {
    match error {
        binary::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        binary::ReadError::DecoderError => ReadError::DecoderError,
        binary::ReadError::TooBig => ReadError::TooBig,
    }
}

fn map_nbt_error(error: nbt::ReadError) -> ReadError {
    match error {
        nbt::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        nbt::ReadError::DecoderError => ReadError::DecoderError,
        nbt::ReadError::EncoderError => ReadError::EncoderError,
        nbt::ReadError::TooBig => ReadError::TooBig,
    }
}
