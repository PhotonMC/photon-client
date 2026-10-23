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

use photon_core::math::Vec3;
use photon_data::item::classes::ItemClass;
use photon_world::block::rules::{self as rules, Rule, UseResult};
use photon_world::local_player::LocalPlayer;
use photon_world::{ClientItemStackState, ClientPlayerGameMode, ClientWorld};

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
    /// The hand whose book and quill was used, whose screen is to open (`openItemGui`).
    pub(super) opened_book: Option<u8>,
    /// The command block whose screen is to open (`Player.openCommandBlock`).
    pub(super) opened_command_block: Option<(i32, i32, i32)>,
    /// The structure block whose screen is to open (`Player.openStructureBlock`).
    pub(super) opened_structure_block: Option<(i32, i32, i32)>,
    /// The jigsaw block whose screen is to open (`Player.openJigsawBlock`).
    pub(super) opened_jigsaw_block: Option<(i32, i32, i32)>,
    /// The test block whose screen is to open (`Player.openTestBlock`).
    pub(super) opened_test_block: Option<(i32, i32, i32)>,
    /// The test instance block whose screen is to open (`Player.openTestInstanceBlock`).
    pub(super) opened_test_instance_block: Option<(i32, i32, i32)>,
    /// The command block minecart whose screen is to open (`Player.openMinecartCommandBlock`).
    pub(super) opened_command_minecart: Option<photon_core::ids::EntityId>,
    /// The block states of the breaks the tutorial is told of, and how far each was.
    pub(super) destroyed: Vec<(photon_core::ids::BlockStateId, f32)>,
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
            opened_book: None,
            opened_command_block: None,
            opened_structure_block: None,
            opened_jigsaw_block: None,
            opened_test_block: None,
            opened_test_instance_block: None,
            opened_command_minecart: None,
            destroyed: Vec::new(),
        }
    }
}

mod base_use;
mod buckets;
mod context;
mod stab;
mod tools;
pub use context::*;
mod destroying;
mod mining;
pub use mining::*;
mod outgoing;
pub use outgoing::*;
mod place_prediction;
mod placing;
mod using;
