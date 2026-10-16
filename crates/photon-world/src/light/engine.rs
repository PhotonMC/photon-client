use super::*;
use photon_core::ids::BlockStateId;

impl Engine {
    pub(super) fn propagate_light_sources(&mut self, level: &Level<'_>, chunk: ColumnKey) {
        match self.storage.layer {
            LightLayer::Block => {
                self.set_light_enabled(level, chunk, true);
                if !level.chunks.has_chunk(chunk.0, chunk.1) {
                    return;
                }
                // `findBlockLightSources`: every emitting state in the
                // chunk's non-empty sections.
                let min_section = self.level_min_y >> 4;
                let section_count = self.section_count;
                for index in 0..section_count {
                    let base_y = (min_section + index) * 16;
                    for y in 0..16 {
                        for z in 0..16 {
                            for x in 0..16 {
                                let block = (chunk.0 * 16 + x, base_y + y, chunk.1 * 16 + z);
                                let state = level.state(block);
                                let emission = emission(state);
                                if emission != 0 {
                                    self.increase_queue.push_back((
                                        block,
                                        entry::increase_light_from_emission(
                                            emission,
                                            is_empty_shape(state),
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            LightLayer::Sky => self.propagate_sky_sources(level, chunk),
        }
    }

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

    pub(super) fn block_emission(&self, block: BlockKey, state: BlockStateId) -> u8 {
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
                    crate::constants::VANILLA_AIR_BLOCK_STATE_ID
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

    pub(super) fn lowest_source_y(level: &Level<'_>, x: i32, z: i32, default: i32) -> i32 {
        Self::chunk_sources(level, x >> 4, z >> 4)
            .map_or(default, |sources| sources.lowest_source_y(x & 15, z & 15))
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

pub(super) fn offset(block: BlockKey, direction: Direction) -> BlockKey {
    let (dx, dy, dz) = direction.step();
    (block.0 + dx, block.1 + dy, block.2 + dz)
}

pub(super) const PROPAGATION_DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// `LightEngine` for one layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Engine {
    pub(super) storage: Storage,
    /// `blockNodesToCheck`, in the order they were added.
    pub(super) nodes_to_check: Vec<BlockKey>,
    pub(super) nodes_seen: HashSet<BlockKey>,
    pub(super) decrease_queue: VecDeque<(BlockKey, u64)>,
    pub(super) increase_queue: VecDeque<(BlockKey, u64)>,
    pub(super) level_min_y: i32,
    pub(super) section_count: i32,
}
