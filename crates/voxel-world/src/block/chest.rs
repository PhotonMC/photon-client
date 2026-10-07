//! The block entities the client draws (`ChestBlockEntity`, `EnderChestBlockEntity`,
//! `ShulkerBoxBlockEntity`, `BellBlockEntity`, `SkullBlockEntity`): which blocks they are, the partner half of a double
//! chest, and the lid or the clapper that moves as the server's block events say.

use std::collections::HashMap;
use std::sync::OnceLock;

use voxel_core::direction::Direction;
use voxel_data::block::classes::BlockClass;

use crate::ClientWorld;

mod animations;
pub(crate) use animations::*;
mod views;
pub use views::*;
mod world;
