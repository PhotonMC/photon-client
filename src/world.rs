//! Queryable client-side world state built from decoded chunk sections.

use std::collections::BTreeMap;

/// The vertical bounds used when mapping a chunk packet's section order to Y.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionBounds {
    pub min_y: i32,
    pub height: i32,
}

impl DimensionBounds {
    pub const fn new(min_y: i32, height: i32) -> Self {
        Self { min_y, height }
    }

    pub fn section_count(self) -> usize {
        if self.height <= 0 || self.height % 16 != 0 {
            return 0;
        }
        (self.height / 16) as usize
    }

    fn min_section_y(self) -> i32 {
        self.min_y.div_euclid(16)
    }
}

impl Default for DimensionBounds {
    fn default() -> Self {
        Self::new(-64, 384)
    }
}

/// The client-side copy of the currently loaded world.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientWorld {
    dimension: DimensionBounds,
    chunks: BTreeMap<(i32, i32), ClientChunk>,
    game_time: u64,
}

impl ClientWorld {
    pub fn new(dimension: DimensionBounds) -> Self {
        Self {
            dimension,
            chunks: BTreeMap::new(),
            game_time: 0,
        }
    }

    pub fn dimension(&self) -> DimensionBounds {
        self.dimension
    }

    pub fn game_time(&self) -> u64 {
        self.game_time
    }

    pub fn tick(&mut self) {
        self.game_time = self.game_time.wrapping_add(1);
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn loaded_section_count(&self) -> usize {
        self.chunks.values().map(|chunk| chunk.sections.len()).sum()
    }

    pub fn contains_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunks.contains_key(&(chunk_x, chunk_z))
    }

    pub fn unload_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunks.remove(&(chunk_x, chunk_z)).is_some()
    }

    /// Return the resolved global block-state ID at a world coordinate.
    pub fn block_state_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunks.get(&chunk_key)?;
        let section_y = y.div_euclid(16);
        let section = chunk.sections.get(&section_y)?;
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        section.block_states.get(index).copied()
    }

    /// Apply a server-verified block-state update to an already loaded section.
    pub(crate) fn update_block_state(&mut self, x: i32, y: i32, z: i32, state_id: i32) -> bool {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let Some(chunk) = self.chunks.get_mut(&chunk_key) else {
            return false;
        };
        let section_y = y.div_euclid(16);
        let Some(section) = chunk.sections.get_mut(&section_y) else {
            return false;
        };
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        let Some(block_state) = section.block_states.get_mut(index) else {
            return false;
        };
        *block_state = state_id;
        true
    }

    pub fn section_counts_at(
        &self,
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
    ) -> Option<(i16, i16)> {
        let chunk = self.chunks.get(&(chunk_x, chunk_z))?;
        let section = chunk.sections.get(&section_y)?;
        Some((section.non_empty_block_count, section.fluid_count))
    }

    /// Return the resolved biome registry ID at a world coordinate.
    pub fn biome_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunks.get(&chunk_key)?;
        let section_y = y.div_euclid(16);
        let section = chunk.sections.get(&section_y)?;
        let local_x = x.rem_euclid(16) as usize / 4;
        let local_y = y.rem_euclid(16) as usize / 4;
        let local_z = z.rem_euclid(16) as usize / 4;
        let index = local_y * 16 + local_z * 4 + local_x;
        section.biomes.get(index).copied()
    }

    pub(crate) fn replace_chunk<'a, I>(&mut self, chunk_x: i32, chunk_z: i32, sections: I)
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
                        block_states: input.block_states.to_vec(),
                        biomes: input.biomes.to_vec(),
                    },
                )
            })
            .collect();
        self.chunks
            .insert((chunk_x, chunk_z), ClientChunk { sections });
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ClientChunk {
    sections: BTreeMap<i32, ClientSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClientSection {
    non_empty_block_count: i16,
    fluid_count: i16,
    block_states: Vec<i32>,
    biomes: Vec<i32>,
}

pub(crate) struct SectionInput<'a> {
    pub(crate) non_empty_block_count: i16,
    pub(crate) fluid_count: i16,
    pub(crate) block_states: &'a [i32],
    pub(crate) biomes: &'a [i32],
}

#[cfg(test)]
mod tests {
    use super::{ClientWorld, DimensionBounds, SectionInput};

    fn section(block_value: i32, biome_value: i32) -> (Vec<i32>, Vec<i32>) {
        let mut blocks = vec![block_value; 4_096];
        blocks[0] = 11;
        blocks[15] = 12;
        blocks[16] = 13;
        blocks[255] = 14;
        blocks[256] = 15;
        blocks[4_095] = 16;

        let mut biomes = vec![biome_value; 64];
        biomes[0] = 21;
        biomes[3] = 22;
        biomes[4] = 23;
        biomes[63] = 24;
        (blocks, biomes)
    }

    #[test]
    fn chunk_sections_are_queryable_with_java_local_order() {
        let (blocks, biomes) = section(7, 8);
        let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
        world.replace_chunk(
            -1,
            2,
            [SectionInput {
                non_empty_block_count: 6,
                fluid_count: 1,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );

        assert_eq!(world.loaded_chunk_count(), 1);
        assert_eq!(world.loaded_section_count(), 1);
        assert_eq!(world.section_counts_at(-1, -4, 2), Some((6, 1)));
        assert_eq!(world.block_state_at(-16, -64, 32), Some(11));
        assert_eq!(world.block_state_at(-1, -64, 32), Some(12));
        assert_eq!(world.block_state_at(-16, -64, 33), Some(13));
        assert_eq!(world.block_state_at(-1, -64, 47), Some(14));
        assert_eq!(world.block_state_at(-16, -63, 32), Some(15));
        assert_eq!(world.block_state_at(-1, -49, 47), Some(16));
        assert_eq!(world.block_state_at(-17, -64, 32), None);
        assert_eq!(world.biome_at(-16, -64, 32), Some(21));
        assert_eq!(world.biome_at(-4, -64, 32), Some(22));
        assert_eq!(world.biome_at(-16, -64, 36), Some(23));
        assert_eq!(world.biome_at(-1, -49, 47), Some(24));
    }

    #[test]
    fn server_block_update_replaces_loaded_state() {
        let (blocks, biomes) = section(7, 8);
        let mut world = ClientWorld::default();
        world.replace_chunk(
            -1,
            2,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );

        assert_eq!(world.block_state_at(-1, -64, 32), Some(12));
        assert!(world.update_block_state(-1, -64, 32, 86));
        assert_eq!(world.block_state_at(-1, -64, 32), Some(86));
        assert!(!world.update_block_state(-17, -64, 32, 86));
        assert!(!world.update_block_state(-1, -48, 32, 86));
    }

    #[test]
    fn replacing_and_unloading_a_chunk_updates_the_section_cache() {
        let (first_blocks, first_biomes) = section(7, 8);
        let (mut second_blocks, mut second_biomes) = section(17, 18);
        second_blocks[0] = 31;
        second_biomes[0] = 32;
        let mut world = ClientWorld::default();
        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &first_blocks,
                biomes: &first_biomes,
            }],
        );
        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 2,
                fluid_count: 0,
                block_states: &second_blocks,
                biomes: &second_biomes,
            }],
        );

        assert_eq!(world.loaded_chunk_count(), 1);
        assert_eq!(world.loaded_section_count(), 1);
        assert_eq!(world.block_state_at(0, -64, 0), Some(31));
        assert_eq!(world.biome_at(0, -64, 0), Some(32));
        assert!(world.unload_chunk(0, 0));
        assert!(!world.contains_chunk(0, 0));
        assert_eq!(world.loaded_section_count(), 0);
        assert_eq!(world.block_state_at(0, -64, 0), None);
        assert!(!world.unload_chunk(0, 0));
    }

    #[test]
    fn tick_counter_is_independent_of_chunk_storage() {
        let mut world = ClientWorld::default();
        assert_eq!(world.game_time(), 0);
        world.tick();
        world.tick();
        assert_eq!(world.game_time(), 2);
        assert_eq!(world.loaded_chunk_count(), 0);
    }
}
