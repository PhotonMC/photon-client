use crate::*;

impl ClientWorld {
    /// Create the bounded chunk cache used by a Java client level.
    pub fn configure_chunk_cache(&mut self, server_view_radius: i32) {
        self.chunks_mut().reset(Some(server_view_radius));
        self.light = new_light_engine(
            self.level().dimension,
            self.level().dimension_environment.has_skylight,
        );
        self.light_tasks.clear();
    }

    /// Return the Java cache range and ring index for a chunk coordinate.
    pub fn chunk_cache_location(&self, chunk_x: i32, chunk_z: i32) -> Option<(bool, usize)> {
        self.chunks().cache().map(|cache| {
            (
                cache.in_range(chunk_x, chunk_z),
                cache.index(chunk_x, chunk_z),
            )
        })
    }

    /// Move the Java client chunk-cache view center without evicting slots.
    pub fn update_chunk_cache_center(&mut self, chunk_x: i32, chunk_z: i32) {
        if let Some(cache) = self.chunks_mut().cache_mut() {
            cache.view_center_x = chunk_x;
            cache.view_center_z = chunk_z;
        }
    }

    /// Resize the Java client chunk cache and retain chunks in the new range.
    pub fn update_chunk_cache_radius(&mut self, server_view_radius: i32) {
        self.chunks_mut().resize(server_view_radius);
    }

    /// Apply the cache's section-emptiness notification path.
    pub fn update_chunk_section_emptiness(
        &mut self,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        empty: bool,
    ) {
        if let Some(cache) = self.chunks_mut().cache_mut() {
            cache.on_section_emptiness_changed(section_x, section_y, section_z, empty);
        }
    }

    /// Swap the Java cache's double-buffered update sets.
    pub fn flip_chunk_cache_update_sets(&mut self) {
        if let Some(cache) = self.chunks_mut().cache_mut() {
            cache.flip_update_tracking_sets();
        }
    }

    /// Snapshot the loaded chunks and cache update sets in deterministic order.
    pub fn client_chunk_cache_snapshot(&self) -> Option<ClientChunkCacheSnapshot> {
        let cache = self.chunks().cache()?;
        let index = cache.updating_sets_index;
        Some(ClientChunkCacheSnapshot {
            storage_radius: cache.storage_radius,
            view_range: cache.view_range,
            view_center_x: cache.view_center_x,
            view_center_z: cache.view_center_z,
            loaded_chunks: self.chunks().positions(),
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
        self.chunks().len()
    }

    pub fn loaded_section_count(&self) -> usize {
        self.chunks()
            .values()
            .map(|chunk| chunk.sections.len())
            .sum()
    }

    pub(crate) fn chunk_for_query(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        self.chunks().get(chunk_x, chunk_z)
    }

    pub(crate) fn chunk_for_update(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&mut ClientChunk> {
        self.chunks_mut().get_mut(chunk_x, chunk_z)
    }

    pub fn contains_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunk_for_query(chunk_x, chunk_z).is_some()
    }

    /// `handleForgetLevelChunk`: the chunk is dropped if the cache holds it,
    /// and its light is queued for removal either way.
    pub fn unload_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> bool {
        self.queue_light_removal(chunk_x, chunk_z);
        let removed = self.chunks_mut().unload(chunk_x, chunk_z);
        if removed {
            self.unload_chunk_light(chunk_x, chunk_z);
            self.entities.for_each_mut(|mut entity| {
                if !entity.always_ticking
                    && entity_chunk_key(entity.position()) == (chunk_x, chunk_z)
                {
                    entity.ticking = false;
                }
            });
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
        self.level_mut().server_simulation_distance = chunks;
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
        self.biome_at(x * 4, self.level().dimension.clamp_quart_y(y) * 4, z * 4)
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
        let section_base_y = self.level().dimension.min_section_y();
        let sections = Sections::new(
            section_base_y,
            sections.into_iter().map(|input| ClientSection {
                non_empty_block_count: input.non_empty_block_count,
                fluid_count: input.fluid_count,
                block_states: paletted::PalettedStates::from_values(input.block_states),
                biomes: input.biomes.to_vec(),
            }),
        );
        // `initializeLightSources`.
        let mut sky_sources = light::ChunkSkyLightSources::new(self.level().dimension.min_y);
        let min_section = self.level().dimension.min_section_y();
        sky_sources.fill_from(self.level().dimension.section_count(), |index| {
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
            heightmaps: crate::heightmap::ChunkHeightmaps::new(self.level().dimension.min_y),
        };
        new_chunk.chests = block::chest::chests_of(chunk_x, chunk_z, &new_chunk.sections);
        let new_section_statuses = new_chunk
            .sections
            .iter()
            .map(|(section_y, section)| (section_y, section.has_only_air()))
            .collect::<Vec<_>>();

        match self
            .chunks_mut()
            .replace((chunk_x, chunk_z), new_chunk, new_section_statuses)
        {
            Replaced::OutOfRange => return false,
            // `Storage.replace`: the chunk pushed out is unloaded.
            Replaced::Stored {
                evicted: Some((old_x, old_z)),
            } => self.unload_chunk_light(old_x, old_z),
            Replaced::Stored { evicted: None } => {}
        }
        self.entities.for_each_mut(|mut entity| {
            if entity_chunk_key(entity.position()) == (chunk_x, chunk_z) {
                entity.ticking = true;
            }
        });
        true
    }
}
