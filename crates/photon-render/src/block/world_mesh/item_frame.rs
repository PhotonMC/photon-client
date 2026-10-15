use super::*;
use photon_core::ids::ItemId;

impl BlockRenderer {
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
        item: ItemId,
        context: crate::model::DisplayContext,
        grass: &dyn Fn(f64, f64) -> u32,
        patch: &[u8],
    ) -> Vec<ItemLayer> {
        let Some(Some(def)) = usize::try_from(item.0)
            .ok()
            .and_then(|index| self.items.get(index))
        else {
            return Vec::new();
        };
        let values = photon_game::item::values::Values::new(item, patch);
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
                for part in crate::entity::special_models::layers(special) {
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
                |_| crate::block::rotation::FLAT_IDENTITY,
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

    /// The particle textures of an item's model layers on the ground (`pickParticleMaterial`'s
    /// choices), in layer order; a layer with none is left out.
    pub fn item_particle_sprites(&self, item: ItemId) -> Vec<String> {
        let Some(Some(def)) = usize::try_from(item.0)
            .ok()
            .and_then(|index| self.items.get(index))
        else {
            return Vec::new();
        };
        let mut layers = Vec::new();
        crate::item_model::layers_for(
            &def.model,
            crate::model::DisplayContext::Ground,
            &mut layers,
        );
        layers
            .iter()
            .filter_map(|layer| self.models.get(&layer.model.as_string()))
            .filter_map(|model| model.textures.material("particle"))
            .map(|material| material.sprite.as_string())
            .collect()
    }

    /// A client item's swap properties: whether a new stack lowers the
    /// hand, and how far. Items without a definition take the defaults.
    pub fn item_swap(&self, item: ItemId) -> (bool, f32) {
        usize::try_from(item.0)
            .ok()
            .and_then(|index| self.items.get(index))
            .and_then(Option::as_ref)
            .map_or((true, 1.0), |def| {
                (def.hand_animation_on_swap, def.swap_animation_scale)
            })
    }
}

/// The item frames' blockstate files, with the models they name read ahead.
pub(super) fn item_frame_files(
    source: &impl AssetSource,
    model_cache: &mut crate::resolve::ModelCache,
) -> Vec<Option<crate::block::blockstate::BlockStateFile>> {
    let files: Vec<_> = ITEM_FRAME_FILES
        .iter()
        .map(|name| {
            Identifier::parse(name)
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
    files
}

/// The models the item frames' variants name, added to `models`.
pub(super) fn add_item_frame_models(
    files: &[Option<crate::block::blockstate::BlockStateFile>],
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
    files: &[Option<crate::block::blockstate::BlockStateFile>],
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
    let items: Vec<Option<crate::item_model::ClientItem>> = photon_data::item::registry::ITEMS
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
                for part in crate::entity::special_models::layers(special) {
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
            let image = crate::texture::png::decode_rgba(&source.texture(sprite)?).ok()?;
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

/// The blockstate files that name no block: the item frames' (`BlockStateDefinitions`), each with
/// a `map` property.
pub(super) const ITEM_FRAME_FILES: [&str; 2] =
    ["minecraft:item_frame", "minecraft:glow_item_frame"];

/// One layer of an item as drawn in one display context.
pub struct ItemLayer {
    /// Quads in the model's unit cube, tinted, with their facings.
    pub mesh: Mesh,
    /// Whether the model lights like a block (`gui_light: side`) rather
    /// than flat from the front.
    pub side_lit: bool,
    /// The context's display transform; `None` is no transform.
    pub transform: Option<crate::model::ItemTransform>,
}
