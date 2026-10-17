//! `PotentSulfurBlockEntity` as the client runs it: the water above a sulfur block, where its gas
//! rises from, and the ticker that sends clouds of it up.

use photon_core::ids::BlockStateId;
use photon_core::math::Vec3;
use photon_data::block::blocks::id;
use photon_data::block::definition::Fluid;

use super::sounds::LevelEvent;
use crate::block::piston::Pos;
use crate::block::{self, rules};
use crate::*;

/// `PotentSulfurState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SulfurState {
    Dry,
    Wet,
    Dormant,
    Erupting,
    Continuous,
}

impl SulfurState {
    /// The state of a potent sulfur block; dry for any other.
    pub(crate) fn of(state: BlockStateId) -> Self {
        match rules::property(state, "potent_sulfur_state") {
            Some("wet") => Self::Wet,
            Some("dormant") => Self::Dormant,
            Some("erupting") => Self::Erupting,
            Some("continuous") => Self::Continuous,
            _ => Self::Dry,
        }
    }
}

/// `PotentSulfurBlock.getTicker` on the client: a dry block has none.
pub(crate) fn has_sulfur_ticker(state: BlockStateId) -> bool {
    block::state_or_air(state).block == id::POTENT_SULFUR as u16
        && SulfurState::of(state) != SulfurState::Dry
}

/// `PotentSulfurBlock.ALLOWED_WATER_BLOCKS_ABOVE`.
const ALLOWED_WATER_BLOCKS_ABOVE: i32 = 4;
/// `PotentSulfurBlockEntity.PARTICLE_FREQUENCY_TICKS`.
const PARTICLE_FREQUENCY_TICKS: u64 = 20;

impl ClientWorld {
    /// `getFluidState(pos).isSourceOfType(Fluids.WATER)`.
    pub(crate) fn is_water_source_at(&self, (x, y, z): Pos) -> bool {
        block::state_or_air(self.block_state_or_air(x, y, z))
            .fluid
            .fluid
            == Fluid::Water
    }

    /// `PotentSulfurBlockEntity.isGeyserPassableBlock`: air, water, or a block with nothing to
    /// collide with. The collision shape here is read without the position context the reference
    /// gives it, which only blocks that look at who asks (scaffolding) answer differently to.
    fn is_geyser_passable(&self, (x, y, z): Pos) -> bool {
        let state = self.block_state_or_air(x, y, z);
        let info = block::state_or_air(state);
        if rules::state_is_air(state) || info.block == id::WATER as u16 {
            return true;
        }
        self.block_state_registry()
            .collision_shape_at(state, [x, y, z])
            .is_empty()
    }

    /// `findNoxiousGasSourceBlock`: the first cell above a sulfur block, within four of water, that
    /// is not water the gas could rise through; none if there is no such cell in reach.
    pub(crate) fn noxious_gas_source(&self, origin: Pos) -> Option<Pos> {
        let max_y = origin.1 + ALLOWED_WATER_BLOCKS_ABOVE + 1;
        let mut pos = (origin.0, origin.1 + 1, origin.2);
        while pos.1 <= max_y {
            let state = self.block_state_or_air(pos.0, pos.1, pos.2);
            let is_water_logged = self.is_water_source_at(pos);
            let is_water = block::state_or_air(state).block == id::WATER as u16;
            if !is_water_logged || (!is_water && !self.is_geyser_passable(pos)) {
                if rules::state_is_air(state) || self.is_geyser_passable(pos) {
                    return Some(pos);
                }
                break;
            }
            pos.1 += 1;
        }
        None
    }

    /// `canBeReachedByNoxiousGas`: a point in a cell the gas passes, within three blocks of the
    /// source, over water the source's floor sees without a block between.
    pub fn noxious_gas_reaches(&self, source: Pos, at: Vec3) -> bool {
        let cell = [at.x, at.y, at.z].map(photon_core::math::floor_f64);
        if !self.is_geyser_passable((cell[0], cell[1], cell[2])) {
            return false;
        }
        let centre = Vec3::new(
            f64::from(source.0) + 0.5,
            f64::from(source.1) + 0.5,
            f64::from(source.2) + 0.5,
        );
        if at.distance_to_sqr(centre) > 9.0 {
            return false;
        }
        let below_source = Vec3::new(centre.x, centre.y - 1.0, centre.z);
        let below = Vec3::new(at.x, at.y - 1.0, at.z);
        let water_cell = [below.x, below.y, below.z].map(photon_core::math::floor_f64);
        self.is_water_source_at((water_cell[0], water_cell[1], water_cell[2]))
            && self.clip_blocks(below_source, below).is_none()
    }

    /// `PotentSulfurBlockEntity.CLIENT_NOXIOUS_GAS_TICKER`: once a second, a cloud at the middle of
    /// the cell the gas rises into.
    fn tick_noxious_gas(&mut self, pos: Pos) {
        if !self.game_time().is_multiple_of(PARTICLE_FREQUENCY_TICKS) {
            return;
        }
        if let Some(source) = self.noxious_gas_source(pos) {
            let at = (
                f64::from(source.0) + 0.5,
                f64::from(source.1) + 0.5,
                f64::from(source.2) + 0.5,
            );
            self.level_events.0.push(LevelEvent::NoxiousGasCloud { at });
        }
    }

    /// Whether the block at a place is a potent sulfur block.
    pub(super) fn is_potent_sulfur_at(&self, (x, y, z): Pos) -> bool {
        block::state_or_air(self.block_state_or_air(x, y, z)).block == id::POTENT_SULFUR as u16
    }

    /// `LevelChunk.updateBlockEntityTicker` for a potent sulfur block whose state was set: the
    /// block has an entity of its own (made when it has none), and a ticker unless it is dry.
    pub(super) fn rebind_sulfur_ticker(&mut self, pos: Pos, state: BlockStateId) {
        if !self.is_potent_sulfur_at(pos) {
            return;
        }
        if self
            .block_entity_or_new(pos, "minecraft:potent_sulfur")
            .is_none()
        {
            return;
        }
        if has_sulfur_ticker(state) {
            self.block_tickers.register(pos);
        } else {
            self.block_tickers.unregister(pos);
        }
    }

    /// What a potent sulfur block's ticker does this tick, by its state.
    pub(super) fn tick_potent_sulfur(&mut self, pos: Pos) {
        let state = self.block_state_or_air(pos.0, pos.1, pos.2);
        if self.block_entity_at(pos.0, pos.1, pos.2).is_none() {
            self.block_tickers.unregister(pos);
            return;
        }
        match SulfurState::of(state) {
            SulfurState::Wet | SulfurState::Dormant => self.tick_noxious_gas(pos),
            SulfurState::Dry | SulfurState::Erupting | SulfurState::Continuous => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::chunk::SectionInput;

    fn sulfur(state: &str) -> BlockStateId {
        let chosen = BTreeMap::from([("potent_sulfur_state".to_owned(), state.to_owned())]);
        photon_data::block::registry::BLOCKS[id::POTENT_SULFUR]
            .state_id(&chosen)
            .expect("a state")
    }

    fn world() -> ClientWorld {
        let blocks = vec![0; 4_096];
        let biomes = vec![0; 64];
        let sections = (0..24).map(|_| SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        });
        let mut world = ClientWorld::default();
        world.replace_chunk(0, 0, sections.collect::<Vec<_>>());
        world
    }

    fn ticking(world: &ClientWorld) -> Vec<Pos> {
        world.block_tickers.positions().collect()
    }

    /// A block that dries has its ticker dropped, and one that gets wet again takes a turn after
    /// the ones that kept theirs.
    #[test]
    fn a_ticker_is_rebound_with_the_state() {
        let mut world = world();
        for x in 0..3 {
            assert!(world.update_block_state(x, 64, 0, sulfur("wet")));
        }
        assert_eq!(ticking(&world), [(0, 64, 0), (1, 64, 0), (2, 64, 0)]);
        assert!(world.update_block_state(0, 64, 0, sulfur("dry")));
        assert_eq!(ticking(&world), [(1, 64, 0), (2, 64, 0)]);
        assert!(world.update_block_state(0, 64, 0, sulfur("dormant")));
        assert_eq!(ticking(&world), [(1, 64, 0), (2, 64, 0), (0, 64, 0)]);
        // A change between states that tick keeps the turn.
        assert!(world.update_block_state(1, 64, 0, sulfur("erupting")));
        assert_eq!(ticking(&world), [(1, 64, 0), (2, 64, 0), (0, 64, 0)]);
    }

    /// A block that is dry has an entity of its own and no ticker.
    #[test]
    fn a_dry_block_has_no_ticker() {
        let mut world = world();
        assert!(world.update_block_state(5, 64, 5, sulfur("dry")));
        assert!(world.block_entity_at(5, 64, 5).is_some());
        assert!(ticking(&world).is_empty());
    }
}
