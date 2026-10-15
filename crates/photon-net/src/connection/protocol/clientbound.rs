//! The play-phase packets the server sends, parsed. Parsing is pure and happens here, on the
//! reader thread; what a packet does to the client's state is the listener's (`PlaySession`).
//! The order the packets came in is kept, so an error surfaces where it always did.

use super::*;

/// A packet that parses itself from its payload (the bytes after the packet id).
pub(crate) trait Decode: Sized {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError>;
}

/// A clientbound play packet as the reader thread hands it over.
pub(crate) enum ClientboundPlay {
    /// A packet whose family is not typed yet: its whole body, parsed where it is handled.
    Raw(Vec<u8>),
    Chunk(Box<ChunkPacketObservation>),
    Light(Box<LightUpdateObservation>),
    InitializeBorder(photon_world::ClientWorldBorderInitialization),
    SetBorderCenter(SetBorderCenter),
    SetBorderLerpSize {
        old_size: f64,
        new_size: f64,
        time: i64,
    },
    SetBorderSize(SetBorderSize),
    SetBorderWarningDelay(SetBorderWarningDelay),
    SetBorderWarningDistance(SetBorderWarningDistance),
}

impl ClientboundPlay {
    /// Parse the packet in `body` (its id, then its payload).
    pub(crate) fn decode(body: Vec<u8>) -> Result<Self, ConnectionError> {
        let (packet_id, payload) = split_packet(&body)?;
        Ok(match packet_id {
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => Self::Chunk(Box::new(parse_chunk_packet(payload)?)),
            PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID => {
                Self::Light(Box::new(parse_light_update_packet(payload)?))
            }
            PLAY_CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID => {
                Self::InitializeBorder(parse_initialize_border_packet(payload)?)
            }
            PLAY_CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID => {
                Self::SetBorderCenter(Decode::decode(payload)?)
            }
            PLAY_CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID => {
                let (old_size, new_size, time) = parse_set_border_lerp_size_packet(payload)?;
                Self::SetBorderLerpSize {
                    old_size,
                    new_size,
                    time,
                }
            }
            PLAY_CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID => {
                Self::SetBorderSize(Decode::decode(payload)?)
            }
            PLAY_CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID => {
                Self::SetBorderWarningDelay(Decode::decode(payload)?)
            }
            PLAY_CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID => {
                Self::SetBorderWarningDistance(Decode::decode(payload)?)
            }
            _ => Self::Raw(body),
        })
    }
}
