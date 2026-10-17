use super::*;
use photon_core::ids::BlockStateId;

impl BlockRenderer {
    /// Read every block's blockstate file and every model any of its
    /// variants names, and pack all their textures into one atlas.
    pub fn new(source: &impl AssetSource, mip_level: i32) -> Result<Self, ResolveError> {
        use rayon::prelude::*;
        let mut model_cache = crate::resolve::ModelCache::default();
        // Every blockstate file, read and parsed in parallel.
        let files: Vec<Option<crate::block::blockstate::BlockStateFile>> = registry::BLOCKS
            .par_iter()
            .map(|block| {
                if is_air_block(block.name) {
                    return None;
                }
                Identifier::parse(block.name)
                    .ok()
                    .and_then(|name| source.block_state(&name))
                    .and_then(|text| crate::block::blockstate::parse_block_state(&text))
            })
            .collect();
        model_cache.prefetch(
            source,
            files
                .iter()
                .flatten()
                .flat_map(variant_lists)
                .flatten()
                .map(|variant| variant.model.clone()),
        );
        let frame_files = item_frame_files(source, &mut model_cache);
        let mut blockstates = BTreeMap::new();
        let mut models: BTreeMap<String, ResolvedModel> = BTreeMap::new();
        let mut skipped: Vec<(String, String)> = Vec::new();
        for (block, file) in registry::BLOCKS.iter().zip(files) {
            if is_air_block(block.name) {
                continue;
            }
            let Some(file) = file else {
                skipped.push((block.name.to_owned(), "no blockstate file".into()));
                continue;
            };
            for variant in variant_lists(&file).flatten() {
                let name = variant.model.as_string();
                if models.contains_key(&name) {
                    continue;
                }
                match model_cache.resolve(source, &variant.model) {
                    Ok(model) => {
                        models.insert(name, model);
                    }
                    Err(error) => {
                        if !skipped
                            .iter()
                            .any(|(block_name, _)| block_name == block.name)
                        {
                            skipped.push((block.name.to_owned(), format!("{error:?}")));
                        }
                    }
                }
            }
            blockstates.insert(block.name.to_owned(), file);
        }
        add_item_frame_models(&frame_files, (source, &mut model_cache), &mut models);
        let (mut pending, mut air) = state_lists(&blockstates);
        let frame_base = pending.len() as u32;
        add_item_frame_states(&frame_files, &mut pending, &mut air);
        let (items, layer_alpha, layer_sprites) =
            item_models(source, &mut model_cache, &mut models);
        let ordered: Vec<&ResolvedModel> = models.values().collect();
        let extra = extra_sprites(layer_sprites);
        let atlas = crate::resolve::build_atlas_with(source, &ordered, &extra, 8192, mip_level)?;
        let (baked, states) = bake_states(&models, &atlas, pending, &air);
        let particle_sprites = particle_sprites(&blockstates, &models);
        Ok(Self {
            models,
            baked,
            items,
            layer_alpha,
            states,
            particle_sprites,
            frame_base,
            atlas,
            skipped,
        })
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Every quad a block's model draws at `position`, culled against
    /// nothing, as the breaking overlay draws them: corners in the model's
    /// own unit cube and each quad's facing, with the block's offset. The
    /// variants are the ones the world mesh picks there.
    pub fn block_quads(
        &self,
        state_id: BlockStateId,
        position: [i32; 3],
    ) -> (Vec<FacedQuad>, [f32; 3]) {
        let Some(StateRender::Drawn(drawn)) = self.state_render(state_id) else {
            return (Vec::new(), [0.0; 3]);
        };
        let info = photon_world::block::state_or_air(state_id);
        let mut random =
            photon_core::random::RandomSource::create(photon_world::block::seed(info, position));
        let mut quads = Vec::new();
        pick_variants(&drawn.lists, &mut random, |model| {
            let Some(model) = model else {
                return;
            };
            quads.extend(
                self.baked[model as usize]
                    .quads
                    .iter()
                    .map(|quad| (quad.corners, quad.direction)),
            );
        });
        let offset = photon_world::block::offset(info.block(), position).map(|value| value as f32);
        (quads, offset)
    }

    /// How a state is drawn; none for a number outside the registry.
    fn state_render(&self, state: BlockStateId) -> Option<&StateRender> {
        self.states.get(usize::try_from(state.0).ok()?)
    }

    /// `BlockModelResolver.update` for a block state drawn on an entity: every
    /// quad of the variants the model's own seed picks (`MODEL_SEED`, 42),
    /// culled against nothing, each tinted by the state's colour outside a
    /// world, and the light the state gives off (`blockLightCoords`). Nothing
    /// for air and for states with no model to draw.
    pub fn entity_block(&self, state: BlockStateId) -> Option<(ItemLayer, i32)> {
        let Some(StateRender::Drawn(drawn)) = self.state_render(state) else {
            return None;
        };
        let tints: Vec<u32> = match &drawn.tints {
            StateTints::Fixed(colours) => colours.clone(),
            StateTints::Biome { constant, .. } => constant.to_vec(),
        };
        let mut random = photon_core::random::RandomSource::create(42);
        let mut mesh = Mesh::default();
        pick_variants(&drawn.lists, &mut random, |model| {
            let Some(model) = model else {
                return;
            };
            for quad in &self.baked[model as usize].quads {
                let tint = usize::try_from(quad.tint_index)
                    .ok()
                    .and_then(|slot| tints.get(slot).copied())
                    .unwrap_or(0xFFFF_FFFF);
                let [red, green, blue] =
                    [16, 8, 0].map(|bits| f32::from(((tint >> bits) & 0xFF) as u8) / 255.0);
                let base = mesh.vertices.len() as u32;
                for corner in 0..4 {
                    mesh.vertices.push(crate::scene::BlockVertex::new(
                        quad.corners[corner],
                        quad.uvs[corner],
                        [red, green, blue, 1.0],
                        [0.0; 2],
                    ));
                }
                mesh.indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                ]);
                mesh.quads.push(crate::scene::QuadInfo {
                    direction: quad.direction,
                    cull: None,
                    shade: quad.shade,
                    tint_index: quad.tint_index,
                });
            }
        });
        // A fake state is of air: it gives off no light.
        let info = photon_world::block::state_or_air(if state.0 as u32 >= self.frame_base {
            photon_world::VANILLA_AIR_BLOCK_STATE_ID
        } else {
            state
        });
        Some((
            ItemLayer {
                mesh,
                side_lit: true,
                transform: None,
            },
            crate::entity::block::block_light_coords(info.light_emission, info.emissive_rendering),
        ))
    }

    /// Mesh every block in a box of the world, positioned relative to
    /// `mesh_origin`.
    ///
    /// `read` answers the block state at a position, or nothing where no
    /// chunk is loaded, which reads as air. Neighbours outside the box are
    /// read from the world too, so a region's faces cull exactly as the
    /// whole world's would.
    pub fn mesh_region<R, L, T>(
        &self,
        world: &MeshWorld<'_, R, L, T>,
        origin: [i32; 3],
        size: [i32; 3],
        mesh_origin: [i32; 3],
    ) -> RegionMesh
    where
        R: Fn(i32, i32, i32) -> Option<BlockStateId>,
        L: Fn(i32, i32, i32) -> (u8, u8),
        T: Fn(crate::block::biome_tint::TintKind, [i32; 3]) -> u32,
    {
        self.mesh_blocks(world, origin, size, mesh_origin, None)
    }

    /// `MovingBlockFeatureRenderer`'s tessellation of one block: the model's quads lit by what
    /// the world holds around `block` (which `read` should answer as the block alone, with air
    /// all about it), with the variants the model picks by `seed_position` (the entity's start
    /// position) rather than by where it is drawn, and no fluid. The vertices are in the block's
    /// own unit cube.
    pub fn mesh_moving_block<R, L, T>(
        &self,
        world: &MeshWorld<'_, R, L, T>,
        block: [i32; 3],
        seed_position: [i32; 3],
    ) -> crate::scene::Mesh
    where
        R: Fn(i32, i32, i32) -> Option<BlockStateId>,
        L: Fn(i32, i32, i32) -> (u8, u8),
        T: Fn(crate::block::biome_tint::TintKind, [i32; 3]) -> u32,
    {
        self.mesh_blocks(world, block, [1, 1, 1], block, Some(seed_position))
            .mesh
    }

    /// [`Self::mesh_region`] and [`Self::mesh_moving_block`]: with a seed position, the
    /// blocks are those of a moving block (no fluids, variants picked from that position).
    pub(super) fn mesh_blocks<R, L, T>(
        &self,
        world: &MeshWorld<'_, R, L, T>,
        origin: [i32; 3],
        size: [i32; 3],
        mesh_origin: [i32; 3],
        seed_position: Option<[i32; 3]>,
    ) -> RegionMesh
    where
        R: Fn(i32, i32, i32) -> Option<BlockStateId>,
        L: Fn(i32, i32, i32) -> (u8, u8),
        T: Fn(crate::block::biome_tint::TintKind, [i32; 3]) -> u32,
    {
        let MeshWorld {
            read,
            light,
            biome_tint,
            tags,
        } = *world;
        let lighter = crate::block::light::Lighter {
            state: |p: [i32; 3]| read(p[0], p[1], p[2]),
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
                    let Some(state_id) = read(position[0], position[1], position[2]) else {
                        continue;
                    };
                    let drawn = match self.states.get(state_id.0 as usize) {
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
                            );
                            !photon_world::block::should_render_face(
                                state_id, neighbour, facing, tags,
                            )
                        })
                    };
                    // Closed in on every side, it would draw no face at all.
                    if drawn.closes_in && Direction::stream().all(&mut is_hidden) {
                        continue;
                    }
                    let info = photon_world::block::state_or_air(state_id);
                    let mut random = photon_core::random::RandomSource::create(
                        photon_world::block::seed(info, seed_position.unwrap_or(position)),
                    );
                    let mut picked = Vec::new();
                    pick_variants(&drawn.lists, &mut random, |model| picked.push(model));
                    if picked.is_empty() {
                        continue;
                    }
                    drawn.tints.colours(position, biome_tint, &mut tints);
                    let nudge = photon_world::block::offset(info.block(), position);
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

/// One lit quad of a block into the mesh. The colour is built in 8-bit
/// steps as the reference builds it: the occlusion grey, then the side
/// shade, then the tint.
pub(super) fn push_block_quad(
    mesh: &mut Mesh,
    quad: &crate::block::bake::BakedQuad,
    lit: &crate::block::light::QuadLight,
    smooth: bool,
    tint: u32,
    shift: [f32; 3],
) {
    let side = if quad.shade {
        crate::block::light::cardinal(quad.direction)
    } else {
        1.0
    };
    let tint = [16, 8, 0, 24].map(|bits| ((tint >> bits) & 0xFF) as i32);
    let white = tint == [255, 255, 255, 255];
    let base = mesh.vertices.len() as u32;
    for corner in 0..4 {
        let grey = if smooth {
            let grey = (lit.shade[corner] * 255.0).floor() as i32;
            ((grey as f32 * side) as i32).clamp(0, 255)
        } else {
            (side * 255.0).floor() as i32
        };
        let channel = |index: usize| {
            if white {
                grey
            } else {
                grey * tint[index] / 255
            }
        };
        mesh.vertices.push(crate::scene::BlockVertex::new(
            std::array::from_fn(|axis| quad.corners[corner][axis] + shift[axis]),
            quad.uvs[corner],
            [
                channel(0) as f32 / 255.0,
                channel(1) as f32 / 255.0,
                channel(2) as f32 / 255.0,
                1.0,
            ],
            crate::block::light::light_uv(lit.light[corner]),
        ));
    }
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Read a box of the world and build one mesh for everything in it, with
/// its own atlas.
pub fn build_world_mesh(
    source: &impl AssetSource,
    read: impl Fn(i32, i32, i32) -> Option<BlockStateId>,
    light: impl Fn(i32, i32, i32) -> (u8, u8),
    biome_tint: impl Fn(crate::block::biome_tint::TintKind, [i32; 3]) -> u32,
    origin: [i32; 3],
    size: [i32; 3],
    mip_level: i32,
) -> Result<WorldMesh, ResolveError> {
    let renderer = BlockRenderer::new(source, mip_level)?;
    // A probe's world has no server, so no tags.
    let tags = photon_world::block::tags::BlockTags::default();
    let world = MeshWorld {
        read: &read,
        light: &light,
        biome_tint: &biome_tint,
        tags: &tags,
    };
    let region = renderer.mesh_region(&world, origin, size, origin);
    let model_count = renderer.model_count();
    Ok(WorldMesh {
        mesh: region.mesh,
        atlas: renderer.atlas,
        solid_count: region.solid_count,
        model_count,
        skipped: renderer.skipped,
    })
}

/// Add one fluid quad to the translucent list, and its reverse when the
/// reference draws the face from behind too.
pub(super) fn add_face(
    mesh: &mut Mesh,
    corners: [[f32; 3]; 4],
    uv: [[f32; 2]; 4],
    shade: f32,
    tint: [f32; 4],
    light: [f32; 2],
    back_face: bool,
) {
    let base = mesh.vertices.len() as u32;
    for (position, uv) in corners.into_iter().zip(uv) {
        // The shade goes into the colour, in bytes, as the reference's does.
        mesh.vertices.push(crate::scene::BlockVertex::new(
            position,
            uv,
            [tint[0] * shade, tint[1] * shade, tint[2] * shade, tint[3]],
            light,
        ));
    }
    mesh.translucent_indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    if back_face {
        mesh.translucent_indices
            .extend([base, base + 3, base + 2, base, base + 2, base + 1]);
    }
}

/// The sprites named by no block model, so they are asked for by name: the fluid renderer's, the
/// sky's sun and moon phases, the conduit's, and those the item models ask for.
fn extra_sprites(layer_sprites: impl IntoIterator<Item = Identifier>) -> Vec<Identifier> {
    FLUID_SPRITES
        .iter()
        .map(|name| (*name).to_owned())
        .chain(std::iter::once(crate::sky::SUN_SPRITE.to_owned()))
        .chain(crate::sky::moon_sprites())
        .chain(
            crate::block_entity::conduit::SPRITES
                .iter()
                .map(|name| (*name).to_owned()),
        )
        .map(|name| Identifier::parse(&name).expect("fixed sprite names are valid"))
        .chain(layer_sprites)
        .collect()
}
