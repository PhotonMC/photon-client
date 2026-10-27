//! `PotentSulfurBlockEntity` as the client runs it: the water above a sulfur block, where its gas
//! rises from, and the ticker that sends clouds of it up.

use photon_core::aabb::Aabb;
use photon_core::ids::BlockStateId;
use photon_core::math::Vec3;
use photon_data::block::blocks::id;
use photon_data::block::definition::Fluid;

use super::sounds::LevelEvent;
use crate::block::piston::Pos;
use crate::block::{self, rules};
use crate::local_player::LocalPlayer;
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
/// `PotentSulfurBlockEntity.SOUND_FREQUENCY_TICKS`.
const SOUND_FREQUENCY_TICKS: i64 = 40;
/// `SoundEvents.GEYSER_ERUPTION_ACTIVE` and `GEYSER_CONTINUOUS_ACTIVE`.
const ERUPTION_SOUND: &str = "minecraft:block.potent_sulfur.geyser_eruption_active";
const CONTINUOUS_SOUND: &str = "minecraft:block.potent_sulfur.geyser_continuous_eruption_active";

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

    /// What a potent sulfur block's ticker does this tick, by its state.
    pub(super) fn tick_potent_sulfur(&mut self, pos: Pos, player: Option<&mut LocalPlayer>) {
        let state = self.block_state_or_air(pos.0, pos.1, pos.2);
        if self.block_entity_at(pos.0, pos.1, pos.2).is_none() {
            self.block_tickers.unregister(pos);
            return;
        }
        match SulfurState::of(state) {
            SulfurState::Wet | SulfurState::Dormant => self.tick_noxious_gas(pos),
            SulfurState::Erupting => {
                self.tick_geyser_plume(pos, ERUPTION_SOUND);
                self.launch_entities(pos, player);
            }
            SulfurState::Continuous => {
                self.tick_geyser_plume(pos, CONTINUOUS_SOUND);
                self.launch_entities(pos, player);
            }
            SulfurState::Dry => {}
        }
    }

    /// `LAUNCH_ENTITY_TICKER`: the entities in the geyser's column go up. The client only
    /// simulates the local player, so the others are left to the server.
    fn launch_entities(&self, pos: Pos, player: Option<&mut LocalPlayer>) {
        let (Some(source), Some(player)) = (self.noxious_gas_source(pos), player) else {
            return;
        };
        let water_blocks = source.1 - pos.1 - 1;
        let height = self.unobstructed_block_count((pos.0, pos.1 + 1, pos.2), water_blocks);
        let column = Aabb::new(
            f64::from(pos.0),
            f64::from(pos.1 + 1),
            f64::from(pos.2),
            f64::from(pos.0 + 1),
            f64::from(pos.1 + 2),
            f64::from(pos.2 + 1),
        )
        .expand_towards(0.0, f64::from(height - 1), 0.0);
        player.launched_by_geyser(column, water_blocks);
    }

    /// `getUnobstructedBlockCount`: how many cells up from a place the geyser's force reaches,
    /// at most six for each block of water.
    fn unobstructed_block_count(&self, (x, y, z): Pos, water_blocks: i32) -> i32 {
        let force_height = 6 * water_blocks;
        (0..force_height)
            .find(|&i| !self.is_geyser_passable((x, y + i, z)))
            .unwrap_or(force_height)
    }

    /// `PotentSulfurBlockEntity.CLIENT_GEYSER_PLUME_TICKER`: every second since it erupted, the
    /// geyser goes up from the cell the gas rises into; every two, its sound plays there.
    fn tick_geyser_plume(&mut self, pos: Pos, sound: &'static str) {
        let Some(source) = self.noxious_gas_source(pos) else {
            return;
        };
        let Some(erupted) = self
            .block_entity_at(pos.0, pos.1, pos.2)
            .map(|entity| entity.events().eruption_tick)
        else {
            return;
        };
        let time = self.game_time() as i64 - erupted;
        let (x, y, z) = (
            f64::from(source.0) + 0.5,
            f64::from(source.1),
            f64::from(source.2) + 0.5,
        );
        if time % PARTICLE_FREQUENCY_TICKS as i64 == 0 {
            let water_blocks = source.1 - pos.1 - 1;
            self.level_events.0.push(LevelEvent::Geyser {
                at: (x, y, z),
                water_blocks,
            });
        }
        if time % SOUND_FREQUENCY_TICKS == 0 {
            let at = (x, y + 0.5, z);
            self.level_events
                .0
                .push(LevelEvent::GeyserSound { at, sound });
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
