//! The heightmaps a client chunk keeps (`Heightmap`): the three kinds the chunk packet carries,
//! each one value per column, stored as the reference packs them.

use photon_core::ids::BlockStateId;

use crate::block;

/// `Heightmap.Types` the server sends to the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    WorldSurface,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

impl HeightmapType {
    pub const ALL: [Self; 3] = [
        Self::WorldSurface,
        Self::MotionBlocking,
        Self::MotionBlockingNoLeaves,
    ];

    /// The type a chunk packet's id names; the world-generation and live-world ones, which the
    /// client chunk never reads, have none here.
    pub fn from_wire(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::WorldSurface),
            4 => Some(Self::MotionBlocking),
            5 => Some(Self::MotionBlockingNoLeaves),
            _ => None,
        }
    }

    fn slot(self) -> usize {
        match self {
            Self::WorldSurface => 0,
            Self::MotionBlocking => 1,
            Self::MotionBlockingNoLeaves => 2,
        }
    }

    /// The type's predicate on a block state (`Heightmap.Types.isOpaque`).
    pub fn is_opaque(self, state: BlockStateId) -> bool {
        match self {
            Self::WorldSurface => !block::rules::state_is_air(state),
            Self::MotionBlocking => crate::motion_blocking(state),
            Self::MotionBlockingNoLeaves => {
                crate::motion_blocking(state)
                    && !block::state_or_air(state).block().name.ends_with("_leaves")
            }
        }
    }
}

/// `Mth.ceillog2`.
fn ceil_log2(value: i32) -> u32 {
    if value <= 1 {
        0
    } else {
        32 - (value - 1).leading_zeros()
    }
}

/// One heightmap: for each of a chunk's 256 columns the first free height above the highest
/// block the type counts, as a number from the bottom of the world (`Heightmap`'s own storage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Heightmap {
    kind: HeightmapType,
    min_y: i32,
    heights: [u32; 256],
}

impl Heightmap {
    /// A map with every column at the bottom of the world.
    fn empty(kind: HeightmapType, min_y: i32) -> Self {
        Self {
            kind,
            min_y,
            heights: [0; 256],
        }
    }

    /// The map a packet's packed longs hold (`Heightmap.setRawData`), or none where the length
    /// is not the one a world of this height packs into.
    fn from_raw(kind: HeightmapType, bounds: (i32, i32), raw: &[i64]) -> Option<Self> {
        let (min_y, height) = bounds;
        let bits = ceil_log2(height + 1);
        let per_long = (64 / bits) as usize;
        if raw.len() != 256_usize.div_ceil(per_long) {
            return None;
        }
        let mask = (1_u64 << bits) - 1;
        let mut map = Self::empty(kind, min_y);
        for (index, height) in map.heights.iter_mut().enumerate() {
            let cell = raw[index / per_long] as u64;
            *height = ((cell >> ((index % per_long) as u32 * bits)) & mask) as u32;
        }
        Some(map)
    }

    /// `Heightmap.primeHeightmaps`: the highest state in each column that counts, found by
    /// looking at the blocks.
    fn primed(
        kind: HeightmapType,
        bounds: (i32, i32),
        state_at: impl Fn(usize, i32) -> BlockStateId,
    ) -> Self {
        let mut map = Self::empty(kind, bounds.0);
        for column in 0..256 {
            for y in (bounds.0..bounds.0 + bounds.1).rev() {
                if kind.is_opaque(state_at(column, y)) {
                    map.heights[column] = (y + 1 - bounds.0) as u32;
                    break;
                }
            }
        }
        map
    }

    /// `Heightmap.getFirstAvailable`: one above the highest counted block of a column.
    pub(crate) fn first_available(&self, column: usize) -> i32 {
        self.heights[column] as i32 + self.min_y
    }

    /// `Heightmap.update`: a block of a column changed to `state` at `y`; whether the column's
    /// height moved. `state_at` reads the column's blocks for the scan down.
    pub(crate) fn update(
        &mut self,
        column: usize,
        y: i32,
        state: BlockStateId,
        state_at: impl Fn(i32) -> BlockStateId,
    ) -> bool {
        let first_available = self.first_available(column);
        if y <= first_available - 2 {
            return false;
        }
        if self.kind.is_opaque(state) {
            if y >= first_available {
                self.set(column, y + 1);
                return true;
            }
        } else if first_available - 1 == y {
            for below in (self.min_y..y).rev() {
                if self.kind.is_opaque(state_at(below)) {
                    self.set(column, below + 1);
                    return true;
                }
            }
            self.set(column, self.min_y);
            return true;
        }
        false
    }

    fn set(&mut self, column: usize, height: i32) {
        self.heights[column] = (height - self.min_y) as u32;
    }
}

/// The heightmaps of one chunk, as `LevelChunk` creates them: every column at the bottom of the
/// world until the packet or the blocks say otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChunkHeightmaps {
    maps: [Heightmap; 3],
}

impl Default for ChunkHeightmaps {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ChunkHeightmaps {
    pub(crate) fn new(min_y: i32) -> Self {
        Self {
            maps: HeightmapType::ALL.map(|kind| Heightmap::empty(kind, min_y)),
        }
    }

    pub(crate) fn get(&self, kind: HeightmapType) -> &Heightmap {
        &self.maps[kind.slot()]
    }

    /// `LevelChunk.setHeightmap`: the packet's data for one type, or the blocks' where it is of
    /// another length.
    pub(crate) fn set_raw(
        &mut self,
        kind: HeightmapType,
        bounds: (i32, i32),
        raw: &[i64],
        state_at: impl Fn(usize, i32) -> BlockStateId,
    ) {
        self.maps[kind.slot()] = Heightmap::from_raw(kind, bounds, raw)
            .unwrap_or_else(|| Heightmap::primed(kind, bounds, state_at));
    }

    /// `Heightmap.primeHeightmaps` for one type.
    pub(crate) fn prime(
        &mut self,
        kind: HeightmapType,
        bounds: (i32, i32),
        state_at: impl Fn(usize, i32) -> BlockStateId,
    ) {
        self.maps[kind.slot()] = Heightmap::primed(kind, bounds, state_at);
    }

    /// `LevelChunk.setBlockState`'s update of every map.
    pub(crate) fn update(
        &mut self,
        column: usize,
        y: i32,
        state: BlockStateId,
        state_at: impl Fn(i32) -> BlockStateId + Copy,
    ) {
        for map in &mut self.maps {
            map.update(column, y, state, state_at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AIR: BlockStateId = BlockStateId(0);

    fn stone() -> BlockStateId {
        photon_data::block::registry::block("minecraft:stone")
            .unwrap()
            .default_state_id()
    }

    /// A world of the overworld's size packs nine bits to a value, seven values to a long, so a
    /// map is 37 longs; none of a long's values spans into the next.
    #[test]
    fn a_map_of_the_overworld_reads_seven_nine_bit_values_a_long() {
        assert_eq!(ceil_log2(385), 9);
        let mut raw = vec![0_i64; 37];
        // Column 0 holds 1, column 6 (the last in the first long) 5, column 7 (the first in the
        // second) 300 and the last column, 255, 384.
        raw[0] = 1 | 5 << (6 * 9);
        raw[1] = 300;
        raw[36] = 384 << (3 * 9);
        let map = Heightmap::from_raw(HeightmapType::WorldSurface, (-64, 384), &raw).unwrap();
        assert_eq!(map.first_available(0), -63);
        assert_eq!(map.first_available(6), -59);
        assert_eq!(map.first_available(7), 236);
        assert_eq!(map.first_available(255), 320);
        assert_eq!(map.first_available(1), -64);
    }

    #[test]
    fn a_map_of_another_length_is_not_taken() {
        let raw = vec![0_i64; 36];
        assert!(Heightmap::from_raw(HeightmapType::WorldSurface, (-64, 384), &raw).is_none());
    }

    #[test]
    fn world_heights_pack_by_their_bit_count() {
        assert_eq!(ceil_log2(257), 9);
        assert_eq!(ceil_log2(256), 8);
        assert_eq!(ceil_log2(17), 5);
    }

    #[test]
    fn a_block_below_the_top_two_leaves_the_map_alone() {
        let mut map = Heightmap::empty(HeightmapType::WorldSurface, 0);
        map.heights[0] = 10;
        // One at 8 sits at `first - 2`: ignored whatever it is; one at 9 is the top.
        let stone = stone();
        assert!(!map.update(0, 8, stone, |_| AIR));
        assert_eq!(map.first_available(0), 10);
        assert!(map.update(0, 12, stone, |_| AIR));
        assert_eq!(map.first_available(0), 13);
    }

    #[test]
    fn removing_the_top_block_scans_down_and_falls_to_the_bottom_when_nothing_is_left() {
        let stone = stone();
        let mut map = Heightmap::empty(HeightmapType::MotionBlocking, -64);
        map.heights[5] = (20 + 64) as u32 + 1;
        // The top is at 20, with stone also at 3.
        let column = |y: i32| if y == 3 { stone } else { AIR };
        assert!(map.update(5, 20, AIR, column));
        assert_eq!(map.first_available(5), 4);
        assert!(map.update(5, 3, AIR, column));
        assert_eq!(map.first_available(5), -64);
    }
}
