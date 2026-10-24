use super::*;
use photon_core::ids::BlockStateId;

impl Storage {
    pub(super) fn new(layer: LightLayer) -> Self {
        Self {
            layer,
            arena: Arena::default(),
            section_states: HashMap::new(),
            columns_with_sources: HashSet::new(),
            updating: LayerMap::default(),
            visible: LayerMap::default(),
            updating_sky: SkyTops::new(),
            visible_sky: SkyTops::new(),
            changed_sections: HashSet::new(),
            affected_sections: HashSet::new(),
            queued_sections: LayerMap::default(),
            columns_to_retain: HashSet::new(),
            to_remove: Vec::new(),
            has_inconsistencies: false,
        }
    }

    pub(super) fn storing_light_for_section(&self, key: SectionKey) -> bool {
        self.updating.get(key).is_some()
    }

    pub(super) fn visible_layer(&self, key: SectionKey) -> Option<&DataLayer> {
        self.visible.get(key).map(|id| self.arena.get(id))
    }

    /// `getDataLayerToWrite`: the updating layer, copied on its first
    /// change of the run.
    pub(super) fn data_layer_to_write(&mut self, key: SectionKey) -> Option<LayerId> {
        let id = self.updating.get(key)?;
        if self.changed_sections.insert(key) {
            let copy = self.arena.alloc(self.arena.get(id).clone());
            self.updating.set(&mut self.arena, key, copy);
            return Some(copy);
        }
        Some(id)
    }

    pub(super) fn get_stored_level(&self, block: BlockKey) -> u8 {
        let id = self
            .updating
            .get(section_of(block))
            .expect("stored light read where no section is stored");
        let (x, y, z) = local(block);
        self.arena.get(id).get(x, y, z)
    }

    pub(super) fn set_stored_level(&mut self, block: BlockKey, level: u8) {
        let key = section_of(block);
        let id = if self.changed_sections.insert(key) {
            // `copyDataLayer`.
            let old = self.updating.get(key).expect("stored section");
            let copy = self.arena.alloc(self.arena.get(old).clone());
            self.updating.set(&mut self.arena, key, copy);
            copy
        } else {
            self.updating.get(key).expect("stored section")
        };
        let (x, y, z) = local(block);
        self.arena.get_mut(id).set(x, y, z, level);
        // `SectionPos.aroundAndAtBlockPos`.
        let (bx, by, bz) = block;
        for sx in (bx - 1) >> 4..=(bx + 1) >> 4 {
            for sy in (by - 1) >> 4..=(by + 1) >> 4 {
                for sz in (bz - 1) >> 4..=(bz + 1) >> 4 {
                    self.affected_sections.insert((sx, sy, sz));
                }
            }
        }
    }

    pub(super) fn mark_section_and_neighbors_as_affected(&mut self, key: SectionKey) {
        for dz in -1..=1 {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    self.affected_sections
                        .insert((key.0 + dx, key.1 + dy, key.2 + dz));
                }
            }
        }
    }

    /// `createDataLayer`: the queued layer itself if there is one.
    pub(super) fn create_data_layer(&mut self, key: SectionKey) -> LayerId {
        if let Some(queued) = self.queued_sections.get(key) {
            return queued;
        }
        match self.layer {
            LightLayer::Block => self.arena.alloc(DataLayer::new(0)),
            LightLayer::Sky => {
                let top = self.updating_sky.top(column(key));
                if top != self.updating_sky.current_lowest_y && key.1 < top {
                    let mut above = (key.0, key.1 + 1, key.2);
                    let above_id = loop {
                        if let Some(id) = self.updating.get(above) {
                            break id;
                        }
                        above.1 += 1;
                    };
                    let layer = repeat_first_layer(self.arena.get(above_id));
                    self.arena.alloc(layer)
                } else if self.light_on_in_section(key) {
                    self.arena.alloc(DataLayer::new(15))
                } else {
                    self.arena.alloc(DataLayer::new(0))
                }
            }
        }
    }

    pub(super) fn mark_new_inconsistencies(&mut self) {
        if !self.has_inconsistencies {
            return;
        }
        self.has_inconsistencies = false;
        let to_remove = std::mem::take(&mut self.to_remove);
        for &node in &to_remove {
            let queued = self.queued_sections.remove(&mut self.arena, node);
            let stored = self.updating.remove(&mut self.arena, node);
            if self.columns_to_retain.contains(&column(node)) {
                if let Some(queued) = queued {
                    self.queued_sections.set(&mut self.arena, node, queued);
                } else if let Some(stored) = stored {
                    self.queued_sections.set(&mut self.arena, node, stored);
                }
            }
            for id in [queued, stored].into_iter().flatten() {
                self.arena.drop_if_unheld(id);
            }
        }
        for &node in &to_remove {
            self.on_node_removed(node);
            self.changed_sections.insert(node);
        }
        let queued: Vec<(SectionKey, LayerId)> = self
            .queued_sections
            .map
            .iter()
            .map(|(key, id)| (*key, *id))
            .collect();
        for (key, id) in queued {
            if self.storing_light_for_section(key) {
                if self.updating.get(key) != Some(id) {
                    self.updating.set(&mut self.arena, key, id);
                    self.changed_sections.insert(key);
                }
                if let Some(id) = self.queued_sections.remove(&mut self.arena, key) {
                    self.arena.drop_if_unheld(id);
                }
            }
        }
    }

    pub(super) fn on_node_added(&mut self, key: SectionKey) {
        if self.layer != LightLayer::Sky {
            return;
        }
        let y = key.1;
        if self.updating_sky.current_lowest_y > y {
            self.updating_sky.current_lowest_y = y;
        }
        let old_top = self.updating_sky.top(column(key));
        if old_top < y + 1 {
            self.updating_sky.top_sections.insert(column(key), y + 1);
        }
    }

    pub(super) fn on_node_removed(&mut self, key: SectionKey) {
        if self.layer != LightLayer::Sky {
            return;
        }
        let col = column(key);
        let mut y = key.1;
        if self.updating_sky.top(col) == y + 1 {
            let mut top = key;
            while !self.storing_light_for_section(top) && self.has_light_data_at_or_below(y) {
                y -= 1;
                top.1 -= 1;
            }
            if self.storing_light_for_section(top) {
                self.updating_sky.top_sections.insert(col, y + 1);
            } else {
                self.updating_sky.top_sections.remove(&col);
            }
        }
    }

    pub(super) fn set_light_enabled(&mut self, col: ColumnKey, enable: bool) {
        if enable {
            self.columns_with_sources.insert(col);
        } else {
            self.columns_with_sources.remove(&col);
        }
    }

    pub(super) fn light_on_in_section(&self, key: SectionKey) -> bool {
        self.columns_with_sources.contains(&column(key))
    }

    pub(super) fn retain_data(&mut self, col: ColumnKey, retain: bool) {
        if retain {
            self.columns_to_retain.insert(col);
        } else {
            self.columns_to_retain.remove(&col);
        }
    }

    pub(super) fn queue_section_data(&mut self, key: SectionKey, data: Option<DataLayer>) {
        match data {
            Some(data) => {
                let id = self.arena.alloc(data);
                self.queued_sections.set(&mut self.arena, key, id);
                self.has_inconsistencies = true;
            }
            None => {
                if let Some(id) = self.queued_sections.remove(&mut self.arena, key) {
                    self.arena.drop_if_unheld(id);
                }
            }
        }
    }

    pub(super) fn update_section_status(&mut self, key: SectionKey, empty: bool) {
        let state = self.section_states.get(&key).copied().unwrap_or(0);
        let new_state = if empty {
            state & !HAS_DATA_BIT
        } else {
            state | HAS_DATA_BIT
        };
        if state == new_state {
            return;
        }
        self.put_section_state(key, new_state);
        let increment: i32 = if empty { -1 } else { 1 };
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let neighbor = (key.0 + dx, key.1 + dy, key.2 + dz);
                    let neighbor_state = self.section_states.get(&neighbor).copied().unwrap_or(0);
                    let count = i32::from(neighbor_state & NEIGHBOR_COUNT_BITS) + increment;
                    assert!(
                        (0..=26).contains(&count),
                        "Neighbor count was not within range [0; 26]"
                    );
                    self.put_section_state(
                        neighbor,
                        (neighbor_state & !NEIGHBOR_COUNT_BITS) | count as u8,
                    );
                }
            }
        }
    }

    pub(super) fn put_section_state(&mut self, key: SectionKey, state: u8) {
        if state != 0 {
            if self.section_states.insert(key, state).unwrap_or(0) == 0 {
                self.initialize_section(key);
            }
        } else if self.section_states.remove(&key).unwrap_or(0) != 0 {
            self.to_remove.push(key);
            self.has_inconsistencies = true;
        }
    }

    pub(super) fn initialize_section(&mut self, key: SectionKey) {
        if let Some(position) = self.to_remove.iter().position(|node| *node == key) {
            self.to_remove.remove(position);
            return;
        }
        let id = self.create_data_layer(key);
        self.updating.set(&mut self.arena, key, id);
        self.changed_sections.insert(key);
        self.on_node_added(key);
        self.mark_section_and_neighbors_as_affected(key);
        self.has_inconsistencies = true;
    }

    /// `swapSectionMap`: publish the updating layers; the sections whose
    /// light changed come back to be redrawn.
    pub(super) fn swap_section_map(&mut self) -> Vec<SectionKey> {
        if !self.changed_sections.is_empty() {
            let old: Vec<LayerId> = self.visible.map.values().copied().collect();
            self.visible.map = self.updating.map.clone();
            for id in self.visible.map.values() {
                self.arena.retain(*id);
            }
            for id in old {
                self.arena.release(id);
            }
            self.visible_sky = self.updating_sky.clone();
            self.changed_sections.clear();
        }
        self.affected_sections.drain().collect()
    }

    pub(super) fn has_light_data_at_or_below(&self, section_y: i32) -> bool {
        section_y >= self.updating_sky.current_lowest_y
    }

    pub(super) fn is_above_data(&self, key: SectionKey) -> bool {
        let top = self.updating_sky.top(column(key));
        top == self.updating_sky.current_lowest_y || key.1 >= top
    }

    /// The level a getter reads, from the published layers.
    pub(super) fn get_light_value(&self, block: BlockKey) -> u8 {
        let mut level = [0];
        self.get_light_run(block, &mut level);
        level[0]
    }

    /// The levels a getter reads along x from `block`, one per entry of `out`; the run stays
    /// inside one section, so the layer is looked up once.
    pub(super) fn get_light_run(&self, block: BlockKey, out: &mut [u8]) {
        let key = section_of(block);
        let (x, y, z) = local(block);
        debug_assert!(x as usize + out.len() <= 16, "a run stays in its section");
        let source = match self.layer {
            LightLayer::Block => self.visible_layer(key).map(|layer| (layer, y)),
            LightLayer::Sky => match self.sky_source(key, y) {
                SkySource::Layer(layer, local_y) => Some((layer, local_y)),
                SkySource::Full => {
                    out.fill(15);
                    return;
                }
            },
        };
        match source {
            None => out.fill(0),
            Some((layer, local_y)) => {
                for (offset, level) in out.iter_mut().enumerate() {
                    *level = layer.get(x + offset as i32, local_y, z);
                }
            }
        }
    }

    /// Where the sky level of a block in section `key` at local height `y` is read from.
    fn sky_source(&self, key: SectionKey, y: i32) -> SkySource<'_> {
        let top = self.visible_sky.top(column(key));
        if top == self.visible_sky.current_lowest_y || key.1 >= top {
            return SkySource::Full;
        }
        let mut section = key;
        let mut local_y = y;
        loop {
            if let Some(layer) = self.visible_layer(section) {
                return SkySource::Layer(layer, local_y);
            }
            // The flat index: the bottom row of the layer above.
            local_y = 0;
            section.1 += 1;
            if section.1 >= top {
                return SkySource::Full;
            }
        }
    }
}

/// `LayerLightSectionStorage` for one layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Storage {
    pub(super) layer: LightLayer,
    pub(super) arena: Arena,
    pub(super) section_states: HashMap<SectionKey, u8>,
    pub(super) columns_with_sources: HashSet<ColumnKey>,
    pub(super) updating: LayerMap,
    pub(super) visible: LayerMap,
    pub(super) updating_sky: SkyTops,
    pub(super) visible_sky: SkyTops,
    pub(super) changed_sections: HashSet<SectionKey>,
    pub(super) affected_sections: HashSet<SectionKey>,
    pub(super) queued_sections: LayerMap,
    pub(super) columns_to_retain: HashSet<ColumnKey>,
    pub(super) to_remove: Vec<SectionKey>,
    pub(super) has_inconsistencies: bool,
}

/// What the engine reads of the level.
pub trait LightChunks {
    /// Whether the chunk is loaded; the engine reads an unloaded one as
    /// bedrock.
    fn has_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool;
    /// A block state in a loaded chunk.
    fn block_state(&self, x: i32, y: i32, z: i32) -> BlockStateId;
    fn sky_sources(&self, chunk_x: i32, chunk_z: i32) -> Option<&ChunkSkyLightSources>;
}

/// The place a sky level is read from.
enum SkySource<'a> {
    /// Above all light data: the sky is full.
    Full,
    /// A layer, at this height in it.
    Layer(&'a DataLayer, i32),
}
