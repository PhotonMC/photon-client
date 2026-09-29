//! Common protocol packet codecs for the pinned Java reference.

use voxel_core::binary;

pub type ReadError = binary::ReadError;

/// Clientbound common ping carrying one big-endian signed int.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPingPacket {
    pub id: i32,
}

impl ClientboundPingPacket {
    pub const fn new(id: i32) -> Self {
        Self { id }
    }

    pub fn encode(self, output: &mut Vec<u8>) {
        write_i32(self.id, output);
    }

    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self::new(read_i32(input)?))
    }
}

/// Serverbound common pong carrying one big-endian signed int.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPongPacket {
    pub id: i32,
}

impl ServerboundPongPacket {
    pub const fn new(id: i32) -> Self {
        Self { id }
    }

    pub fn encode(self, output: &mut Vec<u8>) {
        write_i32(self.id, output);
    }

    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self::new(read_i32(input)?))
    }
}

/// Clientbound common keep-alive carrying one big-endian signed long.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundKeepAlivePacket {
    pub id: i64,
}

impl ClientboundKeepAlivePacket {
    pub const fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn encode(self, output: &mut Vec<u8>) {
        write_i64(self.id, output);
    }

    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self::new(read_i64(input)?))
    }
}

/// Serverbound common keep-alive carrying one big-endian signed long.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundKeepAlivePacket {
    pub id: i64,
}

impl ServerboundKeepAlivePacket {
    pub const fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn encode(self, output: &mut Vec<u8>) {
        write_i64(self.id, output);
    }

    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self::new(read_i64(input)?))
    }
}

fn write_i32(value: i32, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn read_i32(input: &mut &[u8]) -> Result<i32, ReadError> {
    if input.len() < 4 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(4);
    *input = rest;
    Ok(i32::from_be_bytes(
        head.try_into().expect("checked int length"),
    ))
}

fn write_i64(value: i64, output: &mut Vec<u8>) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn read_i64(input: &mut &[u8]) -> Result<i64, ReadError> {
    if input.len() < 8 {
        return Err(ReadError::BufferUnderflow);
    }
    let (head, rest) = input.split_at(8);
    *input = rest;
    Ok(i64::from_be_bytes(
        head.try_into().expect("checked long length"),
    ))
}
