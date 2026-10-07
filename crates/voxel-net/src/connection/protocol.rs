//! Packet codecs, wire observations, and frame serialization for the Java protocol.

use std::collections::BTreeSet;
use std::fmt;
use std::io::{self, Read, Write};
use std::time::Duration;

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::frame::{self, FrameError, MAX_FRAME_LENGTH};
use crate::protocol::ConnectionProtocol;
use voxel_core::binary::{BitSet, Uuid};
use voxel_core::wire;
use voxel_world::minecart_track::MinecartStep;
use voxel_world::{
    ClientAttributeModifierState, ClientAttributeState, ClientItemStackState, ClientPlayerGameMode,
    ClientWorld, DimensionBounds, DimensionEnvironment,
};

#[cfg(test)]
#[path = "protocol_tests/mod.rs"]
mod tests;

mod inventory;
pub(crate) use inventory::*;
mod recipe;
pub(crate) use recipe::*;
mod entity;
pub use entity::*;
mod entity_2;
pub(crate) use entity_2::*;
mod sound;
pub use sound::*;
mod boss;
pub use boss::*;
mod chat;
pub(crate) use chat::*;
mod section;
pub use section::*;

mod component;
pub(crate) use component::*;
mod error;
pub use error::*;
mod entity_3;
pub use entity_3::*;
mod chat_2;
pub use chat_2::*;
mod signature;
pub(crate) use signature::*;
