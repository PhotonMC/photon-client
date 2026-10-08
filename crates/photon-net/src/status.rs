//! Status request and ping packet codecs for the pinned Java protocol.

use photon_core::binary;

pub type ReadError = binary::ReadError;

/// The status request has no payload and always decodes to its singleton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundStatusRequestPacket;

impl ServerboundStatusRequestPacket {
    pub const INSTANCE: Self = Self;

    pub const fn encode(self, _output: &mut Vec<u8>) {}

    pub const fn decode(_input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self)
    }
}

/// Serverbound status ping carrying one big-endian signed long.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPingRequestPacket {
    pub time: i64,
}

impl ServerboundPingRequestPacket {
    pub const fn new(time: i64) -> Self {
        Self { time }
    }

    pub fn encode(self, output: &mut Vec<u8>) {
        write_time(self.time, output);
    }

    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self::new(read_time(input)?))
    }
}

/// Clientbound status pong carrying one big-endian signed long.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPongResponsePacket {
    pub time: i64,
}

impl ClientboundPongResponsePacket {
    pub const fn new(time: i64) -> Self {
        Self { time }
    }

    pub fn encode(self, output: &mut Vec<u8>) {
        write_time(self.time, output);
    }

    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self::new(read_time(input)?))
    }
}

fn write_time(time: i64, output: &mut Vec<u8>) {
    output.extend_from_slice(&time.to_be_bytes());
}

fn read_time(input: &mut &[u8]) -> Result<i64, ReadError> {
    photon_core::wire::take(input)
        .map(i64::from_be_bytes)
        .ok_or(ReadError::BufferUnderflow)
}
