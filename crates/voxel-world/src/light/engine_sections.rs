use super::*;

impl Engine {
    pub(super) fn new(layer: LightLayer, level_min_y: i32) -> Self {
        Self {
            storage: Storage::new(layer),
            nodes_to_check: Vec::new(),
            nodes_seen: HashSet::new(),
            decrease_queue: VecDeque::new(),
            increase_queue: VecDeque::new(),
            level_min_y,
            section_count: 0,
        }
    }

    pub(super) fn opacity(state: i32) -> u8 {
        dampening(state).max(1)
    }

    pub(super) fn shape_occludes(from: i32, to: i32, direction: Direction) -> bool {
        face_shape_occludes(
            occlusion_shape(from, direction),
            occlusion_shape(to, direction.opposite()),
        )
    }

    pub(super) fn check_block(&mut self, block: BlockKey) {
        if self.nodes_seen.insert(block) {
            self.nodes_to_check.push(block);
        }
    }

    pub(super) fn run(&mut self, level: &Level<'_>) -> Vec<SectionKey> {
        let nodes = std::mem::take(&mut self.nodes_to_check);
        self.nodes_seen.clear();
        for node in nodes {
            self.check_node(level, node);
        }
        while let Some((node, data)) = self.decrease_queue.pop_front() {
            self.propagate_decrease(level, node, data);
        }
        while let Some((from, data)) = self.increase_queue.pop_front() {
            let mut from_level = self.storage.get_stored_level(from);
            let target = entry::from_level(data);
            if entry::is_increase_from_emission(data) && from_level < target {
                self.storage.set_stored_level(from, target);
                from_level = target;
            }
            if from_level == target {
                self.propagate_increase(level, from, data, from_level);
            }
        }
        self.storage.mark_new_inconsistencies();
        self.storage.swap_section_map()
    }

    pub(super) fn has_light_work(&self) -> bool {
        self.storage.has_inconsistencies
            || !self.nodes_to_check.is_empty()
            || !self.decrease_queue.is_empty()
            || !self.increase_queue.is_empty()
    }

    pub(super) fn block_emission(&self, block: BlockKey, state: i32) -> u8 {
        let emission = emission(state);
        if emission > 0 && self.storage.light_on_in_section(section_of(block)) {
            emission
        } else {
            0
        }
    }

    pub(super) fn check_node(&mut self, level: &Level<'_>, block: BlockKey) {
        match self.storage.layer {
            LightLayer::Block => {
                if !self.storage.storing_light_for_section(section_of(block)) {
                    return;
                }
                let state = level.state(block);
                let emission = self.block_emission(block, state);
                let old = self.storage.get_stored_level(block);
                if emission < old {
                    self.storage.set_stored_level(block, 0);
                    self.decrease_queue
                        .push_back((block, entry::decrease_all_directions(old)));
                } else {
                    self.decrease_queue
                        .push_back((block, entry::decrease_all_directions(1)));
                }
                if emission > 0 {
                    self.increase_queue.push_back((
                        block,
                        entry::increase_light_from_emission(emission, is_empty_shape(state)),
                    ));
                }
            }
            LightLayer::Sky => self.check_sky_node(level, block),
        }
    }

    pub(super) fn propagate_increase(
        &mut self,
        level: &Level<'_>,
        from: BlockKey,
        data: u64,
        from_level: u8,
    ) {
        let sky = self.storage.layer == LightLayer::Sky;
        let empty_below = if sky {
            self.count_empty_sections_below_if_at_border(from)
        } else {
            0
        };
        let mut from_state = None;
        for direction in PROPAGATION_DIRECTIONS {
            if !entry::propagates(data, direction) {
                continue;
            }
            let to = offset(from, direction);
            if !self.storage.storing_light_for_section(section_of(to)) {
                continue;
            }
            let to_level = self.storage.get_stored_level(to);
            if i32::from(from_level) - 1 <= i32::from(to_level) {
                continue;
            }
            let to_state = level.state(to);
            let new_level = i32::from(from_level) - i32::from(Self::opacity(to_state));
            if new_level <= i32::from(to_level) {
                continue;
            }
            let new_level = new_level as u8;
            let from_state = *from_state.get_or_insert_with(|| {
                if entry::is_from_empty_shape(data) {
                    0
                } else {
                    level.state(from)
                }
            });
            if Self::shape_occludes(from_state, to_state, direction) {
                continue;
            }
            self.storage.set_stored_level(to, new_level);
            if new_level > 1 {
                self.increase_queue.push_back((
                    to,
                    entry::increase_skip_one_direction(
                        new_level,
                        is_empty_shape(to_state),
                        direction.opposite(),
                    ),
                ));
            }
            if sky {
                self.propagate_from_empty_sections(to, direction, new_level, true, empty_below);
            }
        }
    }

    pub(super) fn propagate_decrease(&mut self, level: &Level<'_>, from: BlockKey, data: u64) {
        let sky = self.storage.layer == LightLayer::Sky;
        let empty_below = if sky {
            self.count_empty_sections_below_if_at_border(from)
        } else {
            0
        };
        let old_from_level = entry::from_level(data);
        for direction in PROPAGATION_DIRECTIONS {
            if !entry::propagates(data, direction) {
                continue;
            }
            let to = offset(from, direction);
            if !self.storage.storing_light_for_section(section_of(to)) {
                continue;
            }
            let to_level = self.storage.get_stored_level(to);
            if to_level == 0 {
                continue;
            }
            if to_level < old_from_level {
                if sky {
                    self.storage.set_stored_level(to, 0);
                    self.decrease_queue.push_back((
                        to,
                        entry::decrease_skip_one_direction(to_level, direction.opposite()),
                    ));
                    self.propagate_from_empty_sections(to, direction, to_level, false, empty_below);
                } else {
                    let to_state = level.state(to);
                    let to_emission = self.block_emission(to, to_state);
                    self.storage.set_stored_level(to, 0);
                    if to_emission < to_level {
                        self.decrease_queue.push_back((
                            to,
                            entry::decrease_skip_one_direction(to_level, direction.opposite()),
                        ));
                    }
                    if to_emission > 0 {
                        self.increase_queue.push_back((
                            to,
                            entry::increase_light_from_emission(
                                to_emission,
                                is_empty_shape(to_state),
                            ),
                        ));
                    }
                }
            } else {
                self.increase_queue.push_back((
                    to,
                    entry::increase_only_one_direction(to_level, false, direction.opposite()),
                ));
            }
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

    pub(super) fn lowest_source_y(level: &Level<'_>, x: i32, z: i32, default: i32) -> i32 {
        Self::chunk_sources(level, x >> 4, z >> 4)
            .map_or(default, |sources| sources.lowest_source_y(x & 15, z & 15))
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

    pub(super) fn set_light_enabled(&mut self, level: &Level<'_>, chunk: ColumnKey, enable: bool) {
        self.storage.set_light_enabled(chunk, enable);
        if self.storage.layer != LightLayer::Sky || !enable {
            return;
        }
        let empty = ChunkSkyLightSources::new(self.level_min_y);
        let sources = Self::chunk_sources(level, chunk.0, chunk.1).unwrap_or(&empty);
        let highest_non_source_y = sources.highest_lowest_source_y().wrapping_sub(1);
        let lowest_fully_source_section_y = (highest_non_source_y >> 4) + 1;
        let top = self.storage.updating_sky.top(chunk);
        let bottom = self
            .storage
            .updating_sky
            .current_lowest_y
            .max(lowest_fully_source_section_y);
        let mut section_y = top - 1;
        while section_y >= bottom {
            if let Some(id) = self
                .storage
                .data_layer_to_write((chunk.0, section_y, chunk.1))
                && self.storage.arena.get(id).is_empty()
            {
                self.storage.arena.get_mut(id).fill(15);
            }
            section_y -= 1;
        }
    }
}
