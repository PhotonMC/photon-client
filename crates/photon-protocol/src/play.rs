//! Packet codecs, wire observations, and frame serialization for the Java protocol.

use std::collections::BTreeSet;
use std::fmt;
use std::io::{self, Read, Write};
use std::time::Duration;

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::frame::{self, FrameError, MAX_FRAME_LENGTH};
use crate::phase::ConnectionProtocol;
use photon_core::binary::{BitSet, Uuid};
use photon_core::wire;
use photon_world::objects::minecart_track::MinecartStep;
use photon_world::{
    ClientAttributeModifierState, ClientAttributeState, ClientItemStackState, ClientPlayerGameMode,
};

#[cfg(test)]
mod tests;

mod block;
pub use block::*;
mod border;
pub use border::*;
mod chat;
pub use chat::*;
mod clientbound;
pub use clientbound::*;
mod chunk;
pub use chunk::*;
mod commands;
pub use commands::*;
mod dialog;
pub use dialog::DialogEvent;
pub use dialog::{ClearDialog, DialogHolder, ShowDialog, encode_custom_click_action};
mod container;
pub use container::*;
mod components;
pub use components::*;
mod cookies;
pub use cookies::*;
mod error;
mod explode;
pub use error::*;
pub use explode::*;
mod packet;
pub use packet::*;
mod hud;
pub use hud::*;
mod ids;
pub use ids::*;
mod inventory;
pub use inventory::*;
mod limits;
pub use limits::*;
mod map;
pub use map::*;
mod merchant;
pub use merchant::*;
mod movement;
pub use movement::*;
mod player;
pub use player::*;
mod particles;
pub use particles::*;
mod player_info;
pub use player_info::*;
mod reader;
pub use reader::*;
mod recipe;
pub use recipe::*;
mod resource_pack;
pub use resource_pack::*;
mod scoreboard;
pub use scoreboard::*;
mod serverbound;
pub use serverbound::*;
mod sound;
pub use sound::*;
mod spawn;
pub use spawn::*;
mod state;
pub use state::*;
