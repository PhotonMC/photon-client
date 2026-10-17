//! `ConduitBlockEntity.clientTick`: the conduit counts its ticks, and every two seconds works out
//! whether the water and the frame around it keep it active and hunting.

use photon_data::block::blocks::id;
use photon_data::block::definition::Fluid;

use crate::block::piston::Pos;
use crate::block::{self};
use crate::*;

/// `ConduitBlockEntity.VALID_BLOCKS`.
const FRAME_BLOCKS: [usize; 4] = [
    id::PRISMARINE,
    id::PRISMARINE_BRICKS,
    id::SEA_LANTERN,
    id::DARK_PRISMARINE,
];

/// How many frame blocks wake the conduit (`updateShape`).
const ACTIVE_FRAME_BLOCKS: usize = 16;
/// How many frame blocks make it hunt (`updateHunting`).
const HUNTING_FRAME_BLOCKS: usize = 42;

/// Whether an offset from the conduit is on the frame (`updateShape`): two blocks out along some
/// axis and in line with the conduit along another, which is the lines through the middle of the
/// faces of a cube of five, the inner cube of three left out.
fn on_frame((ox, oy, oz): (i32, i32, i32)) -> bool {
    let reach = ox.abs().max(oy.abs()).max(oz.abs());
    reach == 2 && (ox == 0 || oy == 0 || oz == 0)
}

impl ClientWorld {
    pub(super) fn is_conduit_at(&self, (x, y, z): Pos) -> bool {
        block::state_or_air(self.block_state_or_air(x, y, z)).block == id::CONDUIT as u16
    }

    /// `Level.isWaterAt`: a water fluid in the block.
    fn is_water_at(&self, (x, y, z): Pos) -> bool {
        let fluid = block::state_or_air(self.block_state_or_air(x, y, z)).fluid;
        !fluid.is_empty() && fluid.is_same(Fluid::Water)
    }

    /// `ConduitBlockEntity.updateShape`: the frame blocks the conduit has, or none if it is not
    /// wholly in water.
    fn conduit_frame_blocks(&self, (x, y, z): Pos) -> Vec<Pos> {
        let cube = |reach: i32| {
            (-reach..=reach).flat_map(move |ox| {
                (-reach..=reach).flat_map(move |oy| (-reach..=reach).map(move |oz| (ox, oy, oz)))
            })
        };
        if !cube(1).all(|(ox, oy, oz)| self.is_water_at((x + ox, y + oy, z + oz))) {
            return Vec::new();
        }
        cube(2)
            .filter(|offset| on_frame(*offset))
            .map(|(ox, oy, oz)| (x + ox, y + oy, z + oz))
            .filter(|at| {
                let state = block::state_or_air(self.block_state_or_air(at.0, at.1, at.2));
                FRAME_BLOCKS
                    .iter()
                    .any(|frame| state.block == *frame as u16)
            })
            .collect()
    }

    /// `ConduitBlockEntity.clientTick`: the specks are asked of the particle engine, which has
    /// the level's random source.
    pub(super) fn tick_conduit(&mut self, pos: Pos) {
        let time = self.game_time() as i64;
        let looked = (time % 40 == 0).then(|| self.conduit_frame_blocks(pos));
        let Some(entity) = self.block_entity_mut_at(pos) else {
            self.block_tickers.unregister(pos);
            return;
        };
        let conduit = &mut entity.events.conduit;
        conduit.tick_count += 1;
        if let Some(frame) = looked {
            // `updateShape` leaves the list empty when the water is not whole, and wakes the conduit
            // from sixteen blocks of frame.
            conduit.active = frame.len() >= ACTIVE_FRAME_BLOCKS;
            conduit.hunting = frame.len() >= HUNTING_FRAME_BLOCKS;
            conduit.frame = frame;
        }
        if conduit.active {
            conduit.rotation += 1;
        }
        let (tick_count, frame) = (conduit.tick_count, conduit.frame.clone());
        if !frame.is_empty() {
            self.level_events.0.push(LevelEvent::ConduitSpecks {
                pos,
                tick_count,
                frame,
            });
        }
    }
}
