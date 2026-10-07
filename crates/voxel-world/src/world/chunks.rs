use crate::*;

impl ClientWorld {
    /// Create the bounded chunk cache used by a Java client level.
    pub fn configure_chunk_cache(&mut self, server_view_radius: i32) {
        self.chunks.clear();
        self.light = new_light_engine(self.dimension, self.dimension_environment.has_skylight);
        self.light_tasks.clear();
        self.chunk_cache = Some(ClientChunkCacheState::new(server_view_radius));
    }

    /// Return the Java cache range and ring index for a chunk coordinate.
    pub fn chunk_cache_location(&self, chunk_x: i32, chunk_z: i32) -> Option<(bool, usize)> {
        self.chunk_cache.as_ref().map(|cache| {
            (
                cache.in_range(chunk_x, chunk_z),
                cache.index(chunk_x, chunk_z),
            )
        })
    }

    /// Move the Java client chunk-cache view center without evicting slots.
    pub fn update_chunk_cache_center(&mut self, chunk_x: i32, chunk_z: i32) {
        if let Some(cache) = self.chunk_cache.as_mut() {
            cache.view_center_x = chunk_x;
            cache.view_center_z = chunk_z;
        }
    }

    /// Resize the Java client chunk cache and retain chunks in the new range.
    pub fn update_chunk_cache_radius(&mut self, server_view_radius: i32) {
        let Some(cache) = self.chunk_cache.as_mut() else {
            return;
        };
        let new_storage_radius = server_view_radius.max(2) + 3;
        cache.server_view_radius = server_view_radius;
        if cache.storage_radius == new_storage_radius {
            return;
        }

        let mut new_cache = ClientChunkCacheState::new(server_view_radius);
        new_cache.view_center_x = cache.view_center_x;
        new_cache.view_center_z = cache.view_center_z;
        let retained = self
            .chunks
            .iter()
            .filter(|&(&(chunk_x, chunk_z), _)| new_cache.in_range(chunk_x, chunk_z))
            .map(|(&(chunk_x, chunk_z), chunk)| ((chunk_x, chunk_z), chunk.clone()))
            .collect::<Vec<_>>();
        self.chunks.clear();
        for ((chunk_x, chunk_z), chunk) in retained {
            let index = new_cache.index(chunk_x, chunk_z);
            new_cache.slots.insert(index, (chunk_x, chunk_z));
            new_cache.on_chunk_added(
                chunk_x,
                chunk_z,
                chunk
                    .sections
                    .iter()
                    .map(|(&section_y, section)| (section_y, section.has_only_air())),
            );
            self.chunks.insert((chunk_x, chunk_z), chunk);
        }
        self.chunk_cache = Some(new_cache);
    }

    /// Apply the cache's section-emptiness notification path.
    pub fn update_chunk_section_emptiness(
        &mut self,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        empty: bool,
    ) {
        if let Some(cache) = self.chunk_cache.as_mut() {
            cache.on_section_emptiness_changed(section_x, section_y, section_z, empty);
        }
    }

    /// Swap the Java cache's double-buffered update sets.
    pub fn flip_chunk_cache_update_sets(&mut self) {
        if let Some(cache) = self.chunk_cache.as_mut() {
            cache.flip_update_tracking_sets();
        }
    }

    /// Snapshot the loaded chunks and cache update sets in deterministic order.
    pub fn client_chunk_cache_snapshot(&self) -> Option<ClientChunkCacheSnapshot> {
        let cache = self.chunk_cache.as_ref()?;
        let index = cache.updating_sets_index;
        Some(ClientChunkCacheSnapshot {
            storage_radius: cache.storage_radius,
            view_range: cache.view_range,
            view_center_x: cache.view_center_x,
            view_center_z: cache.view_center_z,
            loaded_chunks: self.chunks.keys().copied().collect(),
            added_empty_sections: cache.added_empty_sections[index].iter().copied().collect(),
            removed_empty_sections: cache.removed_empty_sections[index]
                .iter()
                .copied()
                .collect(),
            added_loaded_chunks: cache.added_loaded_chunks[index].iter().copied().collect(),
            removed_loaded_chunks: cache.removed_loaded_chunks[index].iter().copied().collect(),
        })
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn loaded_section_count(&self) -> usize {
        self.chunks.values().map(|chunk| chunk.sections.len()).sum()
    }

    pub(crate) fn cached_chunk_is_current(&self, chunk_x: i32, chunk_z: i32) -> bool {
        let Some(cache) = self.chunk_cache.as_ref() else {
            return true;
        };
        cache.in_range(chunk_x, chunk_z)
            && cache.slots.get(&cache.index(chunk_x, chunk_z)) == Some(&(chunk_x, chunk_z))
    }

    pub(crate) fn chunk_for_query(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        self.cached_chunk_is_current(chunk_x, chunk_z)
            .then(|| self.chunks.get(&(chunk_x, chunk_z)))
            .flatten()
    }

    pub(crate) fn chunk_for_update(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&mut ClientChunk> {
        if !self.cached_chunk_is_current(chunk_x, chunk_z) {
            return None;
        }
        self.chunks.get_mut(&(chunk_x, chunk_z))
    }

    pub fn contains_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunk_for_query(chunk_x, chunk_z).is_some()
    }

    /// `handleForgetLevelChunk`: the chunk is dropped if the cache holds it,
    /// and its light is queued for removal either way.
    pub fn unload_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> bool {
        self.queue_light_removal(chunk_x, chunk_z);
        if let Some(cache) = self.chunk_cache.as_ref() {
            if !cache.in_range(chunk_x, chunk_z) {
                return false;
            }
            let index = cache.index(chunk_x, chunk_z);
            if cache.slots.get(&index) != Some(&(chunk_x, chunk_z)) {
                return false;
            }
            let removed = self.chunks.remove(&(chunk_x, chunk_z));
            if let Some(chunk) = removed.as_ref() {
                let section_ys = chunk.sections.keys().copied().collect::<Vec<_>>();
                let cache = self
                    .chunk_cache
                    .as_mut()
                    .expect("chunk cache exists during unload");
                cache.slots.remove(&index);
                cache.on_chunk_removed(chunk_x, chunk_z, section_ys);
            }
            if removed.is_some() {
                self.unload_chunk_light(chunk_x, chunk_z);
                for entity in self.entities.values_mut() {
                    if !entity.always_ticking
                        && entity_chunk_key(entity.position()) == (chunk_x, chunk_z)
                    {
                        entity.ticking = false;
                    }
                }
            }
            return removed.is_some();
        }

        let removed = self.chunks.remove(&(chunk_x, chunk_z)).is_some();
        if removed {
            self.unload_chunk_light(chunk_x, chunk_z);
            for entity in self.entities.values_mut() {
                if !entity.always_ticking
                    && entity_chunk_key(entity.position()) == (chunk_x, chunk_z)
                {
                    entity.ticking = false;
                }
            }
        }
        removed
    }

    /// Whether the chunk a position lies in is loaded.
    pub(crate) fn contains_chunk_at(&self, position: (f64, f64, f64)) -> bool {
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        self.contains_chunk(chunk_x, chunk_z)
    }

    /// `ClientLevel.setServerSimulationDistance`.
    pub fn set_server_simulation_distance(&mut self, chunks: i32) {
        self.server_simulation_distance = chunks;
    }

    /// Where the client's own player stands, for the checks that measure from
    /// its chunk. The session moves the player, so it says where each tick.
    pub fn set_local_player_chunk(&mut self, x: f64, z: f64) {
        let chunk = |value: f64| (value.floor() as i32) >> 4;
        self.local_chunk = Some((chunk(x), chunk(z)));
    }

    pub fn section_counts_at(
        &self,
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
    ) -> Option<(i16, i16)> {
        let chunk = self.chunk_for_query(chunk_x, chunk_z)?;
        let section = chunk.sections.get(&section_y)?;
        Some((section.non_empty_block_count, section.fluid_count))
    }

    /// `getNoiseBiome`: the biome at a quart position, its row clamped into
    /// the dimension.
    pub fn noise_biome(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        self.biome_at(x * 4, self.dimension.clamp_quart_y(y) * 4, z * 4)
    }

    /// Return the resolved biome registry ID at a world coordinate.
    pub fn biome_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunk_for_query(chunk_key.0, chunk_key.1)?;
        let section_y = y.div_euclid(16);
        let section = chunk.sections.get(&section_y)?;
        let local_x = x.rem_euclid(16) as usize / 4;
        let local_y = y.rem_euclid(16) as usize / 4;
        let local_z = z.rem_euclid(16) as usize / 4;
        let index = local_y * 16 + local_z * 4 + local_x;
        section.biomes.get(index).copied()
    }

    /// Replace a loaded chunk with decoded sections in bottom-to-top order.
    pub fn replace_chunk<'a, I>(&mut self, chunk_x: i32, chunk_z: i32, sections: I) -> bool
    where
        I: IntoIterator<Item = SectionInput<'a>>,
    {
        let section_base_y = self.dimension.min_section_y();
        let sections = sections
            .into_iter()
            .enumerate()
            .map(|(index, input)| {
                (
                    section_base_y + index as i32,
                    ClientSection {
                        non_empty_block_count: input.non_empty_block_count,
                        fluid_count: input.fluid_count,
                        block_states: paletted::PalettedStates::from_values(input.block_states),
                        biomes: input.biomes.to_vec(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        // `initializeLightSources`.
        let mut sky_sources = light::ChunkSkyLightSources::new(self.dimension.min_y);
        let min_section = self.dimension.min_section_y();
        sky_sources.fill_from(self.dimension.section_count(), |index| {
            sections
                .get(&(min_section + index as i32))
                .filter(|section| !section.has_only_air())
                .map(|section| &section.block_states)
        });
        let mut new_chunk = ClientChunk {
            sections,
            sky_sources: Some(sky_sources),
            block_entities: BTreeMap::new(),
            chests: std::collections::BTreeSet::new(),
        };
        new_chunk.chests = block::chest::chests_of(chunk_x, chunk_z, &new_chunk.sections);
        let new_section_statuses = new_chunk
            .sections
            .iter()
            .map(|(&section_y, section)| (section_y, section.has_only_air()))
            .collect::<Vec<_>>();

        if let Some(cache) = self.chunk_cache.as_ref() {
            if !cache.in_range(chunk_x, chunk_z) {
                return false;
            }
            let index = cache.index(chunk_x, chunk_z);
            let current_chunk = cache.slots.get(&index).copied();
            if current_chunk == Some((chunk_x, chunk_z)) {
                self.chunks.insert((chunk_x, chunk_z), new_chunk);
                self.chunk_cache
                    .as_mut()
                    .expect("chunk cache exists during refresh")
                    .refresh_empty_sections(chunk_x, chunk_z, new_section_statuses);
            } else {
                if let Some((old_x, old_z)) = current_chunk
                    && let Some(old_chunk) = self.chunks.remove(&(old_x, old_z))
                {
                    let old_section_ys = old_chunk.sections.keys().copied().collect::<Vec<_>>();
                    self.chunk_cache
                        .as_mut()
                        .expect("chunk cache exists during replacement")
                        .on_chunk_removed(old_x, old_z, old_section_ys);
                    // `Storage.replace`: the chunk pushed out is unloaded.
                    self.unload_chunk_light(old_x, old_z);
                }
                self.chunks.insert((chunk_x, chunk_z), new_chunk);
                let cache = self
                    .chunk_cache
                    .as_mut()
                    .expect("chunk cache exists during insertion");
                cache.slots.insert(index, (chunk_x, chunk_z));
                cache.on_chunk_added(chunk_x, chunk_z, new_section_statuses);
            }
        } else {
            self.chunks.insert((chunk_x, chunk_z), new_chunk);
        }
        for entity in self.entities.values_mut() {
            if entity_chunk_key(entity.position()) == (chunk_x, chunk_z) {
                entity.ticking = true;
            }
        }
        true
    }
}
