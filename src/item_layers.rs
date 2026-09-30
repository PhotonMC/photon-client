//! Item models turned into layers once and kept.

/// Each stack's layers in one display context, built the first time a slot
/// shows it: by item, then by component patch.
#[derive(Default)]
pub struct ItemLayerCache {
    layers: std::collections::HashMap<i32, Vec<PatchLayers>>,
}

/// The layers of an item with one component patch.
pub type PatchLayers = (Vec<u8>, Vec<voxel_render::world_mesh::ItemLayer>);

impl ItemLayerCache {
    pub fn get(
        &mut self,
        stack: &voxel_world::ClientItemStackState,
        build: impl FnOnce() -> Vec<voxel_render::world_mesh::ItemLayer>,
    ) -> &[voxel_render::world_mesh::ItemLayer] {
        let variants = self.layers.entry(stack.item_registry_id()).or_default();
        let index = match variants
            .iter()
            .position(|(patch, _)| patch == stack.patch_bytes())
        {
            Some(index) => index,
            None => {
                variants.push((stack.patch_bytes().to_vec(), build()));
                variants.len() - 1
            }
        };
        &variants[index].1
    }
}
