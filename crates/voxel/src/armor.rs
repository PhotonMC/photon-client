//! What humanoids wear (`HumanoidArmorLayer`, `EquipmentLayerRenderer`): each
//! worn piece's model, posed as its wearer is, in the textures its equipment
//! asset names, coloured by its dye.

use std::collections::HashMap;

use voxel_game::item_values::Values;
use voxel_render::entity_textures::{EntityTextures, TextureRect};
use voxel_render::equipment::{EquipmentInfo, parse_equipment_info};
use voxel_render::mob_model::MobModel;
use voxel_render::scene::Vertex;
use voxel_render::trim::{PalettedPermutations, TrimData, permute, sprite_path};
use voxel_world::ClientItemStackState;

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
    trim_names: Option<(Vec<String>, Vec<String>)>,
}

/// Turns a posed armor model into triangles, in a texture, multiplied by a
/// colour (`0xRRGGBB`).
pub type Place<'a> = &'a dyn Fn(&MobModel, TextureRect, Option<u32>) -> Vec<Vertex>;

/// The layer types armor is drawn in, by `HumanoidArmorLayer`.
const HUMANOID: &str = "humanoid";
const LEGGINGS: &str = "humanoid_leggings";
const BABY: &str = "humanoid_baby";
const WINGS: &str = "wings";
/// The layer types mobs wear one piece in (`SimpleEquipmentLayer`): the saddles, the
/// happy ghast's harness and the nautilus's armor.
const SIMPLE: [&str; 4] = [
    "strider_saddle",
    "happy_ghast_body",
    "nautilus_body",
    "nautilus_saddle",
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
        }
    }

    /// The server's trim patterns and materials, in the order a `trim` component numbers them.
    pub fn set_trim_registries(&mut self, patterns: Vec<String>, materials: Vec<String>) {
        self.trim_names = Some((patterns, materials));
    }

    /// The pictures of the `armor_trims` atlas: each pattern's grey picture recoloured by each
    /// material's palette (`PalettedPermutations`), named by their path under `textures/`.
    pub fn trim_images(root: &str) -> Vec<(String, voxel_render::DecodedImage)> {
        let base = std::path::Path::new(root).join("minecraft");
        let load = |path: &str| {
            voxel_render::png::decode_rgba(
                &std::fs::read(base.join("textures").join(format!("{path}.png"))).ok()?,
            )
            .ok()
        };
        let Some(source) = voxel_core::json::read_object(&base.join("atlases/armor_trims.json"))
            .as_ref()
            .and_then(PalettedPermutations::parse)
        else {
            return Vec::new();
        };
        let Some(keys) = load(&source.palette_key) else {
            return Vec::new();
        };
        let palettes: Vec<(&String, voxel_render::DecodedImage)> = source
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
        let mut paths: Vec<String> = self
            .infos
            .values()
            .flat_map(|info| {
                [HUMANOID, LEGGINGS, BABY, WINGS]
                    .into_iter()
                    .chain(SIMPLE)
                    .flat_map(|layer_type| {
                        info.layers(layer_type)
                            .iter()
                            .map(move |layer| layer.texture_path(layer_type))
                    })
            })
            .collect();
        paths.sort();
        paths.dedup();
        paths
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
                let Some(layer) = voxel_data::entity_model::layer(&name) else {
                    continue;
                };
                self.models.insert(name.clone(), MobModel::bake(layer));
            }
            let model = self
                .models
                .get_mut(&name)
                .expect("the model was just built");
            pose(model);
            for (texture, colour) in layers {
                // A layer coloured to nothing is not drawn; white is no colour.
                let (true, Some(rect)) = (colour != 0, textures.rect(&texture)) else {
                    continue;
                };
                let rgb = colour & 0x00FF_FFFF;
                vertices.extend(place(model, rect, (colour != 0xFFFF_FFFF).then_some(rgb)));
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

    /// `WingsLayer`: the wings of what is worn on the chest (an elytra), spread by the wearer's
    /// animation state, in their wings texture.
    pub fn wings(
        &mut self,
        chest: Option<&ClientItemStackState>,
        (baby, crouching, spread): (bool, bool, [f32; 3]),
        textures: &EntityTextures,
        place: Place,
    ) -> Vec<Vertex> {
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
            let Some(layer) = voxel_data::entity_model::layer(name) else {
                return Vec::new();
            };
            self.models.insert(name.to_owned(), MobModel::bake(layer));
        }
        let model = self.models.get_mut(name).expect("the model was just built");
        model.setup_elytra(spread, crouching);
        // `WingsLayer.submit`: the wings sit an eighth of a block behind the back.
        model.shift_root([0.0, 0.0, 2.0]);
        let mut vertices = Vec::new();
        for (texture, colour) in layers {
            let (true, Some(rect)) = (colour != 0, textures.rect(&texture)) else {
                continue;
            };
            let rgb = colour & 0x00FF_FFFF;
            vertices.extend(place(model, rect, (colour != 0xFFFF_FFFF).then_some(rgb)));
        }
        vertices
    }

    /// `SimpleEquipmentLayer`: what a mob wears in the equipment slot `slot`
    /// (a saddle) drawn in the layer type `layer_type` of its asset on the
    /// model `model`, posed as the wearer is posed.
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
        let layers: Vec<String> = self.infos[&asset]
            .layers(layer_type)
            .iter()
            .map(|layer| layer.texture_path(layer_type))
            .collect();
        if !self.models.contains_key(model) {
            let Some(layer) = voxel_data::entity_model::layer(model) else {
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
            .filter_map(|texture| textures.rect(texture))
            .flat_map(|rect| place(model, rect, None))
            .collect()
    }

    /// The equipment asset a stack is worn as in a slot: its item's
    /// `equippable` component names it and the slot it goes in
    /// (`HumanoidArmorLayer.shouldRender`).
    fn asset_of(&self, stack: &ClientItemStackState, slot: Slot) -> Option<String> {
        self.asset_for(stack, slot.name())
    }

    /// The equipment asset a stack is worn as in the slot of this name.
    fn asset_for(&self, stack: &ClientItemStackState, slot: &str) -> Option<String> {
        let item = usize::try_from(stack.item_registry_id())
            .ok()
            .and_then(|index| voxel_data::item::registry::ITEMS.get(index))?;
        let equip = item
            .equippable
            .as_ref()
            .filter(|equip| equip.slot == slot)?;
        let asset = equip.asset?.strip_prefix("minecraft:")?;
        self.infos.contains_key(asset).then(|| asset.to_owned())
    }
}
