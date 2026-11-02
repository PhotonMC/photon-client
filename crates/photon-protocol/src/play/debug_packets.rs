//! Packets of the debug overlay's charts and of the game test highlight.

use super::*;

/// `ClientboundPongResponsePacket` in the play state: the time the client's ping request sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PongResponsePacket {
    pub time: i64,
}

impl Decode for PongResponsePacket {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let time = read_i64(input, "pong response time")?;
        if !input.is_empty() {
            return Err(ConnectionError::Codec("pong response trailing payload"));
        }
        Ok(Self { time })
    }
}

/// `ClientboundDebugSamplePacket`: one sample of the server's tick times. The only
/// `RemoteDebugSampleType` is the tick time, so any other number is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSamplePacket {
    pub sample: Vec<i64>,
}

impl Decode for DebugSamplePacket {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let sample = read_long_array(input, "debug sample")?;
        if read_varint(input, "debug sample type")? != 0 {
            return Err(ConnectionError::Codec("debug sample type"));
        }
        if !input.is_empty() {
            return Err(ConnectionError::Codec("debug sample trailing payload"));
        }
        Ok(Self { sample })
    }
}

/// `ClientboundGameTestHighlightPosPacket`: a block to mark and the position the test names it by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameTestHighlightPos {
    pub absolute: (i32, i32, i32),
    pub relative: (i32, i32, i32),
}

impl Decode for GameTestHighlightPos {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let mut pos = |name| -> Result<(i32, i32, i32), ConnectionError> {
            let pos = photon_core::binary::BlockPos::unpack(read_i64(input, name)? as u64);
            Ok((pos.x, pos.y, pos.z))
        };
        let absolute = pos("game test absolute position")?;
        let relative = pos("game test relative position")?;
        if !input.is_empty() {
            return Err(ConnectionError::Codec(
                "game test highlight trailing payload",
            ));
        }
        Ok(Self { absolute, relative })
    }
}
