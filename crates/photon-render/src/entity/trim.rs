//! Armor trims (`ArmorTrim`, `EquipmentLayerRenderer`'s last pass): which sprite a pattern in a
//! material draws, and the sprites themselves, a pattern's grey picture recoloured by a
//! material's palette (`PalettedPermutations`, the `armor_trims` atlas).

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use photon_core::json::{JsonObject, JsonValue, read_object};

use crate::DecodedImage;

/// A `trim_pattern`: where its pictures are, and whether it is drawn as a decal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimPattern {
    /// `asset_id`'s path (`minecraft:sentry` is `sentry`).
    pub asset: String,
    pub decal: bool,
}

/// A `trim_material`'s asset group (`MaterialAssetGroup`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimMaterial {
    /// `asset_name`: the suffix of the pictures it is drawn in.
    pub base: String,
    /// The suffix for armor of an equipment asset (`minecraft:iron`), where it differs.
    pub overrides: BTreeMap<String, String>,
}

impl TrimMaterial {
    /// `MaterialAssetGroup.assetId(equipmentAsset).suffix`.
    pub fn suffix(&self, equipment_asset: &str) -> &str {
        self.overrides
            .get(equipment_asset)
            .map_or(self.base.as_str(), String::as_str)
    }
}

/// The patterns and materials of the vanilla data pack, by name (`minecraft:sentry`).
#[derive(Debug, Clone, Default)]
pub struct TrimData {
    patterns: HashMap<String, TrimPattern>,
    materials: HashMap<String, TrimMaterial>,
}

/// The files of a data pack registry folder, by their names without the extension.
fn entries(folder: &Path) -> impl Iterator<Item = (String, JsonObject)> {
    std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_stem()?.to_str()?.to_owned();
            Some((format!("minecraft:{name}"), read_object(&path)?))
        })
}

fn path_of(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, path)| path)
}

impl TrimData {
    /// `data/minecraft/trim_pattern` and `trim_material` under a data root.
    pub fn load(data_root: &Path) -> Self {
        let base = data_root.join("minecraft");
        let patterns = entries(&base.join("trim_pattern"))
            .filter_map(|(name, json)| {
                let asset = path_of(&json.text("asset_id")?).to_owned();
                Some((
                    name,
                    TrimPattern {
                        asset,
                        decal: json.bool("decal").unwrap_or(false),
                    },
                ))
            })
            .collect();
        let materials = entries(&base.join("trim_material"))
            .filter_map(|(name, json)| {
                let overrides = json
                    .object("override_armor_assets")
                    .map(|object| {
                        object
                            .entries()
                            .filter_map(|(key, value)| match value {
                                JsonValue::String(units) => Some((
                                    String::from_utf16(key).ok()?,
                                    String::from_utf16(units).ok()?,
                                )),
                                _ => None,
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                Some((
                    name,
                    TrimMaterial {
                        base: json.text("asset_name")?,
                        overrides,
                    },
                ))
            })
            .collect();
        Self {
            patterns,
            materials,
        }
    }

    pub fn pattern(&self, name: &str) -> Option<&TrimPattern> {
        self.patterns.get(name)
    }

    pub fn material(&self, name: &str) -> Option<&TrimMaterial> {
        self.materials.get(name)
    }
}

/// `ArmorTrim.layerAssetId`'s path: the pattern's picture in the material's suffix, in the layer
/// type's folder (`trims/entity/humanoid`).
pub fn sprite_path(
    prefix: &str,
    pattern: &TrimPattern,
    material: &TrimMaterial,
    equipment: &str,
) -> String {
    format!("{prefix}/{}_{}", pattern.asset, material.suffix(equipment))
}

/// What `atlases/armor_trims.json` asks of the atlas: pictures recoloured by palettes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PalettedPermutations {
    /// The pictures' paths under `textures/`.
    pub textures: Vec<String>,
    pub palette_key: String,
    /// Each permutation's suffix and the path of its palette.
    pub permutations: BTreeMap<String, String>,
}

impl PalettedPermutations {
    /// The `paletted_permutations` source of an atlas definition.
    pub fn parse(atlas: &JsonObject) -> Option<Self> {
        let source = atlas
            .array("sources")?
            .iter()
            .find_map(|source| match source {
                JsonValue::Object(object)
                    if object.text("type").as_deref()
                        == Some("minecraft:paletted_permutations") =>
                {
                    Some(object)
                }
                _ => None,
            })?;
        let textures = source
            .array("textures")?
            .iter()
            .filter_map(|texture| match texture {
                JsonValue::String(units) => {
                    Some(path_of(&String::from_utf16(units).ok()?).to_owned())
                }
                _ => None,
            })
            .collect();
        let permutations = source
            .object("permutations")?
            .entries()
            .filter_map(|(key, value)| match value {
                JsonValue::String(units) => Some((
                    String::from_utf16(key).ok()?,
                    path_of(&String::from_utf16(units).ok()?).to_owned(),
                )),
                _ => None,
            })
            .collect();
        Some(Self {
            textures,
            palette_key: path_of(&source.text("palette_key")?).to_owned(),
            permutations,
        })
    }
}

/// `PalettedPermutations.createPaletteMapping`'s operator over a picture: a pixel whose colour is
/// one of the key palette's takes the matching colour of the permutation's palette, and keeps its
/// alpha scaled by that colour's; any other opaque colour stays, and a clear pixel stays clear.
pub fn permute(
    image: &DecodedImage,
    keys: &DecodedImage,
    values: &DecodedImage,
) -> Option<DecodedImage> {
    if keys.rgba.len() != values.rgba.len() {
        return None;
    }
    let mut palette: HashMap<[u8; 3], [u8; 4]> = HashMap::new();
    for (key, value) in keys
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(values.rgba.as_chunks::<4>().0.iter())
    {
        if key[3] != 0 {
            palette.insert(
                [key[0], key[1], key[2]],
                [value[0], value[1], value[2], value[3]],
            );
        }
    }
    let mut out = image.clone();
    for pixel in out.rgba.as_chunks_mut::<4>().0.iter_mut() {
        if pixel[3] == 0 {
            continue;
        }
        let colour = [pixel[0], pixel[1], pixel[2]];
        let value = palette
            .get(&colour)
            .copied()
            .unwrap_or([colour[0], colour[1], colour[2], 255]);
        let alpha = (u32::from(pixel[3]) * u32::from(value[3]) / 255) as u8;
        pixel.copy_from_slice(&[value[0], value[1], value[2], alpha]);
    }
    Some(out)
}
