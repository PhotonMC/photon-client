//! `CampfireBlockEntity.particleTick`: a lit campfire sends up smoke, and the food on it smokes.

use super::sounds::CampfireSmoke;
use crate::block::piston::Pos;
use crate::*;

impl ClientWorld {
    /// The ticker a lit campfire has on a client: the smoke is asked of the particle engine,
    /// which draws it from the level's random source. A campfire that went out loses the ticker
    /// with its state; one that is not there any more loses it here.
    pub(super) fn tick_campfire(&mut self, pos: Pos) {
        let state = self.block_state_or_air(pos.0, pos.1, pos.2);
        let property = |name| crate::block::rules::property(state, name);
        if property("lit") != Some("true") {
            self.block_tickers.unregister(pos);
            return;
        }
        let cooking = self
            .campfire_view(pos)
            .map_or([false; 4], |view| view.items.map(|slot| slot.is_some()));
        // `Direction.get2DDataValue`: south, west, north, east.
        let rotation = match property("facing") {
            Some("south") => 0,
            Some("west") => 1,
            Some("north") => 2,
            _ => 3,
        };
        self.level_events
            .0
            .push(LevelEvent::CampfireTick(CampfireSmoke {
                pos,
                signal: property("signal_fire") == Some("true"),
                rotation,
                cooking,
                dowsed: false,
            }));
    }
}
