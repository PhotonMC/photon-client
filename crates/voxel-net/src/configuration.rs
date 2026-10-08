//! Configuration phase packet codecs for the pinned Java protocol.

use voxel_core::binary;
use voxel_core::identifier::{self, Identifier};
use voxel_core::{utf8, wire};

pub type ReadError = binary::ReadError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}

pub use voxel_game::options::ChatVisibility;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainHand {
    Left,
    Right,
}

impl MainHand {
    const fn id(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    const fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Left),
            1 => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

impl ParticleStatus {
    const fn id(self) -> i32 {
        match self {
            Self::All => 0,
            Self::Decreased => 1,
            Self::Minimal => 2,
        }
    }

    const fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::All),
            1 => Some(Self::Decreased),
            2 => Some(Self::Minimal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInformation {
    pub language: Vec<u16>,
    pub view_distance: i32,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customisation: i32,
    pub main_hand: MainHand,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
    pub particle_status: ParticleStatus,
}

impl Default for ClientInformation {
    /// What `Options.buildPlayerInformation` gives with the default options: English, twelve
    /// chunks, full chat in colour, every model part, the right hand, no filtering, listed, all
    /// particles.
    fn default() -> Self {
        Self::new(
            "en_us".encode_utf16().collect(),
            12,
            ChatVisibility::Full,
            true,
            127,
            MainHand::Right,
            false,
            true,
            ParticleStatus::All,
        )
    }
}

impl ClientInformation {
    #[expect(
        clippy::too_many_arguments,
        reason = "one parameter per field of the packet"
    )]
    pub fn new(
        language: Vec<u16>,
        view_distance: i32,
        chat_visibility: ChatVisibility,
        chat_colors: bool,
        model_customisation: i32,
        main_hand: MainHand,
        text_filtering_enabled: bool,
        allows_listing: bool,
        particle_status: ParticleStatus,
    ) -> Self {
        Self {
            language,
            view_distance,
            chat_visibility,
            chat_colors,
            model_customisation,
            main_hand,
            text_filtering_enabled,
            allows_listing,
            particle_status,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundClientInformationPacket {
    pub information: ClientInformation,
}

/// Clientbound configuration features represented by Java's identifier set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateEnabledFeaturesPacket {
    pub features: Vec<Identifier>,
}

impl ClientboundUpdateEnabledFeaturesPacket {
    pub fn new(features: Vec<Identifier>) -> Self {
        Self {
            features: identifier::java_hash_set_order(features, 0),
        }
    }

    /// Encode the HashSet-backed feature collection and each UTF-8 identifier.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        wire::write_varint(self.features.len() as i32, output);
        for feature in &self.features {
            identifier::write_identifier(feature, output).map_err(|_| WriteError::EncoderError)?;
        }
        Ok(())
    }

    /// Decode the count-sized HashSet and preserve Java's iteration order.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let count = wire::read_varint(input).map_err(ReadError::from)?;
        if count < 0 {
            return Err(ReadError::DecoderError);
        }
        let count = count as usize;
        let mut raw = Vec::new();
        for _ in 0..count {
            raw.push(identifier::read_identifier(input)?);
        }
        Ok(Self {
            features: identifier::java_hash_set_order(raw, count),
        })
    }
}

/// Clientbound configuration code-of-conduct text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCodeOfConductPacket {
    pub code_of_conduct: Vec<u16>,
}

impl ClientboundCodeOfConductPacket {
    pub fn new(code_of_conduct: Vec<u16>) -> Self {
        Self { code_of_conduct }
    }

    /// Encode the text with Java's default 32767 UTF-16-unit bound.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        utf8::write_utf8(output, &self.code_of_conduct, 32_767)
            .map_err(|_| WriteError::EncoderError)
    }

    /// Decode the text with Java's default 32767 UTF-16-unit bound.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self::new(
            utf8::read_utf8(input, 32_767).map_err(ReadError::from)?,
        ))
    }
}

impl ServerboundClientInformationPacket {
    pub fn new(information: ClientInformation) -> Self {
        Self { information }
    }

    /// Encode the nine ClientInformation fields in Java's order.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        utf8::write_utf8(output, &self.information.language, 32_767)
            .map_err(|_| WriteError::EncoderError)?;
        output.push(self.information.view_distance as u8);
        write_enum(self.information.chat_visibility.id(), output);
        output.push(u8::from(self.information.chat_colors));
        output.push(self.information.model_customisation as u8);
        write_enum(self.information.main_hand.id(), output);
        output.push(u8::from(self.information.text_filtering_enabled));
        output.push(u8::from(self.information.allows_listing));
        write_enum(self.information.particle_status.id(), output);
        Ok(())
    }

    /// Decode the nine ClientInformation fields in Java's order.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let language = utf8::read_utf8(input, 16).map_err(ReadError::from)?;
        let view_distance = i8::from_be_bytes([read_byte(input)?]) as i32;
        let chat_visibility =
            ChatVisibility::from_id(read_enum(input)?).ok_or(ReadError::DecoderError)?;
        let chat_colors = read_byte(input)? != 0;
        let model_customisation = i32::from(read_byte(input)?);
        let main_hand = MainHand::from_id(read_enum(input)?).ok_or(ReadError::DecoderError)?;
        let text_filtering_enabled = read_byte(input)? != 0;
        let allows_listing = read_byte(input)? != 0;
        let particle_status =
            ParticleStatus::from_id(read_enum(input)?).ok_or(ReadError::DecoderError)?;
        Ok(Self::new(ClientInformation::new(
            language,
            view_distance,
            chat_visibility,
            chat_colors,
            model_customisation,
            main_hand,
            text_filtering_enabled,
            allows_listing,
            particle_status,
        )))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundFinishConfigurationPacket;

impl ServerboundFinishConfigurationPacket {
    pub const INSTANCE: Self = Self;

    pub const fn encode(self, _output: &mut Vec<u8>) {}

    pub const fn decode(_input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundFinishConfigurationPacket;

impl ClientboundFinishConfigurationPacket {
    pub const INSTANCE: Self = Self;

    pub const fn encode(self, _output: &mut Vec<u8>) {}

    pub const fn decode(_input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundAcceptCodeOfConductPacket;

impl ServerboundAcceptCodeOfConductPacket {
    pub const INSTANCE: Self = Self;

    pub const fn encode(self, _output: &mut Vec<u8>) {}

    pub const fn decode(_input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self)
    }
}

/// Empty clientbound reset-chat packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundResetChatPacket;

impl ClientboundResetChatPacket {
    pub const INSTANCE: Self = Self;

    pub const fn encode(self, _output: &mut Vec<u8>) {}

    pub const fn decode(_input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self)
    }
}

fn write_enum(value: i32, output: &mut Vec<u8>) {
    wire::write_varint(value, output);
}

fn read_enum(input: &mut &[u8]) -> Result<i32, ReadError> {
    wire::read_varint(input).map_err(ReadError::from)
}

fn read_byte(input: &mut &[u8]) -> Result<u8, ReadError> {
    let (&value, rest) = input.split_first().ok_or(ReadError::BufferUnderflow)?;
    *input = rest;
    Ok(value)
}
