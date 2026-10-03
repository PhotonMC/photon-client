//! What humanoids wear (`HumanoidArmorLayer`, `EquipmentLayerRenderer`): each
//! worn piece's model, posed as its wearer is, in the textures its equipment
//! asset names, coloured by its dye.

use std::collections::HashMap;

use voxel_game::item_values::Values;
use voxel_render::entity_textures::{EntityTextures, TextureRect};
use voxel_render::equipment::{EquipmentInfo, parse_equipment_info};
use voxel_render::mob_model::MobModel;
use voxel_render::scene::Vertex;
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
}

/// Turns a posed armor model into triangles, in a texture, multiplied by a
/// colour (`0xRRGGBB`).
pub type Place<'a> = &'a dyn Fn(&MobModel, TextureRect, Option<u32>) -> Vec<Vertex>;

/// The layer types armor is drawn in, by `HumanoidArmorLayer`.
const HUMANOID: &str = "humanoid";
const LEGGINGS: &str = "humanoid_leggings";
const BABY: &str = "humanoid_baby";

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
        }
    }

    /// The paths (under `textures/`) of every texture a humanoid's armor wears.
    pub fn textures(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .infos
            .values()
            .flat_map(|info| {
                [HUMANOID, LEGGINGS, BABY]
                    .into_iter()
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
        }
        vertices
    }

    /// The equipment asset a stack is worn as in a slot: its item's
    /// `equippable` component names it and the slot it goes in
    /// (`HumanoidArmorLayer.shouldRender`).
    fn asset_of(&self, stack: &ClientItemStackState, slot: Slot) -> Option<String> {
        let item = usize::try_from(stack.item_registry_id())
            .ok()
            .and_then(|index| voxel_data::item::registry::ITEMS.get(index))?;
        let equip = item
            .equippable
            .as_ref()
            .filter(|equip| equip.slot == slot.name())?;
        let asset = equip.asset?.strip_prefix("minecraft:")?;
        self.infos.contains_key(asset).then(|| asset.to_owned())
    }
}
