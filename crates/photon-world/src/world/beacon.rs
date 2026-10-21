//! `BeaconBlockEntity.tick` as the client runs it: the beam is worked out ten blocks of the
//! column at a time, and shown once the scan has reached the top.

use photon_data::block::blocks::id;
use photon_data::block::classes::BlockClass;

use crate::block::entity::{BeaconState, BeamSection};
use crate::block::piston::Pos;
use crate::block::{self};
use crate::*;

/// `BeaconBlockEntity.BLOCKS_CHECK_PER_TICK`.
const BLOCKS_PER_TICK: i32 = 10;

/// What a beacon's beam is drawn from at a partial tick (`BeaconRenderState`, before the camera's
/// distance scales it).
#[derive(Debug, Clone, PartialEq)]
pub struct BeaconView {
    pub position: Pos,
    pub sections: Vec<BeamSection>,
    /// `Math.floorMod(gameTime, 40) + partialTicks`.
    pub animation_time: f32,
}

/// The colour a block gives the beam, if it is one that lets the beam through and tints it
/// (`BeaconBeamBlock.getColor().getTextureDiffuseColor()`).
fn beam_color(state: BlockStateId) -> Option<u32> {
    let info = block::state_or_air(state).block();
    let class = info.class;
    let dye = if class.is(BlockClass::BeaconBlock) {
        "white"
    } else if class.is(BlockClass::StainedGlassBlock) {
        info.name
            .strip_prefix("minecraft:")?
            .strip_suffix("_stained_glass")?
    } else if class.is(BlockClass::StainedGlassPaneBlock) {
        info.name
            .strip_prefix("minecraft:")?
            .strip_suffix("_stained_glass_pane")?
    } else {
        return None;
    };
    let index = DYES.iter().position(|candidate| *candidate == dye)?;
    photon_data::entity_type::dye_texture_colour(index).map(|colour| 0xFF00_0000 | colour)
}

/// `ARGB.average`: each channel, the alpha too, the two added and halved as integers.
fn average(left: u32, right: u32) -> u32 {
    let channel = |shift: u32| (((left >> shift) & 0xFF) + ((right >> shift) & 0xFF)) / 2;
    (channel(24) << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

impl ClientWorld {
    pub(super) fn is_beacon_at(&self, (x, y, z): Pos) -> bool {
        block::state_or_air(self.block_state_or_air(x, y, z)).block == id::BEACON as u16
    }

    /// `BeaconBlockEntity.tick`, as far as the beam goes: the part that applies effects and plays
    /// sounds belongs to the server.
    pub(super) fn tick_beacon(&mut self, pos: Pos) {
        let Some(mut beacon) = self
            .block_entity_at(pos.0, pos.1, pos.2)
            .map(|entity| entity.events.beacon.clone())
        else {
            self.block_tickers.unregister(pos);
            return;
        };
        self.scan_beam(pos, &mut beacon);
        if let Some(entity) = self.block_entity_mut_at(pos) {
            entity.events.beacon = beacon;
        }
    }

    /// Up to ten blocks of the column above the beacon, picked up where the last tick left off.
    fn scan_beam(&self, pos: Pos, beacon: &mut BeaconState) {
        let (x, y, z) = pos;
        let first = if beacon.last_check_y < y {
            beacon.checking = Vec::new();
            beacon.last_check_y = y - 1;
            y
        } else {
            beacon.last_check_y + 1
        };
        let surface = self.world_surface_height(x, z);
        for check_y in first..first + BLOCKS_PER_TICK {
            if check_y > surface {
                break;
            }
            let state = self.block_state_or_air(x, check_y, z);
            if !self.extend_beam(beacon, state) {
                beacon.checking.clear();
                beacon.last_check_y = surface;
                break;
            }
            beacon.last_check_y += 1;
        }
        // Once in four seconds a beam that is shown has its pyramid looked at.
        if self.game_time().is_multiple_of(80) && !beacon.sections.is_empty() {
            beacon.levels = self.beacon_base_levels(pos);
        }
        if beacon.last_check_y >= surface {
            beacon.last_check_y = self.dimension().min_y - 1;
            beacon.sections = beacon.checking.clone();
        }
    }

    /// One block of the column: it starts a stretch of the beam, adds to one or blends into the
    /// next; false where it stops the beam.
    fn extend_beam(&self, beacon: &mut BeaconState, state: BlockStateId) -> bool {
        if let Some(color) = beam_color(state) {
            if beacon.checking.len() <= 1 {
                beacon.checking.push(BeamSection { color, height: 1 });
            } else if let Some(last) = beacon.checking.last_mut() {
                if color == last.color {
                    last.height += 1;
                } else {
                    let blended = average(last.color, color);
                    beacon.checking.push(BeamSection {
                        color: blended,
                        height: 1,
                    });
                }
            }
            return true;
        }
        let info = block::state_or_air(state);
        let stops = info.light_dampening >= 15 && info.block().name != "minecraft:bedrock";
        match beacon.checking.last_mut() {
            Some(last) if !stops => {
                last.height += 1;
                true
            }
            _ => false,
        }
    }

    /// `BeaconBlockEntity.updateBase`: how many layers of the pyramid under the beacon are whole,
    /// of up to four.
    fn beacon_base_levels(&self, (x, y, z): Pos) -> i32 {
        let is_base = |x, y, z| {
            let state = self.block_state_or_air(x, y, z);
            let block = block::state_or_air(state).block;
            self.block_state_registry()
                .tags()
                .contains(usize::from(block), "minecraft:beacon_base_blocks")
        };
        let mut levels = 0;
        for step in 1..=4 {
            let layer = y - step;
            if layer < self.dimension().min_y {
                break;
            }
            let whole = (x - step..=x + step)
                .all(|lx| (z - step..=z + step).all(|lz| is_base(lx, layer, lz)));
            if !whole {
                break;
            }
            levels = step;
        }
        levels
    }

    /// `Level.getHeight(Heightmap.Types.WORLD_SURFACE, x, z)`: one above the highest block in the
    /// column that is not air.
    pub fn world_surface_height(&self, x: i32, z: i32) -> i32 {
        self.level_height(crate::HeightmapType::WorldSurface, x, z)
    }

    /// The beacons the level ticks, as `BeaconRenderer.extractRenderState` takes them.
    pub fn beacon_views(&self, partial: f32) -> Vec<BeaconView> {
        let animation = (self.game_time() as i64).rem_euclid(40) as f32 + partial;
        self.block_tickers
            .positions()
            .filter(|pos| self.is_beacon_at(*pos))
            .filter_map(|pos| {
                let beacon = &self.block_entity_at(pos.0, pos.1, pos.2)?.events().beacon;
                Some(BeaconView {
                    position: pos,
                    // `BeaconBlockEntity.getBeamSections`: none while it has no levels.
                    sections: if beacon.levels == 0 {
                        Vec::new()
                    } else {
                        beacon.sections.clone()
                    },
                    animation_time: animation,
                })
            })
            .collect()
    }
}
