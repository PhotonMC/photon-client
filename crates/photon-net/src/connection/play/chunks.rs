use super::*;

impl PlaySession {
    /// `handleLevelChunkWithLight`: the chunk, then its light.
    pub(super) fn apply_chunk(&mut self, chunk: &ChunkPacketObservation) {
        self.mark_columns_around(chunk.x, chunk.z);
        self.world.replace_chunk(
            chunk.x,
            chunk.z,
            chunk
                .section_observations
                .iter()
                .map(|section| SectionInput {
                    non_empty_block_count: section.non_empty_block_count,
                    fluid_count: section.fluid_count,
                    block_states: &section.block_state_values,
                    biomes: &section.biome_values,
                }),
        );
        self.world.apply_block_entities(
            chunk.x,
            chunk.z,
            chunk.block_entities.iter().map(|entity| BlockEntityInput {
                x: entity.x,
                y: entity.y,
                z: entity.z,
                type_id: entity.type_id,
                tag: entity.tag.as_deref(),
            }),
        );
        self.world.apply_chunk_light(
            chunk.x,
            chunk.z,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &chunk.light_data.sky_mask,
                    empty_mask: &chunk.light_data.empty_sky_mask,
                    updates: &chunk.light_data.sky_updates,
                },
                block: LightLayerInput {
                    mask: &chunk.light_data.block_mask,
                    empty_mask: &chunk.light_data.empty_block_mask,
                    updates: &chunk.light_data.block_updates,
                },
            },
        );
    }

    /// `handleLightUpdatePacket`: `readSectionList` redraws the sections
    /// whose light arrived when the queued task runs.
    pub(super) fn apply_light_update(&mut self, update: &LightUpdateObservation) {
        self.world.apply_light_updates(
            update.x,
            update.z,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &update.light_data.sky_mask,
                    empty_mask: &update.light_data.empty_sky_mask,
                    updates: &update.light_data.sky_updates,
                },
                block: LightLayerInput {
                    mask: &update.light_data.block_mask,
                    empty_mask: &update.light_data.empty_block_mask,
                    updates: &update.light_data.block_updates,
                },
            },
        );
    }

    /// Whether the level is ready and the player is being ticked.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Sections whose drawing may have changed since the last call: their
    /// blocks, their light, or a neighbour they cull or shade against.
    pub fn take_dirty_sections(&mut self) -> Vec<(i32, i32, i32)> {
        std::mem::take(&mut self.dirty_sections)
            .into_iter()
            .collect()
    }

    /// Chunks the server unloaded since the last call.
    pub fn take_forgotten_chunks(&mut self) -> Vec<(i32, i32)> {
        std::mem::take(&mut self.forgotten_chunks)
    }

    pub(super) fn section_range(&self) -> std::ops::Range<i32> {
        let dimension = self.world.dimension();
        let low = dimension.min_y.div_euclid(16);
        low..low + dimension.height / 16
    }

    /// Mark a chunk column and every column around it.
    pub(super) fn mark_columns_around(&mut self, chunk_x: i32, chunk_z: i32) {
        for section_y in self.section_range() {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    self.dirty_sections
                        .insert((chunk_x + dx, section_y, chunk_z + dz));
                }
            }
        }
    }

    /// `ClientLevel.update`'s light part, once a frame: queued light tasks
    /// and the engine run, and the sections whose light changed redraw.
    pub fn update_light(&mut self) {
        for section in self.world.update_light() {
            self.dirty_sections.insert(section);
        }
    }

    /// Mark the section holding a block and the sections touching it.
    pub(super) fn mark_block(&mut self, x: i32, y: i32, z: i32) {
        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                for dz in [-1, 0, 1] {
                    self.dirty_sections.insert((
                        (x + dx).div_euclid(16),
                        (y + dy).div_euclid(16),
                        (z + dz).div_euclid(16),
                    ));
                }
            }
        }
    }
}
