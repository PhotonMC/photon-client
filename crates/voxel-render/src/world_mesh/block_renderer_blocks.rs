use super::*;

impl BlockRenderer {
    /// [`Self::mesh_region`] and [`Self::mesh_moving_block`]: with a seed position, the
    /// blocks are those of a moving block (no fluids, variants picked from that position).
    #[expect(
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    pub(super) fn mesh_blocks<R, L, T>(
        &self,
        world: &MeshWorld<'_, R, L, T>,
        origin: [i32; 3],
        size: [i32; 3],
        mesh_origin: [i32; 3],
        seed_position: Option<[i32; 3]>,
    ) -> RegionMesh
    where
        R: Fn(i32, i32, i32) -> Option<i32>,
        L: Fn(i32, i32, i32) -> (u8, u8),
        T: Fn(crate::biome_tint::TintKind, [i32; 3]) -> u32,
    {
        let MeshWorld {
            read,
            light,
            biome_tint,
            tags,
        } = *world;
        let lighter = crate::block_light::Lighter {
            state: |p: [i32; 3]| read(p[0], p[1], p[2]).and_then(|state| u32::try_from(state).ok()),
            light: |p: [i32; 3]| light(p[0], p[1], p[2]),
        };
        let mut mesh = Mesh::default();
        let mut solid_count = 0;
        let mut tints = Vec::new();
        for offset_x in 0..size[0] {
            for offset_y in 0..size[1] {
                for offset_z in 0..size[2] {
                    let position = [
                        origin[0] + offset_x,
                        origin[1] + offset_y,
                        origin[2] + offset_z,
                    ];
                    let Some(state_id) = read(position[0], position[1], position[2])
                        .and_then(|state| u32::try_from(state).ok())
                    else {
                        continue;
                    };
                    let drawn = match self.states.get(state_id as usize) {
                        Some(StateRender::Air) => continue,
                        Some(StateRender::Drawn(drawn)) => drawn,
                        _ => {
                            solid_count += 1;
                            continue;
                        }
                    };
                    solid_count += 1;
                    // `shouldRenderFace` towards each side, asked at most once.
                    let mut hidden: [Option<bool>; 6] = [None; 6];
                    let mut is_hidden = |facing: Direction| {
                        *hidden[facing as usize].get_or_insert_with(|| {
                            let (step_x, step_y, step_z) = facing.step();
                            let neighbour = read(
                                position[0] + step_x,
                                position[1] + step_y,
                                position[2] + step_z,
                            )
                            .and_then(|state| u32::try_from(state).ok());
                            !voxel_world::block::should_render_face(
                                state_id, neighbour, facing, tags,
                            )
                        })
                    };
                    // Closed in on every side, it would draw no face at all.
                    if drawn.closes_in && Direction::stream().all(&mut is_hidden) {
                        continue;
                    }
                    let info = voxel_world::block::state_or_air(i64::from(state_id));
                    let mut random = voxel_core::random::RandomSource::create(
                        voxel_world::block::seed(info, seed_position.unwrap_or(position)),
                    );
                    let mut picked = Vec::new();
                    pick_variants(&drawn.lists, &mut random, |model| picked.push(model));
                    if picked.is_empty() {
                        continue;
                    }
                    drawn.tints.colours(position, biome_tint, &mut tints);
                    let nudge = voxel_world::block::offset(info.block(), position);
                    // A model is built in its own block's unit cube, so it
                    // moves to where the block sits, measured from the box's
                    // near corner.
                    let shift: [f32; 3] = std::array::from_fn(|axis| {
                        (position[axis] - mesh_origin[axis]) as f32 + nudge[axis] as f32
                    });
                    for model in picked.into_iter().flatten() {
                        let model = &self.baked[model as usize];
                        // Smooth lighting, the reference's default, unless the
                        // block glows or the model opts out of ambient occlusion.
                        let smooth = info.light_emission == 0 && model.ambient_occlusion;
                        for quad in &model.quads {
                            if quad.cull.is_some_and(&mut is_hidden) {
                                continue;
                            }
                            let tint = usize::try_from(quad.tint_index)
                                .ok()
                                .and_then(|slot| tints.get(slot).copied())
                                .unwrap_or(0xFFFF_FFFF);
                            push_block_quad(
                                &mut mesh,
                                quad,
                                &lighter.light_quad(
                                    state_id,
                                    position,
                                    &quad.corners,
                                    quad.direction,
                                    quad.cull,
                                    smooth,
                                ),
                                smooth,
                                tint,
                                shift,
                            );
                        }
                    }
                }
            }
        }

        let atlas = &self.atlas;
        let fluids = FluidMesher {
            biome_tint,
            read,
            lighter: &lighter,
            atlas,
            origin: mesh_origin,
        };
        // A moving block draws no fluid (`tesselateBlock` alone).
        let fluid_size = if seed_position.is_some() {
            [0; 3]
        } else {
            size
        };
        for offset_x in 0..fluid_size[0] {
            for offset_y in 0..fluid_size[1] {
                for offset_z in 0..fluid_size[2] {
                    fluids.tesselate(
                        [
                            origin[0] + offset_x,
                            origin[1] + offset_y,
                            origin[2] + offset_z,
                        ],
                        &mut mesh,
                    );
                }
            }
        }
        RegionMesh { mesh, solid_count }
    }
}
