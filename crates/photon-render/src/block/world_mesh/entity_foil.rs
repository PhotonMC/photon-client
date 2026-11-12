//! The enchantment glint of the special models: only a trident and a shield draw one, with the
//! entity glint's texture matrix and the model's own coordinates, not the item atlas's.

use super::*;
use crate::texture::atlas::{AtlasSprite, SpriteEntry, Stitcher};
use std::collections::BTreeMap;

/// How a layer of an item draws its glint.
#[derive(Debug, Clone, PartialEq)]
pub enum LayerFoil {
    /// The layer's quads again with the item atlas's coordinates and the item glint's matrix
    /// (`ItemFeatureRenderer.prepareFoilSubmit`).
    Standard,
    /// A model that draws its glint itself (`ShieldSpecialRenderer`, `TridentSpecialRenderer`):
    /// the coordinate of each corner of each quad, moved by the entity glint's matrix.
    Entity(Vec<[f32; 2]>),
    /// A special renderer that takes the foil flag and leaves it unused.
    None,
}

/// What a stack's layers say about its glint, as a whole.
#[derive(Debug, Clone, PartialEq)]
pub enum StackFoil {
    Standard,
    Entity(Vec<[f32; 2]>),
    None,
}

/// The glint the layers of one stack draw. Layers of one kind give that kind; a stack mixing kinds
/// (no vanilla item does) keeps the standard glint.
pub fn stack_foil(layers: &[ItemLayer]) -> StackFoil {
    if layers.is_empty() {
        return StackFoil::Standard;
    }
    if layers.iter().all(|layer| layer.foil == LayerFoil::None) {
        return StackFoil::None;
    }
    let mut corners = Vec::new();
    for layer in layers {
        let LayerFoil::Entity(own) = &layer.foil else {
            return StackFoil::Standard;
        };
        corners.extend_from_slice(own);
    }
    StackFoil::Entity(corners)
}

/// The sprites of the `shield_patterns` atlas stitched as the game stitches it (no mip levels, the
/// missing texture among them), each by name with its rectangle `[u0, v0, u1, v1]`.
pub(super) fn shield_rects(source: &impl AssetSource, bit: i32) -> BTreeMap<String, [f32; 4]> {
    let mut sizes: Vec<(Identifier, (i32, i32))> =
        vec![(Identifier::parse("missingno").expect("fixed name"), {
            let missing = crate::texture::missing::missing_image();
            (missing.width as i32, missing.height as i32)
        })];
    for name in source.atlas_sprites("shield_patterns") {
        let size = source
            .texture(&name)
            .and_then(|bytes| crate::texture::png::decode_rgba(&bytes).ok())
            .map(|image| (image.width as i32, image.height as i32))
            .unwrap_or_else(|| {
                let missing = crate::texture::missing::missing_image();
                (missing.width as i32, missing.height as i32)
            });
        sizes.push((name, size));
    }
    if sizes.len() == 1 {
        return BTreeMap::new();
    }
    let mut stitcher = Stitcher::new(16384, 16384, 0, bit);
    for (name, (width, height)) in &sizes {
        stitcher.register_sprite(SpriteEntry {
            width: *width,
            height: *height,
            name: name.clone(),
        });
    }
    if stitcher.stitch().is_err() {
        return BTreeMap::new();
    }
    let (atlas_width, atlas_height) = (stitcher.width(), stitcher.height());
    stitcher
        .placements()
        .iter()
        .filter_map(|placement| {
            let (_, (width, height)) = sizes.iter().find(|(name, _)| *name == placement.name)?;
            let sprite = AtlasSprite::new(placement, *width, *height, atlas_width, atlas_height);
            Some((
                placement.name.as_string(),
                [sprite.u0, sprite.v0, sprite.u1, sprite.v1],
            ))
        })
        .collect()
}

/// How the layer of a special model draws its glint: a trident over the model's own coordinates, a
/// shield over its sprite's place in the shield atlas (`sprite.wrap`), the rest not at all.
pub(super) fn special_foil(
    kind: &crate::item_model::SpecialModel,
    part: &crate::entity::special_models::ModelLayer,
    shield: &BTreeMap<String, [f32; 4]>,
) -> LayerFoil {
    use crate::item_model::SpecialModel;
    let rect = match kind {
        SpecialModel::Trident => None,
        SpecialModel::Shield => Identifier::parse(&part.sprite)
            .ok()
            .and_then(|sprite| shield.get(&sprite.as_string())),
        _ => return LayerFoil::None,
    };
    let corners = part
        .quads
        .iter()
        .flatten()
        .map(|vertex| match rect {
            Some(rect) => [
                rect[0] + (rect[2] - rect[0]) * vertex.uv[0],
                rect[1] + (rect[3] - rect[1]) * vertex.uv[1],
            ],
            None => vertex.uv,
        })
        .collect();
    LayerFoil::Entity(corners)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(foil: LayerFoil) -> ItemLayer {
        ItemLayer {
            mesh: Default::default(),
            side_lit: false,
            transform: None,
            foil,
        }
    }

    fn part(sprite: &str) -> crate::entity::special_models::ModelLayer {
        let vertex = |uv: [f32; 2]| crate::entity::player_model::ModelVertex {
            position: [0.0; 3],
            uv,
            normal: [0.0, 1.0, 0.0],
        };
        crate::entity::special_models::ModelLayer {
            quads: vec![[
                vertex([0.0, 0.0]),
                vertex([0.5, 0.0]),
                vertex([0.5, 0.5]),
                vertex([0.0, 0.5]),
            ]],
            sprite: sprite.to_owned(),
            tint: 0xFFFF_FFFF,
        }
    }

    #[test]
    fn a_trident_keeps_its_own_coordinates_a_shield_takes_its_sprite_s_place_and_a_chest_none() {
        use crate::item_model::SpecialModel;
        let rects = BTreeMap::from([(
            "minecraft:entity/shield/shield_base_nopattern".to_owned(),
            [0.5, 0.25, 0.75, 0.5],
        )]);
        let trident = special_foil(
            &SpecialModel::Trident,
            &part("entity/trident/trident"),
            &rects,
        );
        assert_eq!(
            trident,
            LayerFoil::Entity(vec![[0.0, 0.0], [0.5, 0.0], [0.5, 0.5], [0.0, 0.5]])
        );
        let shield = special_foil(
            &SpecialModel::Shield,
            &part("entity/shield/shield_base_nopattern"),
            &rects,
        );
        assert_eq!(
            shield,
            LayerFoil::Entity(vec![
                [0.5, 0.25],
                [0.625, 0.25],
                [0.625, 0.375],
                [0.5, 0.375]
            ])
        );
        let chest = SpecialModel::Chest {
            texture: "entity/chest/normal".to_owned(),
        };
        assert_eq!(
            special_foil(&chest, &part("entity/chest/normal"), &rects),
            LayerFoil::None
        );
    }

    #[test]
    fn a_stack_takes_the_glint_all_its_layers_share() {
        let entity = || layer(LayerFoil::Entity(vec![[0.5, 0.25]]));
        assert_eq!(stack_foil(&[]), StackFoil::Standard);
        assert_eq!(
            stack_foil(&[layer(LayerFoil::None), layer(LayerFoil::None)]),
            StackFoil::None
        );
        assert_eq!(
            stack_foil(&[entity(), entity()]),
            StackFoil::Entity(vec![[0.5, 0.25]; 2])
        );
        assert_eq!(
            stack_foil(&[entity(), layer(LayerFoil::Standard)]),
            StackFoil::Standard
        );
    }
}
