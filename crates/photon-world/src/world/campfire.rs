//! `CampfireBlockEntity.particleTick`: a lit campfire sends up smoke, and the food on it smokes.

use crate::block::piston::Pos;
use crate::*;

/// What a lit campfire's tick asks of the particle engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CampfireSmoke {
    pub pos: (i32, i32, i32),
    /// `CampfireBlock.SIGNAL_FIRE`: the tall, dark smoke.
    pub signal: bool,
    /// `Direction.get2DDataValue` of its facing.
    pub rotation: i32,
    /// Which of its four slots hold food.
    pub cooking: [bool; 4],
}

impl ClientWorld {
    /// `CampfireBlock.dowse` on a client: the smoke of a campfire put out, for the particle
    /// engine.
    pub fn push_campfire_dowse(&mut self, pos: (i32, i32, i32), signal: bool) {
        self.level_events
            .0
            .push(LevelEvent::CampfireDowse { pos, signal });
    }

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
            }));
    }
}
