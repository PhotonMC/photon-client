use super::*;

impl ClientWorld {
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

    pub(super) fn light_at(&self, layer: LightLayer, x: i32, y: i32, z: i32) -> Option<u8> {
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
        new_chunk.chests = chest::chests_of(chunk_x, chunk_z, &new_chunk.sections);
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
