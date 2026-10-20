use crate::*;
use photon_data::item::definition::UseEffects;
use photon_data::item::registry::AIR_ITEM;

impl ClientWorld {
    pub fn apply_local_player_container_content(
        &mut self,
        container_id: i32,
        state_id: i32,
        items: &[Option<ClientItemStackState>],
        carried_item: Option<&ClientItemStackState>,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if container_id == 0 {
            player.inventory_state_id = state_id;
            player.inventory_slots = items
                .iter()
                .enumerate()
                .map(|(slot, item)| (slot as i16, item.clone()))
                .collect();
            player.carried_item = carried_item.cloned();
            return true;
        }

        let Some(container) = player
            .open_container
            .as_mut()
            .filter(|container| container.container_id == container_id)
        else {
            return false;
        };
        container.state_id = state_id;
        container.slot_count = items.len();
        container.slots = items
            .iter()
            .enumerate()
            .map(|(slot, item)| (slot as i16, item.clone()))
            .collect();
        container.carried_item = carried_item.cloned();
        player.mirror_container_into_inventory();
        true
    }

    /// The local player's inventory menu: its 46 slots, the carried stack
    /// and the menu's state id.
    pub fn local_player_menu(
        &self,
    ) -> Option<(
        Vec<Option<ClientItemStackState>>,
        Option<ClientItemStackState>,
        i32,
    )> {
        let player = self.local_player.as_ref()?;
        let slots = (0..46)
            .map(|slot| player.inventory_slots.get(&slot).cloned().flatten())
            .collect();
        Some((
            slots,
            player.carried_item.clone(),
            player.inventory_state_id,
        ))
    }

    /// Write back the local player's inventory menu after a predicted click.
    pub fn set_local_player_menu(
        &mut self,
        slots: &[Option<ClientItemStackState>],
        carried: Option<ClientItemStackState>,
    ) {
        let Some(player) = self.local_player.as_mut() else {
            return;
        };
        for (slot, stack) in slots.iter().enumerate() {
            player.inventory_slots.insert(slot as i16, stack.clone());
        }
        player.carried_item = carried;
    }

    /// The menu of the container the server opened: its id and type, every
    /// slot, the carried stack and the state id. `None` with no container
    /// open or before its contents have come.
    pub fn local_player_container_menu(&self) -> Option<OpenContainerMenu> {
        let player = self.local_player.as_ref()?;
        let container = player.open_container.as_ref()?;
        if container.slot_count == 0 {
            return None;
        }
        let mut slots: Vec<Option<ClientItemStackState>> = (0..container.slot_count)
            .map(|slot| container.slots.get(&(slot as i16)).cloned().flatten())
            .collect();
        // The last 36 slots are the player's inventory, which the server can
        // also change without the container's knowing.
        if let Some(top) = container.top_slot_count {
            for index in 0..36 {
                let from = if index < 27 {
                    9 + index
                } else {
                    36 + index - 27
                };
                if let (Some(slot), Some(stack)) = (
                    slots.get_mut(top + index),
                    player.inventory_slots.get(&(from as i16)),
                ) {
                    *slot = stack.clone();
                }
            }
        }
        Some(OpenContainerMenu {
            container_id: container.container_id,
            menu_type_id: container.menu_type_id,
            slots,
            carried: container.carried_item.clone(),
            state_id: container.state_id,
        })
    }

    /// Write back the open container's menu after a predicted click.
    pub fn set_local_player_container_menu(
        &mut self,
        slots: &[Option<ClientItemStackState>],
        carried: Option<ClientItemStackState>,
    ) {
        let Some(player) = self.local_player.as_mut() else {
            return;
        };
        let Some(container) = player.open_container.as_mut() else {
            return;
        };
        for (slot, stack) in slots.iter().enumerate() {
            container.slots.insert(slot as i16, stack.clone());
        }
        container.carried_item = carried;
        player.mirror_container_into_inventory();
    }

    /// `ItemStack.shrink` on a stack in the local player's inventory menu,
    /// as the client predicts using it up. An emptied stack leaves the slot.
    pub fn shrink_local_player_slot(&mut self, slot: i16, amount: i32) {
        let Some(player) = self.local_player.as_mut() else {
            return;
        };
        if let Some(Some(stack)) = player.inventory_slots.get_mut(&slot) {
            stack.item_count -= amount;
            if stack.item_count <= 0 {
                player.inventory_slots.insert(slot, None);
            }
        }
    }

    pub fn apply_local_player_container_slot(
        &mut self,
        container_id: i32,
        state_id: i32,
        slot: i16,
        item: Option<&ClientItemStackState>,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if container_id == 0 {
            player.inventory_state_id = state_id;
            player.inventory_slots.insert(slot, item.cloned());
            return true;
        }

        let Some(container) = player
            .open_container
            .as_mut()
            .filter(|container| container.container_id == container_id)
        else {
            return false;
        };
        container.state_id = state_id;
        container.slots.insert(slot, item.cloned());
        player.mirror_container_into_inventory();
        true
    }

    /// `handleContainerSetData`: a value of the open menu's `ContainerData`.
    pub fn apply_local_player_container_data(
        &mut self,
        container_id: i32,
        index: i16,
        value: i16,
    ) -> bool {
        let Some(container) = self
            .local_player
            .as_mut()
            .and_then(|player| player.open_container.as_mut())
            .filter(|container| container.container_id == container_id)
        else {
            return false;
        };
        container.data.insert(index, value);
        true
    }

    /// What the local player's swing and use of items read of the player state: what the main hand
    /// (the selected hotbar slot) and the off hand hold, the effects that change a swing, and the
    /// server's use flags.
    pub fn local_hand_state(&self, selected_slot: i32) -> local_player::HandState {
        pub(crate) const HASTE: i32 = 2;
        const MINING_FATIGUE: i32 = 3;
        const CONDUIT_POWER: i32 = 28;
        const BLINDNESS: i32 = 14;
        const LEVITATION: i32 = 24;
        const SLOW_FALLING: i32 = 27;
        let Some(state) = self.local_player() else {
            return local_player::HandState::default();
        };
        let item = |slot: i32| {
            state
                .inventory_slot(slot as i16)
                .map_or(AIR_ITEM, |stack| stack.item_registry_id())
        };
        let amplifier = |id: i32| {
            state
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == id)
                .map(|effect| effect.amplifier())
        };
        local_player::HandState {
            main_item: item(36 + selected_slot),
            off_item: item(45),
            dig_speed: match (amplifier(HASTE), amplifier(CONDUIT_POWER)) {
                (None, None) => None,
                (a, b) => Some(a.unwrap_or(0).max(b.unwrap_or(0))),
            },
            mining_fatigue: amplifier(MINING_FATIGUE),
            using_item: state.is_using_item(),
            using_off_hand: state.used_item_is_off_hand(),
            main_use: use_effects(state.inventory_slot(36 + selected_slot as i16)),
            off_use: use_effects(state.inventory_slot(45)),
            food: state.food,
            blinded: amplifier(BLINDNESS).is_some(),
            levitating: amplifier(LEVITATION).is_some(),
            slow_falling: amplifier(SLOW_FALLING).is_some(),
            glider: false,
        }
    }

    pub fn apply_local_player_open_menu(
        &mut self,
        container_id: i32,
        menu_type_id: i32,
        title_nbt: &[u8],
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.open_container = Some(ClientContainerState::new(
            container_id,
            menu_type_id,
            title_nbt,
        ));
        true
    }

    pub fn apply_local_player_container_close(&mut self, _container_id: i32) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if player.open_container.is_some() {
            player.open_container = None;
            true
        } else {
            false
        }
    }

    pub fn apply_local_player_carried_item(&mut self, item: Option<&ClientItemStackState>) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if let Some(container) = player.open_container.as_mut() {
            container.carried_item = item.cloned();
        } else {
            player.carried_item = item.cloned();
        }
        true
    }

    pub fn apply_local_player_inventory_slot(
        &mut self,
        slot: i32,
        item: Option<&ClientItemStackState>,
    ) -> bool {
        let Some(menu_slot) = player_container_menu_slot(slot) else {
            return false;
        };
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.inventory_slots.insert(menu_slot, item.cloned());
        true
    }
}

/// `ItemStack.getOrDefault(USE_EFFECTS, DEFAULT)`: the stack's own component, else the item's,
/// and the default when the patch removes it or the slot is empty.
fn use_effects(stack: Option<&ClientItemStackState>) -> UseEffects {
    let Some(stack) = stack else {
        return UseEffects::DEFAULT;
    };
    match stack.patched(crate::item_components::ids::USE_EFFECTS) {
        Some(Some(value)) => match value.as_slice() {
            [can_sprint, _, speed @ ..] => {
                speed
                    .first_chunk::<4>()
                    .map_or(UseEffects::DEFAULT, |speed| UseEffects {
                        can_sprint: *can_sprint != 0,
                        speed: f32::from_be_bytes(*speed),
                    })
            }
            _ => UseEffects::DEFAULT,
        },
        Some(None) => UseEffects::DEFAULT,
        None => crate::block::rules::item(stack.item_registry_id())
            .and_then(|item| item.use_effects)
            .unwrap_or(UseEffects::DEFAULT),
    }
}

#[cfg(test)]
mod use_effects_tests {
    use super::*;

    #[test]
    fn a_stack_takes_its_own_use_effects_over_the_items() {
        let spear = photon_data::item::registry::named("minecraft:iron_spear");
        let bread = photon_data::item::registry::named("minecraft:bread");
        let bare = ClientItemStackState::new(1, spear);
        assert_eq!(
            use_effects(Some(&bare)),
            UseEffects {
                can_sprint: true,
                speed: 1.0
            }
        );
        assert_eq!(use_effects(None), UseEffects::DEFAULT);
        // One component added: `can_sprint`, `interact_vibrations` and a speed of 0.5.
        let mut patch = vec![1, 0, 5, 1, 0];
        patch.extend(0.5_f32.to_be_bytes());
        let own = ClientItemStackState::new(1, bread).with_patch_bytes(patch);
        assert_eq!(
            use_effects(Some(&own)),
            UseEffects {
                can_sprint: true,
                speed: 0.5
            }
        );
        // The patch removing it leaves the default, also over a spear's own.
        let removed = ClientItemStackState::new(1, spear).with_patch_bytes(vec![0, 1, 5]);
        assert_eq!(use_effects(Some(&removed)), UseEffects::DEFAULT);
    }
}
