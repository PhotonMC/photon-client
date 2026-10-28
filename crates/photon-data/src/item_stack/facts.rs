//! What item model properties read of a stack on its own (`ItemStack.nextDamageWillBreak`,
//! `ItemStack.has`).

use super::ClientItemStackState;
use crate::item_components::ids;
use photon_core::wire::read_varint;

impl ClientItemStackState {
    /// An int component of the stack: its patch's value, else the item's own.
    fn int_component(&self, id: i32, own: i32) -> i32 {
        match self.patched(id) {
            Some(Some(value)) => read_varint(&mut value.as_slice()).unwrap_or(own),
            Some(None) => 0,
            None => own,
        }
    }

    /// `ItemStack.has` of a component the item sets itself when it has durability.
    fn has_durability_component(&self, id: i32, durable: bool) -> bool {
        match self.patched(id) {
            Some(value) => value.is_some(),
            None => durable,
        }
    }

    /// `ItemStack.nextDamageWillBreak`: a damageable stack (a maximum, a damage and no
    /// `unbreakable`) one point from its end.
    pub fn breaks_on_next_damage(&self) -> bool {
        let durability = usize::try_from(self.item_registry_id.0)
            .ok()
            .and_then(|index| crate::item::registry::ITEMS.get(index))
            .map_or(0, |item| item.max_damage);
        let durable = durability > 0;
        let damageable = self.has_durability_component(ids::MAX_DAMAGE, durable)
            && self
                .patched(ids::UNBREAKABLE)
                .is_none_or(|value| value.is_none())
            && self.has_durability_component(ids::DAMAGE, durable);
        if !damageable {
            return false;
        }
        let max = self.int_component(ids::MAX_DAMAGE, durability);
        let damage = self.int_component(ids::DAMAGE, 0);
        // `Mth.clamp`: a maximum below zero wins, where `i32::clamp` panics.
        (if damage < 0 { 0 } else { damage.min(max) }) >= max - 1
    }

    /// `ItemStack.has` for a component only a stack's own patch adds: no item definition the
    /// client tests it on declares it.
    pub fn has_added_component(&self, id: i32) -> bool {
        matches!(self.patched(id), Some(Some(_)))
    }
}
