use super::*;

/// `WeightedList` draws: one variant from each list with the block's random
/// source, a lone variant written outside a list taking no draw.
pub(super) fn pick_variants(
    lists: &[Vec<BakedVariant>],
    random: &mut voxel_core::random::RandomSource,
    mut picked: impl FnMut(Option<u32>),
) {
    for list in lists {
        match list.as_slice() {
            [] => {}
            [only] if !only.listed => picked(only.model),
            _ => {
                let total: i32 = list.iter().map(|variant| variant.weight).sum();
                if let Ok(mut selection) = random.next_int_bound(total)
                    && let Some(variant) = list.iter().find(|variant| {
                        selection -= variant.weight;
                        selection < 0
                    })
                {
                    picked(variant.model);
                }
            }
        }
    }
}

/// Bake every variant the states name, each distinct model and turn once
/// and in parallel, and turn each state's lists into lists of baked models.
pub(super) fn bake_states(
    models: &BTreeMap<String, ResolvedModel>,
    atlas: &BuiltAtlas,
    pending: Vec<PendingState>,
    air: &[bool],
) -> (Vec<BakedModel>, Vec<StateRender>) {
    use rayon::prelude::*;
    let key = |variant: &crate::blockstate::Variant| BakeKey {
        model: variant.model.as_string(),
        rotation: (
            variant.x.degrees(),
            variant.y.degrees(),
            variant.z.degrees(),
        ),
        uv_lock: variant.uvlock,
    };
    // Each distinct model and turn, in the order the states first name it.
    let mut keys: Vec<BakeKey> = Vec::new();
    let mut slots: HashMap<BakeKey, usize> = HashMap::new();
    for (lists, _) in pending.iter().flatten() {
        for variant in lists.iter().flatten() {
            slots.entry(key(variant)).or_insert_with_key(|key| {
                keys.push(key.clone());
                keys.len() - 1
            });
        }
    }
    let results: Vec<Option<BakedModel>> = keys
        .par_iter()
        .map(|key| crate::bake::bake(models.get(&key.model)?, atlas, key))
        .collect();
    let mut baked = Vec::new();
    let indices: Vec<Option<u32>> = results
        .into_iter()
        .map(|model| {
            let model = model?;
            baked.push(model);
            u32::try_from(baked.len() - 1).ok()
        })
        .collect();
    let states = pending
        .into_iter()
        .zip(air)
        .map(|(drawn, is_air)| {
            if *is_air {
                return StateRender::Air;
            }
            let Some((lists, tints)) = drawn else {
                return StateRender::Missing;
            };
            let lists: Vec<Vec<BakedVariant>> = lists
                .iter()
                .map(|list| {
                    list.iter()
                        .map(|variant| BakedVariant {
                            model: indices[slots[&key(variant)]],
                            weight: variant.weight,
                            listed: variant.listed,
                        })
                        .collect()
                })
                .collect();
            let closes_in = lists.iter().flatten().all(|variant| {
                variant
                    .model
                    .is_none_or(|model| baked[model as usize].closes_in())
            });
            StateRender::Drawn(DrawnState {
                lists,
                closes_in,
                tints,
            })
        })
        .collect();
    (baked, states)
}

pub(super) fn is_air_block(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

/// Every weighted list a blockstate file holds: its variants, then its
/// multipart pieces.
pub(super) fn variant_lists(
    file: &crate::blockstate::BlockStateFile,
) -> impl Iterator<Item = &Vec<crate::blockstate::Variant>> {
    file.variants
        .iter()
        .flat_map(|variants| variants.values())
        .chain(
            file.multipart
                .iter()
                .flat_map(|parts| parts.iter().map(|part| &part.apply)),
        )
}

/// Every state's variant lists and colours, block by block in parallel, and
/// which states are air.
pub(super) fn state_lists(
    blockstates: &BTreeMap<String, crate::blockstate::BlockStateFile>,
) -> (Vec<PendingState>, Vec<bool>) {
    use rayon::prelude::*;
    let per_block: Vec<Vec<(PendingState, bool)>> = registry::BLOCKS
        .par_iter()
        .map(|block| {
            let is_air = is_air_block(block.name);
            (0..block.state_count())
                .map(|offset| {
                    let drawn = blockstates.get(block.name).filter(|_| !is_air).map(|file| {
                        let properties: BTreeMap<String, String> = block
                            .properties_at(offset)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|(key, value)| (key.to_owned(), value.to_owned()))
                            .collect();
                        (
                            file.matched_lists(&properties),
                            StateTints::of(block.name, &properties),
                        )
                    });
                    (drawn, is_air)
                })
                .collect()
        })
        .collect();
    per_block.into_iter().flatten().unzip()
}

/// The item frames' blockstate files, with the models they name read ahead.
pub(super) fn item_frame_files(
    source: &impl AssetSource,
    model_cache: &mut crate::resolve::ModelCache,
) -> Vec<Option<crate::blockstate::BlockStateFile>> {
    let files: Vec<_> = ITEM_FRAME_FILES
        .iter()
        .map(|name| {
            Identifier::parse(name)
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
    files
}

/// The models the item frames' variants name, added to `models`.
pub(super) fn add_item_frame_models(
    files: &[Option<crate::blockstate::BlockStateFile>],
    (source, model_cache): (&impl AssetSource, &mut crate::resolve::ModelCache),
    models: &mut BTreeMap<String, ResolvedModel>,
) {
    for variant in files.iter().flatten().flat_map(variant_lists).flatten() {
        if let std::collections::btree_map::Entry::Vacant(slot) =
            models.entry(variant.model.as_string())
            && let Ok(model) = model_cache.resolve(source, &variant.model)
        {
            slot.insert(model);
        }
    }
}

/// The item frames' fake states after the registry's: each file's `map=false` then `map=true`.
pub(super) fn add_item_frame_states(
    files: &[Option<crate::blockstate::BlockStateFile>],
    pending: &mut Vec<PendingState>,
    air: &mut Vec<bool>,
) {
    for file in files {
        for map in [false, true] {
            let properties = BTreeMap::from([("map".to_owned(), map.to_string())]);
            pending.push(file.as_ref().map(|file| {
                (
                    file.matched_lists(&properties),
                    StateTints::Fixed(Vec::new()),
                )
            }));
            air.push(false);
        }
    }
}

/// Every item's client item definition, the models its layers name in any
/// display context (added to `models`), the alpha of every flat item's
/// layers and the sprites no element names.
pub(super) fn item_models(
    source: &impl AssetSource,
    cache: &mut crate::resolve::ModelCache,
    models: &mut BTreeMap<String, ResolvedModel>,
) -> (
    Vec<Option<crate::item_model::ClientItem>>,
    BTreeMap<String, crate::item_model::LayerAlpha>,
    Vec<Identifier>,
) {
    use rayon::prelude::*;
    let items: Vec<Option<crate::item_model::ClientItem>> = voxel_data::item::registry::ITEMS
        .par_iter()
        .map(|item| {
            Identifier::parse(item.name)
                .ok()
                .and_then(|name| source.item(&name))
                .and_then(|text| crate::item_model::parse_client_item(&text))
        })
        .collect();
    let layers_of = |def: &crate::item_model::ClientItem| {
        let mut layers = Vec::new();
        for context in crate::model::DisplayContext::ALL {
            crate::item_model::layers_for(&def.model, context, &mut layers);
        }
        layers
    };
    cache.prefetch(
        source,
        items
            .iter()
            .flatten()
            .flat_map(layers_of)
            .map(|layer| layer.model),
    );
    let mut layer_sprites = Vec::new();
    // The textures a flat item's geometry is made from, decoded below.
    let mut flat_sprites: Vec<Identifier> = Vec::new();
    let mut asked = std::collections::HashSet::new();
    for def in items.iter().flatten() {
        for layer in layers_of(def) {
            if let Some(special) = &layer.special {
                for part in crate::special_models::layers(special) {
                    if let Ok(sprite) = Identifier::parse(&format!("minecraft:{}", part.sprite)) {
                        layer_sprites.push(sprite);
                    }
                }
            }
            let name = layer.model.as_string();
            if models.contains_key(&name) {
                continue;
            }
            let Ok(model) = cache.resolve(source, &layer.model) else {
                continue;
            };
            if model.generated {
                for slot in ["layer0", "layer1", "layer2", "layer3", "layer4"] {
                    let Some(material) = model.textures.material(slot) else {
                        break;
                    };
                    if asked.insert(material.sprite.as_string()) {
                        // The geometry is made from the texture later, so no
                        // element names it for the atlas.
                        layer_sprites.push(material.sprite.clone());
                        flat_sprites.push(material.sprite.clone());
                    }
                }
            }
            models.insert(name, model);
        }
    }
    let layer_alpha = flat_sprites
        .par_iter()
        .filter_map(|sprite| {
            let image = crate::png::decode_rgba(&source.texture(sprite)?).ok()?;
            // An animated strip's first frame decides the sides.
            let frame = image.width.min(image.height);
            let alpha = image.rgba[..(frame * frame * 4) as usize]
                .chunks(4)
                .map(|pixel| pixel[3])
                .collect();
            Some((sprite.as_string(), (alpha, frame as i32, frame as i32)))
        })
        .collect();
    (items, layer_alpha, layer_sprites)
}

/// One lit quad of a block into the mesh. The colour is built in 8-bit
/// steps as the reference builds it: the occlusion grey, then the side
/// shade, then the tint.
pub(super) fn push_block_quad(
    mesh: &mut Mesh,
    quad: &crate::bake::BakedQuad,
    lit: &crate::block_light::QuadLight,
    smooth: bool,
    tint: u32,
    shift: [f32; 3],
) {
    let side = if quad.shade {
        crate::block_light::cardinal(quad.direction)
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
            crate::block_light::light_uv(lit.light[corner]),
        ));
    }
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Read a box of the world and build one mesh for everything in it, with
/// its own atlas.
pub fn build_world_mesh(
    source: &impl AssetSource,
    read: impl Fn(i32, i32, i32) -> Option<i32>,
    light: impl Fn(i32, i32, i32) -> (u8, u8),
    biome_tint: impl Fn(crate::biome_tint::TintKind, [i32; 3]) -> u32,
    origin: [i32; 3],
    size: [i32; 3],
    mip_level: i32,
) -> Result<WorldMesh, ResolveError> {
    let renderer = BlockRenderer::new(source, mip_level)?;
    // A probe's world has no server, so no tags.
    let tags = voxel_world::block::tags::BlockTags::default();
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
