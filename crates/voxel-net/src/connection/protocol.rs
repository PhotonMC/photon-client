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
use voxel_world::objects::minecart_track::MinecartStep;
use voxel_world::{
    ClientAttributeModifierState, ClientAttributeState, ClientItemStackState, ClientPlayerGameMode,
    ClientWorld, DimensionBounds, DimensionEnvironment,
};

#[cfg(test)]
mod tests;

mod block;
pub use block::*;
mod border;
pub(crate) use border::*;
mod chat;
pub use chat::*;
mod chunk;
pub use chunk::*;
mod commands;
pub use commands::*;
mod container;
pub use container::*;
mod components;
pub(crate) use components::*;
mod error;
mod explode;
pub use error::*;
pub(crate) use explode::*;
mod packet;
pub(crate) use packet::*;
mod hud;
pub use hud::*;
mod ids;
pub(crate) use ids::*;
mod inventory;
pub(crate) use inventory::*;
mod limits;
pub(crate) use limits::*;
mod map;
pub(crate) use map::*;
mod merchant;
pub(crate) use merchant::*;
mod movement;
pub use movement::*;
mod player;
pub use player::*;
mod particles;
pub use particles::*;
mod player_info;
pub use player_info::*;
mod reader;
pub(crate) use reader::*;
mod recipe;
pub(crate) use recipe::*;
mod scoreboard;
pub(crate) use scoreboard::*;
mod serverbound;
pub use serverbound::*;
mod sound;
pub use sound::*;
mod spawn;
pub use spawn::*;
mod state;
pub use state::*;
mod trace;
pub use trace::*;
