use super::*;

impl BlockRenderer {
    /// Read every block's blockstate file and every model any of its
    /// variants names, and pack all their textures into one atlas.
    pub fn new(source: &impl AssetSource, mip_level: i32) -> Result<Self, ResolveError> {
        use rayon::prelude::*;
        let mut model_cache = crate::resolve::ModelCache::default();
        // Every blockstate file, read and parsed in parallel.
        let files: Vec<Option<crate::blockstate::BlockStateFile>> = registry::BLOCKS
            .par_iter()
            .map(|block| {
                if is_air_block(block.name) {
                    return None;
                }
                Identifier::parse(block.name)
                    .ok()
                    .and_then(|name| source.block_state(&name))
                    .and_then(|text| crate::blockstate::parse_block_state(&text))
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
        // The fluid renderer's sprites and the sky's sun and moon phases are
        // named by no block model, so they are asked for by name.
        let extra: Vec<Identifier> = FLUID_SPRITES
            .iter()
            .map(|name| (*name).to_owned())
            .chain(std::iter::once(crate::sky::SUN_SPRITE.to_owned()))
            .chain(crate::sky::moon_sprites())
            .map(|name| Identifier::parse(&name).expect("fixed sprite names are valid"))
            .chain(layer_sprites)
            .collect();
        let atlas = crate::resolve::build_atlas_with(source, &ordered, &extra, 8192, mip_level)?;
        let (baked, states) = bake_states(&models, &atlas, pending, &air);
        Ok(Self {
            models,
            baked,
            items,
            layer_alpha,
            states,
            frame_base,
            atlas,
            skipped,
        })
    }

    /// The fake state an item frame's model is drawn from (`BlockStateDefinitions.getItemFrameFakeState`),
    /// for [`Self::entity_block`].
    pub fn item_frame_state(&self, glowing: bool, map: bool) -> u32 {
        self.frame_base + u32::from(glowing) * 2 + u32::from(map)
    }

    /// The quads an item's default stack draws in a display context, layer
    /// by layer, each with its model's GUI light and display transforms.
    /// Corners are in the model's unit cube; tints are already applied to
    /// the vertices' colour. `grass` reads the grass colour map.
    pub fn item_layers(
        &self,
        item: i32,
        context: crate::model::DisplayContext,
        grass: &dyn Fn(f64, f64) -> u32,
        patch: &[u8],
    ) -> Vec<ItemLayer> {
        let Some(Some(def)) = usize::try_from(item)
            .ok()
            .and_then(|index| self.items.get(index))
        else {
            return Vec::new();
        };
        let values = voxel_game::item_values::Values::new(item, patch);
        let mut layers = Vec::new();
        crate::item_model::layers_for(&def.model, context, &mut layers);
        let mut out = Vec::new();
        for layer in layers {
            let Some(model) = self.models.get(&layer.model.as_string()) else {
                continue;
            };
            let tints: Vec<u32> = layer
                .tints
                .iter()
                .map(|tint| crate::item_model::tint_colour(tint, grass, &values))
                .collect();
            if let Some(special) = &layer.special {
                for part in crate::special_models::layers(special) {
                    let Some(rect) = self
                        .atlas
                        .sprites
                        .get(&format!("minecraft:{}", part.sprite))
                    else {
                        continue;
                    };
                    let mut mesh = model_mesh(&part, rect);
                    apply_local(&mut mesh, &layer.local);
                    out.push(ItemLayer {
                        mesh,
                        side_lit: model.gui_light == crate::model::GuiLight::Side,
                        transform: model.display.get(context),
                    });
                }
                continue;
            }
            let geometry;
            let model = if model.generated {
                let mut flat = model.clone();
                flat.elements = crate::item_model::generated_elements(&model.textures, &|sprite| {
                    self.layer_alpha.get(&sprite.as_string()).cloned()
                });
                geometry = flat;
                &geometry
            } else {
                model
            };
            let Ok(mut mesh) = crate::resolve::build_mesh(
                model,
                &self.atlas,
                None,
                &tints,
                |_| crate::rotation::FLAT_IDENTITY,
                |_| false,
            ) else {
                continue;
            };
            apply_local(&mut mesh, &layer.local);
            out.push(ItemLayer {
                mesh,
                side_lit: model.gui_light == crate::model::GuiLight::Side,
                transform: model.display.get(context),
            });
        }
        out
    }

    /// A client item's swap properties: whether a new stack lowers the
    /// hand, and how far. Items without a definition take the defaults.
    pub fn item_swap(&self, item: i32) -> (bool, f32) {
        usize::try_from(item)
            .ok()
            .and_then(|index| self.items.get(index))
            .and_then(Option::as_ref)
            .map_or((true, 1.0), |def| {
                (def.hand_animation_on_swap, def.swap_animation_scale)
            })
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Every quad a block's model draws at `position`, culled against
    /// nothing, as the breaking overlay draws them: corners in the model's
    /// own unit cube and each quad's facing, with the block's offset. The
    /// variants are the ones the world mesh picks there.
    pub fn block_quads(&self, state_id: u32, position: [i32; 3]) -> (Vec<FacedQuad>, [f32; 3]) {
        let Some(StateRender::Drawn(drawn)) = self.states.get(state_id as usize) else {
            return (Vec::new(), [0.0; 3]);
        };
        let info = voxel_world::block::state_or_air(i64::from(state_id));
        let mut random =
            voxel_core::random::RandomSource::create(voxel_world::block::seed(info, position));
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
        let offset = voxel_world::block::offset(info.block(), position).map(|value| value as f32);
        (quads, offset)
    }

    /// `BlockModelResolver.update` for a block state drawn on an entity: every
    /// quad of the variants the model's own seed picks (`MODEL_SEED`, 42),
    /// culled against nothing, each tinted by the state's colour outside a
    /// world, and the light the state gives off (`blockLightCoords`). Nothing
    /// for air and for states with no model to draw.
    pub fn entity_block(&self, state_id: u32) -> Option<(ItemLayer, i32)> {
        let Some(StateRender::Drawn(drawn)) = self.states.get(state_id as usize) else {
            return None;
        };
        let tints: Vec<u32> = match &drawn.tints {
            StateTints::Fixed(colours) => colours.clone(),
            StateTints::Biome { constant, .. } => constant.to_vec(),
        };
        let mut random = voxel_core::random::RandomSource::create(42);
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
        let info = voxel_world::block::state_or_air(if state_id >= self.frame_base {
            0
        } else {
            i64::from(state_id)
        });
        Some((
            ItemLayer {
                mesh,
                side_lit: true,
                transform: None,
            },
            crate::entity_block::block_light_coords(info.light_emission, info.emissive_rendering),
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
        R: Fn(i32, i32, i32) -> Option<i32>,
        L: Fn(i32, i32, i32) -> (u8, u8),
        T: Fn(crate::biome_tint::TintKind, [i32; 3]) -> u32,
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
        R: Fn(i32, i32, i32) -> Option<i32>,
        L: Fn(i32, i32, i32) -> (u8, u8),
        T: Fn(crate::biome_tint::TintKind, [i32; 3]) -> u32,
    {
        self.mesh_blocks(world, block, [1, 1, 1], block, Some(seed_position))
            .mesh
    }
}
