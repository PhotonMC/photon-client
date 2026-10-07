use super::*;

impl ClientPlayerState {
    pub fn food(&self) -> i32 {
        self.food
    }

    pub fn saturation(&self) -> f32 {
        self.saturation.get()
    }

    pub fn inventory_state_id(&self) -> i32 {
        self.inventory_state_id
    }

    pub fn inventory_slot(&self, slot: i16) -> Option<&ClientItemStackState> {
        self.inventory_slots.get(&slot).and_then(Option::as_ref)
    }

    pub fn inventory_slot_count(&self) -> usize {
        self.inventory_slots.len()
    }

    pub fn carried_item(&self) -> Option<&ClientItemStackState> {
        self.carried_item.as_ref()
    }

    /// A container menu's last 36 slots are the player's own inventory (the
    /// 27 of the main inventory, then the hotbar), so what the container
    /// says of them is what the inventory holds.
    pub(crate) fn mirror_container_into_inventory(&mut self) {
        let Some(container) = self.open_container.as_ref() else {
            return;
        };
        let Some(top) = container.top_slot_count else {
            return;
        };
        for index in 0..36 {
            let from = (top + index) as i16;
            let Some(stack) = container.slots.get(&from) else {
                continue;
            };
            // Main inventory slots 9 to 35 keep their menu numbers, and the
            // hotbar 0 to 8 is the player menu's 36 to 44.
            let to = if index < 27 {
                9 + index
            } else {
                36 + index - 27
            };
            self.inventory_slots.insert(to as i16, stack.clone());
        }
    }

    pub fn open_container(&self) -> Option<&ClientContainerState> {
        self.open_container.as_ref()
    }
}
