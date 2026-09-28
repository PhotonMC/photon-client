//! A copy of a small box of the world that can leave the thread the world
//! lives on.
//!
//! The section mesher reads block states, light and biomes a little beyond
//! the section it builds. Copying that box up front, a whole chunk section at
//! a time, lets the meshing run on another thread while the world keeps
//! taking packets, the way the reference copies sections into a render
//! region before compiling them.

use super::ClientWorld;
use super::chunk::ClientChunk;

/// Block states and light in a box of blocks, and the biomes, in quarts,
/// under it.
#[derive(Debug, Clone)]
pub struct RegionCopy {
    min: [i32; 3],
    size: [i32; 3],
    states: Vec<Option<i32>>,
    /// Block light, then sky light.
    light: Vec<(u8, u8)>,
    quart_min: [i32; 3],
    quart_size: [i32; 3],
    biomes: Vec<Option<i32>>,
    /// The dimension's quart rows, which a biome lookup is clamped into.
    quart_rows: (i32, i32),
}

impl RegionCopy {
    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let local = [x - self.min[0], y - self.min[1], z - self.min[2]];
        (0..3)
            .all(|axis| (0..self.size[axis]).contains(&local[axis]))
            .then(|| ((local[1] * self.size[2] + local[2]) * self.size[0] + local[0]) as usize)
    }

    /// The block state, nothing where no chunk was loaded or outside the box.
    pub fn block_state_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        self.states[self.index(x, y, z)?]
    }

    /// Block and sky light, as the world answered them.
    pub fn light_at(&self, x: i32, y: i32, z: i32) -> (u8, u8) {
        self.index(x, y, z)
            .map_or((0, 15), |index| self.light[index])
    }

    /// The biome at a quart position, its height clamped into the dimension
    /// as the world's noise biome lookup does.
    pub fn noise_biome(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        let y = y.clamp(self.quart_rows.0, self.quart_rows.1);
        let local = [
            x - self.quart_min[0],
            y - self.quart_min[1],
            z - self.quart_min[2],
        ];
        if !(0..3).all(|axis| (0..self.quart_size[axis]).contains(&local[axis])) {
            return None;
        }
        self.biomes
            [((local[1] * self.quart_size[2] + local[2]) * self.quart_size[0] + local[0]) as usize]
    }
}

impl ClientWorld {
    /// Whether a section holds nothing but air, or is not loaded at all.
    pub fn section_is_empty(&self, chunk_x: i32, section_y: i32, chunk_z: i32) -> bool {
        self.chunk_for_query(chunk_x, chunk_z)
            .and_then(|chunk| chunk.sections.get(&section_y))
            .is_none_or(|section| section.has_only_air())
    }

    /// Copy the blocks and light in `[min, min + size)` and the quart biomes
    /// in `[quart_min, quart_min + quart_size)`. Every value is what the
    /// world's own lookups answer.
    pub fn copy_region(
        &self,
        min: [i32; 3],
        size: [i32; 3],
        quart_min: [i32; 3],
        quart_size: [i32; 3],
    ) -> RegionCopy {
        let volume = (size[0] * size[1] * size[2]) as usize;
        let mut states = vec![None; volume];
        let mut light = vec![(0, 15); volume];
        for chunk_x in min[0].div_euclid(16)..=(min[0] + size[0] - 1).div_euclid(16) {
            for chunk_z in min[2].div_euclid(16)..=(min[2] + size[2] - 1).div_euclid(16) {
                let Some(chunk) = self.chunk_for_query(chunk_x, chunk_z) else {
                    continue;
                };
                let sky_top = sky_top(chunk);
                let x_range = (chunk_x * 16).max(min[0])..(chunk_x * 16 + 16).min(min[0] + size[0]);
                let z_range = (chunk_z * 16).max(min[2])..(chunk_z * 16 + 16).min(min[2] + size[2]);
                for y in min[1]..min[1] + size[1] {
                    let section_y = y.div_euclid(16);
                    let local_y = y.rem_euclid(16) as usize;
                    let section = chunk.sections.get(&section_y);
                    let light_section = chunk.lights.get(&section_y);
                    let block_light = light_section.and_then(|section| section.block.as_ref());
                    let sky_light = light_section.and_then(|section| section.sky.as_ref());
                    for z in z_range.clone() {
                        for x in x_range.clone() {
                            let within = local_y * 256
                                + z.rem_euclid(16) as usize * 16
                                + x.rem_euclid(16) as usize;
                            let nibble =
                                |data: &Vec<u8>| (data[within / 2] >> ((within & 1) * 4)) & 0x0f;
                            let index = (((y - min[1]) * size[2] + (z - min[2])) * size[0]
                                + (x - min[0])) as usize;
                            states[index] = section
                                .and_then(|section| section.block_states.get(within).copied());
                            let block = block_light.map_or(0, nibble);
                            let sky = match (sky_top, sky_light) {
                                (None, _) => 15,
                                (Some(_), Some(data)) => nibble(data),
                                (Some(top), None) => sky_above(chunk, top, section_y, x, z),
                            };
                            light[index] = (block, sky);
                        }
                    }
                }
            }
        }

        let dimension = self.dimension();
        let quart_rows = (
            dimension.min_y.div_euclid(4),
            dimension.min_y.div_euclid(4) + dimension.height / 4 - 1,
        );
        let mut biomes =
            Vec::with_capacity((quart_size[0] * quart_size[1] * quart_size[2]) as usize);
        for y in quart_min[1]..quart_min[1] + quart_size[1] {
            for z in quart_min[2]..quart_min[2] + quart_size[2] {
                for x in quart_min[0]..quart_min[0] + quart_size[0] {
                    let y = y.clamp(quart_rows.0, quart_rows.1);
                    biomes.push(self.biome_at(x * 4, y * 4, z * 4));
                }
            }
        }
        RegionCopy {
            min,
            size,
            states,
            light,
            quart_min,
            quart_size,
            biomes,
            quart_rows,
        }
    }
}

/// The section above the chunk's highest sky light data, where the sky
/// lookup stops and answers full light.
fn sky_top(chunk: &ClientChunk) -> Option<i32> {
    chunk
        .lights
        .iter()
        .filter(|(_, section)| section.sky.is_some())
        .map(|(section_y, _)| section_y + 1)
        .max()
}

/// `sky_brightness` for a section without its own sky data: the bottom row
/// of the first section above that has some.
fn sky_above(chunk: &ClientChunk, top: i32, section_y: i32, x: i32, z: i32) -> u8 {
    let mut section_y = section_y + 1;
    while section_y < top {
        if let Some(data) = chunk
            .lights
            .get(&section_y)
            .and_then(|section| section.sky.as_ref())
        {
            let index = z.rem_euclid(16) as usize * 16 + x.rem_euclid(16) as usize;
            return (data[index / 2] >> ((index & 1) * 4)) & 0x0f;
        }
        section_y += 1;
    }
    15
}
