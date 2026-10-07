//! Breaking blocks: `MultiPlayerGameMode`'s destroy state and the attack key
//! as `Minecraft.startAttack` and `continueAttack` spend it.
//!
//! Each tick the loop first spends the attack clicks, each starting an
//! attack on what the player aims at, then lets the held key keep breaking
//! the aimed block, then counts the miss wait down. Breaking accumulates the
//! block's destroy progress every tick; a finished block, and every block in
//! creative, is removed at once as a prediction the server confirms or
//! reverts. The packets the reference would send come back in order.
//!
//! Not ported: block particles and hit sounds, `BlockState.attack` and
//! `playerWillDestroy` side effects, neighbour shape updates after a
//! predicted break, entity targets, piercing weapons, item-specific attack
//! charge, adventure mode `can_break` predicates, and game master blocks for
//! operators (the permission level is not tracked; they are never broken).

use voxel_core::math::Vec3;
use voxel_data::item::classes::ItemClass;
use voxel_world::block::rules::{self as rules, Rule, UseResult};
use voxel_world::local_player::LocalPlayer;
use voxel_world::{ClientItemStackState, ClientPlayerGameMode, ClientWorld};

/// `MultiPlayerGameMode`'s breaking state and `Minecraft.missTime`.
#[derive(Debug, Clone)]
pub struct GameMode {
    pub(super) destroy_pos: (i32, i32, i32),
    pub(super) destroying_item: Option<ClientItemStackState>,
    pub(super) destroy_progress: f32,
    pub(super) destroy_ticks: f32,
    pub(super) destroy_delay: i32,
    pub(super) is_destroying: bool,
    pub(super) miss_time: i32,
    /// `Minecraft.rightClickDelay`.
    pub(super) right_click_delay: i32,
    /// Hands whose stack was used since the caller last asked:
    /// `ItemInHandRenderer.itemUsed`.
    pub(super) items_used: Vec<u8>,
}

impl Default for GameMode {
    fn default() -> Self {
        Self {
            destroy_pos: (-1, -1, -1),
            destroying_item: None,
            destroy_progress: 0.0,
            destroy_ticks: 0.0,
            destroy_delay: 0,
            is_destroying: false,
            miss_time: 0,
            right_click_delay: 0,
            items_used: Vec::new(),
        }
    }
}

mod context;
pub use context::*;
mod destroying;
mod mining;
pub use mining::*;
mod outgoing;
pub use outgoing::*;
mod using;
