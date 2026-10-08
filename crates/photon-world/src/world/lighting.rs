use crate::*;

impl ClientWorld {
    /// Sections of loaded chunks with published light data in either layer.
    pub fn loaded_light_section_count(&self) -> usize {
        let mut sections: BTreeSet<light::SectionKey> = BTreeSet::new();
        for layer in [light::LightLayer::Sky, light::LightLayer::Block] {
            sections.extend(
                self.light
                    .visible_sections(layer)
                    .into_iter()
                    .filter(|key| self.contains_chunk(key.0, key.2)),
            );
        }
        sections.len()
    }

    /// Light tasks waiting for a frame.
    pub fn pending_light_update_count(&self) -> usize {
        self.light_tasks.len()
    }

    /// The layer's value where the engine holds or has queued a layer for the
    /// section, as `getDataLayerData` finds it.
    pub fn sky_light_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.light_at(LightLayer::Sky, x, y, z)
    }

    pub fn block_light_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.light_at(LightLayer::Block, x, y, z)
    }

    /// `getBrightness(LightLayer.SKY, pos)`: above the highest section with
    /// data the sky is full, and a section with no data takes the bottom row
    /// of the nearest one above.
    pub fn sky_brightness(&self, x: i32, y: i32, z: i32) -> u8 {
        self.light.light_value(light::LightLayer::Sky, (x, y, z))
    }

    /// `getBrightness(LightLayer.BLOCK, pos)`: nothing where no data arrived.
    pub fn block_brightness(&self, x: i32, y: i32, z: i32) -> u8 {
        self.light.light_value(light::LightLayer::Block, (x, y, z))
    }

    pub fn light_engine(&self) -> &light::LevelLightEngine {
        &self.light
    }

    /// A light update packet: `handleLightUpdatePacket` queues its layers
    /// for the next frame, and their sections are redrawn when it runs.
    pub fn apply_light_updates(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
    ) -> LightApplication {
        self.queue_light_data(chunk_x, chunk_z, updates, true, false)
    }

    /// The light that came with a chunk: `handleLevelChunkWithLight` queues
    /// it, then switches the chunk's sections on for light.
    pub fn apply_chunk_light(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
    ) -> LightApplication {
        self.queue_light_data(chunk_x, chunk_z, updates, false, true)
    }

    pub(crate) fn queue_light_data(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
        schedule_rebuild: bool,
        enable_chunk: bool,
    ) -> LightApplication {
        let light_section_count = self.dimension.section_count().saturating_add(2);
        let invalid_sky_updates =
            count_invalid_light_updates(light_section_count, updates.sky.mask, updates.sky.updates);
        let invalid_block_updates = count_invalid_light_updates(
            light_section_count,
            updates.block.mask,
            updates.block.updates,
        );
        if invalid_sky_updates != 0 || invalid_block_updates != 0 {
            return LightApplication {
                invalid_sky_updates,
                invalid_block_updates,
                ..LightApplication::default()
            };
        }
        let application = LightApplication {
            applied_sky_sections: count_light_sections(
                light_section_count,
                updates.sky.mask,
                updates.sky.empty_mask,
            ),
            applied_block_sections: count_light_sections(
                light_section_count,
                updates.block.mask,
                updates.block.empty_mask,
            ),
            queued: !self.contains_chunk(chunk_x, chunk_z),
            ..LightApplication::default()
        };
        self.light_tasks.push_back(LightTask::Data {
            update: PendingLightUpdate::new(chunk_x, chunk_z, updates),
            schedule_rebuild,
            enable_chunk,
        });
        application
    }

    /// `ClientLevel.unload`: the chunk's column stops lighting at once.
    pub(crate) fn unload_chunk_light(&mut self, chunk_x: i32, chunk_z: i32) {
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        self.light
            .set_light_enabled(&view, (chunk_x, chunk_z), false);
    }

    /// `queueLightRemoval`: whether or not the chunk was loaded, its light
    /// is dropped on a later frame.
    pub(crate) fn queue_light_removal(&mut self, chunk_x: i32, chunk_z: i32) {
        self.light_tasks
            .push_back(LightTask::Removal { chunk_x, chunk_z });
    }

    /// `enableChunkLight`: each of the loaded chunk's sections tells the
    /// engine whether it is empty, and the chunk and its neighbours redraw.
    pub fn enable_chunk_light(&mut self, chunk_x: i32, chunk_z: i32) {
        let Some(chunk) = self.chunk_for_query(chunk_x, chunk_z) else {
            return;
        };
        let statuses: Vec<(i32, bool)> = chunk
            .sections
            .iter()
            .map(|(y, section)| (*y, section.has_only_air()))
            .collect();
        for (section_y, empty) in statuses {
            self.light
                .update_section_status((chunk_x, section_y, chunk_z), empty);
        }
        let min_section = self.dimension.min_section_y();
        let max_section = min_section + self.dimension.section_count() as i32 - 1;
        for x in chunk_x - 1..=chunk_x + 1 {
            for z in chunk_z - 1..=chunk_z + 1 {
                for y in min_section..=max_section {
                    self.light_dirty.push((x, y, z));
                }
            }
        }
    }

    /// `ClientLevel.update`'s light part, once a frame: some queued light
    /// tasks, then the engine. Returns the sections to redraw.
    pub fn update_light(&mut self) -> Vec<light::SectionKey> {
        // `pollLightUpdates`: a tenth of the queue, at least ten, or all of
        // it once it reaches a thousand.
        let size = self.light_tasks.len();
        let budget = if size < 1000 {
            (size / 10).max(10)
        } else {
            size
        };
        for _ in 0..budget {
            let Some(task) = self.light_tasks.pop_front() else {
                break;
            };
            self.run_light_task(task);
        }
        let mut dirty = std::mem::take(&mut self.light_dirty);
        dirty.extend(self.run_light_updates());
        dirty
    }

    /// Every queued light task and the engine, as enough frames would.
    pub fn flush_light(&mut self) -> Vec<light::SectionKey> {
        let mut dirty = Vec::new();
        while !self.light_tasks.is_empty() || self.light.has_light_work() {
            dirty.extend(self.update_light());
        }
        dirty
    }

    /// `LevelLightEngine.runLightUpdates`: the sections whose light changed.
    pub fn run_light_updates(&mut self) -> Vec<light::SectionKey> {
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        self.light.run_light_updates(&view)
    }

    /// Light every loaded chunk from its own sources, as the server first
    /// lights a chunk: the sections holding blocks are marked, every column
    /// is switched on and the sources spread. Not what the client does with
    /// a server's light; it measures the engine on its own.
    pub fn light_from_sources(&mut self) {
        let chunks: Vec<(i32, i32)> = self.chunks.keys().copied().collect();
        for &(chunk_x, chunk_z) in &chunks {
            let filled: Vec<i32> = self.chunks[&(chunk_x, chunk_z)]
                .sections
                .iter()
                .filter(|(_, section)| !section.has_only_air())
                .map(|(y, _)| *y)
                .collect();
            for section_y in filled {
                self.light
                    .update_section_status((chunk_x, section_y, chunk_z), false);
            }
        }
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        for &chunk in &chunks {
            self.light.set_light_enabled(&view, chunk, true);
        }
        for &chunk in &chunks {
            self.light.propagate_light_sources(&view, chunk);
        }
    }

    pub(crate) fn run_light_task(&mut self, task: LightTask) {
        match task {
            LightTask::Data {
                update,
                schedule_rebuild,
                enable_chunk,
            } => {
                // `applyLightData`.
                let (chunk_x, chunk_z) = (update.chunk_x, update.chunk_z);
                let min = self.light.min_light_section();
                let count = self.light.light_section_count();
                for (layer, input) in [
                    (light::LightLayer::Sky, &update.sky),
                    (light::LightLayer::Block, &update.block),
                ] {
                    let mut updates = input.updates.iter();
                    for index in 0..count {
                        let bit = index as u64;
                        let have_data = input.mask.contains(bit);
                        if !have_data && !input.empty_mask.contains(bit) {
                            continue;
                        }
                        let data = if have_data {
                            light::DataLayer::from_bytes(updates.next().expect("counted"))
                        } else {
                            light::DataLayer::new(0)
                        };
                        let section_y = min + index;
                        self.light.queue_section_data(
                            layer,
                            (chunk_x, section_y, chunk_z),
                            Some(data),
                        );
                        if schedule_rebuild {
                            // `setSectionDirtyWithNeighbors`.
                            for dx in -1..=1 {
                                for dy in -1..=1 {
                                    for dz in -1..=1 {
                                        self.light_dirty.push((
                                            chunk_x + dx,
                                            section_y + dy,
                                            chunk_z + dz,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                let view = LightView {
                    chunks: &self.chunks,
                    cache: self.chunk_cache.as_ref(),
                };
                self.light
                    .set_light_enabled(&view, (chunk_x, chunk_z), true);
                if enable_chunk {
                    self.enable_chunk_light(chunk_x, chunk_z);
                }
            }
            LightTask::Removal { chunk_x, chunk_z } => {
                let view = LightView {
                    chunks: &self.chunks,
                    cache: self.chunk_cache.as_ref(),
                };
                self.light
                    .set_light_enabled(&view, (chunk_x, chunk_z), false);
                let min = self.light.min_light_section();
                for section_y in min..min + self.light.light_section_count() {
                    for layer in [light::LightLayer::Block, light::LightLayer::Sky] {
                        self.light
                            .queue_section_data(layer, (chunk_x, section_y, chunk_z), None);
                    }
                }
                let min_section = self.dimension.min_section_y();
                for section_y in min_section..min_section + self.dimension.section_count() as i32 {
                    self.light
                        .update_section_status((chunk_x, section_y, chunk_z), true);
                }
            }
        }
    }

    pub(crate) fn light_at(&self, layer: LightLayer, x: i32, y: i32, z: i32) -> Option<u8> {
        let layer = match layer {
            LightLayer::Sky => light::LightLayer::Sky,
            LightLayer::Block => light::LightLayer::Block,
        };
        self.light
            .data_layer(
                layer,
                (x.div_euclid(16), y.div_euclid(16), z.div_euclid(16)),
            )
            .map(|data| data.get(x.rem_euclid(16), y.rem_euclid(16), z.rem_euclid(16)))
    }
}
