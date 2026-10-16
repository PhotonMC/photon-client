//! Sound for the client: decoding the game's `.ogg` files (`JOrbisAudioStream`), mixing the
//! sounds that play into the stereo the device takes (the model of an OpenAL source with a
//! linear distance fall-off), and the device output.

pub mod channel;
pub mod decode;
pub mod library;
pub mod mixer;
#[cfg(feature = "device")]
pub mod output;
