//! What the flakes of a block are made of: `BlockStateModelSet.getParticleMaterial` and the tint
//! source's `colorAsTerrainParticle`.

use super::*;
use photon_core::ids::BlockStateId;

/// The sprite a blockstate file's model names as its particle for a state: the first model of the
/// weighted list the state selects, or, in a multipart file, of its first selector (whatever the
/// state), through that model's `particle` texture slot.
pub(super) fn particle_sprite(
    file: &crate::block::blockstate::BlockStateFile,
    properties: &BTreeMap<String, String>,
    models: &BTreeMap<String, ResolvedModel>,
) -> Option<String> {
    let variant = match (&file.variants, &file.multipart) {
        (Some(variants), _) => variants
            .iter()
            .find(|(selector, _)| crate::block::blockstate::selector_matches(selector, properties))
            .and_then(|(_, list)| list.first())?,
        (None, Some(multipart)) => multipart.first()?.apply.first()?,
        (None, None) => return None,
    };
    let model = models.get(&variant.model.as_string())?;
    model
        .textures
        .material("particle")
        .map(|material| material.sprite.as_string())
}

impl BlockRenderer {
    /// The sprite of the block atlas a state's flakes show (`getParticleMaterial(state).sprite()`),
    /// nothing where the state has no model to name one.
    pub fn particle_sprite(&self, state: BlockStateId) -> Option<&str> {
        self.particle_sprites
            .get(usize::try_from(state.0).ok()?)?
            .as_deref()
    }

    /// `BlockTintSource.colorAsTerrainParticle` of the state's first tint source, if it has one:
    /// the colour it is drawn in, except that a grass block's flakes are not tinted.
    pub fn particle_tint(
        &self,
        state: BlockStateId,
        position: [i32; 3],
        biome_tint: &impl Fn(crate::block::biome_tint::TintKind, [i32; 3]) -> u32,
    ) -> Option<i32> {
        let (block, offset) = registry::block_for_state(state)?;
        if block.index() == photon_data::block::ids::GRASS_BLOCK {
            return Some(-1);
        }
        let properties: BTreeMap<String, String> = block
            .properties_at(offset)
            .unwrap_or_default()
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect();
        let mut colours = Vec::new();
        StateTints::of(block.name, &properties).colours(position, biome_tint, &mut colours);
        colours.first().map(|colour| *colour as i32)
    }

    /// Where a sprite is in the block atlas, as `u0`, `v0`, `u1`, `v1`.
    pub fn sprite_rect(&self, name: &str) -> Option<[f32; 4]> {
        self.atlas.sprites.get(name).copied()
    }
}
