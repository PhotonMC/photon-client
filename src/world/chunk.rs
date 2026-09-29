//! `ClientWorld` chunk layer split out of the former single-file module.

use super::*;

use std::collections::{BTreeMap, BTreeSet};

use crate::binary::BitSet;

/// The observable state of the Java client chunk cache.
///
/// The cache owns a bounded ring of chunk slots, while the world owns the
/// decoded chunk data. Keeping this snapshot separate lets behavior tests
/// inspect the cache lifecycle without exposing its slot implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientChunkCacheSnapshot {
    pub storage_radius: i32,
    pub view_range: i32,
    pub view_center_x: i32,
    pub view_center_z: i32,
    pub loaded_chunks: Vec<(i32, i32)>,
    pub added_empty_sections: Vec<i64>,
    pub removed_empty_sections: Vec<i64>,
    pub added_loaded_chunks: Vec<(i32, i32)>,
    pub removed_loaded_chunks: Vec<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClientChunkCacheState {
    pub(crate) server_view_radius: i32,
    pub(crate) storage_radius: i32,
    pub(crate) view_range: i32,
    pub(crate) view_center_x: i32,
    pub(crate) view_center_z: i32,
    pub(crate) slots: BTreeMap<usize, (i32, i32)>,
    pub(crate) updating_sets_index: usize,
    pub(crate) added_empty_sections: [BTreeSet<i64>; 2],
    pub(crate) removed_empty_sections: [BTreeSet<i64>; 2],
    pub(crate) added_loaded_chunks: [BTreeSet<(i32, i32)>; 2],
    pub(crate) removed_loaded_chunks: [BTreeSet<(i32, i32)>; 2],
}

impl ClientChunkCacheState {
    pub(crate) fn new(server_view_radius: i32) -> Self {
        let storage_radius = server_view_radius.max(2) + 3;
        let view_range = storage_radius * 2 + 1;
        Self {
            server_view_radius,
            storage_radius,
            view_range,
            view_center_x: 0,
            view_center_z: 0,
            slots: BTreeMap::new(),
            updating_sets_index: 0,
            added_empty_sections: std::array::from_fn(|_| BTreeSet::new()),
            removed_empty_sections: std::array::from_fn(|_| BTreeSet::new()),
            added_loaded_chunks: std::array::from_fn(|_| BTreeSet::new()),
            removed_loaded_chunks: std::array::from_fn(|_| BTreeSet::new()),
        }
    }

    pub(crate) fn in_range(&self, chunk_x: i32, chunk_z: i32) -> bool {
        (i64::from(chunk_x) - i64::from(self.view_center_x)).abs() <= i64::from(self.storage_radius)
            && (i64::from(chunk_z) - i64::from(self.view_center_z)).abs()
                <= i64::from(self.storage_radius)
    }

    pub(crate) fn index(&self, chunk_x: i32, chunk_z: i32) -> usize {
        let range = i64::from(self.view_range);
        let x = i64::from(chunk_x).rem_euclid(range);
        let z = i64::from(chunk_z).rem_euclid(range);
        (z * range + x) as usize
    }

    pub(crate) fn section_node(chunk_x: i32, section_y: i32, chunk_z: i32) -> i64 {
        let x = (chunk_x as u64) & 0x3f_ffff;
        let y = (section_y as u64) & 0x0f_ffff;
        let z = (chunk_z as u64) & 0x3f_ffff;
        ((x << 42) | (z << 20) | y) as i64
    }

    pub(crate) fn on_chunk_removed(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        section_ys: impl IntoIterator<Item = i32>,
    ) {
        let index = self.updating_sets_index;
        self.removed_loaded_chunks[index].insert((chunk_x, chunk_z));
        for section_y in section_ys {
            self.removed_empty_sections[index]
                .insert(Self::section_node(chunk_x, section_y, chunk_z));
        }
    }

    pub(crate) fn on_chunk_added(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        sections: impl IntoIterator<Item = (i32, bool)>,
    ) {
        let index = self.updating_sets_index;
        self.added_loaded_chunks[index].insert((chunk_x, chunk_z));
        for (section_y, has_only_air) in sections {
            if has_only_air {
                self.added_empty_sections[index]
                    .insert(Self::section_node(chunk_x, section_y, chunk_z));
            }
        }
    }

    pub(crate) fn refresh_empty_sections(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        sections: impl IntoIterator<Item = (i32, bool)>,
    ) {
        let index = self.updating_sets_index;
        for (section_y, has_only_air) in sections {
            let node = Self::section_node(chunk_x, section_y, chunk_z);
            if has_only_air {
                self.added_empty_sections[index].insert(node);
            } else {
                self.removed_empty_sections[index].insert(node);
            }
        }
    }

    pub(crate) fn on_section_emptiness_changed(
        &mut self,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        empty: bool,
    ) {
        if !self.in_range(section_x, section_z) {
            return;
        }
        let node = Self::section_node(section_x, section_y, section_z);
        if empty {
            self.added_empty_sections[self.updating_sets_index].insert(node);
        } else {
            self.removed_empty_sections[self.updating_sets_index].insert(node);
        }
    }

    pub(crate) fn flip_update_tracking_sets(&mut self) {
        self.updating_sets_index = (self.updating_sets_index + 1) % 2;
        self.added_empty_sections[self.updating_sets_index].clear();
        self.removed_empty_sections[self.updating_sets_index].clear();
        self.added_loaded_chunks[self.updating_sets_index].clear();
        self.removed_loaded_chunks[self.updating_sets_index].clear();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ClientChunk {
    pub(crate) sections: BTreeMap<i32, ClientSection>,
    /// `ChunkSkyLightSources`, from the sections the chunk arrived with.
    pub(crate) sky_sources: Option<super::light::ChunkSkyLightSources>,
    pub(crate) block_entities: BTreeMap<(i32, i32, i32), ClientBlockEntity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingLightUpdate {
    pub(crate) chunk_x: i32,
    pub(crate) chunk_z: i32,
    pub(crate) sky: OwnedLightLayerInput,
    pub(crate) block: OwnedLightLayerInput,
}

impl PendingLightUpdate {
    pub(crate) fn new(chunk_x: i32, chunk_z: i32, input: LightUpdateInput<'_>) -> Self {
        Self {
            chunk_x,
            chunk_z,
            sky: OwnedLightLayerInput::new(input.sky),
            block: OwnedLightLayerInput::new(input.block),
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnedLightLayerInput {
    pub(crate) mask: BitSet,
    pub(crate) empty_mask: BitSet,
    pub(crate) updates: Vec<Vec<u8>>,
}

impl OwnedLightLayerInput {
    pub(crate) fn new(input: LightLayerInput<'_>) -> Self {
        Self {
            mask: input.mask.clone(),
            empty_mask: input.empty_mask.clone(),
            updates: input.updates.to_vec(),
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClientSection {
    pub(crate) non_empty_block_count: i16,
    pub(crate) fluid_count: i16,
    pub(crate) block_states: Vec<i32>,
    pub(crate) biomes: Vec<i32>,
}

impl ClientSection {
    pub(crate) fn has_only_air(&self) -> bool {
        self.non_empty_block_count == 0
    }

    pub(crate) fn set_block_state(&mut self, index: usize, state_id: i32) -> Option<i32> {
        // The complete block-state registry is still open; this slice has the
        // exact air/non-air and source-water classifications used by its
        // pinned update trace.
        let block_state = self.block_states.get_mut(index)?;
        let previous_state_id = *block_state;
        if previous_state_id != VANILLA_AIR_BLOCK_STATE_ID {
            self.non_empty_block_count -= 1;
            if previous_state_id == VANILLA_WATER_BLOCK_STATE_ID {
                self.fluid_count -= 1;
            }
        }
        if state_id != VANILLA_AIR_BLOCK_STATE_ID {
            self.non_empty_block_count += 1;
            if state_id == VANILLA_WATER_BLOCK_STATE_ID {
                self.fluid_count += 1;
            }
        }
        *block_state = state_id;
        Some(previous_state_id)
    }
}

pub struct SectionInput<'a> {
    pub non_empty_block_count: i16,
    pub fluid_count: i16,
    pub block_states: &'a [i32],
    pub biomes: &'a [i32],
}

pub struct LightUpdateInput<'a> {
    pub sky: LightLayerInput<'a>,
    pub block: LightLayerInput<'a>,
}

pub struct LightLayerInput<'a> {
    pub mask: &'a BitSet,
    pub empty_mask: &'a BitSet,
    pub updates: &'a [Vec<u8>],
}

pub(crate) struct BlockEntityInput<'a> {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z: i32,
    pub(crate) type_id: i32,
    pub(crate) tag: Option<&'a [u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LightLayer {
    Sky,
    Block,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LightApplication {
    pub applied_sky_sections: usize,
    pub applied_block_sections: usize,
    pub invalid_sky_updates: usize,
    pub invalid_block_updates: usize,
    pub queued: bool,
}

pub(crate) fn count_invalid_light_updates(
    section_count: usize,
    mask: &BitSet,
    updates: &[Vec<u8>],
) -> usize {
    let mut update_index = 0;
    let mut invalid_updates = 0;
    for section_index in 0..section_count {
        if mask.contains(section_index as u64) {
            if updates
                .get(update_index)
                .is_none_or(|update| update.len() != 2_048)
            {
                invalid_updates += 1;
            }
            update_index += 1;
        }
    }
    invalid_updates
}

pub(crate) fn count_light_sections(
    section_count: usize,
    mask: &BitSet,
    empty_mask: &BitSet,
) -> usize {
    (0..section_count)
        .filter(|section_index| {
            let bit = *section_index as u64;
            mask.contains(bit) || empty_mask.contains(bit)
        })
        .count()
}

