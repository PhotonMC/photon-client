//! `LevelChunk.setBlockState`'s hand in the block entities the client ticks: a block that has an
//! entity of its own has one when its state is set, and the ticker its state asks for.

use photon_core::ids::BlockStateId;

use super::has_sulfur_ticker;
use crate::block::piston::Pos;
use crate::block::{self};
use crate::*;

/// The type of the block entity a block of this name keeps, where the client ticks it.
fn ticking_entity_of(block_name: &str) -> Option<&'static str> {
    match block_name {
        "minecraft:potent_sulfur" => Some("minecraft:potent_sulfur"),
        "minecraft:enchanting_table" => Some("minecraft:enchanting_table"),
        "minecraft:beacon" => Some("minecraft:beacon"),
        "minecraft:campfire" | "minecraft:soul_campfire" => Some("minecraft:campfire"),
        _ => None,
    }
}

/// Whether the block entity keeps its ticker in this state: a dry sulfur block and an unlit
/// campfire have none.
fn keeps_ticker(kind: &str, state: BlockStateId) -> bool {
    match kind {
        "minecraft:potent_sulfur" => has_sulfur_ticker(state),
        "minecraft:campfire" => block::rules::property(state, "lit") == Some("true"),
        _ => true,
    }
}

impl ClientWorld {
    /// `LevelChunk.updateBlockEntityTicker` for a block whose state was set: its entity is made
    /// when there is none, and has a ticker or has none as the state says (a dry sulfur block has
    /// none), in the slot it had if it had one.
    pub(super) fn rebind_ticker(&mut self, pos: Pos, state: BlockStateId) {
        let Some(kind) = ticking_entity_of(block::state_or_air(state).block().name) else {
            return;
        };
        if self.block_entity_or_new(pos, kind).is_none() {
            return;
        }
        if keeps_ticker(kind, state) {
            self.block_tickers.register(pos);
        } else {
            self.block_tickers.unregister(pos);
        }
    }
}
