//! What the equipment assets say of a worn stack, for the layers that depend on it.

use super::Armor;
use photon_game::item::values::equippable_of;
use photon_world::ClientItemStackState;

impl Armor {
    /// `CapeLayer.hasLayer`: whether the stack names an equipment asset that has layers of
    /// `layer_type` (the wings of an elytra, the humanoid layer of armor), whatever slot the
    /// stack is made for.
    pub fn has_layer(&self, stack: Option<&ClientItemStackState>, layer_type: &str) -> bool {
        let Some(stack) = stack else {
            return false;
        };
        equippable_of(stack.item_registry_id(), stack.patch_bytes())
            .and_then(|equip| equip.asset)
            .and_then(|asset| {
                let name = asset.strip_prefix("minecraft:")?;
                self.infos.get(name)
            })
            .is_some_and(|info| !info.layers(layer_type).is_empty())
    }
}
