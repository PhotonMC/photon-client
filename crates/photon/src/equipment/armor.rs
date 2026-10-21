//! What humanoids wear (`HumanoidArmorLayer`, `EquipmentLayerRenderer`): each
//! worn piece's model, posed as its wearer is, in the textures its equipment
//! asset names, coloured by its dye.

use std::collections::HashMap;

use photon_game::item::values::{Values, equippable_of};
use photon_render::block_entity::skull::SkullKind;
use photon_render::entity::equipment::{EquipmentInfo, parse_equipment_info};
use photon_render::entity::mob_model::MobModel;
use photon_render::entity::textures::{EntityTextures, TextureRect};
use photon_render::entity::trim::{PalettedPermutations, TrimData, permute, sprite_path};
use photon_render::render_types::{
    ARMOR_GLINT_SCALE, ARMOR_GLINT_TEXTURE, glint_texture_matrix, glint_uv,
};
use photon_render::scene::Vertex;
use photon_world::ClientItemStackState;

/// The equipment slots armor is worn in, in the order the layer draws them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Chest,
    Legs,
    Feet,
    Head,
}

impl Slot {
    const ALL: [Slot; 4] = [Slot::Chest, Slot::Legs, Slot::Feet, Slot::Head];

    /// The slot's name in an `equippable` component.
    fn name(self) -> &'static str {
        match self {
            Slot::Chest => "chest",
            Slot::Legs => "legs",
            Slot::Feet => "feet",
            Slot::Head => "head",
        }
    }

    /// The model layer of the piece worn in the slot (`ModelLayers`'
    /// `registerArmorSet`).
    fn model_suffix(self) -> &'static str {
        match self {
            Slot::Chest => "chestplate",
            Slot::Legs => "leggings",
            Slot::Feet => "boots",
            Slot::Head => "helmet",
        }
    }
}

/// What a wearer has on each slot.
#[derive(Clone, Copy, Default)]
pub struct Worn<'a> {
    pub head: Option<&'a ClientItemStackState>,
    pub chest: Option<&'a ClientItemStackState>,
    pub legs: Option<&'a ClientItemStackState>,
    pub feet: Option<&'a ClientItemStackState>,
}

impl<'a> Worn<'a> {
    /// What sits on the head besides armor (`LivingEntityRenderer`'s head
    /// item): a stack the armor layer does not draw.
    pub fn head_item(&self, armor: &Armor) -> Option<&'a ClientItemStackState> {
        self.head
            .filter(|stack| armor.asset_of(stack, Slot::Head).is_none())
    }

    fn get(&self, slot: Slot) -> Option<&'a ClientItemStackState> {
        match slot {
            Slot::Head => self.head,
            Slot::Chest => self.chest,
            Slot::Legs => self.legs,
            Slot::Feet => self.feet,
        }
    }
}

/// The equipment assets of the game's resources, and the armor models built
/// from the layer table.
pub struct Armor {
    infos: HashMap<String, EquipmentInfo>,
    models: HashMap<String, MobModel>,
    /// The trim patterns and materials of the data pack, and the order the server named them in
    /// (a `trim` component holds registry numbers).
    trims: TrimData,
    trim_names: Option<(Vec<String>, std::sync::Arc<Vec<String>>)>,
    /// The glints of the enchanted pieces drawn since they were last taken
    /// (`RenderTypes.armorEntityGlint`), for the renderer's glint pass.
    pub glint: Vec<Vertex>,
    /// What `Util.getMillis` is counted from for the glint's movement.
    clock: std::time::Instant,
    /// `Options.glintSpeed`, what the glint's movement is scaled by.
    pub glint_speed: f64,
}

/// Turns a posed armor model into triangles, in a texture, multiplied by a
/// colour (`0xRRGGBB`).
pub type Place<'a> = &'a dyn Fn(&MobModel, TextureRect, Option<u32>) -> Vec<Vertex>;

/// The layer types armor is drawn in, by `HumanoidArmorLayer`.
const HUMANOID: &str = "humanoid";
const LEGGINGS: &str = "humanoid_leggings";
const BABY: &str = "humanoid_baby";
const WINGS: &str = "wings";
/// The layer types mobs wear one piece in (`SimpleEquipmentLayer`, `LlamaDecorLayer`): the
/// saddles, the happy ghast's harness, the nautilus's and the horse's armor, the llama's carpet.
const SIMPLE: [&str; 11] = [
    "strider_saddle",
    "happy_ghast_body",
    "nautilus_body",
    "nautilus_saddle",
    "horse_body",
    "llama_body",
    "horse_saddle",
    "donkey_saddle",
    "mule_saddle",
    "skeleton_horse_saddle",
    "zombie_horse_saddle",
];

impl Armor {
    /// Every `equipment/<name>.json` under the asset root.
    pub fn load(root: &str) -> Self {
        let folder = std::path::Path::new(root).join("minecraft/equipment");
        let infos = std::fs::read_dir(folder)
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_stem()?.to_str()?.to_owned();
                let info = parse_equipment_info(&std::fs::read_to_string(&path).ok()?)?;
                Some((name, info))
            })
            .collect();
        Self {
            infos,
            models: HashMap::new(),
            trims: TrimData::load(&std::path::Path::new(root).with_file_name("data")),
            trim_names: None,
            glint: Vec::new(),
            clock: std::time::Instant::now(),
            glint_speed: 0.5,
        }
    }

    /// The server's trim patterns and materials, in the order a `trim` component numbers them.
    pub fn set_trim_registries(&mut self, patterns: Vec<String>, materials: Vec<String>) {
        self.trim_names = Some((patterns, std::sync::Arc::new(materials)));
    }

    /// The server's trim materials, in the order a `trim` component numbers them.
    pub fn trim_materials(&self) -> std::sync::Arc<Vec<String>> {
        self.trim_names
            .as_ref()
            .map_or_else(Default::default, |(_, materials)| materials.clone())
    }

    /// The pictures of the `armor_trims` atlas: each pattern's grey picture recoloured by each
    /// material's palette (`PalettedPermutations`), named by their path under `textures/`.
    pub fn trim_images(root: &str) -> Vec<(String, photon_render::DecodedImage)> {
        let base = std::path::Path::new(root).join("minecraft");
        let load = |path: &str| {
            photon_render::texture::png::decode_rgba(
                &std::fs::read(base.join("textures").join(format!("{path}.png"))).ok()?,
            )
            .ok()
        };
        let Some(source) = photon_core::json::read_object(&base.join("atlases/armor_trims.json"))
            .as_ref()
            .and_then(PalettedPermutations::parse)
        else {
            return Vec::new();
        };
        let Some(keys) = load(&source.palette_key) else {
            return Vec::new();
        };
        let palettes: Vec<(&String, photon_render::DecodedImage)> = source
            .permutations
            .iter()
            .filter_map(|(suffix, path)| Some((suffix, load(path)?)))
            .collect();
        let mut images = Vec::new();
        for texture in &source.textures {
            let Some(image) = load(texture) else {
                continue;
            };
            for (suffix, palette) in &palettes {
                if let Some(mapped) = permute(&image, &keys, palette) {
                    images.push((format!("{texture}_{suffix}"), mapped));
                }
            }
        }
        images
    }

    /// `EquipmentLayerRenderer`'s trim pass: the picture of the stack's trim for the layer type
    /// and equipment asset, if it has one.
    fn trim_texture(
        &self,
        stack: &ClientItemStackState,
        (layer_type, asset): (&str, &str),
    ) -> Option<String> {
        let (material, pattern) =
            Values::new(stack.item_registry_id(), stack.patch_bytes()).trim()?;
        let (patterns, materials) = self.trim_names.as_ref()?;
        let pattern = self
            .trims
            .pattern(patterns.get(usize::try_from(pattern).ok()?)?)?;
        let material = self
            .trims
            .material(materials.get(usize::try_from(material).ok()?)?)?;
        Some(sprite_path(
            &format!("trims/entity/{layer_type}"),
            pattern,
            material,
            &format!("minecraft:{asset}"),
        ))
    }

    /// The paths (under `textures/`) of every texture a humanoid's armor wears.
    pub fn textures(&self) -> Vec<String> {
        // The skulls a head wears are drawn in their own mobs' textures.
        let skulls = SkullKind::ALL.map(|kind| kind.texture().to_owned());
        let mut paths: Vec<String> = skulls
            .into_iter()
            .chain(self.infos.values().flat_map(|info| {
                [HUMANOID, LEGGINGS, BABY, WINGS]
                    .into_iter()
                    .chain(SIMPLE)
                    .flat_map(|layer_type| {
                        info.layers(layer_type)
                            .iter()
                            .map(move |layer| layer.texture_path(layer_type))
                    })
            }))
            .collect();
        paths.push(ARMOR_GLINT_TEXTURE.to_owned());
        paths.sort();
        paths.dedup();
        paths
    }

    /// `TextureTransform.ARMOR_ENTITY_GLINT_TEXTURING` now.
    fn glint_matrix(&self) -> [f32; 16] {
        self.glint_matrix_scaled(ARMOR_GLINT_SCALE)
    }

    /// The glint's texture matrix now, at the scale of the render type that draws it.
    pub fn glint_matrix_scaled(&self, scale: f32) -> [f32; 16] {
        let millis = self.clock.elapsed().as_millis() as i64;
        glint_texture_matrix(millis, self.glint_speed, scale)
    }

    /// The triangles of the armor on a wearer. `prefix` names the wearer's
    /// armor models (`minecraft:zombie`), `pose` poses each of them as the
    /// wearer is posed, and `place` turns a posed model into triangles in a
    /// texture, multiplied by a colour (`0xRRGGBB`).
    pub fn vertices(
        &mut self,
        worn: Worn,
        (prefix, baby): (&str, bool),
        textures: &EntityTextures,
        pose: &dyn Fn(&mut MobModel),
        place: Place,
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let glint_matrix = self.glint_matrix();
        for slot in Slot::ALL {
            let Some(stack) = worn.get(slot) else {
                continue;
            };
            let Some(asset) = self.asset_of(stack, slot) else {
                continue;
            };
            let layer_type = if baby {
                BABY
            } else if slot == Slot::Legs {
                LEGGINGS
            } else {
                HUMANOID
            };
            let dye = Values::new(stack.item_registry_id(), stack.patch_bytes())
                .dyed_color()
                .map_or(0, |rgb| rgb as u32 | 0xFF00_0000);
            let layers: Vec<(String, u32)> = self.infos[&asset]
                .layers(layer_type)
                .iter()
                .map(|layer| (layer.texture_path(layer_type), layer.colour(dye)))
                .collect();
            let trim = self.trim_texture(stack, (layer_type, &asset));
            let name = format!("{prefix}#{}", slot.model_suffix());
            if !self.models.contains_key(&name) {
                let Some(layer) = photon_data::entity_model::layer(&name) else {
                    continue;
                };
                self.models.insert(name.clone(), MobModel::bake(layer));
            }
            let model = self
                .models
                .get_mut(&name)
                .expect("the model was just built");
            pose(model);
            // The glint is on the first layer drawn (`renderFoil`).
            let mut foil = Values::new(stack.item_registry_id(), stack.patch_bytes()).has_foil();
            let mut glints = Vec::new();
            for (texture, colour) in layers {
                // A layer coloured to nothing is not drawn; white is no colour.
                let (true, Some(rect)) = (colour != 0, textures.rect(&texture)) else {
                    continue;
                };
                let rgb = colour & 0x00FF_FFFF;
                let triangles = place(model, rect, (colour != 0xFFFF_FFFF).then_some(rgb));
                if std::mem::take(&mut foil) {
                    glints.push((triangles.clone(), rect));
                }
                vertices.extend(triangles);
            }
            for (triangles, rect) in glints {
                push_glint(
                    &mut self.glint,
                    (&triangles, rect),
                    (&glint_matrix, textures),
                );
            }
            // The trim, over the layers, undyed (a baby's armor has none).
            if layer_type != BABY
                && let Some(rect) = trim.as_deref().and_then(|path| textures.rect(path))
            {
                vertices.extend(place(model, rect, None));
            }
        }
        vertices
    }

    /// `CustomHeadLayer`'s skull: the skull's model, posed by the wearer's walk (a piglin's ears and a
    /// dragon's jaw follow it), in its texture, from the pose the layer leaves.
    pub fn skull_vertices(
        &mut self,
        skull: Option<(SkullKind, photon_render::joml::Pose)>,
        animation_pos: f32,
        textures: &EntityTextures,
        (light, red_overlay): (i32, bool),
    ) -> Vec<Vertex> {
        let Some((kind, pose)) = skull else {
            return Vec::new();
        };
        let Some(rect) = textures.rect(kind.texture()) else {
            return Vec::new();
        };
        if !self.models.contains_key(kind.layer()) {
            let Some(layer) = photon_data::entity_model::layer(kind.layer()) else {
                return Vec::new();
            };
            self.models
                .insert(kind.layer().to_owned(), MobModel::bake(layer));
        }
        let model = self
            .models
            .get_mut(kind.layer())
            .expect("the model was just built");
        model.setup_skull(kind, animation_pos);
        photon_render::entity::mob_model::mob_vertices_posed(
            model,
            &pose,
            rect,
            (light, red_overlay),
        )
    }

    /// `WingsLayer`: the wings of what is worn on the chest (an elytra), spread by the wearer's
    /// animation state, in their wings texture.
    pub fn wings(
        &mut self,
        chest: Option<&ClientItemStackState>,
        (baby, crouching, spread): (bool, bool, [f32; 3]),
        textures: &EntityTextures,
        place: Place,
    ) -> Vec<Vertex> {
        let glint_matrix = self.glint_matrix();
        let Some(asset) = chest.and_then(|stack| self.asset_for(stack, "chest")) else {
            return Vec::new();
        };
        let layers: Vec<(String, u32)> = self.infos[&asset]
            .layers(WINGS)
            .iter()
            .map(|layer| (layer.texture_path(WINGS), layer.colour(0)))
            .collect();
        let name = if baby {
            "minecraft:elytra_baby#main"
        } else {
            "minecraft:elytra#main"
        };
        if layers.is_empty() {
            return Vec::new();
        }
        if !self.models.contains_key(name) {
            let Some(layer) = photon_data::entity_model::layer(name) else {
                return Vec::new();
            };
            self.models.insert(name.to_owned(), MobModel::bake(layer));
        }
        let model = self.models.get_mut(name).expect("the model was just built");
        model.setup_elytra(spread, crouching);
        // `WingsLayer.submit`: the wings sit an eighth of a block behind the back.
        model.shift_root([0.0, 0.0, 2.0]);
        let mut vertices = Vec::new();
        // The glint is on the first layer drawn (`renderFoil`).
        let mut foil = chest.is_some_and(|stack| {
            Values::new(stack.item_registry_id(), stack.patch_bytes()).has_foil()
        });
        let mut glints = Vec::new();
        for (texture, colour) in layers {
            let (true, Some(rect)) = (colour != 0, textures.rect(&texture)) else {
                continue;
            };
            let rgb = colour & 0x00FF_FFFF;
            let triangles = place(model, rect, (colour != 0xFFFF_FFFF).then_some(rgb));
            if std::mem::take(&mut foil) {
                glints.push((triangles.clone(), rect));
            }
            vertices.extend(triangles);
        }
        for (triangles, rect) in glints {
            push_glint(
                &mut self.glint,
                (&triangles, rect),
                (&glint_matrix, textures),
            );
        }
        vertices
    }

    /// `SimpleEquipmentLayer`: what a mob wears in the equipment slot `slot`
    /// (a saddle) drawn in the layer type `layer_type` of its asset on the
    /// model `model`, posed as the wearer is posed. A dyed stack colours its dyeable layers.
    pub fn simple_layer(
        &mut self,
        stack: &ClientItemStackState,
        (slot, layer_type, model): (&str, &str, &str),
        textures: &EntityTextures,
        pose: &dyn Fn(&mut MobModel),
        place: Place,
    ) -> Vec<Vertex> {
        let Some(asset) = self.asset_for(stack, slot) else {
            return Vec::new();
        };
        let dye = Values::new(stack.item_registry_id(), stack.patch_bytes())
            .dyed_color()
            .map_or(0, |rgb| rgb as u32 | 0xFF00_0000);
        self.asset_layer(&asset, (layer_type, model, dye), textures, pose, place)
    }

    /// The layers of the equipment asset `asset` (`trader_llama`) with no stack behind it, as
    /// `simple_layer` draws them.
    pub fn named_layer(
        &mut self,
        asset: &str,
        (layer_type, model): (&str, &str),
        textures: &EntityTextures,
        pose: &dyn Fn(&mut MobModel),
        place: Place,
    ) -> Vec<Vertex> {
        self.asset_layer(asset, (layer_type, model, 0), textures, pose, place)
    }

    fn asset_layer(
        &mut self,
        asset: &str,
        (layer_type, model, dye): (&str, &str, u32),
        textures: &EntityTextures,
        pose: &dyn Fn(&mut MobModel),
        place: Place,
    ) -> Vec<Vertex> {
        let Some(info) = self.infos.get(asset) else {
            return Vec::new();
        };
        let layers: Vec<(String, u32)> = info
            .layers(layer_type)
            .iter()
            .map(|layer| (layer.texture_path(layer_type), layer.colour(dye)))
            .collect();
        if !self.models.contains_key(model) {
            let Some(layer) = photon_data::entity_model::layer(model) else {
                return Vec::new();
            };
            self.models.insert(model.to_owned(), MobModel::bake(layer));
        }
        let model = self
            .models
            .get_mut(model)
            .expect("the model was just built");
        pose(model);
        layers
            .iter()
            // A layer coloured to nothing is not drawn; white is no colour.
            .filter(|(_, colour)| *colour != 0)
            .filter_map(|(texture, colour)| Some((textures.rect(texture)?, *colour)))
            .flat_map(|(rect, colour)| {
                place(
                    model,
                    rect,
                    (colour != 0xFFFF_FFFF).then_some(colour & 0x00FF_FFFF),
                )
            })
            .collect()
    }

    /// The equipment asset a stack is worn as in a slot: its item's
    /// `equippable` component names it and the slot it goes in
    /// (`HumanoidArmorLayer.shouldRender`).
    fn asset_of(&self, stack: &ClientItemStackState, slot: Slot) -> Option<String> {
        self.asset_for(stack, slot.name())
    }

    /// The equipment asset a stack is worn as in the slot of this name (its `equippable` component,
    /// the stack's own over the item's).
    fn asset_for(&self, stack: &ClientItemStackState, slot: &str) -> Option<String> {
        let equip = equippable_of(stack.item_registry_id(), stack.patch_bytes())
            .filter(|equip| equip.slot == slot)?;
        let asset = equip.asset?;
        let asset = asset.strip_prefix("minecraft:")?;
        self.infos.contains_key(asset).then(|| asset.to_owned())
    }
}

/// `RenderTypes.armorEntityGlint` over a layer's triangles (drawn in the texture at `rect`): the
/// same triangles with their texture coordinate (of the layer's texture, in 0 to 1) moved by the
/// glint's matrix into the glint texture, which repeats.
fn push_glint(
    out: &mut Vec<Vertex>,
    (triangles, rect): (&[Vertex], TextureRect),
    (matrix, textures): (&[f32; 16], &EntityTextures),
) {
    let Some(glint) = textures.rect(ARMOR_GLINT_TEXTURE) else {
        return;
    };
    let size = [glint[2] - glint[0], glint[3] - glint[1]];
    let layer = [rect[2] - rect[0], rect[3] - rect[1]];
    for vertex in triangles {
        let own = [
            (vertex.uv[0] - rect[0]) / layer[0],
            (vertex.uv[1] - rect[1]) / layer[1],
        ];
        let moved = glint_uv(matrix, own);
        out.push(Vertex {
            position: vertex.position,
            uv: [glint[0] + moved[0] * size[0], glint[1] + moved[1] * size[1]],
            shade: 0.0,
            tint: [glint[0], glint[1], size[0], size[1]],
            light: [0.0; 2],
        });
    }
}
