//! Item models turned into layers once and kept.

use photon_core::ids::BlockStateId;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::{BlockRenderer, ItemLayer};
use photon_render::model::DisplayContext;
use photon_world::ClientItemStackState;

/// Each stack's layers in one display context, built the first time a slot
/// shows it: by item, then by component patch.
pub struct ItemLayerCache {
    context: DisplayContext,
    layers: std::collections::HashMap<i32, Vec<PatchLayers>>,
    /// Block states drawn on entities, by state id, with the light each gives off.
    blocks: std::collections::HashMap<BlockStateId, Option<(ItemLayer, i32)>>,
}

/// The layers of an item with one component patch.
type PatchLayers = (Vec<u8>, Vec<ItemLayer>);

impl ItemLayerCache {
    pub fn new(context: DisplayContext) -> Self {
        Self {
            context,
            layers: std::collections::HashMap::new(),
            blocks: std::collections::HashMap::new(),
        }
    }

    /// The model of a block state drawn on an entity (`BlockModelResolver`),
    /// built the first time it is drawn.
    pub fn block(
        &mut self,
        state: BlockStateId,
        renderer: &BlockRenderer,
    ) -> Option<&(ItemLayer, i32)> {
        self.blocks
            .entry(state)
            .or_insert_with(|| renderer.entity_block(state))
            .as_ref()
    }

    /// A stack's layers, its grass tinted by climate.
    pub fn get(
        &mut self,
        stack: &ClientItemStackState,
        renderer: &BlockRenderer,
        tints: &BiomeTints,
    ) -> &[ItemLayer] {
        let context = self.context;
        let variants = self.layers.entry(stack.item_registry_id().0).or_default();
        let index = if let Some(index) = variants
            .iter()
            .position(|(patch, _)| patch == stack.patch_bytes())
        {
            index
        } else {
            let layers = renderer.item_layers(
                stack.item_registry_id(),
                context,
                &|temperature, downfall| tints.grass_at_climate(temperature, downfall),
                stack.patch_bytes(),
            );
            variants.push((stack.patch_bytes().to_vec(), layers));
            variants.len() - 1
        };
        &variants[index].1
    }
}
