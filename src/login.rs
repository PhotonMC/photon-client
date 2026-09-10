//! The serverbound login hello packet sent after a handshake intention.

use crate::binary::{self, Uuid};
use crate::utf8;

pub const MAX_NAME_LENGTH: usize = 16;

pub type ReadError = binary::ReadError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundHelloPacket {
    pub name: Vec<u16>,
    pub profile_id: Uuid,
}

impl ServerboundHelloPacket {
    pub const fn new(name: Vec<u16>, profile_id: Uuid) -> Self {
        Self { name, profile_id }
    }

    /// Encode the bounded UTF-8 name followed by the two UUID longs.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        utf8::write_utf8(output, &self.name, MAX_NAME_LENGTH)
            .map_err(|_| WriteError::EncoderError)?;
        binary::write_uuid(self.profile_id, output);
        Ok(())
    }

    /// Decode one hello packet with Java's name-then-UUID consumption order.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let name = utf8::read_utf8(input, MAX_NAME_LENGTH).map_err(map_utf8_error)?;
        let profile_id = binary::read_uuid(input)?;
        Ok(Self { name, profile_id })
    }
}

fn map_utf8_error(error: utf8::ReadError) -> ReadError {
    match error {
        utf8::ReadError::DecoderError => ReadError::DecoderError,
        utf8::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        utf8::ReadError::TooBig => ReadError::TooBig,
    }
}
