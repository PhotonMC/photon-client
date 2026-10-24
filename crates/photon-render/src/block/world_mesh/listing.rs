//! Which of the sprites the models name an atlas definition lists.

use super::*;
use crate::resolve::{BuiltAtlas, ResolveError, ResolvedModel, build_atlas_with};
use std::collections::BTreeMap;

/// The one atlas of the models, the extra sprites and the flat items, and the sprites among them
/// that no atlas definition lists.
pub(super) fn atlas_of(
    source: &impl AssetSource,
    (models, extra, layer_alpha): (
        &BTreeMap<String, ResolvedModel>,
        Vec<Identifier>,
        &BTreeMap<String, crate::item_model::LayerAlpha>,
    ),
    (mip, bit): (i32, i32),
) -> Result<(BuiltAtlas, Vec<String>), ResolveError> {
    let models: Vec<&ResolvedModel> = models.values().collect();
    let hidden = unlisted_sprites(source, &models, layer_alpha);
    let hiding = crate::resolve::Hiding {
        inner: source,
        hidden: hidden.iter().cloned().collect(),
    };
    let atlas = build_atlas_with(
        &hiding,
        &models,
        &super::mesh::extra_sprites(extra),
        (8192, mip),
        bit,
    )?;
    Ok((atlas, hidden.into_iter().collect()))
}

/// The sprites that models name, and the flat items extrude, which no atlas definition lists.
fn unlisted_sprites(
    source: &impl AssetSource,
    models: &[&ResolvedModel],
    layer_alpha: &BTreeMap<String, crate::item_model::LayerAlpha>,
) -> std::collections::BTreeSet<String> {
    crate::resolve::face_sprites(models)
        .into_iter()
        .chain(
            layer_alpha
                .keys()
                .filter_map(|key| Identifier::parse(key).ok()),
        )
        .filter(|sprite| !source.sprite_listed(sprite))
        .map(|sprite| sprite.as_string())
        .collect()
}
