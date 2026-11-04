//! Game page video ([ADR 0025](../../../docs/adr/0025-native-video.md)): an operator's WebM file
//! with 8-bit 4:2:0 AV1 video is fetched, demuxed, decoded and paced into RGBA frames. Audio is
//! not played.

mod convert;
mod decode;
mod demux;
mod fetch;
mod player;

pub use demux::{Packet, demux};
pub use fetch::{MAX_VIDEO_BYTES, fetch};
pub use player::{Frame, Player};

/// Why a clip could not be played.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoError {
    /// The address did not answer, or the answer was refused (scheme, redirect, status).
    Unavailable,
    /// More bytes than the cap.
    TooBig,
    /// Not a WebM file, or no AV1 video track in it.
    Container(&'static str),
    /// The decoder refused the stream, or the stream is not 8-bit 4:2:0.
    Codec(&'static str),
}
