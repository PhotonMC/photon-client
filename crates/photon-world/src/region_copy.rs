//! A copy of a small box of the world that can leave the thread the world
//! lives on.
//!
//! The section mesher reads block states, light and biomes a little beyond
//! the section it builds. Copying that box up front, a whole chunk section at
//! a time, lets the meshing run on another thread while the world keeps
//! taking packets, the way the reference copies sections into a render
//! region before compiling them.

use super::ClientWorld;
use photon_core::ids::BlockStateId;

/// Block states and light in a box of blocks, and the biomes, in quarts,
/// under it.
#[derive(Debug, Clone)]
pub struct RegionCopy {
    min: [i32; 3],
    size: [i32; 3],
    states: Vec<Option<BlockStateId>>,
    /// Block light, then sky light.
    light: Vec<(u8, u8)>,
    quart_min: [i32; 3],
    quart_size: [i32; 3],
    biomes: Vec<Option<i32>>,
    /// The dimension's quart rows, which a biome lookup is clamped into.
    quart_rows: (i32, i32),
    /// The block tags the server last sent, which face culling reads.
    tags: std::sync::Arc<crate::block::tags::BlockTags>,
}

impl RegionCopy {
    /// The block tags the server had sent when the copy was made.
    pub fn tags(&self) -> &crate::block::tags::BlockTags {
        &self.tags
    }

    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let local = [x - self.min[0], y - self.min[1], z - self.min[2]];
        (0..3)
            .all(|axis| (0..self.size[axis]).contains(&local[axis]))
            .then(|| ((local[1] * self.size[2] + local[2]) * self.size[0] + local[0]) as usize)
    }

    /// The block state, nothing where no chunk was loaded or outside the box.
    pub fn block_state_at(&self, x: i32, y: i32, z: i32) -> Option<BlockStateId> {
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

    /// [`Self::section_is_empty`] for each of a loaded column's sections,
    /// bottom to top, with the chunk found once; `None` if it is not loaded.
    pub fn column_is_empty(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sections: std::ops::Range<i32>,
    ) -> Option<impl Iterator<Item = bool> + '_> {
        let chunk = self.chunk_for_query(chunk_x, chunk_z)?;
        Some(sections.map(|section_y| {
            chunk
                .sections
                .get(&section_y)
                .is_none_or(|section| section.has_only_air())
        }))
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
                let x_range = (chunk_x * 16).max(min[0])..(chunk_x * 16 + 16).min(min[0] + size[0]);
                let z_range = (chunk_z * 16).max(min[2])..(chunk_z * 16 + 16).min(min[2] + size[2]);
                for y in min[1]..min[1] + size[1] {
                    let section = chunk.sections.get(&y.div_euclid(16));
                    let local_y = y.rem_euclid(16) as usize;
                    for z in z_range.clone() {
                        for x in x_range.clone() {
                            let within = local_y * 256
                                + z.rem_euclid(16) as usize * 16
                                + x.rem_euclid(16) as usize;
                            let index = (((y - min[1]) * size[2] + (z - min[2])) * size[0]
                                + (x - min[0])) as usize;
                            states[index] =
                                section.and_then(|section| section.block_states.get(within));
                        }
                    }
                }
            }
        }
        self.copy_light(min, size, &mut light);

        let quart_rows = self.dimension().quart_rows();
        let mut biomes =
            Vec::with_capacity((quart_size[0] * quart_size[1] * quart_size[2]) as usize);
        for y in quart_min[1]..quart_min[1] + quart_size[1] {
            for z in quart_min[2]..quart_min[2] + quart_size[2] {
                for x in quart_min[0]..quart_min[0] + quart_size[0] {
                    biomes.push(self.noise_biome(x, y, z));
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
            tags: std::sync::Arc::clone(self.block_state_registry.tags()),
        }
    }
}

impl ClientWorld {
    /// Light as the level's getters read it, loaded chunk or not; a row is read a section's
    /// width at a time.
    fn copy_light(&self, min: [i32; 3], size: [i32; 3], light: &mut [(u8, u8)]) {
        let width = size[0] as usize;
        let (mut block_row, mut sky_row) = (vec![0; width], vec![0; width]);
        for y in min[1]..min[1] + size[1] {
            for z in min[2]..min[2] + size[2] {
                let mut start = 0;
                while start < width {
                    let x = min[0] + start as i32;
                    let run = ((16 - x.rem_euclid(16)) as usize).min(width - start);
                    let range = start..start + run;
                    self.light.light_run(
                        crate::light::LightLayer::Block,
                        (x, y, z),
                        &mut block_row[range.clone()],
                    );
                    self.light.light_run(
                        crate::light::LightLayer::Sky,
                        (x, y, z),
                        &mut sky_row[range],
                    );
                    start += run;
                }
                let row = (((y - min[1]) * size[2] + (z - min[2])) * size[0]) as usize;
                for (offset, (block, sky)) in block_row.iter().zip(&sky_row).enumerate() {
                    light[row + offset] = (*block, *sky);
                }
            }
        }
    }
}

impl ClientWorld {
    /// `BlockGetter.clip` against block shapes only, as the third-person
    /// camera asks it: where the segment first meets a block, if anywhere.
    /// The collision shape stands in for the visual shape, which is the same
    /// for every block but a handful (powder snow, barriers, light blocks).
    pub fn clip_blocks(
        &self,
        from: photon_core::math::Vec3,
        to: photon_core::math::Vec3,
    ) -> Option<photon_core::math::Vec3> {
        let registry = &self.block_state_registry;
        self.clip_with(from, to, |state, at, position| {
            registry
                .collision_shape_at(state, at)
                .clip(from, to, position)
        })
        .map(|hit| hit.location)
    }

    /// `BlockGetter.clip` against block outlines, fluids passed through, as
    /// picking asks it: where the segment first meets a block's outline.
    pub fn clip_outlines(
        &self,
        from: photon_core::math::Vec3,
        to: photon_core::math::Vec3,
    ) -> Option<photon_core::aabb::BlockHitResult> {
        self.clip_with(from, to, |state, at, position| {
            crate::outline_shape(state, at).clip(from, to, position)
        })
    }

    /// `BlockGetter.clip` against block outlines and the shapes of fluids
    /// (`ClipContext.Fluid.ANY`): in each block the nearer of the two hits.
    pub fn clip_outlines_and_fluids(
        &self,
        from: photon_core::math::Vec3,
        to: photon_core::math::Vec3,
    ) -> Option<photon_core::aabb::BlockHitResult> {
        self.clip_with_fluids(from, to, false)
    }

    /// `BlockGetter.clip` against block outlines and the shapes of fluid sources
    /// (`ClipContext.Fluid.SOURCE_ONLY`), as `Item.getPlayerPOVHitResult` asks it.
    pub fn clip_outlines_and_sources(
        &self,
        from: photon_core::math::Vec3,
        to: photon_core::math::Vec3,
    ) -> Option<photon_core::aabb::BlockHitResult> {
        self.clip_with_fluids(from, to, true)
    }

    fn clip_with_fluids(
        &self,
        from: photon_core::math::Vec3,
        to: photon_core::math::Vec3,
        sources_only: bool,
    ) -> Option<photon_core::aabb::BlockHitResult> {
        self.clip_with(from, to, |state, at, position| {
            let block = crate::outline_shape(state, at).clip(from, to, position);
            let source = !sources_only || self.fluid_at(at).is_source();
            let fluid = if source {
                self.fluid_shape(state, at).clip(from, to, position)
            } else {
                None
            };
            let distance = |hit: &Option<photon_core::aabb::BlockHitResult>| {
                hit.as_ref()
                    .map_or(f64::MAX, |hit| from.distance_to_sqr(hit.location))
            };
            if distance(&block) <= distance(&fluid) {
                block
            } else {
                fluid
            }
        })
    }

    /// The fluid in a block position (`getFluidState`).
    pub fn fluid_at(&self, [x, y, z]: [i32; 3]) -> crate::block::fluid::FluidState {
        self.block_state_at(x, y, z)
            .and_then(photon_data::block::registry::block_for_state)
            .map_or(crate::block::fluid::FluidState::EMPTY, |(block, offset)| {
                crate::block::fluid::fluid_state(block, offset)
            })
    }

    /// `FlowingFluid.getShape`: a box up to the fluid's height, a full block under
    /// the same fluid; nothing where the fluid is empty.
    fn fluid_shape(&self, state: BlockStateId, at: [i32; 3]) -> photon_core::shape::VoxelShape {
        use photon_core::shape::Shapes;
        let fluid = photon_data::block::registry::block_for_state(state)
            .map_or(crate::block::fluid::FluidState::EMPTY, |(block, offset)| {
                crate::block::fluid::fluid_state(block, offset)
            });
        if fluid.is_empty() {
            return Shapes::empty();
        }
        let above = self.fluid_at([at[0], at[1] + 1, at[2]]);
        let height = if !above.is_empty() && fluid.is_same(above.fluid) {
            1.0
        } else {
            fluid.own_height()
        };
        Shapes::box_shape(0.0, 0.0, 0.0, 1.0, f64::from(height), 1.0)
            .unwrap_or_else(|_| Shapes::empty())
    }

    /// `BlockGetter.traverseBlocks` along a segment, clipping each block's
    /// shape as `shape` gives it.
    fn clip_with(
        &self,
        from: photon_core::math::Vec3,
        to: photon_core::math::Vec3,
        hit: impl Fn(
            BlockStateId,
            [i32; 3],
            crate::BlockPos,
        ) -> Option<photon_core::aabb::BlockHitResult>,
    ) -> Option<photon_core::aabb::BlockHitResult> {
        use photon_core::math::floor_f64;
        if from.x == to.x && from.y == to.y && from.z == to.z {
            return None;
        }
        let lerp = |delta: f64, start: f64, end: f64| start + delta * (end - start);
        let to_x = lerp(-1.0e-7, to.x, from.x);
        let to_y = lerp(-1.0e-7, to.y, from.y);
        let to_z = lerp(-1.0e-7, to.z, from.z);
        let from_x = lerp(-1.0e-7, from.x, to.x);
        let from_y = lerp(-1.0e-7, from.y, to.y);
        let from_z = lerp(-1.0e-7, from.z, to.z);
        let mut block = [floor_f64(from_x), floor_f64(from_y), floor_f64(from_z)];
        let hit_at = |block: [i32; 3]| {
            let state = self.block_state_at(block[0], block[1], block[2])?;
            let position = crate::BlockPos::new(block[0], block[1], block[2]);
            hit(state, block, position)
        };
        if let Some(hit) = hit_at(block) {
            return Some(hit);
        }
        let delta = [to_x - from_x, to_y - from_y, to_z - from_z];
        let start = [from_x, from_y, from_z];
        let (sign, step, mut t) = voxel_walk(start, delta);
        while t[0] <= 1.0 || t[1] <= 1.0 || t[2] <= 1.0 {
            let axis = if t[0] < t[1] {
                if t[0] < t[2] { 0 } else { 2 }
            } else if t[1] < t[2] {
                1
            } else {
                2
            };
            block[axis] += sign[axis];
            t[axis] += step[axis];
            if let Some(hit) = hit_at(block) {
                return Some(hit);
            }
        }
        None
    }
}

/// The signs, the steps along each axis per unit of the segment and the first crossings of a
/// walk through blocks (`BlockGetter.traverseBlocks`).
fn voxel_walk(start: [f64; 3], delta: [f64; 3]) -> ([i32; 3], [f64; 3], [f64; 3]) {
    use photon_core::math::frac_f64;
    let sign = delta.map(|value| {
        if value > 0.0 {
            1
        } else if value < 0.0 {
            -1
        } else {
            0
        }
    });
    let step = [0, 1, 2].map(|axis| {
        if sign[axis] == 0 {
            f64::MAX
        } else {
            f64::from(sign[axis]) / delta[axis]
        }
    });
    let t = [0, 1, 2].map(|axis| {
        step[axis]
            * if sign[axis] > 0 {
                1.0 - frac_f64(start[axis])
            } else {
                frac_f64(start[axis])
            }
    });
    (sign, step, t)
}
