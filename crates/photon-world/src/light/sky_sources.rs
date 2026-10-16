use super::*;
use photon_core::ids::BlockStateId;

impl Engine {
    pub(super) fn propagate_sky_sources(&mut self, level: &Level<'_>, chunk: ColumnKey) {
        self.storage.set_light_enabled(chunk, true);
        let empty = ChunkSkyLightSources::new(self.level_min_y);
        let get = |cx: i32, cz: i32| Self::chunk_sources(level, cx, cz).unwrap_or(&empty);
        let sources = get(chunk.0, chunk.1);
        let north = get(chunk.0, chunk.1 - 1);
        let south = get(chunk.0, chunk.1 + 1);
        let west = get(chunk.0 - 1, chunk.1);
        let east = get(chunk.0 + 1, chunk.1);
        let top = self.storage.updating_sky.top(chunk);
        let bottom = self.storage.updating_sky.current_lowest_y;
        let (min_x, min_z) = (chunk.0 * 16, chunk.1 * 16);
        let mut section_y = top - 1;
        while section_y >= bottom {
            let key = (chunk.0, section_y, chunk.1);
            if let Some(id) = self.storage.data_layer_to_write(key) {
                let section_min_y = section_y * 16;
                let section_max_y = section_min_y + 15;
                let mut sources_below = false;
                for z in 0..16 {
                    for x in 0..16 {
                        let lowest = sources.lowest_source_y(x, z);
                        if lowest > section_max_y {
                            continue;
                        }
                        let north_y = if z == 0 {
                            north.lowest_source_y(x, 15)
                        } else {
                            sources.lowest_source_y(x, z - 1)
                        };
                        let south_y = if z == 15 {
                            south.lowest_source_y(x, 0)
                        } else {
                            sources.lowest_source_y(x, z + 1)
                        };
                        let west_y = if x == 0 {
                            west.lowest_source_y(15, z)
                        } else {
                            sources.lowest_source_y(x - 1, z)
                        };
                        let east_y = if x == 15 {
                            east.lowest_source_y(0, z)
                        } else {
                            sources.lowest_source_y(x + 1, z)
                        };
                        let neighbor = north_y.max(south_y).max(west_y.max(east_y));
                        let mut y = section_max_y;
                        while y >= section_min_y.max(lowest) {
                            self.storage.arena.get_mut(id).set(x, y & 15, z, 15);
                            if y == lowest || y < neighbor {
                                self.increase_queue.push_back((
                                    (min_x + x, y, min_z + z),
                                    entry::increase_sky_source_in_directions(
                                        y == lowest,
                                        y < north_y,
                                        y < south_y,
                                        y < west_y,
                                        y < east_y,
                                    ),
                                ));
                            }
                            y -= 1;
                        }
                        if lowest < section_min_y {
                            sources_below = true;
                        }
                    }
                }
                if !sources_below {
                    break;
                }
            }
            section_y -= 1;
        }
    }

    // Sky light.

    pub(super) fn chunk_sources<'a>(
        level: &'a Level<'_>,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&'a ChunkSkyLightSources> {
        level.chunks.sky_sources(chunk_x, chunk_z)
    }

    pub(super) fn check_sky_node(&mut self, level: &Level<'_>, block: BlockKey) {
        let (x, y, z) = block;
        let key = section_of(block);
        let lowest = if self.storage.light_on_in_section(key) {
            Self::lowest_source_y(level, x, z, i32::MAX)
        } else {
            i32::MAX
        };
        if lowest != i32::MAX {
            self.update_sources_in_column(level, x, z, lowest);
        }
        if !self.storage.storing_light_for_section(key) {
            return;
        }
        if y >= lowest {
            self.decrease_queue
                .push_back((block, entry::decrease_skip_one_direction(15, Direction::Up)));
            self.increase_queue.push_back((
                block,
                entry::increase_skip_one_direction(15, false, Direction::Up),
            ));
        } else {
            let old = self.storage.get_stored_level(block);
            if old > 0 {
                self.storage.set_stored_level(block, 0);
                self.decrease_queue
                    .push_back((block, entry::decrease_all_directions(old)));
            } else {
                self.decrease_queue
                    .push_back((block, entry::decrease_all_directions(1)));
            }
        }
    }

    pub(super) fn update_sources_in_column(
        &mut self,
        level: &Level<'_>,
        x: i32,
        z: i32,
        lowest: i32,
    ) {
        // `sectionToBlockCoord`, wrapping as the reference's int shift does.
        let world_bottom_y = self.storage.updating_sky.current_lowest_y.wrapping_shl(4);
        self.remove_sources_below(x, z, lowest, world_bottom_y);
        self.add_sources_above(level, x, z, lowest, world_bottom_y);
    }

    pub(super) fn remove_sources_below(
        &mut self,
        x: i32,
        z: i32,
        lowest: i32,
        world_bottom_y: i32,
    ) {
        if lowest <= world_bottom_y {
            return;
        }
        let start_y = lowest - 1;
        let mut section_y = start_y >> 4;
        while self.storage.has_light_data_at_or_below(section_y) {
            if self
                .storage
                .storing_light_for_section((x >> 4, section_y, z >> 4))
            {
                let bottom = section_y * 16;
                let top = bottom + 15;
                let mut y = top.min(start_y);
                while y >= bottom {
                    let block = (x, y, z);
                    if self.storage.get_stored_level(block) != 15 {
                        return;
                    }
                    self.storage.set_stored_level(block, 0);
                    let data = if y == lowest - 1 {
                        entry::decrease_all_directions(15)
                    } else {
                        entry::decrease_skip_one_direction(15, Direction::Up)
                    };
                    self.decrease_queue.push_back((block, data));
                    y -= 1;
                }
            }
            section_y -= 1;
        }
    }

    pub(super) fn add_sources_above(
        &mut self,
        level: &Level<'_>,
        x: i32,
        z: i32,
        lowest: i32,
        world_bottom_y: i32,
    ) {
        let neighbor_lowest = Self::lowest_source_y(level, x - 1, z, i32::MIN)
            .max(Self::lowest_source_y(level, x + 1, z, i32::MIN))
            .max(
                Self::lowest_source_y(level, x, z - 1, i32::MIN).max(Self::lowest_source_y(
                    level,
                    x,
                    z + 1,
                    i32::MIN,
                )),
            );
        let start_y = lowest.max(world_bottom_y);
        let mut key = (x >> 4, start_y >> 4, z >> 4);
        while !self.storage.is_above_data(key) {
            if self.storage.storing_light_for_section(key) {
                let bottom = key.1 * 16;
                let top = bottom + 15;
                for y in bottom.max(start_y)..=top {
                    let block = (x, y, z);
                    if self.storage.get_stored_level(block) == 15 {
                        return;
                    }
                    self.storage.set_stored_level(block, 15);
                    if y < neighbor_lowest || y == lowest {
                        self.increase_queue.push_back((
                            block,
                            entry::increase_skip_one_direction(15, false, Direction::Up),
                        ));
                    }
                }
            }
            key.1 += 1;
        }
    }

    pub(super) fn count_empty_sections_below_if_at_border(&self, block: BlockKey) -> i32 {
        let (x, y, z) = block;
        if y & 15 != 0 {
            return 0;
        }
        let (lx, lz) = (x & 15, z & 15);
        if lx != 0 && lx != 15 && lz != 0 && lz != 15 {
            return 0;
        }
        let (sx, sy, sz) = section_of(block);
        let mut empty = 0;
        while !self
            .storage
            .storing_light_for_section((sx, sy - empty - 1, sz))
            && self.storage.has_light_data_at_or_below(sy - empty - 1)
        {
            empty += 1;
        }
        empty
    }

    pub(super) fn propagate_from_empty_sections(
        &mut self,
        to: BlockKey,
        direction: Direction,
        to_level: u8,
        increase: bool,
        empty_below: i32,
    ) {
        if empty_below == 0 {
            return;
        }
        let (x, y, z) = to;
        let crossed = match direction {
            Direction::North => z & 15 == 15,
            Direction::South => z & 15 == 0,
            Direction::West => x & 15 == 15,
            Direction::East => x & 15 == 0,
            _ => false,
        };
        if !crossed {
            return;
        }
        let (sx, sz) = (x >> 4, z >> 4);
        let mut section_y = (y >> 4) - 1;
        let bottom = section_y - empty_below + 1;
        while section_y >= bottom {
            if self.storage.storing_light_for_section((sx, section_y, sz)) {
                let min_y = section_y * 16;
                for local_y in (0..16).rev() {
                    let block = (x, min_y + local_y, z);
                    if increase {
                        self.storage.set_stored_level(block, to_level);
                        if to_level > 1 {
                            self.increase_queue.push_back((
                                block,
                                entry::increase_skip_one_direction(
                                    to_level,
                                    true,
                                    direction.opposite(),
                                ),
                            ));
                        }
                    } else {
                        self.storage.set_stored_level(block, 0);
                        self.decrease_queue.push_back((
                            block,
                            entry::decrease_skip_one_direction(to_level, direction.opposite()),
                        ));
                    }
                }
            }
            section_y -= 1;
        }
    }
}

impl ChunkSkyLightSources {
    /// Every column open to below the world, as a new chunk starts.
    pub fn new(level_min_y: i32) -> Self {
        let min_y = level_min_y - 1;
        Self {
            min_y,
            heights: vec![min_y; 256].into_boxed_slice(),
        }
    }

    pub(super) fn index(x: i32, z: i32) -> usize {
        (x + z * 16) as usize
    }

    /// `fillFrom`: from the chunk's sections, the highest first. `section`
    /// answers a section's block states (y, z, x order, 4096 of them) by
    /// index, or nothing if it holds only air.
    pub(crate) fn fill_from<'a>(
        &mut self,
        section_count: usize,
        section: impl Fn(usize) -> Option<&'a crate::paletted::PalettedStates>,
    ) {
        let sections: Vec<Option<&crate::paletted::PalettedStates>> =
            (0..section_count).map(&section).collect();
        let Some(highest) = sections.iter().rposition(Option::is_some) else {
            self.heights.fill(self.min_y);
            return;
        };
        for z in 0..16 {
            for x in 0..16 {
                let found = self.find_lowest_source_y(&sections, highest, x, z);
                self.heights[Self::index(x, z)] = found.max(self.min_y);
            }
        }
    }

    pub(super) fn find_lowest_source_y(
        &self,
        sections: &[Option<&crate::paletted::PalettedStates>],
        top: usize,
        x: i32,
        z: i32,
    ) -> i32 {
        let level_min_y = self.min_y + 1;
        let section_min = |index: usize| level_min_y + index as i32 * 16;
        let mut top_y = section_min(top) + 16;
        let mut top_state = crate::constants::VANILLA_AIR_BLOCK_STATE_ID;
        for index in (0..=top).rev() {
            match &sections[index] {
                None => {
                    top_state = crate::constants::VANILLA_AIR_BLOCK_STATE_ID;
                    top_y = section_min(index);
                }
                Some(states) => {
                    for y in (0..16).rev() {
                        let bottom_state = states
                            .get((y * 256 + z * 16 + x) as usize)
                            .unwrap_or(crate::constants::VANILLA_AIR_BLOCK_STATE_ID);
                        if is_edge_occluded(top_state, bottom_state) {
                            return top_y;
                        }
                        top_state = bottom_state;
                        top_y -= 1;
                    }
                }
            }
        }
        self.min_y
    }

    /// `update` after the block at local `x`, `z` and height `y` changed.
    /// `state` answers the chunk's block at a local position and height.
    pub fn update(
        &mut self,
        state: impl Fn(i32, i32, i32) -> BlockStateId,
        x: i32,
        y: i32,
        z: i32,
    ) -> bool {
        let index = Self::index(x, z);
        let current = self.heights[index];
        if y + 1 < current {
            return false;
        }
        let top = state(x, y + 1, z);
        let middle = state(x, y, z);
        if self.update_edge(
            &state,
            index,
            current,
            (x, y + 1, z),
            top,
            (x, y, z),
            middle,
        ) {
            return true;
        }
        let bottom = state(x, y - 1, z);
        self.update_edge(
            &state,
            index,
            current,
            (x, y, z),
            middle,
            (x, y - 1, z),
            bottom,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the reference method's parameters, in its order"
    )]
    pub(super) fn update_edge(
        &mut self,
        state: &impl Fn(i32, i32, i32) -> BlockStateId,
        index: usize,
        old_top_edge_y: i32,
        top_pos: (i32, i32, i32),
        top_state: BlockStateId,
        bottom_pos: (i32, i32, i32),
        bottom_state: BlockStateId,
    ) -> bool {
        let checked = top_pos.1;
        if is_edge_occluded(top_state, bottom_state) {
            if checked > old_top_edge_y {
                self.heights[index] = checked;
                return true;
            }
        } else if checked == old_top_edge_y {
            self.heights[index] = self.find_lowest_source_below(state, bottom_pos, bottom_state);
            return true;
        }
        false
    }

    pub(super) fn find_lowest_source_below(
        &self,
        state: &impl Fn(i32, i32, i32) -> BlockStateId,
        start: (i32, i32, i32),
        start_state: BlockStateId,
    ) -> i32 {
        let (x, mut top_y, z) = start;
        let mut top_state = start_state;
        while top_y > self.min_y {
            let bottom_state = state(x, top_y - 1, z);
            if is_edge_occluded(top_state, bottom_state) {
                return top_y;
            }
            top_state = bottom_state;
            top_y -= 1;
        }
        self.min_y
    }

    /// `getLowestSourceY`: below the world reads as negative infinity.
    pub fn lowest_source_y(&self, x: i32, z: i32) -> i32 {
        self.extend(self.heights[Self::index(x, z)])
    }

    pub(super) fn highest_lowest_source_y(&self) -> i32 {
        self.extend(self.heights.iter().copied().max().unwrap_or(self.min_y))
    }

    pub(super) fn extend(&self, value: i32) -> i32 {
        if value == self.min_y { i32::MIN } else { value }
    }
}

/// `ChunkSkyLightSources`: per column, the lowest y the sky reaches
/// straight down to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSkyLightSources {
    pub(super) min_y: i32,
    pub(super) heights: Box<[i32]>,
}
