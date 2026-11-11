//! What a block declares in the active version. The block table is the newest version's; an older
//! version differs in a few blocks (`tables/block_older.rs`), and these read the version's own
//! value.

use super::definition::{Block, PushReaction, Rule};
use crate::versions::block_patches::{self, BlockPatch};

impl Block {
    fn patch(&self) -> Option<&'static BlockPatch> {
        block_patches::of(self.name)
    }

    /// The blast resistance the active version declares.
    pub fn declared_explosion_resistance(&self) -> f32 {
        self.patch()
            .and_then(|patch| patch.explosion_resistance)
            .unwrap_or(self.explosion_resistance)
    }

    /// Whether the active version declares the block solid whatever its shape.
    pub fn declared_force_solid_on(&self) -> bool {
        self.patch()
            .and_then(|patch| patch.force_solid_on)
            .unwrap_or(self.force_solid_on)
    }

    /// The push reaction the active version declares.
    pub fn declared_push_reaction(&self) -> PushReaction {
        self.patch()
            .and_then(|patch| patch.push_reaction)
            .unwrap_or(self.push_reaction)
    }

    /// The view blocking rule the active version declares, as the version states it.
    pub fn declared_view_blocking(&self) -> Rule {
        self.patch()
            .and_then(|patch| patch.view_blocking)
            .unwrap_or(self.view_blocking)
    }

    /// Whether an older version names a view blocking rule for the block itself, where most
    /// blocks derive it. False for every block in the newest version, which has no patches.
    pub fn names_view_blocking(&self) -> bool {
        self.patch()
            .is_some_and(|patch| patch.view_blocking.is_some())
    }
}
