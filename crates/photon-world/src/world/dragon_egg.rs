//! `DragonEggBlock.teleport` as a client runs it (before 26.3): a hit or a use of the egg makes
//! the level search for a place to send it and show the portal specks; the server moves the egg.

use photon_data::block::classes::BlockClass;

use super::sounds::LevelEvent;
use crate::*;

impl ClientWorld {
    /// A hit or a use of the block at `pos`: a dragon egg queues its teleport (the particle engine
    /// searches with the level's random source and draws the specks). Anything else, and a
    /// version whose server tells of the teleport, is left alone.
    pub fn touch_dragon_egg(&mut self, pos: (i32, i32, i32)) {
        if !photon_core::versions::rules()
            .world
            .dragon_egg_teleports_on_client
        {
            return;
        }
        let state = self.block_state_or_air(pos.0, pos.1, pos.2);
        if block::state_or_air(state).block().class == BlockClass::DragonEggBlock {
            self.level_events.0.push(LevelEvent::EggTeleport { pos });
        }
    }
}
