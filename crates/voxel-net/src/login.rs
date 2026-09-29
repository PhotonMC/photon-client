//! The serverbound login hello packet sent after a handshake intention.

use voxel_core::binary::{self, Uuid};
use voxel_core::{utf8, wire};

pub const MAX_NAME_LENGTH: usize = 16;
pub const MAX_SERVER_ID_LENGTH: usize = 20;
pub const MAX_ENCODE_SERVER_ID_LENGTH: usize = 32_767;
pub const MAX_PROFILE_PROPERTIES: usize = 16;
pub const MAX_PROPERTY_NAME_LENGTH: usize = 64;
pub const MAX_PROPERTY_VALUE_LENGTH: usize = 32_767;
pub const MAX_PROPERTY_SIGNATURE_LENGTH: usize = 1_024;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundLoginCompressionPacket {
    pub threshold: i32,
}

impl ClientboundLoginCompressionPacket {
    pub const fn new(threshold: i32) -> Self {
        Self { threshold }
    }

    /// Encode the compression threshold as one Java VarInt.
    pub fn encode(self, output: &mut Vec<u8>) {
        wire::write_varint(self.threshold, output);
    }

    /// Decode the threshold without applying compression to later frames.
    pub fn decode(input: &mut &[u8]) -> Result<Self, wire::ReadError> {
        Ok(Self {
            threshold: wire::read_varint(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundHelloPacket {
    pub server_id: Vec<u16>,
    pub public_key: Vec<u8>,
    pub challenge: Vec<u8>,
    pub should_authenticate: bool,
}

impl ClientboundHelloPacket {
    pub fn new(
        server_id: Vec<u16>,
        public_key: Vec<u8>,
        challenge: Vec<u8>,
        should_authenticate: bool,
    ) -> Self {
        Self {
            server_id,
            public_key,
            challenge,
            should_authenticate,
        }
    }

    /// Encode the server id, two length-prefixed byte arrays and a boolean.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        utf8::write_utf8(output, &self.server_id, MAX_ENCODE_SERVER_ID_LENGTH)
            .map_err(|_| WriteError::EncoderError)?;
        write_byte_array(&self.public_key, output)?;
        write_byte_array(&self.challenge, output)?;
        output.push(u8::from(self.should_authenticate));
        Ok(())
    }

    /// Decode fields in Java's server-id, public-key, challenge, boolean order.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let server_id = utf8::read_utf8(input, MAX_SERVER_ID_LENGTH).map_err(map_utf8_error)?;
        let public_key = read_byte_array(input)?;
        let challenge = read_byte_array(input)?;
        let (&should_authenticate, rest) = input.split_first().ok_or(ReadError::BufferUnderflow)?;
        *input = rest;
        Ok(Self {
            server_id,
            public_key,
            challenge,
            should_authenticate: should_authenticate != 0,
        })
    }
}

/// The empty login acknowledgement packet, decoded as its singleton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundLoginAcknowledgedPacket;

impl ServerboundLoginAcknowledgedPacket {
    pub const INSTANCE: Self = Self;

    pub const fn encode(self, _output: &mut Vec<u8>) {}

    pub const fn decode(_input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileProperty {
    pub name: Vec<u16>,
    pub value: Vec<u16>,
    pub signature: Option<Vec<u16>>,
}

impl ProfileProperty {
    pub fn new(name: Vec<u16>, value: Vec<u16>, signature: Option<Vec<u16>>) -> Self {
        Self {
            name,
            value,
            signature,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: Vec<u16>,
    pub properties: Vec<ProfileProperty>,
}

impl GameProfile {
    pub fn new(id: Uuid, name: Vec<u16>, properties: Vec<ProfileProperty>) -> Self {
        Self {
            id,
            name,
            properties,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginFinishedPacket {
    pub game_profile: GameProfile,
    pub session_id: Uuid,
}

impl ClientboundLoginFinishedPacket {
    pub fn new(game_profile: GameProfile, session_id: Uuid) -> Self {
        Self {
            game_profile,
            session_id,
        }
    }

    /// Encode GameProfile fields followed by the session UUID.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        binary::write_uuid(self.game_profile.id, output);
        utf8::write_utf8(output, &self.game_profile.name, MAX_NAME_LENGTH)
            .map_err(|_| WriteError::EncoderError)?;
        write_properties(&self.game_profile.properties, output)?;
        binary::write_uuid(self.session_id, output);
        Ok(())
    }

    /// Decode GameProfile fields followed by the session UUID.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let id = binary::read_uuid(input)?;
        let name = utf8::read_utf8(input, MAX_NAME_LENGTH).map_err(map_utf8_error)?;
        let properties = read_properties(input)?;
        let session_id = binary::read_uuid(input)?;
        Ok(Self::new(
            GameProfile::new(id, name, properties),
            session_id,
        ))
    }
}

fn write_properties(
    properties: &[ProfileProperty],
    output: &mut Vec<u8>,
) -> Result<(), WriteError> {
    if properties.len() > MAX_PROFILE_PROPERTIES {
        return Err(WriteError::EncoderError);
    }
    wire::write_varint(properties.len() as i32, output);
    for property in properties {
        utf8::write_utf8(output, &property.name, MAX_PROPERTY_NAME_LENGTH)
            .map_err(|_| WriteError::EncoderError)?;
        utf8::write_utf8(output, &property.value, MAX_PROPERTY_VALUE_LENGTH)
            .map_err(|_| WriteError::EncoderError)?;
        match &property.signature {
            Some(signature) => {
                output.push(1);
                utf8::write_utf8(output, signature, MAX_PROPERTY_SIGNATURE_LENGTH)
                    .map_err(|_| WriteError::EncoderError)?;
            }
            None => output.push(0),
        }
    }
    Ok(())
}

fn read_properties(input: &mut &[u8]) -> Result<Vec<ProfileProperty>, ReadError> {
    let count = wire::read_varint(input).map_err(map_wire_error)?;
    if count > MAX_PROFILE_PROPERTIES as i32 {
        return Err(ReadError::DecoderError);
    }
    let mut properties = Vec::new();
    for _ in 0..count {
        let name = utf8::read_utf8(input, MAX_PROPERTY_NAME_LENGTH).map_err(map_utf8_error)?;
        let value = utf8::read_utf8(input, MAX_PROPERTY_VALUE_LENGTH).map_err(map_utf8_error)?;
        let (&has_signature, rest) = input.split_first().ok_or(ReadError::BufferUnderflow)?;
        *input = rest;
        let signature = if has_signature != 0 {
            Some(utf8::read_utf8(input, MAX_PROPERTY_SIGNATURE_LENGTH).map_err(map_utf8_error)?)
        } else {
            None
        };
        properties.push(ProfileProperty::new(name, value, signature));
    }
    Ok(properties)
}

fn write_byte_array(value: &[u8], output: &mut Vec<u8>) -> Result<(), WriteError> {
    let length = i32::try_from(value.len()).map_err(|_| WriteError::EncoderError)?;
    wire::write_varint(length, output);
    output.extend_from_slice(value);
    Ok(())
}

fn read_byte_array(input: &mut &[u8]) -> Result<Vec<u8>, ReadError> {
    let available_before_length = input.len();
    let length = wire::read_varint(input).map_err(map_wire_error)?;
    if length < 0 {
        return Err(ReadError::DecoderError);
    }
    let length = length as usize;
    if length > available_before_length {
        return Err(ReadError::DecoderError);
    }
    if length > input.len() {
        return Err(ReadError::BufferUnderflow);
    }
    let (value, rest) = input.split_at(length);
    *input = rest;
    Ok(value.to_vec())
}

fn map_utf8_error(error: utf8::ReadError) -> ReadError {
    match error {
        utf8::ReadError::DecoderError => ReadError::DecoderError,
        utf8::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        utf8::ReadError::TooBig => ReadError::TooBig,
    }
}

fn map_wire_error(error: wire::ReadError) -> ReadError {
    match error {
        wire::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        wire::ReadError::TooBig => ReadError::TooBig,
    }
}
