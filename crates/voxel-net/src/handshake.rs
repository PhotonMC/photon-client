//! The serverbound handshake intention packet used before login or status.

use voxel_core::{utf8, wire};

pub const MAX_HOST_LENGTH: usize = 255;
pub const MAX_ENCODE_HOST_LENGTH: usize = 32_767;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIntent {
    Status,
    Login,
    Transfer,
}

impl ClientIntent {
    pub const fn id(self) -> i32 {
        match self {
            Self::Status => 1,
            Self::Login => 2,
            Self::Transfer => 3,
        }
    }

    pub const fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Status),
            2 => Some(Self::Login),
            3 => Some(Self::Transfer),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Login => "login",
            Self::Transfer => "transfer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    BufferUnderflow,
    DecoderError,
    TooBig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientIntentionPacket {
    pub protocol_version: i32,
    pub host_name: Vec<u16>,
    pub port: i32,
    pub intention: ClientIntent,
}

impl ClientIntentionPacket {
    pub fn new(
        protocol_version: i32,
        host_name: Vec<u16>,
        port: i32,
        intention: ClientIntent,
    ) -> Self {
        Self {
            protocol_version,
            host_name,
            port,
            intention,
        }
    }

    /// Encode fields in the order used by Java's handshake stream codec.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        wire::write_varint(self.protocol_version, output);
        utf8::write_utf8(output, &self.host_name, MAX_ENCODE_HOST_LENGTH)
            .map_err(|_| WriteError::EncoderError)?;
        output.extend_from_slice(&(self.port as u16).to_be_bytes());
        wire::write_varint(self.intention.id(), output);
        Ok(())
    }

    /// Decode one packet and retain Java's field-by-field consumption order.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let protocol_version = wire::read_varint(input).map_err(map_wire_error)?;
        let host_name = utf8::read_utf8(input, MAX_HOST_LENGTH).map_err(map_utf8_error)?;
        if input.len() < 2 {
            return Err(ReadError::BufferUnderflow);
        }
        let (port_bytes, rest) = input.split_at(2);
        let port = u16::from_be_bytes([port_bytes[0], port_bytes[1]]) as i32;
        *input = rest;
        let intention_id = wire::read_varint(input).map_err(map_wire_error)?;
        let intention = ClientIntent::from_id(intention_id).ok_or(ReadError::DecoderError)?;
        Ok(Self {
            protocol_version,
            host_name,
            port,
            intention,
        })
    }
}

fn map_wire_error(error: wire::ReadError) -> ReadError {
    match error {
        wire::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        wire::ReadError::TooBig => ReadError::TooBig,
    }
}

fn map_utf8_error(error: utf8::ReadError) -> ReadError {
    match error {
        utf8::ReadError::DecoderError => ReadError::DecoderError,
        utf8::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        utf8::ReadError::TooBig => ReadError::TooBig,
    }
}
