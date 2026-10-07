use super::*;

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
}
