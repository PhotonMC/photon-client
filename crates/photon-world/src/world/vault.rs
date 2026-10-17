//! `VaultBlockEntity.Client.tick`: the display item's spin, and what the vault asks of the level's
//! random source every tick, which the particle engine holds.

use crate::block::piston::Pos;
use crate::*;

/// `VaultClientData.ROTATION_SPEED`.
const ROTATION_SPEED: i32 = 10;

/// `Mth.wrapDegrees` of a whole number of degrees: into a half turn either way, the half turn
/// back itself wrapping round.
fn wrap_degrees(degrees: i32) -> i32 {
    photon_core::math::wrap_degrees_i32(degrees)
}

impl ClientWorld {
    /// `VaultBlockEntity.Client.tick`: the spin turns, and the vault's particles and sound are
    /// asked of the particle engine.
    pub(super) fn tick_vault(&mut self, pos: Pos) {
        let Some(entity) = self.block_entity_mut_at(pos) else {
            self.block_tickers.unregister(pos);
            return;
        };
        let spin = &mut entity.events.vault;
        spin.previous = spin.current;
        spin.current = wrap_degrees(spin.current + ROTATION_SPEED);
        self.level_events.0.push(LevelEvent::VaultTick { pos });
    }
}
