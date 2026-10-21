//! The client chunk's heightmaps (`LevelChunk`, `ChunkAccess.getHeight`).

use crate::chunk::sections_state;
use crate::heightmap::HeightmapType;
use crate::*;

impl ClientWorld {
    /// `LevelChunk.replaceWithPacketData`'s heightmaps: each type the packet carries is taken
    /// as sent, or worked out from the blocks where its data is of another length.
    pub fn set_chunk_heightmaps<'a>(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        maps: impl IntoIterator<Item = (i32, &'a [i64])>,
    ) {
        let bounds = (self.dimension().min_y, self.dimension().height);
        let Some(chunk) = self.chunk_for_update(chunk_x, chunk_z) else {
            return;
        };
        for (id, raw) in maps {
            let Some(kind) = HeightmapType::from_wire(id) else {
                continue;
            };
            let sections = &chunk.sections;
            chunk.heightmaps.set_raw(kind, bounds, raw, |column, y| {
                sections_state(sections, column, y)
            });
        }
    }

    /// `Heightmap.primeHeightmaps`: the maps of the types worked out from the chunk's blocks.
    pub fn prime_chunk_heightmaps(&mut self, chunk_x: i32, chunk_z: i32, kinds: &[HeightmapType]) {
        let bounds = (self.dimension().min_y, self.dimension().height);
        let Some(chunk) = self.chunk_for_update(chunk_x, chunk_z) else {
            return;
        };
        for &kind in kinds {
            let sections = &chunk.sections;
            chunk.heightmaps.prime(kind, bounds, |column, y| {
                sections_state(sections, column, y)
            });
        }
    }

    /// `ChunkAccess.getHeight`: the height of the highest block a type counts at a column, one
    /// below the map's first available; none where the chunk is not loaded.
    pub fn chunk_height(&self, kind: HeightmapType, x: i32, z: i32) -> Option<i32> {
        let chunk = self.chunk_for_query(x.div_euclid(16), z.div_euclid(16))?;
        let column = z.rem_euclid(16) as usize * 16 + x.rem_euclid(16) as usize;
        Some(chunk.heightmaps.get(kind).first_available(column) - 1)
    }
}
