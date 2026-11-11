//! 26.3's armor trims (`PalettedTextureManager`, `EquipmentLayerRenderer.TrimTextureKey`): a
//! pattern's grey picture is recoloured by a palette when a trim asks for it, instead of being
//! one of the sprites of an `armor_trims` atlas. The pictures every trim could ask for are made
//! up front here and named by the picture and the palette.

use photon_core::identifier::Identifier;
use photon_core::resources::Resources;
use photon_render::DecodedImage;
use photon_render::entity::equipment::EquipmentInfo;
use photon_render::entity::trim::{TrimMaterial, TrimPattern, permute};

/// The layer types a trim is drawn on (a baby's armor has none).
const LAYERS: [&str; 2] = ["humanoid", "humanoid_leggings"];

/// What names a palette in a picture's name: its path without the vanilla `trim/` folder.
fn palette_suffix(path: &str) -> &str {
    path.strip_prefix("trim/").unwrap_or(path)
}

fn name_of(id: &Identifier, folder: &str) -> Option<String> {
    let path = id.path().strip_prefix(folder)?.strip_suffix(".png")?;
    Some(path.to_owned())
}

/// The path (under `textures/palettes/`) of the palette a picture's `.mcmeta` names as its base,
/// the one its own colours are in.
fn base_palette_path(stack: &dyn Resources, picture: &Identifier) -> Option<String> {
    let meta = Identifier::parse(&format!("{}.mcmeta", picture.path())).ok()?;
    let text = String::from_utf8(stack.read(&meta)?).ok()?;
    let palette = photon_core::json::parse_object(&text)?
        .object("palette")?
        .text("base_palette")?;
    Some(
        palette
            .split_once(':')
            .map_or(palette.as_str(), |(_, path)| path)
            .to_owned(),
    )
}

/// Every pattern picture of the trim layers, as it is and in every palette. A base palette is
/// only ever the colours a picture already has, so no picture is made in it.
pub(super) fn images(stack: &dyn Resources) -> Vec<(String, DecodedImage)> {
    let load = |id: &Identifier| photon_render::texture::png::decode_rgba(&stack.read(id)?).ok();
    let pictures: Vec<(String, DecodedImage, Option<String>)> = LAYERS
        .iter()
        .flat_map(|layer| stack.list(&format!("textures/trims/entity/{layer}"), ".png"))
        .filter_map(|id| {
            let texture = name_of(&id, "textures/")?;
            Some((texture, load(&id)?, base_palette_path(stack, &id)))
        })
        .collect();
    let palettes: Vec<(String, DecodedImage)> = stack
        .list("textures/palettes", ".png")
        .iter()
        .filter(|id| id.namespace() == Identifier::DEFAULT_NAMESPACE)
        .filter_map(|id| {
            let name = name_of(id, "textures/palettes/")?;
            let is_base = pictures
                .iter()
                .any(|(_, _, base)| base.as_deref() == Some(&name));
            (!is_base).then(|| Some((palette_suffix(&name).to_owned(), load(id)?)))?
        })
        .collect();
    let mut out = Vec::new();
    for (texture, image, base) in pictures {
        let keys = base.and_then(|path| {
            load(&Identifier::parse(&format!("textures/palettes/{path}.png")).ok()?)
        });
        for (suffix, palette) in &palettes {
            // A picture without a base palette, or one of another size, is not recoloured.
            let mapped = keys
                .as_ref()
                .and_then(|keys| permute(&image, keys, palette))
                .unwrap_or_else(|| image.clone());
            out.push((format!("{texture}_{suffix}"), mapped));
        }
        out.push((texture, image));
    }
    out
}

/// The picture of a trim on a layer type: the pattern's (or an override's) in the material's
/// palette (or an override's), after the first of the equipment asset's overrides that matches.
/// An override without a palette leaves the picture as it is.
pub(super) fn sprite(
    info: &EquipmentInfo,
    layer_type: &str,
    (pattern_name, pattern): (&str, &TrimPattern),
    (material_name, material): (&str, &TrimMaterial),
) -> String {
    let found = info
        .trim_overrides
        .iter()
        .find(|entry| entry.matches(material_name, pattern_name));
    let texture = found
        .and_then(|entry| entry.texture.as_deref())
        .unwrap_or(&pattern.asset);
    let palette = match found {
        Some(entry) => entry.palette.as_deref().map(palette_suffix),
        None => Some(material.base.as_str()),
    };
    let picture = format!("trims/entity/{layer_type}/{texture}");
    palette.map_or(picture.clone(), |palette| format!("{picture}_{palette}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_render::entity::trim::TrimOverride;

    fn pattern() -> TrimPattern {
        TrimPattern {
            asset: "sentry".to_owned(),
            decal: false,
        }
    }

    fn material() -> TrimMaterial {
        TrimMaterial {
            base: "iron".to_owned(),
            overrides: Default::default(),
        }
    }

    fn picked(info: &EquipmentInfo) -> String {
        sprite(
            info,
            "humanoid",
            ("minecraft:sentry", &pattern()),
            ("minecraft:iron", &material()),
        )
    }

    #[test]
    fn a_trim_takes_its_materials_palette_unless_an_override_matches() {
        let mut info = EquipmentInfo::default();
        assert_eq!(picked(&info), "trims/entity/humanoid/sentry_iron");
        info.trim_overrides = vec![
            TrimOverride {
                material: Some("minecraft:gold".to_owned()),
                palette: Some("trim/gold_darker".to_owned()),
                ..Default::default()
            },
            TrimOverride {
                material: Some("minecraft:iron".to_owned()),
                palette: Some("trim/iron_darker".to_owned()),
                ..Default::default()
            },
        ];
        assert_eq!(picked(&info), "trims/entity/humanoid/sentry_iron_darker");
    }

    #[test]
    fn an_override_without_a_palette_leaves_the_picture_as_it_is() {
        let info = EquipmentInfo {
            trim_overrides: vec![TrimOverride {
                pattern: Some("minecraft:sentry".to_owned()),
                texture: Some("other".to_owned()),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(picked(&info), "trims/entity/humanoid/other");
    }

    #[test]
    fn the_overrides_of_an_equipment_asset_are_read_in_order() {
        let asset = photon_core::json::parse_object(
            r#"{"trim_overrides":[{"when":{"material":"minecraft:iron"},"palette":"minecraft:trim/iron_darker"}]}"#,
        )
        .unwrap();
        let parsed = TrimOverride::parse_all(&asset);
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].matches("minecraft:iron", "minecraft:any"));
        assert!(!parsed[0].matches("minecraft:gold", "minecraft:any"));
        assert_eq!(parsed[0].palette.as_deref(), Some("trim/iron_darker"));
    }
}
